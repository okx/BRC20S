use crate::okx::datastore::brc20::balance::Balance as BRC20Balance;
use crate::okx::datastore::brc20::BRC20DataStoreReadWrite;
use crate::okx::datastore::brc30::Balance as BRC30Balance;
use crate::okx::datastore::brc30::{BRC30DataStoreReadWrite, PledgedTick};
use crate::okx::datastore::ScriptKey;
use crate::okx::protocol::brc30::params::{
  BIGDECIMAL_TEN, MAX_DECIMAL_WIDTH, NATIVE_TOKEN_DECIMAL,
};
use crate::okx::protocol::brc30::{BRC30Error, Error, Num};
use anyhow::anyhow;
use bigdecimal::num_bigint::Sign;
use std::str::FromStr;

pub fn get_user_common_balance<'a, L: BRC30DataStoreReadWrite, M: BRC20DataStoreReadWrite>(
  script: &ScriptKey,
  token: &PledgedTick,
  brc30ledger: &'a L,
  brc20ledger: &'a M,
) -> Num {
  match token {
    PledgedTick::Native => Num::from(0_u128),
    PledgedTick::BRC30Tick(tickid) => {
      let balance = match brc30ledger.get_balance(&script, tickid) {
        Ok(Some(brc30_balance)) => brc30_balance,
        _ => BRC30Balance::default(tickid),
      };
      Num::from(balance.overall_balance)
    }
    PledgedTick::BRC20Tick(tick) => {
      let balance = match brc20ledger.get_balance(&script, tick) {
        Ok(Some(brc20_balance)) => brc20_balance,
        _ => BRC20Balance::new(&tick),
      };
      Num::from(balance.overall_balance)
    }
    PledgedTick::Unknown => Num::from(0_u128),
  }
}

pub fn get_stake_dec<'a, L: BRC30DataStoreReadWrite, M: BRC20DataStoreReadWrite>(
  token: &PledgedTick,
  brc30ledger: &'a L,
  brc20ledger: &'a M,
) -> u8 {
  match token {
    PledgedTick::Native => NATIVE_TOKEN_DECIMAL,
    PledgedTick::BRC30Tick(tickid) => match brc30ledger.get_tick_info(&tickid) {
      Ok(info) => match info {
        Some(info) => info.decimal,
        None => 0_u8,
      },
      Err(_) => 0_u8,
    },
    PledgedTick::BRC20Tick(tick) => match brc20ledger.get_token_info(tick) {
      Ok(info) => match info {
        Some(info) => info.decimal,
        None => 0_u8,
      },
      Err(_) => 0_u8,
    },
    PledgedTick::Unknown => 0_u8,
  }
}

