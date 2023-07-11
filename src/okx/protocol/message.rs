use crate::okx::datastore::brc20 as store_brc20;
use crate::okx::datastore::brc20s as store_brc20s;
use crate::okx::protocol::brc20 as proto_brc20;
use crate::okx::protocol::brc20s as proto_brc20s;

pub enum Message {
  BRC20(proto_brc20::Message),
  BRC20S(proto_brc20s::Message),
}

pub enum Receipt {
  BRC20(store_brc20::Receipt),
  BRC20S(store_brc20s::Receipt),
}
