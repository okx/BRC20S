use crate::okx::datastore::brc30::{PoolInfo, PoolType, UserInfo};
use crate::okx::protocol::brc30::{params::BIGDECIMAL_TEN, BRC30Error, Num};

#[cfg(not(test))]
use log::{info, warn};
#[cfg(test)]
use std::{println as info, println as warn};

// demo
// | Pool type | earn rate | total stake      | user stake     | block | reward                                        |
// |-----------|-----------|------------------|----------------|-------|-----------------------------------------------|
// | Fix       |  100(1e2) | 10000(1e3)       | 2000(1e3)      | 1     | 2000/1e3 * 100 * 1 = 200  (need stake's DECIMAL)  |
// | Pool      |  100(1e2) | 10000(1e3)       | 2000(1e3)      | 1     | 2000 * 100 / 10000 =  20                          |

const REWARD_PER_DECIMAL: u64 = 10000000000;

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

// need to save pool_info, when call success
pub fn update_pool(
  pool: &mut PoolInfo,
  block_num: u64,
  staked_decimal: u8,
) -> Result<(), BRC30Error> {
  info!("update_pool in");
  let pool_minted = Into::<Num>::into(pool.minted);
  let pool_dmax = Into::<Num>::into(pool.dmax);
  let nums = Into::<Num>::into(block_num - pool.last_update_block);
  let erate = Into::<Num>::into(pool.erate);
  let pool_stake = Into::<Num>::into(pool.staked);
  let acc_reward_per_share = Into::<Num>::into(pool.acc_reward_per_share);

  info!("  pool:{:?}", pool);
  //1 check block num of pool is latest
  if block_num <= pool.last_update_block {
    info!("update_pool out");
    return Ok(());
  }

  //2 check allocated has been minted
  if pool_minted >= pool_dmax {
    pool.last_update_block = block_num;
    info!("update_pool out");
    return Ok(());
  }

  //3 pool type: fixed and pool, for calculating accRewardPerShare
  let mut reward_per_token_stored = Num::zero();
  let multiplier_earn = erate.checked_mul(&nums)?.checked_mul(&get_muliplier())?;
  let mut multiplier_mint = Num::zero();
  if pool.ptype == PoolType::Fixed {
    reward_per_token_stored = multiplier_earn.clone();
    let base = get_staked_decimal_base(staked_decimal)?;
    multiplier_mint = Into::<Num>::into(pool.staked)
      .checked_mul(&reward_per_token_stored)?
      .checked_div(&base)?;
  } else if pool.ptype == PoolType::Pool && pool.staked != 0 {
    reward_per_token_stored = multiplier_earn.checked_div(&pool_stake)?;
    multiplier_mint = multiplier_earn.clone();
  }

  pool.acc_reward_per_share = reward_per_token_stored
    .checked_add(&acc_reward_per_share)?
    .truncate_to_u128()?;

  //4 update latest block num
  pool.last_update_block = block_num;

  //5 update minted
  pool.minted = pool_minted
    .checked_mul(&get_muliplier())?
    .checked_add(&multiplier_mint)?
    .checked_div(&get_muliplier())?
    .truncate_to_u128()?;

  info!(
    "  staked_decimal:{}, acc_reward_per_share:{}, reward_per_token_stored:{}, multiplier_earn:{}, multiplier_mint:{}",
    staked_decimal, pool.acc_reward_per_share, reward_per_token_stored, multiplier_earn, multiplier_mint
  );

  info!("update_pool out");
  return Ok(());
}

