use serde::{Deserialize, Serialize};
use web3::types::{H256, U256, U64, Bytes, Address};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Evm {
  #[serde(rename = "d")]
  pub d: String,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Transaction {
  pub to: Option<Address>,
  pub nonce: U256,
  pub gas: U256,
  #[serde(rename = "gasPrice")]
  pub gas_price: U256,
  pub value: U256,
  pub input: Bytes,
  pub v: U64,
  pub r: U256,
  pub s: U256,
}