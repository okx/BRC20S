use super::*;
use crate::okx::datastore::balance::{
  convert_amount_with_decimal, convert_pledged_tick_with_decimal,
  convert_pledged_tick_without_decimal,
};
use crate::okx::datastore::BRC30DataStoreReadWrite;
use crate::okx::datastore::BRC20::{BRC20Event, BRC20Receipt};
use crate::okx::datastore::BRC30::{BRC30Event, PledgedTick};
use crate::okx::protocol::BRC20::BRC20Message;
use crate::okx::protocol::BRC30::operation::BRC30Operation;
use crate::okx::protocol::BRC30::{BRC30Message, PassiveUnStake};
use crate::{
  okx::datastore::{BRC20DataStoreReadWrite, OrdDataStoreReadWrite},
  Result,
};

pub struct CallManager<
  'a,
  O: OrdDataStoreReadWrite,
  N: BRC20DataStoreReadWrite,
  M: BRC30DataStoreReadWrite,
> {
  ord_store: &'a O,
  brc20_store: &'a N,
  brc30_store: &'a M,
}

impl<'a, O: OrdDataStoreReadWrite, N: BRC20DataStoreReadWrite, M: BRC30DataStoreReadWrite>
  CallManager<'a, O, N, M>
{
  pub fn new(ord_store: &'a O, brc20_store: &'a N, brc30_store: &'a M) -> Self {
    Self {
      ord_store,
      brc20_store,
      brc30_store,
    }
  }

  pub fn execute_message(&self, msg: &Message) -> Result {
    let receipt = match msg {
      Message::BRC20(msg) => {
        BRC20::execute(self.ord_store, self.brc20_store, &msg).map(|v| Receipt::BRC20(v))?
      }

      Message::BRC30(msg) => {
        todo!("add later");
      }
    };

    match receipt {
      Receipt::BRC20(brc20_receipt) => {
        match brc20_receipt.result {
          Ok(BRC20Event::InscripbeTransfer(brc20_transfer)) => {
            let ptick = PledgedTick::BRC20Tick(brc20_transfer.tick.clone());
            match convert_pledged_tick_without_decimal(
              &ptick,
              brc20_transfer.amount,
              self.brc30_store,
              self.brc20_store,
            ) {
              Ok(amt) => {
                let passive_unstake = PassiveUnStake {
                  stake: brc20_transfer.tick.as_str().to_string(),
                  amount: amt.to_string(),
                };
                if let Message::BRC20(old_brc20_msg) = msg {
                  let passive_msg = convert_brc20msg_to_brc30msg(old_brc20_msg, passive_unstake);
                  //todo need to execute passive_msg
                }
              }
              Err(e) => {
                log::error!("brc30 receipt failed:{}", e);
              }
            }
          }
          _ => { /*others operation ,we we do nothing */ }
        }
        Ok(())
      }
      Receipt::BRC30(brc30_recipt) => {
        match brc30_recipt.result {
          Ok(BRC30Event::InscribeTransfer(brc30_transfer)) => {
            let ptick = PledgedTick::BRC30Tick(brc30_transfer.tick_id.clone());
            match convert_pledged_tick_without_decimal(
              &ptick,
              brc30_transfer.amt,
              self.brc30_store,
              self.brc20_store,
            ) {
              Ok(amt) => {
                let passive_unstake = PassiveUnStake {
                  stake: ptick.to_string(),
                  amount: amt.to_string(),
                };
                if let Message::BRC30(old_brc30_msg) = msg {
                  let passive_msg = convert_brc30msg_to_brc30msg(old_brc30_msg, passive_unstake);
                  //todo need to execute passive_msg
                }
              }
              Err(e) => {
                log::error!("brc30 receipt failed:{}", e);
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

fn convert_brc20msg_to_brc30msg(msg: &BRC20Message, op: PassiveUnStake) -> BRC30Message {
  BRC30Message {
    txid: msg.txid.clone(),
    block_height: msg.block_height,
    block_time: msg.block_time,
    inscription_id: msg.inscription_id.clone(),
    inscription_number: msg.inscription_number,
    from: msg.from.clone(),
    to: msg.to.clone(),
    old_satpoint: msg.old_satpoint.clone(),
    new_satpoint: msg.new_satpoint.clone(),
    op: BRC30Operation::PassiveUnStake(op),
  }
}

fn convert_brc30msg_to_brc30msg(msg: &BRC30Message, op: PassiveUnStake) -> BRC30Message {
  BRC30Message {
    txid: msg.txid.clone(),
    block_height: msg.block_height,
    block_time: msg.block_time,
    inscription_id: msg.inscription_id.clone(),
    inscription_number: msg.inscription_number,
    from: msg.from.clone(),
    to: msg.to.clone(),
    old_satpoint: msg.old_satpoint.clone(),
    new_satpoint: msg.new_satpoint.clone(),
    op: BRC30Operation::PassiveUnStake(op),
  }
}
