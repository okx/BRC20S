use super::Tick;
use crate::brc20::custom_serde::U32StringSerde;
use crate::brc20::error::BRC20Error;
use crate::brc20::ledger::{BRC20Event, DeployEvent};
use crate::brc20::num::Num;
use crate::brc20::params::*;
use crate::brc20::{Error, Ledger};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Deploy {
  #[serde(rename = "tick")]
  pub tick: Tick,
  #[serde(rename = "max")]
  pub max_supply: Num,
  #[serde(rename = "lim")]
  pub mint_limit: Option<Num>,
  #[serde(rename = "dec", default = "default_decimals", with = "U32StringSerde")]
  pub decimals: u32,
}

impl Deploy {
  pub(super) fn check(&self) -> Result<(), BRC20Error> {
    if self.max_supply > *MAXIMUM_SUPPLY.deref() {
      return Err(BRC20Error::InvalidMaxSupply(self.max_supply.clone()));
    }
    if self.decimals > MAX_DECIMAL_WIDTH {
      return Err(BRC20Error::InvalidDecimals(self.decimals));
    }
    self.tick.check()?;

    Ok(())
  }

  pub(super) fn reset_decimals(&mut self) {
    self.max_supply.rescale(self.decimals);
  }

  pub(super) fn update_ledger<L: Ledger>(
    mut self,
    tx_id: &str,
    ledger: &mut L,
  ) -> Result<(), Error<L>> {
    let result = self.update_ledger_inner(ledger);

    let status = if let Err(Error::<L>::BRC20Error(e)) = &result {
      Some(e.clone())
    } else {
      None
    };

    ledger
      .set_events_in_tx(
        tx_id,
        &[BRC20Event::Deploy {
          event: DeployEvent {
            inscription_id: [0; 36], // TODO: how to get inscription_id?
            supply: self.max_supply,
            limit_per_mint: self.mint_limit,
            decimal: self.decimals,
            tick: self.tick.to_string(),
            deploy_by: "".to_string(), // TODO: how to get deploy_by ?
          },
          status,
        }],
      )
      .map_err(|e| Error::LedgerError(e))?;

    result
  }
}

impl Deploy {
  fn update_ledger_inner<L: Ledger>(&mut self, ledger: &mut L) -> Result<(), Error<L>> {
    self.check()?;
    self.reset_decimals();

    todo!("not implemented")
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::str::FromStr;

  #[test]
  fn test_invalid_decimals() {
    let deploy = Deploy {
      tick: Tick::from("abcd"),
      max_supply: Num::from_str("21000000").unwrap(),
      mint_limit: Some(Num::from_str("1000").unwrap()),
      decimals: 19,
    };

    assert_eq!(deploy.check(), Err(BRC20Error::InvalidDecimals(19)));
  }
}
