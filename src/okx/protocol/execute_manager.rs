use super::*;
use crate::okx::datastore::balance::{
  convert_amount_with_decimal, convert_pledged_tick_with_decimal,
};
use crate::okx::datastore::BRC30DataStoreReadWrite;
use crate::okx::datastore::BRC20::{BRC20Event, BRC20Receipt};
use crate::okx::protocol::BRC30::{Operation as BRC30Operation, PassiveUnStake};
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
            //todo need convert amt with decimal
            //let amt = convert_pledged_tick_with_decimal()
            let passive_unstake = BRC30Operation::PassiveUnStake(PassiveUnStake {
              stake: brc20_transfer.tick.as_str().to_string(),
              amount: brc20_transfer.amount.to_string(),
            });
            // let mut msg = msg.clone();
          }
          Err(e) => {
            // if err ,we log it
            log::error!("error log:{}", e);
          }
          _ => {
            //others operation ,we we do nothing
          }
        }
        Ok(())
      }
    }
  }
}
