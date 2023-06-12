use crate::{Inscription, Result};
use error::JSONError;
pub mod datastore;
pub mod error;
pub mod num;
mod operation;
pub mod params;
pub mod updater;

pub use self::{
  error::{BRC30Error, Error},
  num::Num,
  operation::{deserialize_brc30, Deploy, Mint, Operation, Stake, Transfer, UnStake},
  updater::{Action, InscriptionData},
};

use crate::okx::datastore::BRC30::{BRC30DbReadAPI, BRC30DbReadWriteAPI};

pub fn deserialize_brc30_operation(
  inscription: Inscription,
  is_transfer: bool,
) -> Result<Operation> {
  let content_body = std::str::from_utf8(inscription.body().ok_or(JSONError::InvalidJson)?)?;
  if content_body.len() < 40 {
    return Err(JSONError::NotBRC30Json.into());
  }

  let content_type = inscription
    .content_type()
    .ok_or(JSONError::InvalidContentType)?;

  if content_type != "text/plain"
    && content_type != "text/plain;charset=utf-8"
    && content_type != "text/plain;charset=UTF-8"
    && content_type != "application/json"
  {
    if !content_type.starts_with("text/plain;") {
      return Err(JSONError::UnSupportContentType.into());
    }
  }
  deserialize_brc30(content_body).map(|op| {
    if is_transfer {
      match op {
        Operation::Transfer(_) => Ok(op),
        _ => Err(JSONError::NotBRC30Json.into()),
      }
    } else {
      Ok(op)
    }
  })?
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::Inscription;
  #[test]
  fn test_ignore_non_transfer_brc20() {
    let content_type = "text/plain;charset=utf-8".as_bytes().to_vec();
    assert_eq!(
      deserialize_brc30_operation(
        Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc-30","op":"deploy","t":"12000","pid":"12000","stake":"12","earn":"12","erate":"12","dmax":"12","total":"12","only":"12","dec":"11"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        false
      )
      .unwrap(),
      Operation::Deploy(Deploy {
        pool_type: "12000".to_string(),
        pool_id: "12000".to_string(),
        stake: "12".to_string(),
        earn: "12".to_string(),
        earn_rate: "12".to_string(),
        distribution_max: "12".to_string(),
        total_supply: "12".to_string(),
        only: "12".to_string(),
        decimals: Some("11".to_string()),
      }),
    );

    assert_eq!(
      deserialize_brc30_operation(
        Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc-30","op":"stake","pid":"pool_id","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        false
      )
      .unwrap(),
      Operation::Stake(Stake {
        pool_id: "pool_id".to_string(),
        amount: "12000".to_string()
      })
    );

    assert_eq!(
      deserialize_brc30_operation(
        Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc-30","op":"mint","tick":"tick","pid":"pool_id","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        false
      )
      .unwrap(),
      Operation::Mint(Mint {
        tick: "tick".to_string(),
        pool_id: "pool_id".to_string(),
        amount: "12000".to_string()
      })
    );

    assert_eq!(
      deserialize_brc30_operation(
        Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc-30","op":"unstake","pid":"pool_id","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        false
      )
      .unwrap(),
      Operation::UnStake(UnStake {
        pool_id: "pool_id".to_string(),
        amount: "12000".to_string()
      })
    );

    assert!(deserialize_brc30_operation(
      Inscription::new(
        Some(content_type.clone()),
        Some(
          r##"{"p":"brc-20","op":"deploy","tick":"abcd","max":"12000","lim":"12","dec":"11"}"##
            .as_bytes()
            .to_vec(),
        ),
      ),
      true
    )
    .is_err());

    assert!(deserialize_brc30_operation(
      Inscription::new(
        Some(content_type.clone()),
        Some(
          r##"{"p":"brc-30","op":"mint","tick":"abcd","amt":"12000"}"##
            .as_bytes()
            .to_vec(),
        ),
      ),
      true
    )
    .is_err());

    assert_eq!(
      deserialize_brc30_operation(
        Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc-30","op":"transfer","pid":"pool_id","tick":"abcd","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        true
      )
      .unwrap(),
      Operation::Transfer(Transfer {
        pool_id: "pool_id".to_string(),
        tick: "abcd".to_string(),
        amount: "12000".to_string()
      })
    );
  }
}
