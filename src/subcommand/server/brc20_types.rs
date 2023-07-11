use super::{types::ScriptPubkey, *};
use crate::okx::protocol::brc20::{BRC20Deploy, BRC20Mint, BRC20Transfer, Operation};

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
impl From<Operation> for Brc20RawOperation {
  fn from(op: Operation) -> Self {
    match op {
      Operation::Deploy(deploy) => Brc20RawOperation::Deploy(deploy.into()),
      Operation::Mint(mint) => Brc20RawOperation::Mint(mint.into()),
      Operation::InscribeTransfer(transfer) => Brc20RawOperation::InscribeTransfer(transfer.into()),
      Operation::Transfer(transfer) => Brc20RawOperation::Transfer(transfer.into()),
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

impl From<BRC20Deploy> for Deploy {
  fn from(deploy: BRC20Deploy) -> Self {
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

impl From<BRC20Mint> for Mint {
  fn from(mint: BRC20Mint) -> Self {
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

impl From<BRC20Transfer> for Transfer {
  fn from(transfer: BRC20Transfer) -> Self {
    Transfer {
      tick: transfer.tick,
      amt: transfer.amount,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::okx::datastore::ScriptKey;
  use bitcoin::hashes::hex::FromHex;

  use super::*;
  #[test]
  fn serialize_script_pubkey() {
    let script_pubkey: ScriptPubkey = ScriptKey::from_script(
      &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
        .unwrap()
        .script_pubkey(),
      Network::Bitcoin,
    )
    .into();
    assert_eq!(
      serde_json::to_string(&script_pubkey).unwrap(),
      r#"{"address":"bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"}"#
    );
    let script_pubkey: ScriptPubkey = ScriptKey::from_script(
      &Script::from_hex(
        "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
      )
      .unwrap(),
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
          .script_pubkey(),
        Network::Bitcoin,
      )
      .into(),
      to: Some(
        ScriptKey::from_script(
          &Script::from_hex(
            "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
          )
          .unwrap(),
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
          .script_pubkey(),
        Network::Bitcoin,
      )
      .into(),
      to: Some(
        ScriptKey::from_script(
          &Script::from_hex(
            "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
          )
          .unwrap(),
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
          .script_pubkey(),
        Network::Bitcoin,
      )
      .into(),
      to: Some(
        ScriptKey::from_script(
          &Script::from_hex(
            "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
          )
          .unwrap(),
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
