use super::{types::ScriptPubkey, *};
use crate::okx::datastore::brc20s;
use std::{convert::From, vec};

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{okx::datastore::ScriptKey, txid, InscriptionId, SatPoint};
  use bitcoin::{Address, Network};
  use std::str::FromStr;

  #[test]
  fn serialize_brc20s_receipts() {
    let receipt = Receipt {
      inscription_id: Some(InscriptionId {
        txid: txid(1),
        index: 0xFFFFFFFF,
      }),
      inscription_number: Some(10),
      op: brc20s::OperationType::Deploy.into(),
      old_satpoint: Some(
        SatPoint::from_str(
          "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
        )
        .unwrap(),
      ),
      new_satpoint: Some(
        SatPoint::from_str(
          "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
        )
        .unwrap(),
      ),
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
          &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
            .unwrap()
            .assume_checked()
            .script_pubkey(),
          Network::Bitcoin,
        )
        .into(),
      ),
      valid: true,
      msg: "ok".to_string(),
      events: vec![
        Event::DeployTick(DeployTickEvent {
          tick: Tick {
            id: "aabbccddee".to_string(),
            name: "abcdef".to_string(),
          },
          supply: "1000000".to_string(),
          decimal: 18,
          deployer: ScriptKey::from_script(
            &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
              .unwrap()
              .assume_checked()
              .script_pubkey(),
            Network::Bitcoin,
          )
          .into(),
        }),
        Event::DeployPool(DeployPoolEvent {
          pid: "aabbccddee#1f".to_string(),
          stake: Stake {
            type_field: brc20s::PledgedTick::BRC20STick(
              brc20s::TickId::from_str("aabbccddee").unwrap(),
            )
            .to_type(),
            tick: "aabbccddee".to_string(),
          },
          earn: Earn {
            id: "aabbccddee".to_string(),
            name: "abcdef".to_string(),
          },
          pool: "pool".to_string(),
          erate: "1000000".to_string(),
          only: 0,
          dmax: "10000".to_string(),
          deployer: ScriptKey::from_script(
            &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
              .unwrap()
              .assume_checked()
              .script_pubkey(),
            Network::Bitcoin,
          )
          .into(),
        }),
      ],
    };
    pretty_assert_eq!(
      serde_json::to_string_pretty(&receipt).unwrap(),
      r#"{
  "op": "deploy",
  "inscriptionNumber": 10,
  "inscriptionId": "1111111111111111111111111111111111111111111111111111111111111111i4294967295",
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "newSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "valid": true,
  "msg": "ok",
  "events": [
    {
      "type": "deployTick",
      "tick": {
        "id": "aabbccddee",
        "name": "abcdef"
      },
      "supply": "1000000",
      "decimal": 18,
      "deployer": {
        "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
      }
    },
    {
      "type": "deployPool",
      "pid": "aabbccddee#1f",
      "stake": {
        "type": "BRC20-S",
        "tick": "aabbccddee"
      },
      "earn": {
        "id": "aabbccddee",
        "name": "abcdef"
      },
      "pool": "pool",
      "erate": "1000000",
      "only": 0,
      "dmax": "10000",
      "deployer": {
        "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
      }
    }
  ]
}"#
    )
  }
}
