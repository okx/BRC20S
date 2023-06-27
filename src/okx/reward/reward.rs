use crate::okx::datastore::brc30::{PoolInfo, PoolType, UserInfo};
use crate::okx::protocol::brc30::{params::BIGDECIMAL_TEN, BRC30Error, Num};
use std::str::FromStr;

const PER_SHARE_MULTIPLIER: u8 = 18;

#[cfg(not(test))]
use log::{info, warn};
#[cfg(test)]
use std::{println as info, println as warn};

// demo
// | Pool type | earn rate | total stake      | user stake     | block | reward                                        |
// |-----------|-----------|------------------|----------------|-------|-----------------------------------------------|
// | Fix       |  100(1e2) | 10000(1e3)       | 2000(1e3)      | 1     | 2000/1e3 * 100 * 1 = 200  (need stake's DECIMAL)  |
// | Pool      |  100(1e2) | 10000(1e3)       | 2000(1e3)      | 1     | 2000 * 100 / 10000 =  20                          |

pub fn query_reward(
  user: UserInfo,
  pool: PoolInfo,
  block_num: u64,
  staked_decimal: u8,
) -> Result<u128, BRC30Error> {
  let mut user_temp = user;
  let mut pool_temp = pool;
  update_pool(&mut pool_temp, block_num, staked_decimal)?;
  return withdraw_user_reward(&mut user_temp, &mut pool_temp, staked_decimal);
}

// do not save pool_info when failed
pub fn update_pool(
  pool: &mut PoolInfo,
  block_num: u64,
  staked_decimal: u8,
) -> Result<(), BRC30Error> {
  if pool.ptype != PoolType::Pool && pool.ptype != PoolType::Fixed {
    return Err(BRC30Error::UnknownPoolType);
  }
  info!("update_pool in");
  let pool_minted = Into::<Num>::into(pool.minted);
  let pool_dmax = Into::<Num>::into(pool.dmax);
  let erate = Into::<Num>::into(pool.erate);
  let pool_stake = Into::<Num>::into(pool.staked);
  let acc_reward_per_share = Num::from_str(pool.acc_reward_per_share.as_str())?;

  info!("  {}", pool);
  //1 check block num, minted, stake
  if block_num <= pool.last_update_block {
    info!("update_pool out");
    return Ok(());
  }
  if pool_stake <= Num::zero() || pool_minted >= pool_dmax {
    info!("update_pool out");
    pool.last_update_block = block_num;
    return Ok(());
  }

  let nums = Into::<Num>::into(block_num - pool.last_update_block);
  //2 calc reward, update minted and block num
  let mut rewards = erate.checked_mul(&nums)?;
  if pool.ptype == PoolType::Pool {
    if pool_minted.checked_add(&rewards)? > pool_dmax {
      rewards = pool_dmax.checked_sub(&pool_minted)?;
    }
    pool.minted = pool_minted.checked_add(&rewards)?.truncate_to_u128()?;

    // calculating accRewardPerShare
    pool.acc_reward_per_share = rewards
      .clone()
      .checked_mul(&get_per_share_multiplier())?
      .checked_div(&pool_stake)? // pool's per share = reward / all stake
      .checked_add(&acc_reward_per_share)?
      .truncate_to_str()?;
  } else if pool.ptype == PoolType::Fixed {
    let mut estimate_reward = pool_stake
      .checked_mul(&rewards)?
      .checked_mul(&get_per_share_multiplier())?
      .checked_div(&get_num_by_decimal(staked_decimal)?)?
      .checked_div(&get_per_share_multiplier())?;
    info!("  estimate_reward:{}, rewards:{}", estimate_reward, rewards);

    if pool_minted.checked_add(&estimate_reward)? > pool_dmax {
      estimate_reward = pool_dmax.checked_sub(&pool_minted)?;
      rewards = estimate_reward
        .checked_mul(&get_per_share_multiplier())?
        .checked_mul(&get_num_by_decimal(staked_decimal)?)?
        .checked_div(&pool_stake)?
        .checked_div(&get_per_share_multiplier())?;
    }

    pool.minted = pool_minted
      .checked_add(&estimate_reward)?
      .truncate_to_u128()?;

    // calculating accRewardPerShare
    pool.acc_reward_per_share = rewards
      .clone()
      .checked_mul(&get_per_share_multiplier())?
      .checked_add(&acc_reward_per_share)?
      .truncate_to_str()?;
  }

  pool.last_update_block = block_num;

  info!(
    "  pool's acc_reward_per_share:{}, rewards:{}",
    pool.acc_reward_per_share, rewards
  );

  info!("update_pool out");
  return Ok(());
}

// do not save pool and user info when failed
pub fn withdraw_user_reward(
  user: &mut UserInfo,
  pool: &mut PoolInfo,
  staked_decimal: u8,
) -> Result<u128, BRC30Error> {
  if pool.ptype != PoolType::Pool && pool.ptype != PoolType::Fixed {
    return Err(BRC30Error::UnknownPoolType);
  }

  info!("withdraw_user_reward in");
  let user_staked = Into::<Num>::into(user.staked);
  let acc_reward_per_share = Num::from_str(pool.acc_reward_per_share.as_str())?;
  let reward_debt = Into::<Num>::into(user.reward_debt);
  let user_reward = Into::<Num>::into(user.reward);
  info!("  {}", pool);
  info!("  {}", user);

  //1 check user's staked gt 0
  if user_staked <= Num::zero() {
    info!("withdraw_user_reward out");
    return Err(BRC30Error::NoStaked(
      user.pid.to_lowercase().as_str().to_string(),
    ));
  }

  //2 pending reward = staked * accRewardPerShare - user reward_debt
  let mut pending_reward = Num::zero();
  if pool.ptype == PoolType::Pool {
    pending_reward = user_staked
      .checked_mul(&acc_reward_per_share)?
      .checked_div(&get_per_share_multiplier())?
      .checked_sub(&reward_debt)?;
  } else if pool.ptype == PoolType::Fixed {
    pending_reward = user_staked
       .checked_mul(&acc_reward_per_share)?
       .checked_div(&get_num_by_decimal(staked_decimal)?)? //fix's pending reward need calc how many staked
       .checked_div(&get_per_share_multiplier())?
       .checked_sub(&reward_debt)?;
  }

  if pending_reward > Num::zero() {
    //3 update minted of user_info and pool
    user.reward = user_reward
      .checked_add(&pending_reward)?
      .truncate_to_u128()?;
  }

  info!("  pending reward:{}", pending_reward.clone());

  info!("withdraw_user_reward out");
  return pending_reward.truncate_to_u128();
}

