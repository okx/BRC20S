use super::{types::ScriptPubkey, *};
use crate::okx::{datastore::brc20 as brc20_store, protocol::brc20};

#[derive(Debug, thiserror::Error)]
pub enum BRC20Error {
  #[error("tick must be 4 bytes length")]
  IncorrectTickFormat,
  #[error("tick not found")]
  TickNotFound,
  #[error("balance not found")]
  BalanceNotFound,
  #[error("operation not found")]
  OperationNotFound,
  #[error("events not found")]
  EventsNotFound,
  #[error("block not found")]
  BlockNotFound,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxInscriptionInfo {
  pub txid: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub blockhash: Option<String>,
  pub confirmed: bool,
  pub inscriptions: Vec<InscriptionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionType {
  Transfer,
  Inscribe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscriptionInfo {
  pub action: ActionType,
  // if the transaction not committed to the blockchain, the following fields are None
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inscription_number: Option<i64>,
  pub inscription_id: String,
  pub from: ScriptPubkey,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub to: Option<ScriptPubkey>,
  pub old_satpoint: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  // if transfer to coinbase new_satpoint is None
  pub new_satpoint: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub operation: Option<RawOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum RawOperation {
  Brc20Operation(Brc20RawOperation),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Brc20RawOperation {
  Deploy(Deploy),
  Mint(Mint),
  InscribeTransfer(Transfer),
  Transfer(Transfer),
}

// action to raw operation
impl From<brc20::Operation> for Brc20RawOperation {
  fn from(op: brc20::Operation) -> Self {
    match op {
      brc20::Operation::Deploy(deploy) => Brc20RawOperation::Deploy(deploy.into()),
      brc20::Operation::Mint(mint) => Brc20RawOperation::Mint(mint.into()),
      brc20::Operation::InscribeTransfer(transfer) => {
        Brc20RawOperation::InscribeTransfer(transfer.into())
      }
      brc20::Operation::Transfer(transfer) => Brc20RawOperation::Transfer(transfer.into()),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deploy {
  pub tick: String,
  pub max: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lim: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dec: Option<String>,
}

impl From<brc20::Deploy> for Deploy {
  fn from(deploy: brc20::Deploy) -> Self {
    Deploy {
      tick: deploy.tick,
      max: deploy.max_supply,
      lim: deploy.mint_limit,
      dec: deploy.decimals,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mint {
  pub tick: String,
  pub amt: String,
}

impl From<brc20::Mint> for Mint {
  fn from(mint: brc20::Mint) -> Self {
    Mint {
      tick: mint.tick,
      amt: mint.amount,
    }
  }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
  pub tick: String,
  pub amt: String,
}

impl From<brc20::Transfer> for Transfer {
  fn from(transfer: brc20::Transfer) -> Self {
    Transfer {
      tick: transfer.tick,
      amt: transfer.amount,
    }
  }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickInfo {
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub supply: String,
  pub limit_per_mint: String,
  pub minted: String,
  pub decimal: u64,
  pub deploy_by: ScriptPubkey,
  pub txid: String,
  pub deploy_height: u64,
  pub deploy_blocktime: u64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllTickInfo {
  pub tokens: Vec<TickInfo>,
}

impl From<&brc20_store::TokenInfo> for TickInfo {
  fn from(tick_info: &brc20_store::TokenInfo) -> Self {
    Self {
      tick: tick_info.tick.to_string(),
      inscription_id: tick_info.inscription_id.to_string(),
      inscription_number: tick_info.inscription_number,
      supply: tick_info.supply.to_string(),
      limit_per_mint: tick_info.limit_per_mint.to_string(),
      minted: tick_info.minted.to_string(),
      decimal: tick_info.decimal as u64,
      deploy_by: tick_info.deploy_by.clone().into(),
      txid: tick_info.inscription_id.txid.to_string(),
      deploy_height: tick_info.deployed_number,
      deploy_blocktime: tick_info.deployed_timestamp as u64,
    }
  }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
  pub tick: String,
  pub available_balance: String,
  pub transferable_balance: String,
  pub overall_balance: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllBalance {
  pub balance: Vec<Balance>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxEvents {
  pub events: Vec<TxEvent>,
  pub txid: String,
}

impl From<&brc20_store::Receipt> for TxEvent {
  fn from(event: &brc20_store::Receipt) -> Self {
    match &event.result {
      Ok(result) => match result {
        brc20_store::Event::Deploy(deploy_event) => Self::Deploy(DeployEvent {
          tick: deploy_event.tick.to_string(),
          inscription_id: event.inscription_id.to_string(),
          inscription_number: event.inscription_number,
          old_satpoint: event.old_satpoint,
          new_satpoint: event.new_satpoint,
          supply: deploy_event.supply.to_string(),
          limit_per_mint: deploy_event.limit_per_mint.to_string(),
          decimal: deploy_event.decimal as u64,
          from: event.from.clone().into(),
          to: event.to.clone().into(),
          valid: true,
          msg: "ok".to_string(),
          event: String::from("deploy"),
        }),
        brc20_store::Event::Mint(mint_event) => Self::Mint(MintEvent {
          tick: mint_event.tick.to_string(),
          inscription_id: event.inscription_id.to_string(),
          inscription_number: event.inscription_number,
          old_satpoint: event.old_satpoint,
          new_satpoint: event.new_satpoint,
          amount: mint_event.amount.to_string(),
          from: event.from.clone().into(),
          to: event.to.clone().into(),
          valid: true,
          msg: mint_event.msg.clone().unwrap_or("ok".to_string()),
          event: String::from("mint"),
        }),
        brc20_store::Event::InscribeTransfer(trans1) => {
          Self::InscribeTransfer(InscribeTransferEvent {
            tick: trans1.tick.to_string(),
            inscription_id: event.inscription_id.to_string(),
            inscription_number: event.inscription_number,
            old_satpoint: event.old_satpoint,
            new_satpoint: event.new_satpoint,
            amount: trans1.amount.to_string(),
            from: event.from.clone().into(),
            to: event.to.clone().into(),
            valid: true,
            msg: "ok".to_string(),
            event: String::from("inscribeTransfer"),
          })
        }
        brc20_store::Event::Transfer(trans2) => Self::Transfer(TransferEvent {
          tick: trans2.tick.to_string(),
          inscription_id: event.inscription_id.to_string(),
          inscription_number: event.inscription_number,
          old_satpoint: event.old_satpoint,
          new_satpoint: event.new_satpoint,
          amount: trans2.amount.to_string(),
          from: event.from.clone().into(),
          to: event.to.clone().into(),
          valid: true,
          msg: trans2.msg.clone().unwrap_or("ok".to_string()),
          event: String::from("transfer"),
        }),
      },
      Err(err) => Self::Error(ErrorEvent {
        inscription_id: event.inscription_id.to_string(),
        inscription_number: event.inscription_number,
        old_satpoint: event.old_satpoint,
        new_satpoint: event.new_satpoint,
        valid: false,
        from: event.from.clone().into(),
        to: event.to.clone().into(),
        msg: err.to_string(),
        event: match event.op {
          brc20_store::OperationType::Deploy => "deploy",
          brc20_store::OperationType::Mint => "mint",
          brc20_store::OperationType::InscribeTransfer => "inscribeTransfer",
          brc20_store::OperationType::Transfer => "transfer",
        }
        .to_string(),
      }),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum TxEvent {
  Deploy(DeployEvent),
  Mint(MintEvent),
  InscribeTransfer(InscribeTransferEvent),
  Transfer(TransferEvent),
  Error(ErrorEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub supply: String,
  pub limit_per_mint: String,
  pub decimal: u64,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub amount: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscribeTransferEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub amount: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub amount: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockEvents {
  pub block: Vec<TxEvents>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferableInscriptions {
  pub inscriptions: Vec<TransferableInscription>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferableInscription {
  pub inscription_id: String,
  pub inscription_number: i64,
  pub amount: String,
  pub tick: String,
  pub owner: String,
}

impl From<&brc20_store::TransferableLog> for TransferableInscription {
  fn from(trans: &brc20_store::TransferableLog) -> Self {
    Self {
      inscription_id: trans.inscription_id.to_string(),
      inscription_number: trans.inscription_number,
      amount: trans.amount.to_string(),
      tick: trans.tick.as_str().to_string(),
      owner: trans.owner.to_string(),
    }
  }
}
#[cfg(test)]
mod tests {
  use crate::okx::datastore::ScriptKey;

  use super::*;
  #[test]
  fn serialize_script_pubkey() {
    let script_pubkey: ScriptPubkey = ScriptKey::from_script(
      &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
        .unwrap()
        .assume_checked()
        .script_pubkey(),
      Network::Bitcoin,
    )
    .into();
    assert_eq!(
      serde_json::to_string(&script_pubkey).unwrap(),
      r#"{"address":"bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"}"#
    );
    let script_pubkey: ScriptPubkey = ScriptKey::from_script(
      &Script::from_bytes(
        hex::decode(
          "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
        )
        .unwrap()
        .as_slice(),
      ),
      Network::Bitcoin,
    )
    .into();

    assert_eq!(
      serde_json::to_string(&script_pubkey).unwrap(),
      r#"{"nonStandard":"df65c8a338dce7900824e7bd18c336656ca19e57"}"#
    );
  }

  #[test]
  fn serialize_deploy() {
    let deploy = Deploy {
      tick: "ordi".to_string(),
      max: "1000".to_string(),
      lim: Some("1000".to_string()),
      dec: Some("18".to_string()),
    };
    assert_eq!(
      serde_json::to_string(&deploy).unwrap(),
      r#"{"tick":"ordi","max":"1000","lim":"1000","dec":"18"}"#
    );
    let deploy = Deploy {
      tick: "ordi".to_string(),
      max: "1000".to_string(),
      lim: None,
      dec: None,
    };
    assert_eq!(
      serde_json::to_string(&deploy).unwrap(),
      r#"{"tick":"ordi","max":"1000"}"#
    );
  }

  #[test]
  fn serialize_mint() {
    let mint = Mint {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    };
    assert_eq!(
      serde_json::to_string(&mint).unwrap(),
      r#"{"tick":"ordi","amt":"1000"}"#
    );
  }

  #[test]
  fn serialize_transfer() {
    let transfer = Transfer {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    };
    assert_eq!(
      serde_json::to_string(&transfer).unwrap(),
      r#"{"tick":"ordi","amt":"1000"}"#
    );
  }

  #[test]
  fn serialize_raw_operation() {
    let deploy = Brc20RawOperation::Deploy(Deploy {
      tick: "ordi".to_string(),
      max: "1000".to_string(),
      lim: Some("1000".to_string()),
      dec: Some("18".to_string()),
    });
    assert_eq!(
      serde_json::to_string(&deploy).unwrap(),
      r#"{"type":"deploy","tick":"ordi","max":"1000","lim":"1000","dec":"18"}"#
    );
    let mint = Brc20RawOperation::Mint(Mint {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    });
    assert_eq!(
      serde_json::to_string(&mint).unwrap(),
      r#"{"type":"mint","tick":"ordi","amt":"1000"}"#
    );
    let inscribe_transfer = Brc20RawOperation::InscribeTransfer(Transfer {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    });
    assert_eq!(
      serde_json::to_string(&inscribe_transfer).unwrap(),
      r#"{"type":"inscribeTransfer","tick":"ordi","amt":"1000"}"#
    );
    let transfer = Brc20RawOperation::Transfer(Transfer {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    });
    assert_eq!(
      serde_json::to_string(&transfer).unwrap(),
      r#"{"type":"transfer","tick":"ordi","amt":"1000"}"#
    );
  }

  #[test]
  fn serialize_inscription_info() {
    let info = InscriptionInfo {
      action: ActionType::Inscribe,
      inscription_number: None,
      inscription_id: InscriptionId::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
      )
      .unwrap()
      .to_string(),
      from: ScriptKey::from_script(
        &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
          .unwrap()
          .assume_checked()
          .script_pubkey(),
        Network::Bitcoin,
      )
      .into(),
      to: Some(
        ScriptKey::from_script(
          &Script::from_bytes(
            hex::decode(
              "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
            )
            .unwrap()
            .as_slice(),
          ),
          Network::Bitcoin,
        )
        .into(),
      ),
      old_satpoint: SatPoint::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
      )
      .unwrap()
      .to_string(),
      new_satpoint: None,
      operation: None,
    };
    assert_eq!(
      serde_json::to_string_pretty(&info).unwrap(),
      r#"{
  "action": "inscribe",
  "inscriptionId": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "nonStandard": "df65c8a338dce7900824e7bd18c336656ca19e57"
  },
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000"
}"#
    );
    let info = InscriptionInfo {
      action: ActionType::Inscribe,
      inscription_number: Some(1),
      inscription_id: InscriptionId::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
      )
      .unwrap()
      .to_string(),
      from: ScriptKey::from_script(
        &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
          .unwrap()
          .assume_checked()
          .script_pubkey(),
        Network::Bitcoin,
      )
      .into(),
      to: Some(
        ScriptKey::from_script(
          &Script::from_bytes(
            hex::decode(
              "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
            )
            .unwrap()
            .as_slice(),
          ),
          Network::Bitcoin,
        )
        .into(),
      ),
      old_satpoint: SatPoint::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
      )
      .unwrap()
      .to_string(),
      new_satpoint: Some(
        SatPoint::from_str(
          "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
        )
        .unwrap()
        .to_string(),
      ),
      operation: None,
    };
    assert_eq!(
      serde_json::to_string_pretty(&info).unwrap(),
      r#"{
  "action": "inscribe",
  "inscriptionNumber": 1,
  "inscriptionId": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "nonStandard": "df65c8a338dce7900824e7bd18c336656ca19e57"
  },
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "newSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000"
}"#
    );
    let info = InscriptionInfo {
      action: ActionType::Inscribe,
      inscription_number: Some(1),
      inscription_id: InscriptionId::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
      )
      .unwrap()
      .to_string(),
      from: ScriptKey::from_script(
        &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
          .unwrap()
          .assume_checked()
          .script_pubkey(),
        Network::Bitcoin,
      )
      .into(),
      to: Some(
        ScriptKey::from_script(
          &Script::from_bytes(
            hex::decode(
              "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
            )
            .unwrap()
            .as_slice(),
          ),
          Network::Bitcoin,
        )
        .into(),
      ),
      old_satpoint: SatPoint::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
      )
      .unwrap()
      .to_string(),
      new_satpoint: Some(
        SatPoint::from_str(
          "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
        )
        .unwrap()
        .to_string(),
      ),
      operation: Some(RawOperation::Brc20Operation(Brc20RawOperation::Deploy(
        Deploy {
          tick: "ordi".to_string(),
          max: "1000".to_string(),
          lim: Some("1000".to_string()),
          dec: Some("18".to_string()),
        },
      ))),
    };
    assert_eq!(
      serde_json::to_string_pretty(&info).unwrap(),
      r#"{
  "action": "inscribe",
  "inscriptionNumber": 1,
  "inscriptionId": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "nonStandard": "df65c8a338dce7900824e7bd18c336656ca19e57"
  },
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "newSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "operation": {
    "type": "deploy",
    "tick": "ordi",
    "max": "1000",
    "lim": "1000",
    "dec": "18"
  }
}"#
    );
  }
}
