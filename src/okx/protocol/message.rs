use crate::okx::datastore::brc0 as brc0_store;
use crate::okx::datastore::brc20 as brc20_store;
use crate::okx::datastore::brc20s as brc20s_store;
use crate::okx::protocol::brc0 as brc0_proto;
use crate::okx::protocol::brc20 as brc20_proto;
use crate::okx::protocol::brc20s as brc20s_proto;
use crate::{
  okx::datastore::{
    ScriptKey,
  },
  InscriptionId, SatPoint, Serialize, Deserialize
};
use bitcoin::{Txid, BlockHash};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrcZeroMsg {
  pub btc_fee: u128,
  pub msg: MsgInscription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgInscription{
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
// pub struct InscriptionContext {
//   pub txid: Txid,
//   pub inscription_id: InscriptionId,
//   pub inscription_number: i64,
//   pub old_satpoint: SatPoint,
//   pub new_satpoint: Option<SatPoint>,
//   pub from: ScriptKey,
//   pub to: Option<ScriptKey>,
//   pub sat_in_outputs: bool,
//   pub is_transfer: bool,
//   pub blockheight: u64,
//   pub blocktime: u32,
//   pub blockhash: BlockHash,
// }