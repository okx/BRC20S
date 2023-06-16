use super::error::RewardError;
use crate::okx::datastore::ScriptKey;
use crate::okx::datastore::BRC30::{
  BRC30DataStoreReadOnly, BRC30DataStoreReadWrite, Balance, InscriptionOperation, Pid, PledgedTick,
  PoolInfo, PoolType, Receipt, StakeInfo, TickId, TickInfo, TransferableAsset, UserInfo,
};

use crate::okx::datastore::{
  BRC20::redb::BRC20DataStore, BRC30::redb::BRC30DataStore, ORD::OrdDbReader,
};

use redb::WriteTransaction;

pub struct Pool<'db, 'a> {
  wtx: &'a WriteTransaction<'db>,
  brc30db: BRC30DataStore<'db, 'a>,
}

impl<'db, 'a> Pool<'db, 'a> {
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    let brc30db = BRC30DataStore::new(&wtx);
    Self { wtx, brc30db }
  }
}

impl<'db, 'a> Pool<'db, 'a> {
  pub fn query_reward(&mut self, pid: &Pid, script_key: &ScriptKey) -> (String, u128) {
    //TODO query_reward
    return (script_key.to_string(), 1u128);
  }

  pub fn update_pool(&mut self, pid: &Pid, block_num: u64) -> Result<(), RewardError> {
    //1 check pool is exist
    let mut pool = self
      .brc30db
      .get_pid_to_poolinfo(&pid.clone())
      .unwrap()
      .unwrap();

    //2 check block num of pool is latest
    if block_num <= pool.last_update_block {
      return Ok(());
    }

    //3 check allocated has been minted
    if pool.minted >= pool.dmax {
      pool.last_update_block = block_num;
      return Ok(());
    }

    //4 pool type: fixed and pool, for calculating accRewardPerShare
    let mut reward_per_token_stored = 0;
    let mut nums = (block_num - pool.last_update_block) as u128;
    if pool.ptype == PoolType::Fixed {
      reward_per_token_stored = pool.erate * nums;
    } else if pool.ptype == PoolType::Pool && pool.staked != 0 {
      reward_per_token_stored = pool.erate * nums / pool.staked;
    }

    pool.acc_reward_per_share += reward_per_token_stored;

    //5 update latest block num
    pool.last_update_block = block_num;

    println!(
      "update_pool-block:{}, acc_reward_per_share:{}, reward_per_token_stored:{}",
      block_num, pool.acc_reward_per_share, reward_per_token_stored
    );
    //6 update pool
    self.brc30db.set_pid_to_poolinfo(&pid.clone(), &pool);
    return Ok(());
  }

  pub fn withdraw_user_reward(
    &mut self,
    pid: &Pid,
    script_key: &ScriptKey,
  ) -> Result<u128, RewardError> {
    //1 check user's staked gt 0
    let mut user = self
      .brc30db
      .get_pid_to_use_info(&script_key.clone(), &pid.clone())
      .unwrap()
      .unwrap();

    if user.staked <= 0 {
      return Err(RewardError::NoStaked(
        pid.to_lowercase().hex(),
        script_key.to_string(),
      ));
    }

    let mut pool = self
      .brc30db
      .get_pid_to_poolinfo(&pid.clone())
      .unwrap()
      .unwrap();

    //2 reward = staked * accRewardPerShare - user reward_debt
    let mut a = 0;
    match user.staked.checked_mul(pool.acc_reward_per_share) {
      Some(result) => a = result,
      None => println!("Multiplication failed!"),
    }

    let mut reward: u128 = 0;
    match a.checked_sub(user.reward_debt) {
      Some(result) => reward = result,
      None => println!("Division failed!"),
    }

    if reward > 0 {
      //3 update minted of user_info and pool, TODO check overflow
      user.reward += reward;
      pool.minted += reward;
      self
        .brc30db
        .set_pid_to_use_info(&script_key.clone(), &pid.clone(), &user);
      self.brc30db.set_pid_to_poolinfo(&pid.clone(), &pool);
    }

    println!(
      "withdraw_user_reward-reward:{}, user.reward_debt:{}, user.staked:{}, pool.acc_reward_per_share:{}",
      reward, user.reward_debt, user.staked, pool.acc_reward_per_share
    );

    return Ok(reward);
  }

