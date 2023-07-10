use super::*;
use crate::okx::datastore::balance::convert_pledged_tick_without_decimal;
use crate::okx::datastore::brc20::BRC20Event;
use crate::okx::datastore::brc20s::{Event, PledgedTick};
use crate::okx::datastore::DataStoreReadWrite;
use crate::okx::protocol::brc20::{BRC20ExecutionMessage, BRC20Message};
use crate::okx::protocol::brc20s::operation::BRC20SOperation;
use crate::okx::protocol::brc20s::{BRC20SMessage, ExecutionMessage, PassiveUnStake};
use crate::{
  okx::datastore::{BRC20DataStoreReadWrite, OrdDataStoreReadWrite},
  Result,
};

pub struct CallManager<
  'a,
  O: OrdDataStoreReadWrite,
  N: BRC20DataStoreReadWrite,
  M: DataStoreReadWrite,
> {
  ord_store: &'a O,
  brc20_store: &'a N,
  brc20s_store: &'a M,
}

impl<'a, O: OrdDataStoreReadWrite, N: BRC20DataStoreReadWrite, M: DataStoreReadWrite>
  CallManager<'a, O, N, M>
{
  pub fn new(ord_store: &'a O, brc20_store: &'a N, brc20s_store: &'a M) -> Self {
    Self {
      ord_store,
      brc20_store,
      brc20s_store,
    }
  }

  pub fn execute_message(&self, context: BlockContext, msg: &Message) -> Result {
    // execute message
    let receipt = match msg {
      Message::BRC20(msg) => brc20::execute(
        context,
        self.ord_store,
        self.brc20_store,
        &BRC20ExecutionMessage::from_message(self.ord_store, &msg, context.network)?,
      )
      .map(|v| v.map(Receipt::BRC20))?,
      Message::BRC20S(msg) => brc20s::execute(
        context,
        self.brc20_store,
        self.brc20s_store,
        &ExecutionMessage::from_message(self.ord_store, &msg, context.network)?,
      )
      .map(|v| v.map(Receipt::BRC20S))?,
    };

    if receipt.is_none() {
      return Ok(());
    };

    // convert receipt to internal call message
    match receipt.unwrap() {
      Receipt::BRC20(brc20_receipt) => {
        match brc20_receipt.result {
          Ok(BRC20Event::Transfer(brc20_transfer)) => {
            let ptick = PledgedTick::BRC20Tick(brc20_transfer.tick.clone());
            match convert_pledged_tick_without_decimal(
              &ptick,
              brc20_transfer.amount,
              self.brc20s_store,
              self.brc20_store,
            ) {
              Ok(amt) => {
                let passive_unstake = PassiveUnStake {
                  stake: brc20_transfer.tick.as_str().to_string(),
                  amount: amt.to_string(),
                };
                if let Message::BRC20(old_brc20_msg) = msg {
                  let passive_msg = convert_brc20msg_to_brc30msg(old_brc20_msg, passive_unstake);
                  brc20s::execute(
                    context,
                    self.brc20_store,
                    self.brc20s_store,
                    &ExecutionMessage::from_message(self.ord_store, &passive_msg, context.network)?,
                  )?;
                }
              }
              Err(e) => {
                log::error!("brc20s receipt failed: {e}");
              }
            }
          }
          _ => { /*others operation ,we we do nothing */ }
        }
        Ok(())
      }
      Receipt::BRC20S(brc20s_receipt) => {
        match brc20s_receipt.result {
          Ok(events) => {
            let mut events = events.into_iter();
            while let Some(Event::Transfer(brc20s_transfer)) = events.next() {
              let ptick = PledgedTick::BRC20STick(brc20s_transfer.tick_id.clone());
              match convert_pledged_tick_without_decimal(
                &ptick,
                brc20s_transfer.amt,
                self.brc20s_store,
                self.brc20_store,
              ) {
                Ok(amt) => {
                  let passive_unstake = PassiveUnStake {
                    stake: ptick.to_string(),
                    amount: amt.to_string(),
                  };
                  if let Message::BRC20S(old_brc20s_msg) = msg {
                    let passive_msg = convert_brc20s_msg(old_brc20s_msg, passive_unstake);
                    brc20s::execute(
                      context,
                      self.brc20_store,
                      self.brc20s_store,
                      &ExecutionMessage::from_message(
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
          _ => { /*others operation ,we we do nothing */ }
        }
        Ok(())
      }
    }
  }
}

fn convert_brc20msg_to_brc30msg(msg: &BRC20Message, op: PassiveUnStake) -> BRC20SMessage {
  BRC20SMessage {
    txid: msg.txid.clone(),
    inscription_id: msg.inscription_id.clone(),
    commit_input_satpoint: None,
    old_satpoint: msg.old_satpoint.clone(),
    new_satpoint: msg.new_satpoint.clone(),
    op: BRC20SOperation::PassiveUnStake(op),
  }
}

fn convert_brc20s_msg(msg: &BRC20SMessage, op: PassiveUnStake) -> BRC20SMessage {
  BRC20SMessage {
    txid: msg.txid.clone(),
    inscription_id: msg.inscription_id.clone(),
    commit_input_satpoint: None,
    old_satpoint: msg.old_satpoint.clone(),
    new_satpoint: msg.new_satpoint.clone(),
    op: BRC20SOperation::PassiveUnStake(op),
  }
}
