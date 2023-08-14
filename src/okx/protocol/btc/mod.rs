mod error;

use crate::{
  okx::datastore::btc::{Event, Receipt, TransferEvent},
  okx::datastore::ScriptKey,
  Result,
};
// use anyhow::anyhow;
use bitcoin::Txid;

pub use self::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
  pub txid: Txid,
  pub from: ScriptKey,
  pub amt: u128,
}

pub fn gen_receipt(msg: &Message) -> Result<Option<Receipt>> {
  let event = Event::Transfer(TransferEvent {
    amt: msg.amt,
    msg: None,
  });

  let receipt = Receipt {
    from: msg.from.clone(),
    result: Ok(event),
  };

  log::debug!("BTC message receipt: {:?}", receipt);
  Ok(Some(receipt))
}
