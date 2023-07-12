use crate::okx::datastore::brc20s::TickId;
use crate::okx::datastore::ScriptKey;
use crate::okx::protocol::brc20s::params::TICK_ID_BYTE_COUNT;
use bitcoin::hashes::hex::ToHex;
use bitcoin::hashes::{sha256, Hash, HashEngine};
use std::str::FromStr;

pub fn caculate_tick_id(
  tick: &str,
  total_supply: u128,
  decimals: u8,
  from: &ScriptKey,
  to: &ScriptKey,
) -> TickId {
  let mut enc = sha256::Hash::engine();
  enc.input(tick.as_bytes());
  enc.input(total_supply.to_string().as_bytes());
  enc.input(decimals.to_string().as_bytes());
  enc.input(from.to_string().as_bytes());
  enc.input(to.to_string().as_bytes());
  let hash = sha256::Hash::from_engine(enc);
  TickId::from_str(hash[0..TICK_ID_BYTE_COUNT].to_hex().as_str()).unwrap()
}

#[cfg(test)]
mod tests {
  use super::*;
  use bitcoin::Address;
  use std::str::FromStr;

  #[test]
  fn test_serialize() {
    let addr1 = Address::from_str("bcrt1qvd26a8c26d4mu5fzyh74pvcp9ykgutxt9fktqf").unwrap();

    let addr2 = Address::from_str("bcrt1qvd26a8c26d4mu5fzyh74pvcp9ykgutxt9fktqf").unwrap();
    let tick_id = caculate_tick_id(
      "ordi",
      40000000,
      2,
      &ScriptKey::from_address(addr1),
      &ScriptKey::from_address(addr2),
    );
    println!("tick_id:{}", tick_id.hex());

    let addr1 =
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();

    let addr2 =
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();
    let tick_id = caculate_tick_id(
      "ordi",
      10,
      18,
      &ScriptKey::from_address(addr1),
      &ScriptKey::from_address(addr2),
    );
    println!("tick_id:{}", tick_id.hex());

    let addr1 =
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();

    let addr2 =
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();
    let tick_id = caculate_tick_id(
      "ordi",
      100,
      1,
      &ScriptKey::from_address(addr1),
      &ScriptKey::from_address(addr2),
    );
    println!("tick_id:{}", tick_id.hex());
  }

  //TODO test
}