// need to update staked  before, do not user info when failed
pub fn update_user_stake(
  user: &mut UserInfo,
  pool: &PoolInfo,
  staked_decimal: u8,
) -> Result<(), BRC30Error> {
  if pool.ptype != PoolType::Pool && pool.ptype != PoolType::Fixed {
    return Err(BRC30Error::UnknownPoolType);
  }

  info!("update_user_stake in");
  let user_staked = Into::<Num>::into(user.staked);
  let acc_reward_per_share = Num::from_str(pool.acc_reward_per_share.as_str())?;
  info!("  {}", user);
  info!("  {}", pool);

  //1 update user's reward_debt
  if pool.ptype == PoolType::Pool {
    user.reward_debt = user_staked
      .checked_mul(&acc_reward_per_share)?
      .checked_div(&get_per_share_multiplier())?
      .truncate_to_u128()?;
  } else if pool.ptype == PoolType::Fixed {
    user.reward_debt = user_staked
      .checked_mul(&acc_reward_per_share)?
      .checked_div(&get_num_by_decimal(staked_decimal)?)?
      .checked_div(&get_per_share_multiplier())?
      .truncate_to_u128()?;
  }

  user.latest_updated_block = pool.last_update_block;

  info!("  reward_debt:{}", user.reward_debt.clone());

  info!("update_user_stake out");
  return Ok(());
}

fn get_per_share_multiplier() -> Num {
  return get_num_by_decimal(PER_SHARE_MULTIPLIER).unwrap();
}

