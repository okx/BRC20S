use crate::okx::protocol::brc20s::params::{PID_BYTE_COUNT, TICK_ID_BYTE_COUNT};
use crate::okx::protocol::brc20s::BRC20SError;
use crate::okx::protocol::brc20s::Num;
use std::str::FromStr;

pub fn validate_pool_str(s: &str) -> Result<(), BRC20SError> {
  if s.len() != PID_BYTE_COUNT {
    return Err(BRC20SError::InvalidPoolId(
      s.to_string(),
      "pool id length is not 13".to_string(),
    ));
  }

  if !s.contains("#") {
    return Err(BRC20SError::InvalidPoolId(
      s.to_string(),
      "pool id must contains '#'".to_string(),
    ));
  }

  let strs = s.split("#").collect::<Vec<&str>>();
  if strs.len() != 2 {
    return Err(BRC20SError::InvalidPoolId(
      s.to_string(),
      "pool id must contains only one '#'".to_string(),
    ));
  }

  let prefix = hex::decode(strs[0]);
  if prefix.is_err() {
    return Err(BRC20SError::InvalidPoolId(
      s.to_string(),
      "the prefix of pool id is not hex".to_string(),
    ));
  }
  let prefix = prefix.unwrap();
  if prefix.len() != TICK_ID_BYTE_COUNT {
    return Err(BRC20SError::InvalidPoolId(
      s.to_string(),
      "the prefix of pool id must contains 10 letter identifier".to_string(),
    ));
  }

  if strs[1].len() != 2 {
    return Err(BRC20SError::InvalidPoolId(
      s.to_string(),
      "the suffix ofpool id must contains 2 letter of pool number".to_string(),
    ));
  }

  let suffix = hex::decode(strs[1]);
  if suffix.is_err() {
    return Err(BRC20SError::InvalidPoolId(
      s.to_string(),
      "the suffix of pool id is not hex".to_string(),
    ));
  }
  Ok(())
}

// validate input amount by user
// it's must be less than max of u64 and positive integer
// eg. 2.000001
pub fn validate_amount(amount: &str) -> Result<(), BRC20SError> {
  let amt = Num::from_str(amount)?;
  if !amt.is_less_than_max_u64() || !amt.is_positive() {
    return Err(BRC20SError::InvalidNum(amount.to_string()));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::super::*;
  use super::*;

  #[test]
  fn test_validate_amount() {
    let amt = "1";
    assert_eq!(Ok(()), validate_amount(amt));

    let amt = "1.0";
    assert_eq!(Ok(()), validate_amount(amt));

    let amt = "1.000000000000000000";
    assert_eq!(Ok(()), validate_amount(amt));

    let amt = "1.0000000000000000000";
    assert_eq!(
      Err(BRC20SError::InvalidNum(amt.to_string())),
      validate_amount(amt)
    );

    let amt = "18446744073709551615";
    assert_eq!(Ok(()), validate_amount(amt));

    let amt = "18446744073709551616";
    assert_eq!(
      Err(BRC20SError::InvalidNum(amt.to_string())),
      validate_amount(amt)
    );

    let amt = "18446744073709551615.0";
    assert_eq!(Ok(()), validate_amount(amt));

    let amt = "0";
    assert_eq!(
      Err(BRC20SError::InvalidNum(amt.to_string())),
      validate_amount(amt)
    );

    let amt = "-1";
    assert_eq!(
      Err(BRC20SError::InvalidNum(amt.to_string())),
      validate_amount(amt)
    );
  }

  #[test]
  fn test_validate_pool_str() {
    assert_eq!(
      validate_pool_str(""),
      Err(BRC20SError::InvalidPoolId(
        "".to_string(),
        "pool id length is not 13".to_string(),
      ))
    );

    assert_eq!(
      validate_pool_str("123"),
      Err(BRC20SError::InvalidPoolId(
        "123".to_string(),
        "pool id length is not 13".to_string(),
      ))
    );

    assert_eq!(
      validate_pool_str("fdsfasfdsfafdfsfadfs"),
      Err(BRC20SError::InvalidPoolId(
        "fdsfasfdsfafdfsfadfs".to_string(),
        "pool id length is not 13".to_string(),
      ))
    );

    assert_eq!(
      validate_pool_str("1234567890001"),
      Err(BRC20SError::InvalidPoolId(
        "1234567890001".to_string(),
        "pool id must contains '#'".to_string(),
      ))
    );

    assert_eq!(
      validate_pool_str("1234#67#89001"),
      Err(BRC20SError::InvalidPoolId(
        "1234#67#89001".to_string(),
        "pool id must contains only one '#'".to_string(),
      ))
    );

    assert_eq!(
      validate_pool_str("1234#67890011"),
      Err(BRC20SError::InvalidPoolId(
        "1234#67890011".to_string(),
        "the prefix of pool id must contains 10 letter identifier".to_string(),
      ))
    );

    assert_eq!(
      validate_pool_str("01234*6789#01"),
      Err(BRC20SError::InvalidPoolId(
        "01234*6789#01".to_string(),
        "the prefix of pool id is not hex".to_string(),
      ))
    );

    assert_eq!(validate_pool_str("1234567890#01"), Ok(()));
  }
}
