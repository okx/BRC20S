use crate::okx::datastore::brc30;
use crate::okx::protocol::brc30::BRC30Message;
use crate::okx::{datastore::brc20::BRC20Receipt, protocol::brc20::BRC20Message};

pub enum Message {
  BRC20(BRC20Message),
  BRC30(BRC30Message),
}

pub enum Receipt {
  BRC20(BRC20Receipt),
  BRC30(brc30::Receipt),
}
