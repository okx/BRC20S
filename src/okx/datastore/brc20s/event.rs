use crate::okx::datastore::brc20s::{Pid, PledgedTick, PoolType, Tick, TickId};
use crate::okx::datastore::ScriptKey;
use crate::okx::protocol::brc20s::BRC20SError;
use crate::{InscriptionId, SatPoint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum OperationType {
  Deploy,
  Mint,
  Stake,
  UnStake,
  PassiveUnStake,
  InscribeTransfer,
  Transfer,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Receipt {
  pub inscription_id: InscriptionId,
  pub inscription_number: i64,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub op: OperationType,
  pub from: ScriptKey,
  pub to: ScriptKey,
  pub result: Result<Vec<Event>, BRC20SError>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Event {
  DeployTick(DeployTickEvent),
  DeployPool(DeployPoolEvent),
  Deposit(DepositEvent),
  Withdraw(WithdrawEvent),
  PassiveWithdraw(PassiveWithdrawEvent),
  Mint(MintEvent),
  InscribeTransfer(InscribeTransferEvent),
  Transfer(TransferEvent),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeployTickEvent {
  pub tick_id: TickId,
  pub name: Tick,
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
  pub only: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DepositEvent {
  pub pid: Pid,
  pub amt: u128,
  pub period_settlement_reward: u128,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct WithdrawEvent {
  pub pid: Pid,
  pub amt: u128,
  pub period_settlement_reward: u128,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PassiveWithdrawEvent {
  pub pid: Pid,
  pub amt: u128,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MintEvent {
  pub pid: Pid,
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
  pub msg: Option<String>,
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
    let action_receipt = Receipt {
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
      op: OperationType::Deploy,
      from: ScriptKey::Address(addr.clone()),
      to: ScriptKey::Address(addr),
      result: Err(BRC20SError::InvalidTickLen("abcde".to_string())),
    };
    assert_eq!(
      serde_json::to_string_pretty(&action_receipt).unwrap(),
      r##"{
  "inscription_id": "9991111111111111111111111111111111111111111111111111111111111111i1",
  "inscription_number": 0,
  "old_satpoint": "0000000000000000000000000000000000000000000000000000000000000000:4294967295:0",
  "new_satpoint": "0000000000000000000000000000000000000000000000000000000000000000:4294967295:0",
  "op": "Deploy",
  "from": {
    "Address": "bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e"
  },
  "to": {
    "Address": "bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e"
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
