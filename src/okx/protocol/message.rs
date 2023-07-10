use crate::okx::datastore::brc20s;
use crate::okx::protocol::brc20s::BRC20SMessage;
use crate::okx::{datastore::brc20::BRC20Receipt, protocol::brc20::BRC20Message};

pub enum Message {
  BRC20(BRC20Message),
  BRC20S(BRC20SMessage),
}

pub enum Receipt {
  BRC20(BRC20Receipt),
  BRC20S(brc20s::Receipt),
}
