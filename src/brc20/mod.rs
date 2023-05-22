use crate::{Inscription, Result};
use error::JSONError;
mod custom_serde;
mod error;
pub mod ledger;
mod num;
mod operation;
mod params;
mod types;
pub mod updater;

pub use self::{
  error::{BRC20Error, Error},
  num::Num,
  operation::{deserialize_brc20, Deploy, Mint, Operation, Transfer},
  types::*,
  updater::{Action, InscriptionData},
};

use ledger::{LedgerRead, LedgerReadWrite};

pub fn deserialize_brc20_operation(
  inscription: Inscription,
  is_transfer: bool,
) -> Result<Operation> {
  let content_body = std::str::from_utf8(inscription.body().ok_or(JSONError::InvalidJson)?)?;
  if content_body.len() < 40 {
    return Err(JSONError::NotBRC20Json.into());
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
  deserialize_brc20(content_body).map(|op| {
    if is_transfer {
      match op {
        Operation::Transfer(_) => Ok(op),
        _ => Err(JSONError::NotBRC20Json.into()),
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
      deserialize_brc20_operation(
        Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc-20","op":"deploy","tick":"abcd","max":"12000","lim":"12","dec":"11"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        false
      )
      .unwrap(),
      Operation::Deploy(Deploy {
        tick: "abcd".to_string(),
        max_supply: "12000".to_string(),
        mint_limit: Some("12".to_string()),
        decimals: Some("11".to_string()),
      }),
    );

    assert_eq!(
      deserialize_brc20_operation(
        Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc-20","op":"mint","tick":"abcd","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        false
      )
      .unwrap(),
      Operation::Mint(Mint {
        tick: "abcd".to_string(),
        amount: "12000".to_string()
      })
    );

    assert_eq!(
      deserialize_brc20_operation(
        Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc-20","op":"transfer","tick":"abcd","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        false
      )
      .unwrap(),
      Operation::Transfer(Transfer {
        tick: "abcd".to_string(),
        amount: "12000".to_string()
      })
    );

    assert!(deserialize_brc20_operation(
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

    assert!(deserialize_brc20_operation(
      Inscription::new(
        Some(content_type.clone()),
        Some(
          r##"{"p":"brc-20","op":"mint","tick":"abcd","amt":"12000"}"##
            .as_bytes()
            .to_vec(),
        ),
      ),
      true
    )
    .is_err());

    assert_eq!(
      deserialize_brc20_operation(
        Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc-20","op":"transfer","tick":"abcd","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        true
      )
      .unwrap(),
      Operation::Transfer(Transfer {
        tick: "abcd".to_string(),
        amount: "12000".to_string()
      })
    );
  }
}
