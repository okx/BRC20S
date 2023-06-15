use crate::okx::datastore::BRC20::{BRC20DataStoreReadOnly, BRC20DataStoreReadWrite};
use crate::okx::datastore::BRC20::BRC20Error::InsufficientBalance;
use crate::okx::datastore::BRC20::redb::BRC20DataStore;
use crate::okx::datastore::BRC20::balance::Balance as BRC20Balance;
use crate::okx::datastore::BRC30::{Balance as BRC30Balance, BRC30DataStoreReadOnly, Pid, UserInfo};
use crate::okx::datastore::BRC30::{ BRC30DataStoreReadWrite, PledgedTick};
use crate::okx::datastore::ScriptKey;
use crate::okx::protocol::BRC30::BRC30Error;


pub fn get_user_common_balance<'a, L: BRC30DataStoreReadWrite, M:BRC20DataStoreReadWrite>
  (script: &ScriptKey,token: &PledgedTick, brc30ledger: &'a L, brc20ledger: &'a M) -> u128 {
  match token {
    PledgedTick::NATIVE => {0_u128},
    PledgedTick::BRC30Tick(tickid) => {
      let balance = brc30ledger
        .get_balance(&script,&tickid)
        .map_or(Some(BRC30Balance::default(tickid)), |v | v).unwrap();
      balance.overall_balance
    },
    PledgedTick::BRC20Tick(tick) => {
      let balance = brc20ledger
        .get_balance(&script,tick)
        .map_or(Some(BRC20Balance::new()),|v|v).unwrap();
      balance.overall_balance
    },
    PledgedTick::UNKNOWN => {0_u128}
  }
}

pub fn get_user_avaliable_balance<'a, L: BRC30DataStoreReadWrite, M:BRC20DataStoreReadWrite>
      (script: &ScriptKey,token: &PledgedTick, pid:&Pid, brc30ledger: &'a L, brc20ledger: &'a M)
  -> Result<u128,BRC30Error> {
  let balance = get_user_common_balance(script,token,brc30ledger,brc20ledger);

  let userinfo = brc30ledger
    .get_pid_to_use_info(script, pid)
    .map_or(Some(UserInfo::default(pid)),|v|v).unwrap();
  if balance < userinfo.staked {
    return Err(BRC30Error::InValidStakeInfo(userinfo.staked,balance))
  }
  Ok(balance-userinfo.staked)
}