  pub fn update_user_stake(
    &mut self,
    pid: &Pid,
    script_key: &ScriptKey,
    stake_alter: u128,
    is_add: bool,
  ) -> Result<(), RewardError> {
    let mut user = self
      .brc30db
      .get_pid_to_use_info(&script_key.clone(), &pid.clone())
      .unwrap()
      .unwrap();

    let mut pool = self
      .brc30db
      .get_pid_to_poolinfo(&pid.clone())
      .unwrap()
      .unwrap();

    //1 check stake_alter gt 0
    if (stake_alter > 0) {
      //2 add or sub, TODO check overflow
      if is_add {
        user.staked += stake_alter;
        pool.staked += stake_alter;
      } else {
        user.staked -= stake_alter;
        pool.staked -= stake_alter;
      }
    }

    //3 update user's reward_debt，TODO check overflow
    user.reward_debt = user.staked * pool.acc_reward_per_share;

    println!(
      "update_user_stake--reward_debt:{}, user staked:{}, acc_reward_per_share:{}, pool staked:{}",
      user.reward_debt, user.staked, pool.acc_reward_per_share, pool.staked
    );

    //4 save user info and pool info
    self
      .brc30db
      .set_pid_to_use_info(&script_key.clone(), &pid.clone(), &user);
    self.brc30db.set_pid_to_poolinfo(&pid.clone(), &pool);

    return Ok(());
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::InscriptionId;
  use bitcoin::Address;
  use redb::Database;
  use std::str::FromStr;
  use tempfile::NamedTempFile;

  #[test]
  fn test_fix_one_user() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30DataStore::new(&wtx);

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let script_key =
      ScriptKey::from_address(Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap());
    new_pid(&wtx, &pid.clone(), PoolType::Fixed, 10, 100000000000);
    new_user(&wtx, &pid, &script_key);

    let mut pool = Pool::new(&wtx);
    // stake-1
    do_nonce(
      &wtx,
      &mut pool,
      &pid.clone(),
      1u64,
      &script_key.clone(),
      1u128,
      true,
      0,
      1,
      1,
      0,
      1,
    );

    // stake-2
    do_nonce(
      &wtx,
      &mut pool,
      &pid.clone(),
      2u64,
      &script_key.clone(),
      1u128,
      true,
      10,
      2,
      2,
      10,
      2,
    );

    // stake-3
    do_nonce(
      &wtx,
      &mut pool,
      &pid.clone(),
      3u64,
      &script_key.clone(),
      1u128,
      true,
      30,
      3,
      3,
      30,
      3,
    );

    // withdraw-1
    do_nonce(
      &wtx,
      &mut pool,
      &pid.clone(),
      4u64,
      &script_key.clone(),
      1u128,
      false,
      60,
      2,
      2,
      60,
      4,
    );

    // withdraw-2
    do_nonce(
      &wtx,
      &mut pool,
      &pid.clone(),
      5u64,
      &script_key.clone(),
      1u128,
      false,
      80,
      1,
      1,
      80,
      5,
    );

    // withdraw-3
    do_nonce(
      &wtx,
      &mut pool,
      &pid.clone(),
      6u64,
      &script_key.clone(),
      1u128,
      false,
      90,
      0,
      0,
      90,
      6,
    );
  }

  fn do_nonce<'db, 'a>(
    wtx: &'a WriteTransaction<'db>,
    pool: &mut Pool,
    pid: &Pid,
    block_mum: u64,
    script_key: &ScriptKey,
    stake_alter: u128,
    is_add: bool,
    expect_user_reward: u128,
    expert_user_staked: u128,
    expect_pool_staked: u128,
    expect_pool_minted: u128,
    expect_pool_block: u64,
  ) {
    let brc30db = BRC30DataStore::new(&wtx);
    pool.update_pool(&pid, block_mum);
    pool.withdraw_user_reward(&pid, &script_key);
    pool.update_user_stake(&pid, &script_key, stake_alter, is_add);

    let user = brc30db
      .get_pid_to_use_info(&script_key.clone(), &pid.clone())
      .unwrap()
      .unwrap();
    let pool = brc30db.get_pid_to_poolinfo(&pid.clone()).unwrap().unwrap();
    assert_eq!(user.reward, expect_user_reward);
    assert_eq!(user.staked, expert_user_staked);
    assert_eq!(user.reward, expect_user_reward);
    assert_eq!(pool.staked, expect_pool_staked);
    assert_eq!(pool.minted, expect_pool_minted);
    assert_eq!(pool.last_update_block, expect_pool_block);
  }

  fn new_pid<'db, 'a>(
    wtx: &'a WriteTransaction<'db>,
    pid: &Pid,
    pool_type: PoolType,
    erate_new: u128,
    dmax: u128,
  ) {
    let brc30db = BRC30DataStore::new(&wtx);
    let pool_info = PoolInfo {
      pid: pid.clone(),
      ptype: pool_type,
      inscription_id: InscriptionId::from_str(
        "2111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      stake: PledgedTick::NATIVE,
      erate: erate_new,
      minted: 0,
      staked: 0,
      dmax,
      acc_reward_per_share: 0,
      last_update_block: 0,
      only: true,
    };
    brc30db.set_pid_to_poolinfo(&pid, &pool_info).unwrap();
    assert_eq!(
      brc30db.get_pid_to_poolinfo(&pid).unwrap().unwrap(),
      pool_info
    );
  }

  fn new_user<'db, 'a>(wtx: &'a WriteTransaction<'db>, pid: &Pid, script_key: &ScriptKey) {
    let brc30db = BRC30DataStore::new(&wtx);
    let user_info = UserInfo {
      pid: pid.clone(),
      staked: 0,
      reward: 0,
      reward_debt: 0,
      latest_updated_block: 0,
    };
    brc30db
      .set_pid_to_use_info(&script_key, &pid, &user_info)
      .unwrap();
    assert_eq!(
      brc30db
        .get_pid_to_use_info(&script_key, &pid)
        .unwrap()
        .unwrap(),
      user_info
    );
  }
}
