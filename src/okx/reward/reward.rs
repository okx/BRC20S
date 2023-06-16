// use super::error::RewardError;
use crate::okx::datastore::BRC30::{PoolInfo, PoolType, UserInfo};
use crate::okx::protocol::BRC30::{BRC30Error, Num};

pub fn query_reward(user: UserInfo, pool: PoolInfo, block_num: u64) -> Result<u128, BRC30Error> {
  let mut user_temp = user;
  let mut pool_temp = pool;
  update_pool(&mut pool_temp, block_num).expect("TODO: panic message");
  return withdraw_user_reward(&mut user_temp, &mut pool_temp);
}

// need to save pool_info, when call success
pub fn update_pool(pool: &mut PoolInfo, block_num: u64) -> Result<(), BRC30Error> {
  //1 check block num of pool is latest
  if block_num <= pool.last_update_block {
    return Ok(());
  }

  //2 check allocated has been minted
  let pool_minted = Into::<Num>::into(pool.minted);
  let pool_dmax = Into::<Num>::into(pool.dmax);
  if pool_minted >= pool_dmax {
    pool.last_update_block = block_num;
    return Ok(());
  }

  //3 pool type: fixed and pool, for calculating accRewardPerShare
  let mut reward_per_token_stored = Num::zero();
  let mut nums = Into::<Num>::into(block_num - pool.last_update_block);
  let erate = Into::<Num>::into(pool.erate);
  let pool_stake = Into::<Num>::into(pool.staked);
  if pool.ptype == PoolType::Fixed {
    reward_per_token_stored = erate.checked_mul(&nums).unwrap();
  } else if pool.ptype == PoolType::Pool && pool.staked != 0 {
    // reward_per_token_stored = pool.erate * nums / pool.staked);
    reward_per_token_stored = erate
      .checked_mul(&nums)
      .unwrap()
      .checked_div(&pool_stake)
      .unwrap();
  } else {
    return Err(BRC30Error::UnknownPoolType);
  }

  let acc_reward_per_share = Into::<Num>::into(pool.acc_reward_per_share);
  pool.acc_reward_per_share = reward_per_token_stored
    .checked_add(&acc_reward_per_share)?
    .checked_to_u128()?;

  //4 update latest block num
  pool.last_update_block = block_num;

  println!(
    "update_pool-block:{}, acc_reward_per_share:{}, reward_per_token_stored:{}",
    block_num, pool.acc_reward_per_share, reward_per_token_stored
  );
  return Ok(());
}

// need to save pool_info and user_info, when call success
pub fn withdraw_user_reward(user: &mut UserInfo, pool: &mut PoolInfo) -> Result<u128, BRC30Error> {
  //1 check user's staked gt 0
  if user.staked <= 0 {
    return Err(BRC30Error::NoStaked(user.pid.to_lowercase().hex()));
  }

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
  }

  println!(
      "withdraw_user_reward-reward:{}, user.reward_debt:{}, user.staked:{}, pool.acc_reward_per_share:{}",
      reward, user.reward_debt, user.staked, pool.acc_reward_per_share
    );

  return Ok(reward);
}

// need to update staked  before, and save pool_info and user_info when call success
pub fn update_user_stake(user: &mut UserInfo, pool: &PoolInfo) -> Result<(), BRC30Error> {
  //1 update user's reward_debt，TODO check overflow
  user.reward_debt = user.staked * pool.acc_reward_per_share;
  println!(
    "update_user_stake--reward_debt:{}, user staked:{}, acc_reward_per_share:{}, pool staked:{}",
    user.reward_debt, user.staked, pool.acc_reward_per_share, pool.staked
  );
  return Ok(());
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::okx::datastore::ScriptKey;
  use crate::okx::datastore::BRC30::{Pid, PledgedTick, PoolInfo, PoolType, UserInfo};
  use crate::InscriptionId;
  use bitcoin::Address;
  use std::str::FromStr;

  #[test]
  fn test_hello() {
    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let script_key =
      ScriptKey::from_address(Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap());
    let mut pool = new_pool(&pid.clone(), PoolType::Fixed, 10, 100000000000);
    let mut user = new_user(&pid, &script_key);

    //stake, no reward
    {
      assert_eq!(update_pool(&mut pool, 1), Ok(()));
      assert_eq!(
        withdraw_user_reward(&mut user, &mut pool).expect_err(""),
        BRC30Error::NoStaked("62636131646162636131642331".to_string())
      );
      user.staked += 2;
      pool.staked += 2;
      assert_eq!(update_user_stake(&mut user, &mut pool), Ok(()));
    }

    //withdraw, has reward
    {
      assert_eq!(update_pool(&mut pool, 2), Ok(()));
      assert_eq!(withdraw_user_reward(&mut user, &mut pool).unwrap(), 20);
      user.staked -= 1;
      pool.staked -= 1;
      assert_eq!(update_user_stake(&mut user, &mut pool), Ok(()));
    }

    // query reward
    {
      assert_eq!(query_reward(user, pool, 3).unwrap(), 10);
    }
  }

  #[test]
  fn test_fix_one_user() {
    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let script_key =
      ScriptKey::from_address(Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap());
    let mut pool = new_pool(&pid.clone(), PoolType::Fixed, 10, 100000000000);
    let mut user = new_user(&pid, &script_key);

    // stake-1
    do_nonce(
      &mut user,
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
      &mut user,
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
      &mut user,
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
      &mut user,
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
      &mut user,
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
      &mut user,
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

  fn do_nonce(
    user: &mut UserInfo,
    pool: &mut PoolInfo,
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
    update_pool(pool, block_mum);
    withdraw_user_reward(user, pool);
    if is_add {
      user.staked += stake_alter;
      pool.staked += stake_alter;
    } else {
      user.staked -= stake_alter;
      pool.staked -= stake_alter;
    }
    update_user_stake(user, pool);

    assert_eq!(user.reward, expect_user_reward);
    assert_eq!(user.staked, expert_user_staked);
    assert_eq!(user.reward, expect_user_reward);
    assert_eq!(pool.staked, expect_pool_staked);
    assert_eq!(pool.minted, expect_pool_minted);
    assert_eq!(pool.last_update_block, expect_pool_block);
  }

  fn new_pool(pid: &Pid, pool_type: PoolType, erate_new: u128, dmax: u128) -> PoolInfo {
    PoolInfo {
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
    }
  }

  fn new_user(pid: &Pid, script_key: &ScriptKey) -> UserInfo {
    UserInfo {
      pid: pid.clone(),
      staked: 0,
      reward: 0,
      reward_debt: 0,
      latest_updated_block: 0,
    }
  }
}
