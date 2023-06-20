use crate::okx::datastore::BRC30::PledgedTick;
use crate::okx::protocol::BRC30::params::{PID_BYTE_COUNT, TICK_ID_BYTE_COUNT};
use crate::okx::protocol::BRC30::BRC30Error;
use crate::okx::protocol::BRC30::Num;
use bigdecimal::num_bigint::Sign;
use std::str::FromStr;

pub fn validate_hex(s: &str) -> Result<(), BRC30Error> {
  let prefix = hex::decode(s);
  if prefix.is_err() {
    return Err(BRC30Error::InvalidHexStr(s.to_string()));
  }
  Ok(())
}

pub fn validate_pool_str(s: &str) -> Result<(), BRC30Error> {
  if s.len() != PID_BYTE_COUNT {
    return Err(BRC30Error::InvalidPoolId(
      s.to_string(),
      "pool id length is not 13".to_string(),
    ));
  }

  if !s.contains("#") {
    return Err(BRC30Error::InvalidPoolId(
      s.to_string(),
      "pool id must contains '#'".to_string(),
    ));
  }

  let strs = s.split("#").collect::<Vec<&str>>();
  if strs.len() != 2 {
    return Err(BRC30Error::InvalidPoolId(
      s.to_string(),
      "pool id must contains only one '#'".to_string(),
    ));
  }

  let prefix = hex::decode(strs[0]);
  if prefix.is_err() {
    return Err(BRC30Error::InvalidPoolId(
      s.to_string(),
      "the prefix of pool id is not hex".to_string(),
    ));
  }
  let prefix = prefix.unwrap();
  if prefix.len() != TICK_ID_BYTE_COUNT {
    return Err(BRC30Error::InvalidPoolId(
      s.to_string(),
      "the prefix of pool id must contains 10 letter identifier".to_string(),
    ));
  }

  if strs[1].len() != 2 {
    return Err(BRC30Error::InvalidPoolId(
      s.to_string(),
      "the suffix ofpool id must contains 2 letter of pool number".to_string(),
    ));
  }

  let suffix = hex::decode(strs[1]);
  if suffix.is_err() {
    return Err(BRC30Error::InvalidPoolId(
      s.to_string(),
      "the suffix of pool id is not hex".to_string(),
    ));
  }
  Ok(())
}
