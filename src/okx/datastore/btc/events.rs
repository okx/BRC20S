use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Receipt {
  // pub op: OperationType,
  // pub from: ScriptKey,
  // pub to: ScriptKey,
  pub result: Result<Event, BTCError>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Event {
  Transfer(TransferEvent),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TransferEvent {
  pub amt: u128,
  pub msg: Option<String>,
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//   use bitcoin::Address;
//   use std::str::FromStr;
//
//   #[test]
//   fn action_receipt_serialize() {
//     let addr =
//       Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();
//     let action_receipt = Receipt {
//       inscription_id: InscriptionId::from_str(
//         "9991111111111111111111111111111111111111111111111111111111111111i1",
//       )
//       .unwrap(),
//       inscription_number: 0,
//       old_satpoint: SatPoint {
//         outpoint: Default::default(),
//         offset: 0,
//       },
//       new_satpoint: SatPoint {
//         outpoint: Default::default(),
//         offset: 0,
//       },
//       op: OperationType::Deploy,
//       from: ScriptKey::Address(addr.clone()),
//       to: ScriptKey::Address(addr),
//       result: Err(BRC20SError::InvalidTickLen("abcde".to_string())),
//     };
//     assert_eq!(
//       serde_json::to_string_pretty(&action_receipt).unwrap(),
//       r##"{
//   "inscription_id": "9991111111111111111111111111111111111111111111111111111111111111i1",
//   "inscription_number": 0,
//   "old_satpoint": "0000000000000000000000000000000000000000000000000000000000000000:4294967295:0",
//   "new_satpoint": "0000000000000000000000000000000000000000000000000000000000000000:4294967295:0",
//   "op": "Deploy",
//   "from": {
//     "Address": "bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e"
//   },
//   "to": {
//     "Address": "bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e"
//   },
//   "result": {
//     "Err": {
//       "InvalidTickLen": "abcde"
//     }
//   }
// }"##
//     );
//   }
// }
