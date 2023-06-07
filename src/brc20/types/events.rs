use super::{super::error::*, *};
use crate::{InscriptionId, SatPoint};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ActionReceipt {
  pub inscription_id: InscriptionId,
  pub inscription_number: i64,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
  pub op: EventType,
  pub from: ScriptKey,
  pub to: ScriptKey,
  pub result: Result<BRC20Event, BRC20Error>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum BRC20Event {
  Deploy(DeployEvent),
  Mint(MintEvent),
  TransferPhase1(TransferPhase1Event),
  TransferPhase2(TransferPhase2Event),
}

#[derive(Debug, PartialEq)]
pub enum EventType {
  Deploy,
  Mint,
  TransferPhase1,
  TransferPhase2,
}

impl Serialize for EventType {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      Self::Deploy => "deploy".serialize(serializer),
      Self::Mint => "mint".serialize(serializer),
      Self::TransferPhase1 => "inscribeTransfer".serialize(serializer),
      Self::TransferPhase2 => "transfer".serialize(serializer),
    }
  }
}

impl<'de> Deserialize<'de> for EventType {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    match String::deserialize(deserializer)?.as_str() {
      "deploy" => Ok(Self::Deploy),
      "mint" => Ok(Self::Mint),
      "inscribeTransfer" => Ok(Self::TransferPhase1),
      "transfer" => Ok(Self::TransferPhase2),
      _ => Err("no such event type"),
    }
    .map_err(|e| de::Error::custom(format!("deserialize event type error: {}", e)))
  }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DeployEvent {
  pub supply: u128,
  pub limit_per_mint: u128,
  pub decimal: u8,
  pub tick: Tick,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct MintEvent {
  pub tick: Tick,
  pub amount: u128,
  pub msg: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TransferPhase1Event {
  pub tick: Tick,
  pub amount: u128,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TransferPhase2Event {
  pub tick: Tick,
  pub amount: u128,
  pub msg: Option<String>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use bitcoin::Address;
  use std::str::FromStr;

  #[test]
  fn action_receipt_serialize() {
    let action_receipt = ActionReceipt {
      inscription_id: InscriptionId::from_str(
        "9991111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      inscription_number: 1,
      old_satpoint: SatPoint::from_str(
        "1111111111111111111111111111111111111111111111111111111111111111:1:1",
      )
      .unwrap(),
      new_satpoint: Some(
        SatPoint::from_str("2111111111111111111111111111111111111111111111111111111111111111:1:1")
          .unwrap(),
      ),
      op: EventType::Deploy,
      from: ScriptKey::from_address(
        Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
      ),
      to: ScriptKey::from_address(
        Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
      ),
      result: Err(BRC20Error::InvalidTickLen("abcde".to_string())),
    };
    println!("{}", serde_json::to_string_pretty(&action_receipt).unwrap());
    assert_eq!(
      serde_json::to_string_pretty(&action_receipt).unwrap(),
      r##"{
  "inscription_id": "9991111111111111111111111111111111111111111111111111111111111111i1",
  "inscription_number": 1,
  "old_satpoint": "1111111111111111111111111111111111111111111111111111111111111111:1:1",
  "new_satpoint": "2111111111111111111111111111111111111111111111111111111111111111:1:1",
  "op": "deploy",
  "from": {
    "Address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "Address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "result": {
    "Err": {
      "InvalidTickLen": "abcde"
    }
  }
}"##
    );
  }
}
