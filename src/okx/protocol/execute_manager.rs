use super::*;
use crate::{
  okx::datastore::{BRC20DataStoreReadWrite, OrdDataStoreReadWrite},
  Result,
};
pub struct CallManager<'a, O: OrdDataStoreReadWrite, N: BRC20DataStoreReadWrite> {
  ord_store: &'a O,
  brc20_store: &'a N,
}

impl<'a, O: OrdDataStoreReadWrite, N: BRC20DataStoreReadWrite> CallManager<'a, O, N> {
  pub fn new(ord_store: &'a O, brc20_store: &'a N) -> Self {
    Self {
      ord_store,
      brc20_store,
    }
  }

  pub fn execute_message(&self, msg: &Message) -> Result {
    let receipt = match msg {
      Message::BRC20(msg) => {
        BRC20::execute(self.ord_store, self.brc20_store, &msg).map(|v| Receipt::BRC20(v))?
      }
    };

    match receipt {
      Receipt::BRC20(_receipt) => {
        // TODO: Internal message call from BRC20 to BRC30
        // if enable_brc30 {
        //   if let Ok(event) = receipt.result {
        //     if let BRC20Event::Transfer(transfer) = event {
        //       self.execute_message(BRC30::convert_message(receipt));
        //     }
        //   }
        // }
        Ok(())
      }
    }
  }
}