fn get_num_by_decimal(decimal: u8) -> Result<Num, BRC30Error> {
  BIGDECIMAL_TEN.checked_powu(decimal as u64)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::okx::datastore::brc30::{Pid, PledgedTick, PoolInfo, PoolType, UserInfo};
  use crate::InscriptionId;
  use std::str::FromStr;

  #[test]
  fn test_hello() {
    const STAKED_DECIMAL: u8 = 3;
    const ERATE_DECIMAL: u8 = 3;
    let stake_base = get_base_decimal(STAKED_DECIMAL);
    let erate_base = get_base_decimal(ERATE_DECIMAL);
    let erate = 1 * erate_base;
    let dmax = 100 * erate_base;

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Fixed, erate, dmax);
    let mut user = new_user(&pid);

    //stake, no reward
    {
      assert_eq!(update_pool(&mut pool, 1, STAKED_DECIMAL), Ok(()));
      assert_eq!(
        withdraw_user_reward(&mut user, &mut pool, STAKED_DECIMAL).expect_err(""),
        BRC30Error::NoStaked("bca1dabca1d#1".to_string())
      );
      user.staked += 2 * stake_base;
      pool.staked += 2 * stake_base;
      assert_eq!(
        update_user_stake(&mut user, &mut pool, STAKED_DECIMAL),
        Ok(())
      );
    }

    //withdraw, has reward
    {
      assert_eq!(update_pool(&mut pool, 2, STAKED_DECIMAL), Ok(()));
      assert_eq!(
        withdraw_user_reward(&mut user, &mut pool, STAKED_DECIMAL).unwrap(),
        2 * erate_base
      );
      user.staked -= 1 * stake_base;
      pool.staked -= 1 * stake_base;
      assert_eq!(
        update_user_stake(&mut user, &mut pool, STAKED_DECIMAL),
        Ok(())
      );
    }

    // query reward
    {
      assert_eq!(
        query_reward(user, pool, 100, STAKED_DECIMAL).unwrap(),
        98 * erate_base
      );
    }
  }

  #[test]
  fn test_complex_fix_one_user() {
    const STAKED_DECIMAL: u8 = 3;
    const ERATE_DECIMAL: u8 = 3;
    let stake_base = get_base_decimal(STAKED_DECIMAL);
    let erate_base = get_base_decimal(ERATE_DECIMAL);
    let erate = 10 * erate_base;
    let dmax = 10000 * erate_base;

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Fixed, erate, dmax);
    let mut user = new_user(&pid);

    // case-1-A deposit 0
    do_one_case(
      &mut user,
      &mut pool,
      1,
      0,
      true,
      0,
      0,
      0,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );
    // case-2-A deposit 1
    do_one_case(
      &mut user,
      &mut pool,
      2,
      1,
      true,
      0,
      1,
      1,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-3-A deposit 10
    do_one_case(
      &mut user,
      &mut pool,
      3,
      9,
      true,
      10,
      10,
      10,
      10,
      Ok(10),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-4-A same block
    do_one_case(
      &mut user,
      &mut pool,
      3,
      0,
      true,
      10,
      10,
      10,
      10,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-5-A  jump block
    do_one_case(
      &mut user,
      &mut pool,
      10,
      0 * stake_base,
      true,
      710,
      10,
      10,
      710,
      Ok(700),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-6-A deposit 90
    do_one_case(
      &mut user,
      &mut pool,
      11,
      90,
      true,
      810,
      100,
      100,
      810,
      Ok(100),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-7-A withdraw 10
    do_one_case(
      &mut user,
      &mut pool,
      12,
      10,
      false,
      1810,
      90,
      90,
      1810,
      Ok(1000),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-8-A withdraw 10, jump block
    do_one_case(
      &mut user,
      &mut pool,
      20,
      10,
      false,
      9010,
      80,
      80,
      9010,
      Ok(7200),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-9-A ,same block
    do_one_case(
      &mut user,
      &mut pool,
      20,
      0,
      false,
      9010,
      80,
      80,
      9010,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-11-A withdraw 9
    do_one_case(
      &mut user,
      &mut pool,
      22,
      80,
      false,
      10610,
      0,
      0,
      10610,
      Ok(1600),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-13-A do nothing
    do_one_case(
      &mut user,
      &mut pool,
      24,
      0 * stake_base,
      false,
      10610,
      0,
      0,
      10610,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-14-A deposit 100, jump block
    do_one_case(
      &mut user,
      &mut pool,
      50,
      100 * stake_base,
      true,
      10610,
      100 * stake_base,
      100 * stake_base,
      10610,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-15-A mint, jump block
    do_one_case(
      &mut user,
      &mut pool,
      100,
      0 * stake_base,
      true,
      10000 * erate_base,
      100 * stake_base,
      100 * stake_base,
      10000 * erate_base,
      Ok(9989390),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-16-A mint, same block
    do_one_case(
      &mut user,
      &mut pool,
      100,
      0 * stake_base,
      true,
      10000 * erate_base,
      100 * stake_base,
      100 * stake_base,
      10000 * erate_base,
      Ok(0 * erate_base),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-17-A mint, jump block
    do_one_case(
      &mut user,
      &mut pool,
      200,
      0 * stake_base,
      true,
      10000 * erate_base,
      100 * stake_base,
      100 * stake_base,
      10000 * erate_base,
      Ok(0 * erate_base),
      Ok(()),
      STAKED_DECIMAL,
    );
  }

  #[test]
  fn test_complex_fix_three_user() {
    const STAKED_DECIMAL: u8 = 3;
    const ERATE_DECIMAL: u8 = 3;
    let stake_base = get_base_decimal(STAKED_DECIMAL);
    let erate_base = get_base_decimal(ERATE_DECIMAL);
    let dmax = 1000;
    let erate = 100;

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Fixed, erate, dmax);
    let mut user_a = new_user(&pid);
    let mut user_b = new_user(&pid);
    let mut user_c = new_user(&pid);

    // case-1-A deposit 100
    do_one_case(
      &mut user_a,
      &mut pool,
      1,
      100,
      true,
      0,
      100,
      100,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-2-B deposit 100
    do_one_case(
      &mut user_b,
      &mut pool,
      2,
      100,
      true,
      0,
      100,
      200,
      10,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-3-C deposit 100
    do_one_case(
      &mut user_c,
      &mut pool,
      3,
      100,
      true,
      0,
      100,
      300,
      30,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-4-A depoist 100
    do_one_case(
      &mut user_a,
      &mut pool,
      4,
      100,
      true,
      30,
      200,
      400,
      60,
      Ok(30),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-5-A withdraw 100
    do_one_case(
      &mut user_a,
      &mut pool,
      4,
      100,
      true,
      30,
      300,
      500,
      60,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-6-B depoist 100
    do_one_case(
      &mut user_b,
      &mut pool,
      4,
      100,
      true,
      20,
      200,
      600,
      60,
      Ok(20),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-7-B withdraw 100
    do_one_case(
      &mut user_b,
      &mut pool,
      4,
      100,
      false,
      20,
      100,
      500,
      60,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-8-B withdraw 100
    do_one_case(
      &mut user_b,
      &mut pool,
      4,
      100,
      false,
      20,
      0,
      400,
      60,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-9-A, dothing
    do_one_case(
      &mut user_a,
      &mut pool,
      5,
      0,
      false,
      60,
      300,
      400,
      100,
      Ok(30),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-10-B dothing
    do_one_case(
      &mut user_b,
      &mut pool,
      5,
      0,
      false,
      20,
      0,
      400,
      100,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-11-C dothing
    do_one_case(
      &mut user_c,
      &mut pool,
      5,
      0,
      false,
      20,
      100,
      400,
      100,
      Ok(20),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-12-A, depoist 100
    do_one_case(
      &mut user_a,
      &mut pool,
      6,
      100,
      true,
      90,
      400,
      500,
      140,
      Ok(30),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-13-B depoist 100
    do_one_case(
      &mut user_b,
      &mut pool,
      6,
      100,
      true,
      20,
      100,
      600,
      140,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-14-C depoist 100
    do_one_case(
      &mut user_c,
      &mut pool,
      6,
      100,
      true,
      30,
      200,
      700,
      140,
      Ok(10),
      Ok(()),
      STAKED_DECIMAL,
    );
  }

  #[test]
  fn test_complex_pool_one_user() {
    const STAKED_DECIMAL: u8 = 3;
    const ERATE_DECIMAL: u8 = 3;
    let stake_base = get_base_decimal(STAKED_DECIMAL);
    let erate_base = get_base_decimal(ERATE_DECIMAL);
    let erate = 10 * erate_base;
    let dmax = 10000 * erate_base;

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Pool, erate, dmax);
    let mut user = new_user(&pid);

    // case-1-A deposit 0
    do_one_case(
      &mut user,
      &mut pool,
      1,
      0,
      true,
      0,
      0,
      0,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );
    // case-2-A deposit 1
    do_one_case(
      &mut user,
      &mut pool,
      2,
      1,
      true,
      0,
      1,
      1,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-3-A deposit 10
    do_one_case(
      &mut user,
      &mut pool,
      3,
      9,
      true,
      10000,
      10,
      10,
      10000,
      Ok(10000),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-4-A same block
    do_one_case(
      &mut user,
      &mut pool,
      3,
      0,
      true,
      10000,
      10,
      10,
      10000,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-5-A  jump block
    do_one_case(
      &mut user,
      &mut pool,
      10,
      0,
      true,
      80000,
      10,
      10,
      80000,
      Ok(70000),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-6-A deposit 90
    do_one_case(
      &mut user,
      &mut pool,
      11,
      90,
      true,
      90000,
      100,
      100,
      90000,
      Ok(10000),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-7-A withdraw 10
    do_one_case(
      &mut user,
      &mut pool,
      12,
      10,
      false,
      100000,
      90,
      90,
      100000,
      Ok(10000),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-8-A withdraw 10, jump block
    do_one_case(
      &mut user,
      &mut pool,
      20,
      10,
      false,
      179999,
      80,
      80,
      180000,
      Ok(79999),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-9-A withdraw 70
    do_one_case(
      &mut user,
      &mut pool,
      21,
      70,
      false,
      189999,
      10,
      10,
      190000,
      Ok(10000),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-10-A ,same block
    do_one_case(
      &mut user,
      &mut pool,
      21,
      0,
      false,
      189999,
      10,
      10,
      190000,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-11-A withdraw 9
    do_one_case(
      &mut user,
      &mut pool,
      22,
      9,
      false,
      199999,
      1,
      1,
      200000,
      Ok(10000),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-12-A withdraw  1
    do_one_case(
      &mut user,
      &mut pool,
      23,
      1,
      false,
      209999,
      0,
      0,
      210000,
      Ok(10000),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-13-A do nothing
    do_one_case(
      &mut user,
      &mut pool,
      24,
      0,
      false,
      209999,
      0,
      0,
      210000,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-14-A deposit 100, jump block
    do_one_case(
      &mut user,
      &mut pool,
      50,
      100,
      true,
      209999,
      100,
      100,
      210000,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-15-A mint, jump block
    do_one_case(
      &mut user,
      &mut pool,
      100,
      0,
      true,
      709999,
      100,
      100,
      710000,
      Ok(500000),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-16-A mint, same block
    do_one_case(
      &mut user,
      &mut pool,
      100,
      0,
      true,
      709999,
      100,
      100,
      710000,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-17-A mint, jump block
    do_one_case(
      &mut user,
      &mut pool,
      999,
      0,
      true,
      9699999,
      100,
      100,
      9700000,
      Ok(8990000),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-18-A mint
    do_one_case(
      &mut user,
      &mut pool,
      1050,
      0,
      true,
      9999999,
      100,
      100,
      10000000,
      Ok(300000),
      Ok(()),
      STAKED_DECIMAL,
    );

    //case-19-A mint, jump block
    do_one_case(
      &mut user,
      &mut pool,
      1080,
      0,
      true,
      9999999,
      100,
      100,
      10000000,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );
  }

  #[test]
  fn test_complex_pool_three_user() {
    const STAKED_DECIMAL: u8 = 3;
    const ERATE_DECIMAL: u8 = 3;
    let erate = 100;
    let dmax = 1000;

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Pool, erate, dmax);
    let mut user_a = new_user(&pid);
    let mut user_b = new_user(&pid);
    let mut user_c = new_user(&pid);

    // case-1-A deposit 100
    do_one_case(
      &mut user_a,
      &mut pool,
      1,
      100,
      true,
      0,
      100,
      100,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-2-B deposit 100
    do_one_case(
      &mut user_b,
      &mut pool,
      2,
      100,
      true,
      0,
      100,
      200,
      100,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-3-C deposit 100
    do_one_case(
      &mut user_c,
      &mut pool,
      3,
      100,
      true,
      0,
      100,
      300,
      200,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-4-A depoist 100
    do_one_case(
      &mut user_a,
      &mut pool,
      4,
      100,
      true,
      183,
      200,
      400,
      300,
      Ok(183),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-5-A withdraw 100
    do_one_case(
      &mut user_a,
      &mut pool,
      4,
      100,
      true,
      183,
      300,
      500,
      300,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-6-B depoist 100
    do_one_case(
      &mut user_b,
      &mut pool,
      4,
      100,
      true,
      83,
      200,
      600,
      300,
      Ok(83),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-7-B withdraw 100
    do_one_case(
      &mut user_b,
      &mut pool,
      4,
      100,
      false,
      83,
      100,
      500,
      300,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-8-B withdraw 100
    do_one_case(
      &mut user_b,
      &mut pool,
      4,
      100,
      false,
      83,
      0,
      400,
      300,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-9-A, dothing
    do_one_case(
      &mut user_a,
      &mut pool,
      5,
      0,
      false,
      258,
      300,
      400,
      400,
      Ok(75),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-10-B dothing
    do_one_case(
      &mut user_b,
      &mut pool,
      5,
      0,
      false,
      83,
      0,
      400,
      400,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-11-C dothing
    do_one_case(
      &mut user_c,
      &mut pool,
      5,
      0,
      false,
      58,
      100,
      400,
      400,
      Ok(58),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-12-A, depoist 100
    do_one_case(
      &mut user_a,
      &mut pool,
      6,
      100,
      true,
      333,
      400,
      500,
      500,
      Ok(75),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-13-B depoist 100
    do_one_case(
      &mut user_b,
      &mut pool,
      6,
      100,
      true,
      83,
      100,
      600,
      500,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    // case-14-C depoist 100
    do_one_case(
      &mut user_c,
      &mut pool,
      6,
      100,
      true,
      83,
      200,
      700,
      500,
      Ok(25),
      Ok(()),
      STAKED_DECIMAL,
    );
  }

  #[test]
  fn test_unknown_pool() {
    const STAKED_DECIMAL: u8 = 3;
    const ERATE_DECIMAL: u8 = 3;
    let stake_base = get_base_decimal(STAKED_DECIMAL);
    let erate_base = get_base_decimal(ERATE_DECIMAL);
    let erate = 10 * erate_base;
    let dmax = 10000 * erate_base;

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Unknown, erate, dmax);
    let mut user = new_user(&pid);

    assert_eq!(
      update_pool(&mut pool, 1, STAKED_DECIMAL),
      Err(BRC30Error::UnknownPoolType)
    );

    assert_eq!(
      withdraw_user_reward(&mut user, &mut pool, STAKED_DECIMAL),
      Err(BRC30Error::UnknownPoolType)
    );

    assert_eq!(
      update_user_stake(&mut user, &mut pool, STAKED_DECIMAL),
      Err(BRC30Error::UnknownPoolType)
    );
  }

  #[test]
  fn test_pool_stake() {
    const STAKED_DECIMAL: u8 = 3;
    const ERATE_DECIMAL: u8 = 3;
    let stake_base = get_base_decimal(STAKED_DECIMAL);
    let erate_base = get_base_decimal(ERATE_DECIMAL);
    let erate = 10 * erate_base;
    let dmax = 10000 * erate_base;

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Pool, erate, dmax);
    let mut user = new_user(&pid);
    pool.acc_reward_per_share = "123".to_string();
    pool.last_update_block = 123;
    pool.last_update_block = 123;

    assert_eq!(update_pool(&mut pool, 1, STAKED_DECIMAL), Ok(()));
    assert_eq!(pool.last_update_block, 123);
    assert_eq!(pool.acc_reward_per_share, "123".to_string());

    assert_eq!(update_pool(&mut pool, 125, STAKED_DECIMAL), Ok(()));
    assert_eq!(pool.last_update_block, 125);
    assert_eq!(pool.acc_reward_per_share, "123".to_string());
  }

  #[test]
  fn test_block() {
    const STAKED_DECIMAL: u8 = 3;
    const ERATE_DECIMAL: u8 = 3;
    let stake_base = get_base_decimal(STAKED_DECIMAL);
    let erate_base = get_base_decimal(ERATE_DECIMAL);
    let erate = 10 * erate_base;
    let dmax = 10000 * erate_base;

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Pool, erate, dmax);
    let mut user = new_user(&pid);
    pool.acc_reward_per_share = "123".to_string();
    pool.last_update_block = 123;

    assert_eq!(update_pool(&mut pool, 1, STAKED_DECIMAL), Ok(()));
    assert_eq!(pool.last_update_block, 123);
    assert_eq!(pool.acc_reward_per_share, "123".to_string());

    assert_eq!(update_pool(&mut pool, 100, STAKED_DECIMAL), Ok(()));
    assert_eq!(pool.last_update_block, 123);
    assert_eq!(pool.acc_reward_per_share, "123".to_string());
  }

  #[test]
  fn test_pool_minted() {
    const STAKED_DECIMAL: u8 = 3;
    const ERATE_DECIMAL: u8 = 3;
    let stake_base = get_base_decimal(STAKED_DECIMAL);
    let erate_base = get_base_decimal(ERATE_DECIMAL);
    let erate = 10 * erate_base;
    let dmax = 10000 * erate_base;

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Pool, erate, dmax);
    let mut user1 = new_user(&pid);
    let mut user2 = new_user(&pid);
    let mut user3 = new_user(&pid);

    do_one_case(
      &mut user1,
      &mut pool,
      1,
      100 * stake_base,
      true,
      0,
      100 * stake_base,
      100 * stake_base,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );
    do_one_case(
      &mut user2,
      &mut pool,
      1,
      200 * stake_base,
      true,
      0,
      200 * stake_base,
      300 * stake_base,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );
    do_one_case(
      &mut user3,
      &mut pool,
      1,
      300 * stake_base,
      true,
      0,
      300 * stake_base,
      600 * stake_base,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user1,
      &mut pool,
      100,
      0,
      true,
      165 * stake_base,
      100 * stake_base,
      600 * stake_base,
      990 * stake_base,
      Ok(165 * stake_base),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user2,
      &mut pool,
      1000,
      0,
      true,
      3330 * stake_base,
      200 * stake_base,
      600 * stake_base,
      9990 * stake_base,
      Ok(3330 * stake_base),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user3,
      &mut pool,
      1001,
      0,
      true,
      4999999,
      300 * stake_base,
      600 * stake_base,
      10000 * stake_base,
      Ok(4999999),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user1,
      &mut pool,
      2001,
      0,
      true,
      1666666,
      100 * stake_base,
      600 * stake_base,
      10000 * stake_base,
      Ok(1501666),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user2,
      &mut pool,
      2001,
      0,
      true,
      3333333,
      200 * stake_base,
      600 * stake_base,
      10000 * stake_base,
      Ok(3333),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user3,
      &mut pool,
      2001,
      0,
      true,
      4999999,
      300 * stake_base,
      600 * stake_base,
      10000 * stake_base,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    assert_eq!(user1.reward + user2.reward + user3.reward, 9999998);
  }

  #[test]
  fn test_fix_minted() {
    const STAKED_DECIMAL: u8 = 3;
    const ERATE_DECIMAL: u8 = 3;
    let stake_base = get_base_decimal(STAKED_DECIMAL);
    let erate_base = get_base_decimal(ERATE_DECIMAL);
    let erate = 10 * erate_base;
    let dmax = 10000 * erate_base;

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Fixed, erate, dmax);
    let mut user1 = new_user(&pid);
    let mut user2 = new_user(&pid);
    let mut user3 = new_user(&pid);

    do_one_case(
      &mut user1,
      &mut pool,
      1,
      10 * stake_base,
      true,
      0,
      10 * stake_base,
      10 * stake_base,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );
    do_one_case(
      &mut user2,
      &mut pool,
      1,
      20 * stake_base,
      true,
      0,
      20 * stake_base,
      30 * stake_base,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );
    do_one_case(
      &mut user3,
      &mut pool,
      1,
      30 * stake_base,
      true,
      0,
      30 * stake_base,
      60 * stake_base,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user1,
      &mut pool,
      5,
      0,
      true,
      400 * stake_base,
      10 * stake_base,
      60 * stake_base,
      2400 * stake_base,
      Ok(400 * stake_base),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user2,
      &mut pool,
      15,
      0,
      true,
      2800 * erate_base,
      20 * stake_base,
      60 * stake_base,
      8400 * erate_base,
      Ok(2800 * stake_base),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user3,
      &mut pool,
      20,
      0,
      true,
      4999999,
      30 * stake_base,
      60 * stake_base,
      10000 * erate_base,
      Ok(4999999),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user1,
      &mut pool,
      30,
      0,
      true,
      1666666,
      10 * stake_base,
      60 * stake_base,
      10000 * erate_base,
      Ok(1266666),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user2,
      &mut pool,
      2001,
      0,
      true,
      3333333,
      20 * stake_base,
      60 * stake_base,
      10000 * stake_base,
      Ok(533333),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user3,
      &mut pool,
      2001,
      0,
      true,
      4999999,
      30 * stake_base,
      60 * stake_base,
      10000 * stake_base,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    assert_eq!(user1.reward + user2.reward + user3.reward, 9999998);
  }

  #[test]
  fn test_user_staked() {
    const STAKED_DECIMAL: u8 = 3;
    const ERATE_DECIMAL: u8 = 3;
    let stake_base = get_base_decimal(STAKED_DECIMAL);
    let erate_base = get_base_decimal(ERATE_DECIMAL);
    let erate = 10 * erate_base;
    let dmax = 10000 * erate_base;

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Fixed, erate, dmax);
    let mut user = new_user(&pid);

    pool.staked = 100;
    pool.minted = 100;
    pool.acc_reward_per_share = "100".to_string();
    pool.last_update_block = 1;

    assert_eq!(update_pool(&mut pool, 100, STAKED_DECIMAL), Ok(()));

    assert_eq!(
      withdraw_user_reward(&mut user, &mut pool, STAKED_DECIMAL),
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string()
      ))
    );

    assert_eq!(
      update_user_stake(&mut user, &mut pool, STAKED_DECIMAL),
      Ok(())
    );
    assert_eq!(user.staked, 0);
    assert_eq!(user.reward, 0);
    assert_eq!(user.latest_updated_block, 100);
    assert_eq!(user.reward_debt, 0);
  }

  #[test]
  fn test_start_big_block() {
    const STAKED_DECIMAL: u8 = 3;
    const ERATE_DECIMAL: u8 = 3;
    let stake_base = get_base_decimal(STAKED_DECIMAL);
    let erate_base = get_base_decimal(ERATE_DECIMAL);
    let erate = 10 * erate_base;
    let dmax = 10000 * erate_base;

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Pool, erate, dmax);
    let mut user1 = new_user(&pid);
    let mut user2 = new_user(&pid);
    let mut user3 = new_user(&pid);

    do_one_case(
      &mut user1,
      &mut pool,
      101,
      100 * stake_base,
      true,
      0,
      100 * stake_base,
      100 * stake_base,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );
    do_one_case(
      &mut user2,
      &mut pool,
      101,
      200 * stake_base,
      true,
      0,
      200 * stake_base,
      300 * stake_base,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );
    do_one_case(
      &mut user3,
      &mut pool,
      101,
      300 * stake_base,
      true,
      0,
      300 * stake_base,
      600 * stake_base,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user1,
      &mut pool,
      200,
      0,
      true,
      165 * stake_base,
      100 * stake_base,
      600 * stake_base,
      990 * stake_base,
      Ok(165 * stake_base),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user2,
      &mut pool,
      1100,
      0,
      true,
      3330 * stake_base,
      200 * stake_base,
      600 * stake_base,
      9990 * stake_base,
      Ok(3330 * stake_base),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user3,
      &mut pool,
      1101,
      0,
      true,
      4999999,
      300 * stake_base,
      600 * stake_base,
      10000 * stake_base,
      Ok(4999999),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user1,
      &mut pool,
      2101,
      0,
      true,
      1666666,
      100 * stake_base,
      600 * stake_base,
      10000 * stake_base,
      Ok(1501666),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user2,
      &mut pool,
      2101,
      0,
      true,
      3333333,
      200 * stake_base,
      600 * stake_base,
      10000 * stake_base,
      Ok(3333),
      Ok(()),
      STAKED_DECIMAL,
    );

    do_one_case(
      &mut user3,
      &mut pool,
      2101,
      0,
      true,
      4999999,
      300 * stake_base,
      600 * stake_base,
      10000 * stake_base,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    assert_eq!(user1.reward + user2.reward + user3.reward, 9999998);
  }

  #[test]
  fn test_query() {
    const STAKED_DECIMAL: u8 = 3;
    const ERATE_DECIMAL: u8 = 3;
    let stake_base = get_base_decimal(STAKED_DECIMAL);
    let erate_base = get_base_decimal(ERATE_DECIMAL);
    let erate = 10 * erate_base;
    let dmax = 10000 * erate_base;

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Pool, erate, dmax);
    let mut user1 = new_user(&pid);
    let mut user2 = new_user(&pid);
    let mut user3 = new_user(&pid);

    assert_eq!(
      query_reward(user1.clone(), pool.clone(), 101, STAKED_DECIMAL).unwrap_err(),
      BRC30Error::NoStaked(pid.to_lowercase().as_str().to_string(),)
    );
    do_one_case(
      &mut user1,
      &mut pool,
      101,
      100 * stake_base,
      true,
      0,
      100 * stake_base,
      100 * stake_base,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    assert_eq!(
      query_reward(user2.clone(), pool.clone(), 101, STAKED_DECIMAL).unwrap_err(),
      BRC30Error::NoStaked(pid.to_lowercase().as_str().to_string(),)
    );
    do_one_case(
      &mut user2,
      &mut pool,
      101,
      200 * stake_base,
      true,
      0,
      200 * stake_base,
      300 * stake_base,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    assert_eq!(
      query_reward(user3.clone(), pool.clone(), 101, STAKED_DECIMAL).unwrap_err(),
      BRC30Error::NoStaked(pid.to_lowercase().as_str().to_string(),)
    );
    do_one_case(
      &mut user3,
      &mut pool,
      101,
      300 * stake_base,
      true,
      0,
      300 * stake_base,
      600 * stake_base,
      0,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
      STAKED_DECIMAL,
    );

    assert_eq!(
      query_reward(user1.clone(), pool.clone(), 200, STAKED_DECIMAL).unwrap(),
      165 * stake_base,
    );
    do_one_case(
      &mut user1,
      &mut pool,
      200,
      0,
      true,
      165 * stake_base,
      100 * stake_base,
      600 * stake_base,
      990 * stake_base,
      Ok(165 * stake_base),
      Ok(()),
      STAKED_DECIMAL,
    );

    assert_eq!(
      query_reward(user2.clone(), pool.clone(), 1100, STAKED_DECIMAL).unwrap(),
      3330 * stake_base
    );
    do_one_case(
      &mut user2,
      &mut pool,
      1100,
      0,
      true,
      3330 * stake_base,
      200 * stake_base,
      600 * stake_base,
      9990 * stake_base,
      Ok(3330 * stake_base),
      Ok(()),
      STAKED_DECIMAL,
    );

    assert_eq!(
      query_reward(user3.clone(), pool.clone(), 1101, STAKED_DECIMAL).unwrap(),
      4999999
    );
    do_one_case(
      &mut user3,
      &mut pool,
      1101,
      0,
      true,
      4999999,
      300 * stake_base,
      600 * stake_base,
      10000 * stake_base,
      Ok(4999999),
      Ok(()),
      STAKED_DECIMAL,
    );

    assert_eq!(
      query_reward(user1.clone(), pool.clone(), 2101, STAKED_DECIMAL).unwrap(),
      1501666
    );
    do_one_case(
      &mut user1,
      &mut pool,
      2101,
      0,
      true,
      1666666,
      100 * stake_base,
      600 * stake_base,
      10000 * stake_base,
      Ok(1501666),
      Ok(()),
      STAKED_DECIMAL,
    );

    assert_eq!(
      query_reward(user2.clone(), pool.clone(), 2101, STAKED_DECIMAL).unwrap(),
      3333
    );
    do_one_case(
      &mut user2,
      &mut pool,
      2101,
      0,
      true,
      3333333,
      200 * stake_base,
      600 * stake_base,
      10000 * stake_base,
      Ok(3333),
      Ok(()),
      STAKED_DECIMAL,
    );

    assert_eq!(
      query_reward(user3.clone(), pool.clone(), 2101, STAKED_DECIMAL).unwrap(),
      0
    );
    do_one_case(
      &mut user3,
      &mut pool,
      2101,
      0,
      true,
      4999999,
      300 * stake_base,
      600 * stake_base,
      10000 * stake_base,
      Ok(0),
      Ok(()),
      STAKED_DECIMAL,
    );

    assert_eq!(user1.reward + user2.reward + user3.reward, 9999998);
  }

  #[test]
  fn test_pool_one_user_18() {}

  fn to_num(s: &str) -> Num {
    Num::from_str(s).unwrap()
  }

  #[test]
  fn test_precision_18_18_block10() {
    //Fix
    do_one_precision(
      PoolType::Fixed,
      to_num("10.0"),
      to_num("20.0"),
      18,
      18,
      to_num("10.0"),
      to_num("10000.0"),
      "1000000000000000000000",
      "2000000000000000000000",
    );
    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      18,
      to_num("10.0"),
      to_num("10000.0"),
      "100",
      "2000000000000000000000",
    );
    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      18,
      to_num("1.0"),
      to_num("10000.0"),
      "10",
      "200000000000000000000",
    );
    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      18,
      to_num("0.1"),
      to_num("10000.0"),
      "1",
      "20000000000000000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      18,
      to_num("0.01"),
      to_num("10000.0"),
      "0",
      "2000000000000000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "200",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("2.0"),
      18,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "20",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("0.2"),
      18,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "2",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("0.02"),
      18,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "0",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("100000000.0"),
      18,
      18,
      to_num("0.000001"),
      to_num("10000.0"),
      "0",
      "1000000000000000000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000001"),
      to_num("100000000.0"),
      18,
      18,
      to_num("0.000001"),
      to_num("10000.0"),
      "10",
      "1000000000000000000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("100000000.0"),
      18,
      18,
      to_num("1"),
      to_num("10000000000.0"),
      "10",
      "1000000000000000000000000000",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("10.0"),
      to_num("20.0"),
      18,
      18,
      to_num("10.0"),
      to_num("10000.0"),
      "33333333333333333330",
      "66666666666666666660",
    );
    do_one_precision(
      PoolType::Pool,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      18,
      to_num("10.0"),
      to_num("10000.0"),
      "4",
      "99999999999999999980",
    );
    do_one_precision(
      PoolType::Pool,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      18,
      to_num("1.0"),
      to_num("10000.0"),
      "0",
      "9999999999999999980",
    );
    do_one_precision(
      PoolType::Pool,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      18,
      to_num("0.1"),
      to_num("10000.0"),
      "0",
      "999999999999999980",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.00000000001"),
      to_num("20.0"),
      18,
      18,
      to_num("0.01"),
      to_num("10000.0"),
      "49999",
      "99999999999950000",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.00000000001"),
      to_num("20.0"),
      18,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "0",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.00000000001"),
      to_num("2.0"),
      18,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "8",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.00000000001"),
      to_num("0.2"),
      18,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "9",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.000000000000000001"),
      to_num("0.02"),
      18,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "9",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.000000000000000001"),
      to_num("100000000.0"),
      18,
      18,
      to_num("0.000001"),
      to_num("10000.0"),
      "0",
      "9999900000000",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.0001"),
      to_num("100000000.0"),
      18,
      18,
      to_num("0.000001"),
      to_num("10000.0"),
      "9",
      "9999900000000",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.0000000001"),
      to_num("100000000.0"),
      18,
      18,
      to_num("1"),
      to_num("10000.0"),
      "9",
      "9999999999900000000",
    );
  }

  #[test]
  fn test_precision_3_6_block10() {
    //Fix
    do_one_precision(
      PoolType::Fixed,
      to_num("10.0"),
      to_num("20.0"),
      3,
      6,
      to_num("10.0"),
      to_num("10000.0"),
      "1000000000",
      "2000000000",
    );
    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("20.0"),
      3,
      6,
      to_num("10.0"),
      to_num("10000.0"),
      "100000",
      "2000000000",
    );
    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("20.0"),
      3,
      6,
      to_num("1.0"),
      to_num("10000.0"),
      "10000",
      "200000000",
    );
    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("20.0"),
      3,
      6,
      to_num("0.1"),
      to_num("10000.0"),
      "1000",
      "20000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("20.0"),
      3,
      6,
      to_num("0.01"),
      to_num("10000.0"),
      "100",
      "2000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("20.0"),
      3,
      6,
      to_num("0.000001"),
      to_num("10000.0"),
      "0",
      "200",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("2.0"),
      3,
      6,
      to_num("0.001"),
      to_num("10000.0"),
      "10",
      "20000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("0.2"),
      3,
      6,
      to_num("0.000001"),
      to_num("10000.0"),
      "0",
      "2",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("0.02"),
      3,
      6,
      to_num("0.000001"),
      to_num("10000.0"),
      "0",
      "0",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("100000000.0"),
      3,
      6,
      to_num("0.000001"),
      to_num("10000.0"),
      "0",
      "1000000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("1"),
      to_num("100000000.0"),
      3,
      6,
      to_num("0.000001"),
      to_num("10000.0"),
      "10",
      "1000000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("1"),
      to_num("100000000.0"),
      3,
      6,
      to_num("10"),
      to_num("10000000000.0"),
      "99999999",
      "9999999900000000",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("10.0"),
      to_num("20.0"),
      3,
      6,
      to_num("10.0"),
      to_num("10000.0"),
      "33333333",
      "66666666",
    );
    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("20.0"),
      3,
      6,
      to_num("10.0"),
      to_num("10000.0"),
      "4999",
      "99995000",
    );
    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("20.0"),
      3,
      6,
      to_num("1.0"),
      to_num("10000.0"),
      "499",
      "9999500",
    );
    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("20.0"),
      3,
      6,
      to_num("0.1"),
      to_num("10000.0"),
      "49",
      "999950",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("20.0"),
      3,
      6,
      to_num("0.01"),
      to_num("10000.0"),
      "4",
      "99995",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("20.0"),
      3,
      6,
      to_num("0.001"),
      to_num("10000.0"),
      "0",
      "9999",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("20.0"),
      3,
      6,
      to_num("0.000001"),
      to_num("10000.0"),
      "0",
      "9",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("2.0"),
      3,
      6,
      to_num("0.000001"),
      to_num("10000.0"),
      "0",
      "9",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("0.2"),
      3,
      6,
      to_num("0.000001"),
      to_num("10000.0"),
      "0",
      "9",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("0.02"),
      3,
      6,
      to_num("0.000001"),
      to_num("10000.0"),
      "0",
      "9",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("100000000.0"),
      3,
      6,
      to_num("0.000001"),
      to_num("10000.0"),
      "0",
      "9",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("100000"),
      to_num("100000000.0"),
      3,
      6,
      to_num("0.001"),
      to_num("10000.0"),
      "9",
      "9990",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("100"),
      to_num("100000000.0"),
      3,
      6,
      to_num("1"),
      to_num("10000.0"),
      "9",
      "9999990",
    );
  }

  #[test]
  fn test_precision_18_3_block10() {
    //Fix
    do_one_precision(
      PoolType::Fixed,
      to_num("10.0"),
      to_num("20.0"),
      18,
      3,
      to_num("10.0"),
      to_num("10000.0"),
      "1000000",
      "2000000",
    );
    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      3,
      to_num("10.0"),
      to_num("10000.0"),
      "0",
      "2000000",
    );
    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      3,
      to_num("1.0"),
      to_num("10000.0"),
      "0",
      "200000",
    );
    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      3,
      to_num("0.1"),
      to_num("10000.0"),
      "0",
      "20000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      3,
      to_num("0.01"),
      to_num("10000.0"),
      "0",
      "2000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      3,
      to_num("0.001"),
      to_num("10000.0"),
      "0",
      "200",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("2.0"),
      18,
      3,
      to_num("0.001"),
      to_num("10000.0"),
      "0",
      "20",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("0.2"),
      18,
      3,
      to_num("0.001"),
      to_num("100000000000.0"),
      "0",
      "2",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("0.02"),
      18,
      3,
      to_num("0.001"),
      to_num("10000.0"),
      "0",
      "0",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.000000000000000001"),
      to_num("100000000.0"),
      18,
      3,
      to_num("0.001"),
      to_num("100000000000.0"),
      "0",
      "1000000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("1"),
      to_num("100000000.0"),
      18,
      3,
      to_num("0.001"),
      to_num("100000000000.0"),
      "10",
      "1000000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("1"),
      to_num("100000000.0"),
      18,
      3,
      to_num("10"),
      to_num("100000000000.0"),
      "100000",
      "10000000000000",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("10.0"),
      to_num("20.0"),
      18,
      3,
      to_num("10.0"),
      to_num("10000.0"),
      "33330",
      "66660",
    );
    do_one_precision(
      PoolType::Pool,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      3,
      to_num("10.0"),
      to_num("10000.0"),
      "0",
      "99980",
    );
    do_one_precision(
      PoolType::Pool,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      3,
      to_num("1.0"),
      to_num("10000.0"),
      "0",
      "9980",
    );
    do_one_precision(
      PoolType::Pool,
      to_num("0.000000000000000001"),
      to_num("20.0"),
      18,
      3,
      to_num("0.1"),
      to_num("10000.0"),
      "0",
      "980",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.00000000001"),
      to_num("20.0"),
      18,
      3,
      to_num("0.01"),
      to_num("80.0"),
      "0",
      "80",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.00000000001"),
      to_num("2.0"),
      18,
      3,
      to_num("0.001"),
      to_num("10000.0"),
      "0",
      "8",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.00000000001"),
      to_num("0.2"),
      18,
      3,
      to_num("0.001"),
      to_num("10000.0"),
      "0",
      "9",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.000000000000000001"),
      to_num("0.02"),
      18,
      3,
      to_num("0.001"),
      to_num("10000.0"),
      "0",
      "9",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("1"),
      to_num("10.0"),
      18,
      3,
      to_num("0.001"),
      to_num("10000000000000000.0"),
      "0",
      "0",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("100000000"),
      to_num("100000000"),
      18,
      3,
      to_num("0.001"),
      to_num("10000.0"),
      "0",
      "0",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("100000"),
      to_num("100000000.0"),
      18,
      3,
      to_num("1"),
      to_num("10000.0"),
      "0",
      "0",
    );
  }

  #[test]
  fn test_precision_3_18_block10() {
    //Fix
    do_one_precision(
      PoolType::Fixed,
      to_num("10.0"),
      to_num("20.0"),
      3,
      18,
      to_num("10.0"),
      to_num("10000.0"),
      "1000000000000000000000",
      "2000000000000000000000",
    );
    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("20.0"),
      3,
      18,
      to_num("10.0"),
      to_num("10000.0"),
      "100000000000000000",
      "2000000000000000000000",
    );
    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("20.0"),
      3,
      18,
      to_num("1.0"),
      to_num("10000.0"),
      "10000000000000000",
      "200000000000000000000",
    );
    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("20.0"),
      3,
      18,
      to_num("0.1"),
      to_num("10000.0"),
      "1000000000000000",
      "20000000000000000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("20.0"),
      3,
      18,
      to_num("0.01"),
      to_num("10000.0"),
      "100000000000000",
      "2000000000000000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("20.0"),
      3,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "200",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("2.0"),
      3,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "20",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("0.2"),
      3,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "2",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("0.02"),
      3,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "0",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("100000000.0"),
      3,
      18,
      to_num("0.000001"),
      to_num("10000.0"),
      "10000000000",
      "1000000000000000000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.01"),
      to_num("100000000.0"),
      3,
      18,
      to_num("0.000001"),
      to_num("10000.0"),
      "100000000000",
      "1000000000000000000000",
    );

    do_one_precision(
      PoolType::Fixed,
      to_num("0.001"),
      to_num("100000000.0"),
      3,
      18,
      to_num("1"),
      to_num("10000000000.0"),
      "10000000000000000",
      "1000000000000000000000000000",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("10.0"),
      to_num("20.0"),
      3,
      18,
      to_num("10.0"),
      to_num("10000.0"),
      "33333333333333333333",
      "66666666666666666666",
    );
    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("20.0"),
      3,
      18,
      to_num("10.0"),
      to_num("10000.0"),
      "4999750012499375",
      "99995000249987500624",
    );
    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("20.0"),
      3,
      18,
      to_num("1.0"),
      to_num("10000.0"),
      "499975001249937",
      "9999500024998750062",
    );
    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("20.0"),
      3,
      18,
      to_num("0.1"),
      to_num("10000.0"),
      "49997500124993",
      "999950002499875006",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.01"),
      to_num("20.0"),
      3,
      18,
      to_num("0.01"),
      to_num("10000.0"),
      "49975012493753",
      "99950024987506246",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.01"),
      to_num("20.0"),
      3,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "9",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.01"),
      to_num("2.0"),
      3,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "9",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.01"),
      to_num("0.2"),
      3,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "0",
      "9",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.01"),
      to_num("0.02"),
      3,
      18,
      to_num("0.000000000000000001"),
      to_num("10000.0"),
      "3",
      "6",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.001"),
      to_num("100000000.0"),
      3,
      18,
      to_num("0.000001"),
      to_num("10000.0"),
      "99",
      "9999999999900",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.01"),
      to_num("100000000.0"),
      3,
      18,
      to_num("0.000001"),
      to_num("10000.0"),
      "999",
      "9999999999000",
    );

    do_one_precision(
      PoolType::Pool,
      to_num("0.01"),
      to_num("100000000.0"),
      3,
      18,
      to_num("1"),
      to_num("10000.0"),
      "999999999",
      "9999999999000000000",
    );
  }

  fn do_one_precision(
    ptype: PoolType,
    stake1: Num,
    stake2: Num,
    staked_decimal: u8,
    earte_decimal: u8,
    erate: Num,
    dmax: Num,
    expect1: &str,
    expect2: &str,
  ) {
    println!("--------------");
    let stake_base = Num::from(get_base_decimal(staked_decimal));
    let erate_base = Num::from(get_base_decimal(earte_decimal));
    let erate = erate_base.checked_mul(&erate).unwrap();
    let dmax = erate_base.checked_mul(&dmax).unwrap();

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(
      &pid.clone(),
      ptype,
      erate.truncate_to_u128().unwrap(),
      dmax.truncate_to_u128().unwrap(),
    );
    let mut user1 = new_user(&pid);
    let mut user2 = new_user(&pid);

    //first
    let _ = update_pool(&mut pool, 1, staked_decimal);
    let _ = withdraw_user_reward(&mut user1, &mut pool, staked_decimal);
    user1.staked += stake_base
      .checked_mul(&stake1)
      .unwrap()
      .truncate_to_u128()
      .unwrap();
    pool.staked += stake_base
      .checked_mul(&stake1)
      .unwrap()
      .truncate_to_u128()
      .unwrap();
    let _ = withdraw_user_reward(&mut user1, &mut pool, staked_decimal);
    let _ = update_user_stake(&mut user1, &mut pool, staked_decimal);

    let _ = update_pool(&mut pool, 1, staked_decimal);
    let _ = withdraw_user_reward(&mut user2, &mut pool, staked_decimal);
    user2.staked += stake_base
      .checked_mul(&stake2)
      .unwrap()
      .truncate_to_u128()
      .unwrap();
    pool.staked += stake_base
      .checked_mul(&stake2)
      .unwrap()
      .truncate_to_u128()
      .unwrap();
    let _ = withdraw_user_reward(&mut user2, &mut pool, staked_decimal);
    let _ = update_user_stake(&mut user2, &mut pool, staked_decimal);

    //second
    let _ = update_pool(&mut pool, 11, staked_decimal);
    let reward1 = withdraw_user_reward(&mut user1, &mut pool, staked_decimal).unwrap();
    let reward2 = withdraw_user_reward(&mut user2, &mut pool, staked_decimal).unwrap();
    assert_eq!(
      reward1,
      Num::from_str(expect1).unwrap().truncate_to_u128().unwrap()
    );
    assert_eq!(
      reward2,
      Num::from_str(expect2).unwrap().truncate_to_u128().unwrap()
    );
  }

  fn do_one_case(
    user: &mut UserInfo,
    pool: &mut PoolInfo,
    block_mum: u64,
    stake_alter: u128,
    is_add: bool,
    expect_user_remain_reward: u128,
    expert_user_staked: u128,
    expect_pool_staked: u128,
    expect_pool_minted: u128,
    expect_withdraw_reward_result: Result<u128, BRC30Error>,
    expect_update_stake_result: Result<(), BRC30Error>,
    staked_decimal: u8,
  ) {
    assert_eq!(update_pool(pool, block_mum, staked_decimal), Ok(()));

    let result = withdraw_user_reward(user, pool, staked_decimal);
    match result {
      Ok(value) => {
        assert_eq!(value, expect_withdraw_reward_result.clone().unwrap());
      }
      Err(err) => {
        println!("err:{:?}", err);
        assert_eq!(err, expect_withdraw_reward_result.clone().unwrap_err())
      }
    }

    if is_add {
      user.staked += stake_alter;
      pool.staked += stake_alter;
    } else {
      user.staked -= stake_alter;
      pool.staked -= stake_alter;
    }
    let u_result = update_user_stake(user, pool, staked_decimal);
    match u_result {
      Ok(value) => {}
      Err(err) => {
        println!("err:{:?}", err);
        assert_eq!(err, expect_update_stake_result.clone().unwrap_err())
      }
    }

    assert_eq!(user.reward, expect_user_remain_reward);
    assert_eq!(user.staked, expert_user_staked);
    assert_eq!(pool.staked, expect_pool_staked);
    assert_eq!(pool.minted, expect_pool_minted);
    assert_eq!(pool.last_update_block, block_mum);
  }

  fn new_pool(pid: &Pid, pool_type: PoolType, erate: u128, dmax: u128) -> PoolInfo {
    PoolInfo {
      pid: pid.clone(),
      ptype: pool_type,
      inscription_id: InscriptionId::from_str(
        "2111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      stake: PledgedTick::NATIVE,
      erate,
      minted: 0,
      staked: 0,
      dmax,
      acc_reward_per_share: "0".to_string(),
      last_update_block: 0,
      only: true,
    }
  }

  fn new_user(pid: &Pid) -> UserInfo {
    UserInfo {
      pid: pid.clone(),
      staked: 0,
      reward: 0,
      reward_debt: 0,
      latest_updated_block: 0,
    }
  }

  fn get_base_decimal(decimal: u8) -> u128 {
    BIGDECIMAL_TEN
      .checked_powu(decimal as u64)
      .unwrap()
      .truncate_to_u128()
      .unwrap()
  }
}