// need to save pool_info and user_info, when call success
pub fn withdraw_user_reward(
  user: &mut UserInfo,
  pool: &mut PoolInfo,
  staked_decimal: u8,
) -> Result<u128, BRC30Error> {
  info!("withdraw_user_reward in");
  let user_staked = Into::<Num>::into(user.staked);
  let acc_reward_per_share = Into::<Num>::into(pool.acc_reward_per_share);
  let reward_debt = Into::<Num>::into(user.reward_debt);
  let user_reward = Into::<Num>::into(user.reward);
  info!("  user{:?}", user);
  info!("  pool{:?}", pool);

  //1 check user's staked gt 0
  if user_staked <= Num::zero() {
    info!("withdraw_user_reward out");
    return Err(BRC30Error::NoStaked(
      user.pid.to_lowercase().as_str().to_string(),
    ));
  }

  //2 reward = staked * accRewardPerShare - user reward_debt
  let mut reward = Num::zero();
  if pool.ptype == PoolType::Fixed {
    reward = user_staked
      .checked_mul(&acc_reward_per_share)?
      .checked_div(&get_staked_decimal_base(staked_decimal)?)?
      .checked_sub(&reward_debt)?;
  } else if pool.ptype == PoolType::Pool && pool.staked != 0 {
    reward = user_staked
      .checked_mul(&acc_reward_per_share)?
      .checked_sub(&reward_debt)?;
  } else {
    info!("withdraw_user_reward out");
    return Err(BRC30Error::UnknownPoolType);
  }

  reward = reward.checked_div(&get_muliplier())?;

  if reward > Num::zero() {
    //3 update minted of user_info and pool
    user.reward = user_reward.checked_add(&reward)?.truncate_to_u128()?;
  }

  info!("  reward:{}", reward.clone());

  info!("withdraw_user_reward out");
  return reward.truncate_to_u128();
}

// need to update staked  before, and save pool_info and user_info when call success
pub fn update_user_stake(
  user: &mut UserInfo,
  pool: &PoolInfo,
  staked_decimal: u8,
) -> Result<(), BRC30Error> {
  info!("update_user_stake in");
  let user_staked = Into::<Num>::into(user.staked);
  let acc_reward_per_share = Into::<Num>::into(pool.acc_reward_per_share);
  info!("  user{:?}", user);
  info!("  pool{:?}", pool);

  //1 update user's reward_debt
  if pool.ptype == PoolType::Fixed {
    user.reward_debt = user_staked
      .checked_mul(&acc_reward_per_share)?
      .checked_div(&get_staked_decimal_base(staked_decimal)?)?
      .truncate_to_u128()?;
  } else if pool.ptype == PoolType::Pool {
    user.reward_debt = user_staked
      .checked_mul(&acc_reward_per_share)?
      .truncate_to_u128()?;
  } else {
    info!("update_user_stake out");
    return Err(BRC30Error::UnknownPoolType);
  }

  user.latest_updated_block = pool.last_update_block;

  info!("  reward_debt:{}", user.reward_debt.clone());

  info!("update_user_stake out");
  return Ok(());
}

fn get_muliplier() -> Num {
  Num::from(REWARD_PER_DECIMAL)
}