pub fn stake_is_exist<'a, L: BRC30DataStoreReadWrite, M: BRC20DataStoreReadWrite>(
  token: &PledgedTick,
  brc30ledger: &'a L,
  brc20ledger: &'a M,
) -> bool {
  match token {
    PledgedTick::Native => true,
    PledgedTick::BRC30Tick(tickid) => {
      let tickinfo = brc30ledger.get_tick_info(&tickid);
      match tickinfo {
        Ok(info) => match info {
          Some(_) => true,
          _ => false,
        },
        _ => false,
      }
    }
    PledgedTick::BRC20Tick(tick) => {
      let tokeninfo = brc20ledger.get_token_info(&tick);
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

pub fn tick_can_staked(token: &PledgedTick) -> bool {
  match token {
    PledgedTick::Native => false,
    PledgedTick::BRC30Tick(_) => false,
    PledgedTick::BRC20Tick(_) => true,
    PledgedTick::Unknown => false,
  }
}

// pub fn get_user_available_balance<'a, L: BRC30DataStoreReadWrite, M:BRC20DataStoreReadWrite>
//       (script: &ScriptKey, token: &PledgedTick, pid:&Pid, brc30ledger: &'a L, brc20ledger: &'a M)
//   -> Result<u128,BRC30Error> {
//   let balance = get_user_common_balance(script,token,brc30ledger,brc20ledger);
//
//   let userinfo = brc30ledger
//     .get_pid_to_use_info(script, pid)
//     .map_or(Some(UserInfo::default(pid)),|v|v).unwrap();
//   if balance < userinfo.staked {
//     return Err(BRC30Error::InValidStakeInfo(userinfo.staked,balance))
//   }
//   Ok(balance-userinfo.staked)
// }

pub fn convert_pledged_tick_with_decimal<
  'a,
  L: BRC30DataStoreReadWrite,
  M: BRC20DataStoreReadWrite,
>(
  tick: &PledgedTick,
  amount: &str,
  brc30ledger: &'a L,
  brc20ledger: &'a M,
) -> Result<Num, Error<L>> {
  match tick {
    PledgedTick::Unknown => Err(Error::BRC30Error(BRC30Error::UnknownStakeType)),
    PledgedTick::Native => convert_amount_with_decimal(amount, NATIVE_TOKEN_DECIMAL),
    PledgedTick::BRC20Tick(tick) => {
      let token = brc20ledger
        .get_token_info(tick)
        .map_err(|e| Error::Others(anyhow!("brc20_query failed:{}", e)))?
        .ok_or(BRC30Error::TickNotFound(tick.hex()))?;

      convert_amount_with_decimal(amount, token.decimal)
    }
    PledgedTick::BRC30Tick(tickid) => {
      let tick = brc30ledger
        .get_tick_info(tickid)
        .map_err(|e| Error::LedgerError(e))?
        .ok_or(BRC30Error::TickNotFound(tickid.to_lowercase().hex()))?;

      convert_amount_with_decimal(amount, tick.decimal)
    }
  }
}

pub fn convert_amount_with_decimal<L: BRC30DataStoreReadWrite>(
  amount: &str,
  decimal: u8,
) -> Result<Num, Error<L>> {
  if decimal > MAX_DECIMAL_WIDTH {
    return Err(Error::BRC30Error(BRC30Error::DecimalsTooLarge(decimal)));
  }
  let base = BIGDECIMAL_TEN.checked_powu(decimal as u64)?;
  let mut amt = Num::from_str(amount)?;

  if amt.scale() > decimal as i64 {
    return Err(Error::from(BRC30Error::InvalidNum(amount.to_string())));
  }

  if !amt.is_less_than_max_u64() || !amt.is_positive() {
    return Err(Error::from(BRC30Error::InvalidNum(amount.to_string())));
  }

  amt = amt.checked_mul(&base)?;
  if amt.sign() == Sign::NoSign {
    return Err(Error::from(BRC30Error::InvalidZeroAmount));
  }
  if !amt.is_positive_integer() {
    return Err(Error::from(BRC30Error::InvalidNum(amount.to_string())));
  }

  Ok(amt)
}

pub fn convert_pledged_tick_without_decimal<
  'a,
  L: BRC30DataStoreReadWrite,
  M: BRC20DataStoreReadWrite,
>(
  tick: &PledgedTick,
  amount: u128,
  brc30ledger: &'a L,
  brc20ledger: &'a M,
) -> Result<Num, Error<L>> {
  match tick {
    PledgedTick::Unknown => Err(Error::BRC30Error(BRC30Error::UnknownStakeType)),
    PledgedTick::Native => convert_amount_without_decimal(amount, NATIVE_TOKEN_DECIMAL),
    PledgedTick::BRC20Tick(tick) => {
      let token = brc20ledger
        .get_token_info(tick)
        .map_err(|e| Error::Others(anyhow!("brc20_query failed:{}", e)))?
        .ok_or(BRC30Error::TickNotFound(tick.hex()))?;

      convert_amount_without_decimal(amount, token.decimal)
    }
    PledgedTick::BRC30Tick(tickid) => {
      let tick = brc30ledger
        .get_tick_info(tickid)
        .map_err(|e| Error::LedgerError(e))?
        .ok_or(BRC30Error::TickNotFound(tickid.to_lowercase().hex()))?;

      convert_amount_without_decimal(amount, tick.decimal)
    }
  }
}

pub fn convert_amount_without_decimal<L: BRC30DataStoreReadWrite>(
  amount: u128,
  decimal: u8,
) -> Result<Num, Error<L>> {
  if decimal > MAX_DECIMAL_WIDTH {
    return Err(Error::BRC30Error(BRC30Error::DecimalsTooLarge(decimal)));
  }

  let base = BIGDECIMAL_TEN.checked_powu(decimal as u64)?;
  let mut amt = Num::from(amount);

  //amount must be plus integer
  if !amt.is_positive_integer() {
    return Err(Error::from(BRC30Error::InvalidNum(amount.to_string())));
  }

  amt = amt.checked_div(&base)?;
  if amt.sign() == Sign::NoSign {
    return Err(Error::from(BRC30Error::InvalidZeroAmount));
  }

  if !amt.is_less_than_max_u64() || !amt.is_positive() {
    return Err(Error::from(BRC30Error::InvalidNum(amount.to_string())));
  }

  Ok(amt)
}
