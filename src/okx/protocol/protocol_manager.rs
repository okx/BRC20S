use std::collections::HashMap;

use super::*;
use crate::{
  index::BlockData,
  okx::datastore::ord::operation::InscriptionOp,
  okx::datastore::{brc20, brc20s, btc::{self, Balance}, ord, ScriptKey},
  Instant, Result,
};
use anyhow::anyhow;
use bitcoin::{Network, Txid, Script};
use bitcoincore_rpc::Client;

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

    let (coinbase_tx, _) = block.txdata.get(0).unwrap();
    //todo: coinbase_tx.output.len() must be 1
    for output in &coinbase_tx.output {
      let sk = ScriptKey::from_script(
        &output.script_pubkey,
        context.network,
      );
      // todo: enhanced security
      // record coinbase tx output rewards
      let new_balance = match self.btc_store.get_balance(&sk).unwrap() {
        Some(balance) => balance.overall_balance,
        None => 0u64,
      } + output.value;
      
      let new_balance = Balance {
        overall_balance: new_balance,
      };

      self.btc_store.update_balance(&sk, new_balance).unwrap();
    }
    // skip the coinbase transaction.
    for (tx, txid) in block.txdata.iter().skip(1) {

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
      
      // update btc balance
      for input in &tx.input {
        let prev_output = &self.ord_store
        .get_outpoint_to_txout(input.previous_output)
        .map_err(|e| anyhow!("failed to get tx out from state! error: {e}",))?
        .unwrap();

        let sk = ScriptKey::from_script(
          &prev_output.script_pubkey,
          context.network,
        );

        // todo: enhanced security
        let new_balance = match self.btc_store.get_balance(&sk).unwrap() {
          Some(balance) => balance.overall_balance,
          None => 0u64,
        } - prev_output.value;
        
        let new_balance = Balance {
          overall_balance: new_balance,
        };

        self.btc_store.update_balance(&sk, new_balance).unwrap();
      }

      for output in &tx.output {
        let sk = ScriptKey::from_script(
          &output.script_pubkey,
          context.network,
        );
        // todo: enhanced security
        let new_balance = match self.btc_store.get_balance(&sk).unwrap() {
          Some(balance) => balance.overall_balance,
          None => 0u64,
        } + output.value;
        
        let new_balance = Balance {
          overall_balance: new_balance,
        };
  
        self.btc_store.update_balance(&sk, new_balance).unwrap();
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