fn get_staked_decimal_base(staked_decimal: u8) -> Result<Num, BRC30Error> {
  BIGDECIMAL_TEN.checked_powu(staked_decimal as u64)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::okx::datastore::brc30::{Pid, PledgedTick, PoolInfo, PoolType, UserInfo};
  use crate::InscriptionId;
  use std::str::FromStr;
  const STAKED_DECIMAL: u8 = 18;
  const ERATE_DECIMAL: u8 = 18;

  #[test]
  fn test_hello() {
    let stake_base = BIGDECIMAL_TEN
      .checked_powu(STAKED_DECIMAL as u64)
      .unwrap()
      .truncate_to_u128()
      .unwrap();
    let erate_base = BIGDECIMAL_TEN
      .checked_powu(ERATE_DECIMAL as u64)
      .unwrap()
      .truncate_to_u128()
      .unwrap();

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(
      &pid.clone(),
      PoolType::Fixed,
      1 * erate_base,
      100 * erate_base,
    );
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
  fn test_fix_one_user() {
    let base = BIGDECIMAL_TEN
      .checked_powu(STAKED_DECIMAL as u64)
      .unwrap()
      .truncate_to_u128()
      .unwrap();
    let dmax = 1000;
    let erate = 100;

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
    );

    // case-3-A deposit 10
    do_one_case(
      &mut user,
      &mut pool,
      3,
      9,
      true,
      0,
      10,
      10,
      0,
      Ok((0)),
      Ok(()),
    );

    //case-4-A same block
    do_one_case(
      &mut user,
      &mut pool,
      3,
      0,
      true,
      0,
      10,
      10,
      0,
      Ok((0)),
      Ok(()),
    );

    //case-5-A  jump block
    do_one_case(
      &mut user,
      &mut pool,
      10,
      0,
      true,
      7,
      10,
      10,
      7,
      Ok((7)),
      Ok(()),
    );

    //case-6-A deposit 90
    do_one_case(
      &mut user,
      &mut pool,
      11,
      90,
      true,
      8,
      100,
      100,
      8,
      Ok((1)),
      Ok(()),
    );

    //case-7-A withdraw 10
    do_one_case(
      &mut user,
      &mut pool,
      12,
      10,
      false,
      18,
      90,
      90,
      18,
      Ok((10)),
      Ok(()),
    );

    //case-8-A withdraw 10, jump block
    do_one_case(
      &mut user,
      &mut pool,
      20,
      10,
      false,
      90,
      80,
      80,
      90,
      Ok((72)),
      Ok(()),
    );

    //case-9-A withdraw 70
    do_one_case(
      &mut user,
      &mut pool,
      21,
      70,
      false,
      98,
      10,
      10,
      98,
      Ok((8)),
      Ok(()),
    );

    //case-10-A ,same block
    do_one_case(
      &mut user,
      &mut pool,
      21,
      0,
      false,
      98,
      10,
      10,
      98,
      Ok((0)),
      Ok(()),
    );

    //case-11-A withdraw 9
    do_one_case(
      &mut user,
      &mut pool,
      22,
      9,
      false,
      99,
      1,
      1,
      99,
      Ok((1)),
      Ok(()),
    );

    //case-12-A withdraw  1
    do_one_case(
      &mut user,
      &mut pool,
      23,
      1,
      false,
      99,
      0,
      0,
      99,
      Ok((0)),
      Ok(()),
    );

    //case-13-A do nothing
    do_one_case(
      &mut user,
      &mut pool,
      24,
      0,
      false,
      99,
      0,
      0,
      99,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
    );

    //case-14-A deposit 100, jump block
    do_one_case(
      &mut user,
      &mut pool,
      50,
      100,
      true,
      99,
      100,
      100,
      99,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
    );

    //case-15-A mint, jump block
    do_one_case(
      &mut user,
      &mut pool,
      100,
      0,
      true,
      599,
      100,
      100,
      599,
      Ok((500)),
      Ok(()),
    );

    //case-16-A mint, same block
    do_one_case(
      &mut user,
      &mut pool,
      100,
      0,
      true,
      599,
      100,
      100,
      599,
      Ok((0)),
      Ok(()),
    );

    //case-17-A mint, jump block
    do_one_case(
      &mut user,
      &mut pool,
      200,
      0,
      true,
      1599,
      100,
      100,
      1599,
      Ok((1000)),
      Ok(()),
    );

    //case-18-A mint
    do_one_case(
      &mut user,
      &mut pool,
      201,
      0,
      true,
      1599,
      100,
      100,
      1599,
      Ok((0)),
      Ok(()),
    );

    //case-19-A mint, jump block
    do_one_case(
      &mut user,
      &mut pool,
      300,
      0,
      true,
      1599,
      100,
      100,
      1599,
      Ok((0)),
      Ok(()),
    );
  }

  #[test]
  fn test_fix_three_user() {
    let base = BIGDECIMAL_TEN
      .checked_powu(STAKED_DECIMAL as u64)
      .unwrap()
      .truncate_to_u128()
      .unwrap();
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
      Ok((30)),
      Ok(()),
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
      Ok((0)),
      Ok(()),
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
      Ok((20)),
      Ok(()),
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
      Ok((0)),
      Ok(()),
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
      Ok((0)),
      Ok(()),
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
      Ok((30)),
      Ok(()),
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
      Ok((20)),
      Ok(()),
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
      Ok((30)),
      Ok(()),
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
      Ok((10)),
      Ok(()),
    );

    // todo go on
  }

  #[test]
  fn test_pool_one_user() {
    let base = BIGDECIMAL_TEN
      .checked_powu(STAKED_DECIMAL as u64)
      .unwrap()
      .truncate_to_u128()
      .unwrap();
    let dmax = 10000;
    let erate = 100;

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
    );

    // case-3-A deposit 10
    do_one_case(
      &mut user,
      &mut pool,
      3,
      9,
      true,
      100,
      10,
      10,
      100,
      Ok((100)),
      Ok(()),
    );

    //case-4-A same block
    do_one_case(
      &mut user,
      &mut pool,
      3,
      0,
      true,
      100,
      10,
      10,
      100,
      Ok((0)),
      Ok(()),
    );

    //case-5-A  jump block
    do_one_case(
      &mut user,
      &mut pool,
      10,
      0,
      true,
      800,
      10,
      10,
      800,
      Ok((700)),
      Ok(()),
    );

    //case-6-A deposit 90
    do_one_case(
      &mut user,
      &mut pool,
      11,
      90,
      true,
      900,
      100,
      100,
      900,
      Ok((100)),
      Ok(()),
    );

    //case-7-A withdraw 10
    do_one_case(
      &mut user,
      &mut pool,
      12,
      10,
      false,
      1000,
      90,
      90,
      1000,
      Ok((100)),
      Ok(()),
    );

    //case-8-A withdraw 10, jump block
    do_one_case(
      &mut user,
      &mut pool,
      20,
      10,
      false,
      1799,
      80,
      80,
      1800,
      Ok((799)),
      Ok(()),
    );

    //case-9-A withdraw 70
    do_one_case(
      &mut user,
      &mut pool,
      21,
      70,
      false,
      1899,
      10,
      10,
      1900,
      Ok((100)),
      Ok(()),
    );

    //case-10-A ,same block
    do_one_case(
      &mut user,
      &mut pool,
      21,
      0,
      false,
      1899,
      10,
      10,
      1900,
      Ok((0)),
      Ok(()),
    );

    //case-11-A withdraw 9
    do_one_case(
      &mut user,
      &mut pool,
      22,
      9,
      false,
      1999,
      1,
      1,
      2000,
      Ok((100)),
      Ok(()),
    );

    //case-12-A withdraw  1
    do_one_case(
      &mut user,
      &mut pool,
      23,
      1,
      false,
      2099,
      0,
      0,
      2100,
      Ok((100)),
      Ok(()),
    );

    //case-13-A do nothing
    do_one_case(
      &mut user,
      &mut pool,
      24,
      0,
      false,
      2099,
      0,
      0,
      2100,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
    );

    //case-14-A deposit 100, jump block
    do_one_case(
      &mut user,
      &mut pool,
      50,
      100,
      true,
      2099,
      100,
      100,
      2100,
      Err(BRC30Error::NoStaked(
        pid.to_lowercase().as_str().to_string(),
      )),
      Ok(()),
    );

    //case-15-A mint, jump block
    do_one_case(
      &mut user,
      &mut pool,
      100,
      0,
      true,
      7099,
      100,
      100,
      7100,
      Ok((5000)),
      Ok(()),
    );

    //case-16-A mint, same block
    do_one_case(
      &mut user,
      &mut pool,
      100,
      0,
      true,
      7099,
      100,
      100,
      7100,
      Ok((0)),
      Ok(()),
    );

    //case-17-A mint, jump block
    do_one_case(
      &mut user,
      &mut pool,
      200,
      0,
      true,
      17099,
      100,
      100,
      17100,
      Ok((10000)),
      Ok(()),
    );

    //case-18-A mint
    do_one_case(
      &mut user,
      &mut pool,
      201,
      0,
      true,
      17099,
      100,
      100,
      17100,
      Ok((0)),
      Ok(()),
    );

    //case-19-A mint, jump block
    do_one_case(
      &mut user,
      &mut pool,
      300,
      0,
      true,
      17099,
      100,
      100,
      17100,
      Ok((0)),
      Ok(()),
    );
  }

  #[test]
  fn test_pool_three_user() {
    let base = BIGDECIMAL_TEN
      .checked_powu(STAKED_DECIMAL as u64)
      .unwrap()
      .truncate_to_u128()
      .unwrap();
    let dmax = 1000;
    let erate = 100;

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
      Ok((183)),
      Ok(()),
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
      Ok((0)),
      Ok(()),
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
      Ok((83)),
      Ok(()),
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
      Ok((0)),
      Ok(()),
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
      Ok((0)),
      Ok(()),
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
      Ok((75)),
      Ok(()),
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
      Ok((58)),
      Ok(()),
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
      Ok((75)),
      Ok(()),
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
      Ok((25)),
      Ok(()),
    );

    // todo go on
  }

  #[test]
  fn test_pool_one_user_18() {
    // let base = BIGDECIMAL_TEN
    //   .checked_powu(18 as u64)
    //   .unwrap()
    //   .truncate_to_u128()
    //   .unwrap();
    // let dmax = 10000;
    // let erate = 100;
    //
    // let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    // let mut pool = new_pool(&pid.clone(), PoolType::Pool, erate, dmax);
    // let mut user = new_user(&pid);
    //
    // let mut case;
    //
    // // case-1-A deposit 0
    // case = Case::new(
    //   1,
    //   0,
    //   true,
    //   0,
    //   0,
    //   0,
    //   0,
    //   Err(BRC30Error::NoStaked(user.pid.to_lowercase().as_str().to_string())),
    //   Ok(()),
    // );
    // do_one_case(&mut user, &mut pool, &case);
    // // case-2-A deposit 1
    // case = Case::new(
    //   2,
    //   100000,
    //   true,
    //   0,
    //   100000,
    //   100000,
    //   0,
    //   Err(BRC30Error::NoStaked(user.pid.to_lowercase().as_str().to_string())),
    //   Ok(()),
    // );
    // do_one_case(&mut user, &mut pool, &case);
    //
    // // case-3-A deposit 10
    // case = Case::new(
    //   3,
    //   900000,
    //   true,
    //   100,
    //   1000000,
    //   1000000,
    //   100,
    //   Ok((100)),
    //   Ok(()),
    // );
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-4-A same block
    // case = Case::new(3, 0, true, 100, 10000000, 10000000, 100, Ok((0)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);

    // //case-5-A  jump block
    // case = Case::new(10, 0, true, 800, 10, 10, 800, Ok((700)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-6-A deposit 90
    // case = Case::new(11, 90, true, 900, 100, 100, 900, Ok((100)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-7-A withdraw 10
    // case = Case::new(12, 10, false, 1000, 90, 90, 1000, Ok((100)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-8-A withdraw 10, jump block
    // case = Case::new(20, 10, false, 1799, 80, 80, 1800, Ok((799)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-9-A withdraw 70
    // case = Case::new(21, 70, false, 1899, 10, 10, 1900, Ok((100)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-10-A ,same block
    // case = Case::new(21, 0, false, 1899, 10, 10, 1900, Ok((0)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-11-A withdraw 9
    // case = Case::new(22, 9, false, 1999, 1, 1, 2000, Ok((100)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-12-A withdraw  1
    // case = Case::new(23, 1, false, 2099, 0, 0, 2100, Ok((100)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-13-A do nothing
    // case = Case::new(
    //   24,
    //   0,
    //   false,
    //   2099,
    //   0,
    //   0,
    //   2100,
    //   Err(BRC30Error::NoStaked(user.pid.to_lowercase().as_str().to_string())),
    //   Ok(()),
    // );
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-14-A deposit 100, jump block
    // case = Case::new(
    //   50,
    //   100,
    //   true,
    //   2099,
    //   100,
    //   100,
    //   2100,
    //   Err(BRC30Error::NoStaked(user.pid.to_lowercase().as_str().to_string())),
    //   Ok(()),
    // );
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-15-A mint, jump block
    // case = Case::new(100, 0, true, 7099, 100, 100, 7100, Ok((5000)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-16-A mint, same block
    // case = Case::new(100, 0, true, 7099, 100, 100, 7100, Ok((0)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-17-A mint, jump block
    // case = Case::new(200, 0, true, 17099, 100, 100, 17100, Ok((10000)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-18-A mint
    // case = Case::new(201, 0, true, 17099, 100, 100, 17100, Ok((0)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);
    //
    // //case-19-A mint, jump block
    // case = Case::new(300, 0, true, 17099, 100, 100, 17100, Ok((0)), Ok(()));
    // do_one_case(&mut user, &mut pool, &case);
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
  ) {
    assert_eq!(update_pool(pool, block_mum, STAKED_DECIMAL), Ok(()));

    let result = withdraw_user_reward(user, pool, STAKED_DECIMAL);
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
    let u_result = update_user_stake(user, pool, STAKED_DECIMAL);
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

  fn new_user(pid: &Pid) -> UserInfo {
    UserInfo {
      pid: pid.clone(),
      staked: 0,
      reward: 0,
      reward_debt: 0,
      latest_updated_block: 0,
    }
  }
}
