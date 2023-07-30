use crate::okx::datastore::brc20;
use crate::okx::datastore::brc20s;
use crate::okx::datastore::brc20s::PledgedTick;
use crate::okx::datastore::ScriptKey;
use crate::okx::protocol::brc20s::params::{
  BIGDECIMAL_TEN, MAX_DECIMAL_WIDTH, NATIVE_TOKEN_DECIMAL,
};
use crate::okx::protocol::brc20s::{BRC20SError, Error, Num};
use anyhow::anyhow;
use bigdecimal::num_bigint::Sign;
use std::str::FromStr;

pub fn get_user_common_balance<'a, L: brc20s::DataStoreReadWrite, M: brc20::DataStoreReadWrite>(
  script: &ScriptKey,
  token: &PledgedTick,
  brc20s_ledger: &'a L,
  brc20_ledger: &'a M,
) -> Num {
  match token {
    PledgedTick::Native => Num::from(0_u128),
    PledgedTick::BRC20STick(tickid) => {
      let balance = match brc20s_ledger.get_balance(&script, tickid) {
        Ok(Some(brc20s_balance)) => brc20s_balance,
        _ => brc20s::Balance::default(tickid),
      };
      Num::from(balance.overall_balance)
    }
    PledgedTick::BRC20Tick(tick) => {
      let balance = match brc20_ledger.get_balance(&script, tick) {
        Ok(Some(brc20_balance)) => brc20_balance,
        _ => brc20::Balance::new(&tick),
      };
      Num::from(balance.overall_balance)
    }
    PledgedTick::Unknown => Num::from(0_u128),
  }
}

pub fn get_stake_dec<'a, L: brc20s::DataStoreReadWrite, M: brc20::DataStoreReadWrite>(
  token: &PledgedTick,
  brc20s_ledger: &'a L,
  brc20_ledger: &'a M,
) -> u8 {
  match token {
    PledgedTick::Native => NATIVE_TOKEN_DECIMAL,
    PledgedTick::BRC20STick(tickid) => match brc20s_ledger.get_tick_info(&tickid) {
      Ok(info) => match info {
        Some(info) => info.decimal,
        None => 0_u8,
      },
      Err(_) => 0_u8,
    },
    PledgedTick::BRC20Tick(tick) => match brc20_ledger.get_token_info(tick) {
      Ok(info) => match info {
        Some(info) => info.decimal,
        None => 0_u8,
      },
      Err(_) => 0_u8,
    },
    PledgedTick::Unknown => 0_u8,
  }
}

pub fn stake_is_exist<'a, L: brc20s::DataStoreReadWrite, M: brc20::DataStoreReadWrite>(
  token: &PledgedTick,
  brc20s_ledger: &'a L,
  brc20_ledger: &'a M,
) -> bool {
  match token {
    PledgedTick::Native => true,
    PledgedTick::BRC20STick(tickid) => {
      let tickinfo = brc20s_ledger.get_tick_info(&tickid);
      match tickinfo {
        Ok(info) => match info {
          Some(_) => true,
          _ => false,
        },
        _ => false,
      }
    }
    PledgedTick::BRC20Tick(tick) => {
      let tokeninfo = brc20_ledger.get_token_info(&tick);
      match tokeninfo {
        Ok(info) => match info {
          Some(_) => true,
          _ => false,
        },
        _ => false,
      }
    }
    PledgedTick::Unknown => false,
  }
}

pub fn get_raw_brc20_tick<M: brc20::DataStoreReadWrite>(
  token: PledgedTick,
  brc20_ledger: &M,
) -> Option<brc20::Tick> {
  let mut a = None;
  match token {
    PledgedTick::BRC20Tick(tick) => {
      let token_info = brc20_ledger.get_token_info(&tick);
      match token_info {
        Ok(info) => match info {
          Some(store_token) => a = Some(store_token.tick),
          _ => {}
        },
        _ => {}
      };
    }
    _ => {}
  }
  a
}

pub fn tick_can_staked(token: &PledgedTick) -> bool {
  match token {
    PledgedTick::Native => false,
    PledgedTick::BRC20STick(_) => false,
    PledgedTick::BRC20Tick(_) => true,
    PledgedTick::Unknown => false,
  }
}

pub fn convert_pledged_tick_with_decimal<
  'a,
  L: brc20s::DataStoreReadWrite,
  M: brc20::DataStoreReadWrite,
