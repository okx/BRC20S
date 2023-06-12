use crate::okx::datastore::brc30::{BRC30Tick, Pid, PledgedTick, PoolType, TickId};
use crate::okx::protocol::brc30::BRC30Error;
use crate::InscriptionId;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Receipt {
  pub inscription_id: InscriptionId,
  pub result: Result<BRC30Event, BRC30Error>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum BRC30Event {
  DeployTick(DeployTickEvent),
  DeployPool(DeployPoolEvent),
  Deposit(DepositEvent),
  Withdraw(WithdrawEvent),
  Mint(MintEvent),
  InscribeTransfer(InscribeTransferEvent),
  Transfer(TransferEvent),
}

#[derive(Debug, PartialEq)]
pub enum EventType {
  DeployTick,
  DeployPool,
  Deposit,
  Withdraw,
  Mint,
  InscribeTransfer,
  Transfer,
}

impl Serialize for EventType {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      Self::DeployTick => "deployTick".serialize(serializer),
      Self::DeployPool => "deployPool".serialize(serializer),
      Self::Deposit => "deposit".serialize(serializer),
      Self::Withdraw => "withdraw".serialize(serializer),
      Self::Mint => "mint".serialize(serializer),
      Self::InscribeTransfer => "inscribeTransfer".serialize(serializer),
      Self::Transfer => "transfer".serialize(serializer),
    }
  }
}

impl<'de> Deserialize<'de> for EventType {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    match String::deserialize(deserializer)?.as_str() {
      "deployTick" => Ok(Self::DeployTick),
      "deployPool" => Ok(Self::DeployPool),
      "deposit" => Ok(Self::Deposit),
      "withdraw" => Ok(Self::Withdraw),
      "mint" => Ok(Self::Mint),
      "inscribeTransfer" => Ok(Self::InscribeTransfer),
      "transfer" => Ok(Self::Transfer),
      _ => Err("no such event type"),
    }
    .map_err(|e| de::Error::custom(format!("deserialize event type error: {}", e)))
  }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DeployTickEvent {
  pub tick_id: TickId,
  pub name: BRC30Tick,
  pub supply: u128,
  pub decimal: u8,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DeployPoolEvent {
  pid: Pid,           // 池子id
  ptype: PoolType,    // 池子类型
  stake: PledgedTick, // 质押的代币名称
  erate: u128,        // 产币速率
  dmax: u128,         // 可挖矿数量
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DepositEvent {
  pid: Pid, // 池子id
  amt: u128,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct WithdrawEvent {
  pid: Pid, // 池子id
  amt: u128,
  initiative: bool,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct MintEvent {
  pid: Pid, // tick id
  amt: u128,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct InscribeTransferEvent {
  tick_id: TickId, // tick id
  amt: u128,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TransferEvent {
  tick_id: TickId, // tick id
  amt: u128,
}

#[cfg(test)]
mod tests {
  use super::*;
  use bitcoin::Address;
  use std::str::FromStr;

  #[test]
  fn action_receipt_serialize() {
    let action_receipt = Receipt {
      inscription_id: InscriptionId::from_str(
        "9991111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      result: Err(BRC30Error::InvalidTickLen("abcde".to_string())),
    };
    println!("{}", serde_json::to_string_pretty(&action_receipt).unwrap());
    assert_eq!(
      serde_json::to_string_pretty(&action_receipt).unwrap(),
      r##"{
  "inscription_id": "9991111111111111111111111111111111111111111111111111111111111111i1",
  "result": {
    "Err": {
      "InvalidTickLen": "abcde"
    }
  }
}"##
    );
  }
}
