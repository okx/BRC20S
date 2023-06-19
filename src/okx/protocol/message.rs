use crate::okx::protocol::BRC30::BRC30Message;
use crate::okx::{datastore::BRC20::BRC20Receipt, protocol::BRC20::BRC20Message};

pub enum Message {
  BRC20(BRC20Message),
  BRC30(BRC30Message),
}

pub enum Receipt {
  BRC20(BRC20Receipt),
  // BRC30(BRC30Receipt),
}
