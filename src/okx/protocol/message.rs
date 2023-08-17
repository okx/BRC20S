use crate::okx::datastore::brc0 as brc0_store;
use crate::okx::datastore::brc20 as brc20_store;
use crate::okx::datastore::brc20s as brc20s_store;
use crate::okx::protocol::brc0 as brc0_proto;
use crate::okx::protocol::brc20 as brc20_proto;
use crate::okx::protocol::brc20s as brc20s_proto;

pub enum Message {
  BRC20(brc20_proto::Message),
  BRC20S(brc20s_proto::Message),
  BRC0(brc0_proto::Message),
}

pub enum Receipt {
  BRC20(brc20_store::Receipt),
  BRC20S(brc20s_store::Receipt),
  BRC0(brc0_store::Receipt),
}
