use crate::okx::datastore::brc30::{BRC30Tick, Pid, PledgedTick, PoolType, TickId};
use crate::okx::datastore::ScriptKey;
use crate::okx::protocol::brc30::BRC30Error;
use crate::{InscriptionId, SatPoint};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum BRC30OperationType {
  Deploy,
  Mint,
  Stake,
  UnStake,
  PassiveUnStake,
  InscribeTransfer,
  Transfer,
}

impl BRC30OperationType {
  pub fn to_string(&self) -> String {
    match self {
      BRC30OperationType::Deploy => "deploy".to_string(),
      BRC30OperationType::Mint => "mint".to_string(),
      BRC30OperationType::Stake => "stake".to_string(),
      BRC30OperationType::UnStake => "unStake".to_string(),
      BRC30OperationType::PassiveUnStake => "passiveUnStake".to_string(),
      BRC30OperationType::InscribeTransfer => "inscribeTransfer".to_string(),
      BRC30OperationType::Transfer => "transfer".to_string(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BRC30Receipt {
  pub inscription_id: InscriptionId,
  pub inscription_number: i64,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub op: BRC30OperationType,
  pub from: ScriptKey,
  pub to: ScriptKey,
  pub result: Result<BRC30Event, BRC30Error>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum BRC30Event {
  DeployTick(DeployTickEvent),
  DeployPool(DeployPoolEvent),
  Deposit(DepositEvent),
  Withdraw(WithdrawEvent),
  PassiveWithdraw(PassiveWithdrawEvent),
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
  PassiveWithdraw,
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
      Self::PassiveWithdraw => "passive_withdraw".serialize(serializer),
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
      "passive_withdraw" => Ok(Self::PassiveWithdraw),
      "mint" => Ok(Self::Mint),
      "inscribeTransfer" => Ok(Self::InscribeTransfer),
      "transfer" => Ok(Self::Transfer),
      _ => Err("no such event type"),
    }
    .map_err(|e| de::Error::custom(format!("deserialize event type error: {}", e)))
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeployTickEvent {
  pub tick_id: TickId,
  pub name: BRC30Tick,
  pub supply: u128,
  pub decimal: u8,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeployPoolEvent {
  pub pid: Pid,
  pub ptype: PoolType,
  pub stake: PledgedTick,
  pub erate: u128,
  pub dmax: u128,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DepositEvent {
  pub(crate) pid: Pid,
  pub(crate) amt: u128,
  pub(crate) reward: u128,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct WithdrawEvent {
  pub(crate) pid: Pid,
  pub(crate) amt: u128,
  pub(crate) initiative: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PassiveWithdrawEvent {
  pub(crate) pid: Vec<(Pid, u128)>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MintEvent {
  pub tick_id: TickId,
  pub amt: u128,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct InscribeTransferEvent {
  pub tick_id: TickId,
  pub amt: u128,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TransferEvent {
  pub tick_id: TickId,
  pub amt: u128,
}

#[cfg(test)]
mod tests {
  use super::*;
  use bitcoin::Address;
  use std::str::FromStr;

  #[test]
  fn action_receipt_serialize() {
    let addr =
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();
    let action_receipt = BRC30Receipt {
      inscription_id: InscriptionId::from_str(
        "9991111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      inscription_number: 0,
      old_satpoint: SatPoint {
        outpoint: Default::default(),
        offset: 0,
      },
      new_satpoint: SatPoint {
        outpoint: Default::default(),
        offset: 0,
      },
      op: BRC30OperationType::Deploy,
      from: ScriptKey::Address(addr.clone()),
      to: ScriptKey::Address(addr.clone()),
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
