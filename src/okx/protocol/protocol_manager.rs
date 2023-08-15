use std::collections::HashMap;

use super::*;
use crate::{
  index::BlockData,
  okx::datastore::ord::operation::InscriptionOp,
  okx::datastore::{
    brc20, brc20s,
    btc::{self, Balance},
    ord, ScriptKey,
  },
  okx::protocol::btc:: {
    self as btc_proto, num::Num, Error
  },
  Instant, Result,
};
use anyhow::anyhow;
use bitcoin::{Network, Script, Txid};
use bitcoincore_rpc::Client;
use serde_json;
use crate::okx::datastore::btc::DataStoreReadOnly;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BlockContext {
  pub network: Network,
  pub blockheight: u64,
  pub blocktime: u32,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum ProtocolKind {
  BRC20,
  BRC20S,
  BTC,
}

pub struct ProtocolManager<
  'a,
  O: ord::OrdDataStoreReadWrite,
  L: btc::DataStoreReadWrite,
  P: brc20::DataStoreReadWrite,
  M: brc20s::DataStoreReadWrite,
> {
  ord_store: &'a O,
  btc_store: &'a L,
  first_inscription_height: u64,
  call_man: CallManager<'a, O, L, P, M>,
  resolve_man: MsgResolveManager<'a, O, L, P, M>,
}

impl<
    'a,
    O: ord::OrdDataStoreReadWrite,
    L: btc::DataStoreReadWrite,
    P: brc20::DataStoreReadWrite,
    M: brc20s::DataStoreReadWrite,
  > ProtocolManager<'a, O, L, P, M>
{
  // Need three datastore, and they're all in the same write transaction.
  pub fn new(
    client: &'a Client,
    ord_store: &'a O,
    btc_store: &'a L,
    brc20_store: &'a P,
    brc20s_store: &'a M,
    first_inscription_height: u64,
    first_brc20_height: u64,
    first_brc20s_height: u64,
  ) -> Self {
    Self {
      resolve_man: MsgResolveManager::new(
        client,
        ord_store,
        btc_store,
        brc20_store,
        brc20s_store,
        first_brc20_height,
        first_brc20s_height,
      ),
      ord_store,
      btc_store,
      first_inscription_height,
      call_man: CallManager::new(ord_store, btc_store, brc20_store, brc20s_store),
    }
  }

  pub(crate) fn index_block(
    &self,
    context: BlockContext,
    block: &BlockData,
    mut operations: HashMap<Txid, Vec<InscriptionOp>>,
  ) -> Result {
    let start = Instant::now();
    let mut inscriptions_size = 0;
    let mut messages_size = 0;
    let mut balance_change_map: HashMap<String, i64> = HashMap::new();

    let (coinbase_tx, _) = block.txdata.get(0).unwrap();
    for output in &coinbase_tx.output {
      let sk = ScriptKey::from_script(&output.script_pubkey, context.network);
      let num_difference = balance_change_map.entry(serde_json::to_string(&sk)?).or_insert(0);
      *num_difference += output.value as i64;
    }
    // skip the coinbase transaction.
    for (tx, txid) in block.txdata.iter().skip(1) {
      // update address btc balance
      for input in &tx.input {
        let prev_output = &self
          .ord_store
          .get_outpoint_to_txout(input.previous_output)
          .map_err(|e| anyhow!("failed to get tx out from state! error: {e}",))?
          .unwrap();

        let sk = ScriptKey::from_script(&prev_output.script_pubkey, context.network);
        let num_difference = balance_change_map.entry(serde_json::to_string(&sk)?).or_insert(0);
        *num_difference -= prev_output.value as i64;
      }

      for output in &tx.output {
        let sk = ScriptKey::from_script(&output.script_pubkey, context.network);
        let num_difference = balance_change_map.entry(serde_json::to_string(&sk)?).or_insert(0);
        *num_difference += output.value as i64;
      }

      for (sk, diff) in balance_change_map.iter() {
        let sk: ScriptKey = serde_json::from_str(sk)?;
        let mut balance = self
          .btc_store
          .get_balance(&sk)
          .map_err(|e| anyhow!("failed to get balance from state! error: {e}"))?
          .map_or(Balance::new(), |v| v);

        let amt = Num::from(diff.clone().abs() as u64);

        if diff.clone() < 0i64 {
          // sub amount to available balance.
          balance.overall_balance = Into::<Num>::into(balance.overall_balance)
            .checked_sub(&amt)?
            .checked_to_u64()?;

          // BTC transfer to BRC20S passive withdrawal
          let msg = Message::BTC(btc_proto::Message {
            txid: tx.txid(),
            from: sk.clone(),
            amt: diff.clone().abs() as u128,
          });
          self.call_man.execute_message(context, &msg)?;
        } else {
          // add amount to available balance.
          balance.overall_balance = Into::<Num>::into(balance.overall_balance)
            .checked_add(&amt)?
            .checked_to_u64()?;
        }

        // store to database.
        self
          .btc_store.update_balance(&sk, balance)
          .map_err(|e| {
            anyhow!("failed to update balance to state! error: {e}")
          })?;
      }

      if let Some(tx_operations) = operations.remove(txid) {
        // save transaction operations.
        if context.blockheight >= self.first_inscription_height {
          self
            .ord_store
            .save_transaction_operations(txid, &tx_operations)
            .map_err(|e| {
              anyhow!("failed to set transaction ordinals operations to state! error: {e}")
            })?;
          inscriptions_size += tx_operations.len();
        }

        // Resolve and execute messages.
        let messages = self
          .resolve_man
          .resolve_message(context, tx, tx_operations)?;
        for msg in messages.iter() {
          self.call_man.execute_message(context, msg)?;
        }
        messages_size += messages.len();

      }
    }

    log::info!(
      "Protocol Manager indexed block {} with {} messages, ord inscriptions {} in {} ms",
      context.blockheight,
      messages_size,
      inscriptions_size,
      (Instant::now() - start).as_millis(),
    );
    Ok(())
  }
}
