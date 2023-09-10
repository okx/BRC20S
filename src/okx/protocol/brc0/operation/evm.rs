use serde::{Deserialize, Serialize};
use web3::types::{H256, U256, U64, Bytes, Address};
use rlp::RlpStream;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Evm {
  #[serde(rename = "d")]
  pub d: Transaction,
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

impl Transaction {
  pub fn encode_rlp(&self) -> Vec<u8> {
    let mut stream = RlpStream::new();
    stream.begin_list(9);

    stream.append(&self.nonce);
    stream.append(&self.gas_price);
    stream.append(&self.gas);
    if let Some(to) = self.to {
      stream.append(&to);
    } else {
      stream.append(&"");
    }
    stream.append(&self.value);
    stream.append(&self.input.0);

    stream.append(&self.v);
    stream.append(&self.r);
    stream.append(&self.s);

    stream.out().to_vec()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_json_lrp() {
    let tx_str = r#"{
    "gas":"0x2faf080",
    "gasPrice":"0x2540be400",
    "input":"0x6057361d0000000000000000000000000000000000000000000000000000000000000146",
    "nonce":"0x3",
    "to":"0x45dd91b0289e60d89cec94df0aac3a2f539c514a",
    "value":"0x0",
    "v":"0xa9",
    "r":"0xfc44c59fa57d225beec4a7f05e0d36f1f3d8fd11364e3f271b2ec9c27e7c1f0e",
    "s":"0x2db439a54fe12abefc235ad576da18d8d8ce37975ac2bd2659c5ebb3fa0912d6"
}"#;

    let tx: Transaction = serde_json::from_str(tx_str).unwrap();

    let rlp_tx = tx.encode_rlp();
    println!("{}", hex::encode(&rlp_tx));

    let tx_str = r#"{
    "gas":"0x2faf080",
    "gasPrice":"0x2540be400",
    "input":"0x608060405234801561001057600080fd5b50610150806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80632e64cec11461003b5780636057361d14610059575b600080fd5b610043610075565b60405161005091906100a1565b60405180910390f35b610073600480360381019061006e91906100ed565b61007e565b005b60008054905090565b8060008190555050565b6000819050919050565b61009b81610088565b82525050565b60006020820190506100b66000830184610092565b92915050565b600080fd5b6100ca81610088565b81146100d557600080fd5b50565b6000813590506100e7816100c1565b92915050565b600060208284031215610103576101026100bc565b5b6000610111848285016100d8565b9150509291505056fea2646970667358221220322c78243e61b783558509c9cc22cb8493dde6925aa5e89a08cdf6e22f279ef164736f6c63430008120033",
    "nonce":"0x2",
    "to":null,
    "value":"0x0",
    "v":"0xaa",
    "r":"0xa480fef9650e270a717db2a74fa02b16f8f72497e58b1377049334b57b9caf6b",
    "s":"0x16909fbbd11886eb66907fae69ea1c919138298ee85a0ac023475d56312fb8c2"
}"#;

    let tx: Transaction = serde_json::from_str(tx_str).unwrap();

    let rlp_tx = tx.encode_rlp();
    println!("{}", hex::encode(&rlp_tx));
  }
}