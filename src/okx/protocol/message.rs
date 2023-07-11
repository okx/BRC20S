use crate::okx::datastore::brc20s as store_brc20s;
use crate::okx::protocol::brc20s as proto_brc20s;
use crate::okx::{datastore::brc20::BRC20Receipt, protocol::brc20::BRC20Message};

pub enum Message {
  BRC20(BRC20Message),
  BRC20S(proto_brc20s::Message),
}

pub enum Receipt {
  BRC20(BRC20Receipt),
  BRC20S(store_brc20s::Receipt),
}