>(
  tick: &PledgedTick,
  amount: &str,
  brc20s_ledger: &'a L,
  brc20_ledger: &'a M,
) -> Result<Num, Error<L>> {
  match tick {
    PledgedTick::Unknown => Err(Error::BRC20SError(BRC20SError::UnknownStakeType)),
    PledgedTick::Native => convert_amount_with_decimal(amount, NATIVE_TOKEN_DECIMAL),
    PledgedTick::BRC20Tick(tick) => {
      let token = brc20_ledger
        .get_token_info(tick)
        .map_err(|e| Error::Others(anyhow!("brc20_query failed:{e}")))?
        .ok_or(BRC20SError::TickNotFound(tick.as_str().to_string()))?;

      convert_amount_with_decimal(amount, token.decimal)
    }
    PledgedTick::BRC20STick(tickid) => {
      let tick = brc20s_ledger
        .get_tick_info(tickid)
        .map_err(|e| Error::LedgerError(e))?
        .ok_or(BRC20SError::TickNotFound(tickid.hex()))?;

      convert_amount_with_decimal(amount, tick.decimal)
    }
  }
}

pub fn convert_amount_with_decimal<L: brc20s::DataStoreReadWrite>(
  amount: &str,
  decimal: u8,
) -> Result<Num, Error<L>> {
  if decimal > MAX_DECIMAL_WIDTH {
    return Err(Error::BRC20SError(BRC20SError::DecimalsTooLarge(decimal)));
  }
  let base = BIGDECIMAL_TEN.checked_powu(decimal as u64)?;
  let mut amt = Num::from_str(amount)?;

  if amt.scale() > decimal as i64 {
    return Err(Error::from(BRC20SError::InvalidNum(amount.to_string())));
  }

  if !amt.is_less_than_max_u64() || !amt.is_positive() {
    return Err(Error::from(BRC20SError::InvalidNum(amount.to_string())));
  }

  amt = amt.checked_mul(&base)?;
  if amt.sign() == Sign::NoSign {
    return Err(Error::from(BRC20SError::InvalidZeroAmount));
  }
  if !amt.is_positive_integer() {
    return Err(Error::from(BRC20SError::InvalidNum(amount.to_string())));
  }

  Ok(amt)
}

pub fn convert_pledged_tick_without_decimal<
  'a,
  L: brc20s::DataStoreReadWrite,
  M: brc20::DataStoreReadWrite,
>(
  tick: &PledgedTick,
  amount: u128,
  brc20s_ledger: &'a L,
  brc20_ledger: &'a M,
) -> Result<Num, Error<L>> {
  match tick {
    PledgedTick::Unknown => Err(Error::BRC20SError(BRC20SError::UnknownStakeType)),
    PledgedTick::Native => convert_amount_without_decimal(amount, NATIVE_TOKEN_DECIMAL),
    PledgedTick::BRC20Tick(tick) => {
      let token = brc20_ledger
        .get_token_info(tick)
        .map_err(|e| Error::Others(anyhow!("brc20_query failed:{e}")))?
        .ok_or(BRC20SError::TickNotFound(tick.as_str().to_string()))?;

      convert_amount_without_decimal(amount, token.decimal)
    }
    PledgedTick::BRC20STick(tickid) => {
      let tick = brc20s_ledger
        .get_tick_info(tickid)
        .map_err(|e| Error::LedgerError(e))?
        .ok_or(BRC20SError::TickNotFound(tickid.hex()))?;

      convert_amount_without_decimal(amount, tick.decimal)
    }
  }
}

pub fn convert_amount_without_decimal<L: brc20s::DataStoreReadWrite>(
  amount: u128,
  decimal: u8,
) -> Result<Num, Error<L>> {
  if decimal > MAX_DECIMAL_WIDTH {
    return Err(Error::BRC20SError(BRC20SError::DecimalsTooLarge(decimal)));
  }

  let base = BIGDECIMAL_TEN.checked_powu(decimal as u64)?;
  let mut amt = Num::from(amount);

  //amount must be plus integer
  if !amt.is_positive_integer() {
    return Err(Error::from(BRC20SError::InvalidNum(amount.to_string())));
  }

  amt = amt.checked_div(&base)?;
  if amt.sign() == Sign::NoSign {
    return Err(Error::from(BRC20SError::InvalidZeroAmount));
  }

  if !amt.is_less_than_max_u64() || !amt.is_positive() {
    return Err(Error::from(BRC20SError::InvalidNum(amount.to_string())));
  }

  Ok(amt)
}
