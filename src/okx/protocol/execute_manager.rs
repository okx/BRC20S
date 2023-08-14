use super::*;
use crate::inscription_id::InscriptionId;
use crate::okx::datastore::balance::convert_pledged_tick_without_decimal;
use crate::okx::datastore::brc20 as brc20_store;
use crate::okx::datastore::brc20s as brc20s_store;
use crate::okx::datastore::brc20s::PledgedTick;
use crate::okx::datastore::btc as btc_store;
use crate::okx::datastore::ord as ord_store;
use crate::okx::protocol::brc20 as brc20_proto;
use crate::okx::protocol::brc20s as brc20s_proto;
use crate::okx::protocol::brc20s::params::NATIVE_TOKEN;
use crate::okx::protocol::btc as btc_proto;
use crate::{Result, SatPoint};
use bitcoin::OutPoint;

pub struct CallManager<
  'a,
  O: ord_store::OrdDataStoreReadWrite,
  L: btc_store::DataStoreReadWrite,
  N: brc20_store::DataStoreReadWrite,
  M: brc20s_store::DataStoreReadWrite,
> {
  ord_store: &'a O,
  btc_store: &'a L,
  brc20_store: &'a N,
  brc20s_store: &'a M,
}

impl<
    'a,
    O: ord_store::OrdDataStoreReadWrite,
    L: btc_store::DataStoreReadWrite,
    N: brc20_store::DataStoreReadWrite,
    M: brc20s_store::DataStoreReadWrite,
  > CallManager<'a, O, L, N, M>
{
  pub fn new(ord_store: &'a O, btc_store: &'a L, brc20_store: &'a N, brc20s_store: &'a M) -> Self {
    Self {
      ord_store,
      btc_store,
      brc20_store,
      brc20s_store,
    }
  }

  pub fn execute_message(&self, context: BlockContext, msg: &Message) -> Result {
    // execute message
    let receipt = match msg {
      Message::BRC20(msg) => brc20_proto::execute(
        context,
        self.ord_store,
        self.brc20_store,
        &brc20_proto::ExecutionMessage::from_message(self.ord_store, msg, context.network)?,
      )
      .map(|v| v.map(Receipt::BRC20))?,
      Message::BRC20S(msg) => brc20s::execute(
        context,
        self.brc20_store,
        self.brc20s_store,
        self.btc_store,
        &brc20s::ExecutionMessage::from_message(self.ord_store, msg, context.network)?,
      )
      .map(|v| v.map(Receipt::BRC20S))?,
      Message::BTC(msg) => btc_proto::gen_receipt(msg).map(|v| v.map(Receipt::BTC))?,
    };

    if receipt.is_none() {
      return Ok(());
    };

    // convert receipt to internal call message
    match receipt.unwrap() {
      Receipt::BRC20(brc20_receipt) => {
        if let Ok(brc20_store::Event::Transfer(brc20_transfer)) = brc20_receipt.result {
          let ptick = brc20s_store::PledgedTick::BRC20Tick(brc20_transfer.tick.clone());
          match convert_pledged_tick_without_decimal(
            &ptick,
            brc20_transfer.amount,
            self.brc20s_store,
            self.brc20_store,
          ) {
            Ok(amt) => {
              let passive_unstake = brc20s_proto::PassiveUnStake {
                stake: brc20_transfer.tick.as_str().to_string(),
                amount: amt.to_string(),
              };
              if let Message::BRC20(old_brc20_msg) = msg {
                let passive_msg = convert_msg_brc20_to_brc20s(old_brc20_msg, passive_unstake);
                brc20s::execute(
                  context,
                  self.brc20_store,
                  self.brc20s_store,
                  self.btc_store,
                  &brc20s::ExecutionMessage::from_message(
                    self.ord_store,
                    &passive_msg,
                    context.network,
                  )?,
                )?;
              }
            }
            Err(e) => {
              log::error!("brc20s receipt failed: {e}");
            }
          }
        }
        Ok(())
      }
      Receipt::BRC20S(brc20s_receipt) => {
        if let Ok(events) = brc20s_receipt.result {
          let mut events = events.into_iter();
          while let Some(brc20s_store::Event::Transfer(brc20s_transfer)) = events.next() {
            let ptick = brc20s_store::PledgedTick::BRC20STick(brc20s_transfer.tick_id);
            match convert_pledged_tick_without_decimal(
              &ptick,
              brc20s_transfer.amt,
              self.brc20s_store,
              self.brc20_store,
            ) {
              Ok(amt) => {
                let passive_unstake = brc20s_proto::PassiveUnStake {
                  stake: ptick.to_string(),
                  amount: amt.to_string(),
                };
                if let Message::BRC20S(old_brc20s_msg) = msg {
                  let passive_msg = convert_msg_brc20s(old_brc20s_msg, passive_unstake);
                  brc20s::execute(
                    context,
                    self.brc20_store,
                    self.brc20s_store,
                    self.btc_store,
                    &brc20s::ExecutionMessage::from_message(
                      self.ord_store,
                      &passive_msg,
                      context.network,
                    )?,
                  )?;
                }
              }
              Err(e) => {
                log::error!("brc20s receipt failed:{}", e);
              }
            }
          }
        }
        Ok(())
      }
      Receipt::BTC(btc_receipt) => {
        if let Ok(btc_store::Event::Transfer(btc_transfer)) = btc_receipt.result {
          match convert_pledged_tick_without_decimal(
            &PledgedTick::Native,
            btc_transfer.amt,
            self.brc20s_store,
            self.brc20_store,
          ) {
            Ok(amt) => {
              let passive_unstake = brc20s_proto::PassiveUnStake {
                stake: NATIVE_TOKEN.to_string(),
                amount: amt.to_string(),
              };
              if let Message::BTC(old_btc_msg) = msg {
                let passive_msg = convert_msg_btc_to_brc20s(old_btc_msg, passive_unstake);
                brc20s::execute(
                  context,
                  self.brc20_store,
                  self.brc20s_store,
                  self.btc_store,
                  &brc20s::ExecutionMessage::from_btc_message(
                    &passive_msg,
                    old_btc_msg.from.clone(),
                    context.network,
                  )?,
                )?;
              }
            }
            Err(e) => {
              log::error!("brc20s receipt failed: {e}");
            }
          }
        }
        Ok(())
      }
    }
  }
}

