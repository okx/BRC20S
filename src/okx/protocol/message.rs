use crate::okx::datastore::brc20 as brc20_store;
use crate::okx::datastore::brc20s as brc20s_store;
use crate::okx::protocol::brc20 as brc20_proto;
use crate::okx::protocol::brc20s as brc20s_proto;
use crate::{Deserialize, Serialize};
#[allow(clippy::upper_case_acronyms)]
pub enum Message {
  BRC20(brc20_proto::Message),
  BRC20S(brc20s_proto::Message),
}

#[allow(clippy::upper_case_acronyms)]
pub enum Receipt {
  BRC20(brc20_store::Receipt),
  BRC20S(brc20s_store::Receipt),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrcZeroMsg {
  pub btc_fee: u128,
  pub msg: MsgInscription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgInscription {
  pub inscription: String,
  pub inscription_context: InscriptionContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InscriptionContext {
  pub txid: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_sat_point: String,
  pub new_sat_point: String,
  pub sender: String,
  pub receiver: String,
  pub is_transfer: bool,
  pub block_height: u64,
  pub block_time: u32,
  pub block_hash: String,
}
