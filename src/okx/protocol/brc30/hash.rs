use crate::okx::datastore::brc30::TickId;
use crate::okx::datastore::ScriptKey;
use crate::okx::protocol::brc30::params::TICK_ID_BYTE_COUNT;
use bitcoin::hashes::hex::ToHex;
use bitcoin::hashes::{sha256, Hash, HashEngine};

pub fn caculate_tick_id(
  total_supply: u128,
  decimals: u8,
  from: &ScriptKey,
  to: &ScriptKey,
) -> TickId {
  let mut enc = sha256::Hash::engine();
  enc.input(total_supply.to_string().as_bytes());
  enc.input(decimals.to_string().as_bytes());
  enc.input(from.to_string().as_bytes());
  enc.input(to.to_string().as_bytes());
  let hash = sha256::Hash::from_engine(enc).to_vec();
  println!("hash:{}", hash.to_hex());
  TickId::from_bytes(&hash[0..TICK_ID_BYTE_COUNT]).unwrap()
}

#[cfg(test)]
mod tests {
  use super::super::*;
  use super::*;
  use bitcoin::util::address::Payload;
  use bitcoin::Address;
  use bitcoin::Network::Bitcoin;

  #[test]
  fn test_serialize() {
    let addr1 =
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();

    let addr2 =
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();
    let tick_id = caculate_tick_id(
      100,
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
      100,
      1,
      &ScriptKey::from_address(addr1),
      &ScriptKey::from_address(addr2),
    );
    println!("tick_id:{}", tick_id.hex());
  }

  //TODO test
}