fn convert_msg_brc20_to_brc20s(
  msg: &brc20_proto::Message,
  op: brc20s_proto::PassiveUnStake,
) -> brc20s::Message {
  brc20s::Message {
    txid: msg.txid,
    inscription_id: msg.inscription_id,
    commit_input_satpoint: None,
    old_satpoint: msg.old_satpoint,
    new_satpoint: msg.new_satpoint,
    op: brc20s::Operation::PassiveUnStake(op),
    sat_in_outputs: msg.sat_in_outputs,
  }
}

fn convert_msg_brc20s(msg: &brc20s::Message, op: brc20s_proto::PassiveUnStake) -> brc20s::Message {
  brc20s::Message {
    txid: msg.txid,
    inscription_id: msg.inscription_id,
    commit_input_satpoint: None,
    old_satpoint: msg.old_satpoint,
    new_satpoint: msg.new_satpoint,
    op: brc20s::Operation::PassiveUnStake(op),
    sat_in_outputs: msg.sat_in_outputs,
  }
}

fn convert_msg_btc_to_brc20s(
  msg: &btc::Message,
  op: brc20s_proto::PassiveUnStake,
) -> brc20s::Message {
  brc20s::Message {
    txid: msg.txid,
    inscription_id: InscriptionId::from(msg.txid),
    commit_input_satpoint: None,
    old_satpoint: SatPoint {
      outpoint: OutPoint::null(),
      offset: 0,
    },
    new_satpoint: None,
    op: brc20s::Operation::PassiveUnStake(op),
    sat_in_outputs: false,
  }
}
