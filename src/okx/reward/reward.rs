use crate::okx::datastore::BRC30::{PoolInfo, PoolType, UserInfo};
use crate::okx::protocol::BRC30::{params::BIGDECIMAL_TEN, BRC30Error, Num};

// demo
// | Pool type | earn rate | total stake      | user stake     | block | reward                                        |
// |-----------|-----------|------------------|----------------|-------|-----------------------------------------------|
// | Fix       |  100(1e2) | 10000(1e3)       | 2000(1e3)      | 1     | 2000/1e3 * 100 * 1 = 200  (need stake's DECIMAL)  |
// | Pool      |  100(1e2) | 10000(1e3)       | 2000(1e3)      | 1     | 2000 * 100 / 10000 =  20                          |

// TODO need add stake's decimal when it's fixed type
const STAKED_DECIMAL: u8 = 3;
const EARN_DECIMAL: u8 = 2;

pub fn query_reward(user: UserInfo, pool: PoolInfo, block_num: u64) -> Result<u128, BRC30Error> {
  let mut user_temp = user;
  let mut pool_temp = pool;
  update_pool(&mut pool_temp, block_num)?;
  return withdraw_user_reward(&mut user_temp, &mut pool_temp);
}

// need to save pool_info, when call success
pub fn update_pool(pool: &mut PoolInfo, block_num: u64) -> Result<(), BRC30Error> {
  let pool_minted = Into::<Num>::into(pool.minted);
  let pool_dmax = Into::<Num>::into(pool.dmax);
  let nums = Into::<Num>::into(block_num - pool.last_update_block);
  let erate = Into::<Num>::into(pool.erate);
  let pool_stake = Into::<Num>::into(pool.staked);
  let acc_reward_per_share = Into::<Num>::into(pool.acc_reward_per_share);

  //1 check block num of pool is latest
  if block_num <= pool.last_update_block {
    return Ok(());
  }

  //2 check allocated has been minted
  if pool_minted >= pool_dmax {
    pool.last_update_block = block_num;
    return Ok(());
  }

  //3 pool type: fixed and pool, for calculating accRewardPerShare
  let mut reward_per_token_stored = Num::zero();
  if pool.ptype == PoolType::Fixed {
    reward_per_token_stored = erate.checked_mul(&nums)?;
  } else if pool.ptype == PoolType::Pool && pool.staked != 0 {
    reward_per_token_stored = erate.checked_mul(&nums)?.checked_div(&pool_stake)?;
  } else {
    return Err(BRC30Error::UnknownPoolType);
  }

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
  let user_staked = Into::<Num>::into(user.staked);
  let acc_reward_per_share = Into::<Num>::into(pool.acc_reward_per_share);
  let reward_debt = Into::<Num>::into(user.reward_debt);
  let user_reward = Into::<Num>::into(user.reward);
  let pool_minted = Into::<Num>::into(pool.minted);

  //1 check user's staked gt 0
  if user_staked <= Num::zero() {
    return Err(BRC30Error::NoStaked(user.pid.to_lowercase().hex()));
  }

  //2 reward = staked * accRewardPerShare - user reward_debt
  let mut reward = Num::zero();
  if pool.ptype == PoolType::Fixed {
    let base = BIGDECIMAL_TEN.checked_powu(STAKED_DECIMAL as u64)?;
    println!("withdraw_user_reward-fixed-staked:{}, acc_reward_per_share:{}", user_staked, acc_reward_per_share);
    let a = user_staked.checked_mul(&acc_reward_per_share)?;
    println!("withdraw_user_reward-fixed-(stake * per_share):{}", a);
    let b = a.checked_div(&base)?;
    println!("withdraw_user_reward-fixed-(stake * per_share / base):{}", b);
    reward = b.checked_sub(&reward_debt)?;
    println!("withdraw_user_reward-fixed-reward:{}, reward_debt:{}", reward, reward_debt);
  } else if pool.ptype == PoolType::Pool && pool.staked != 0 {
    reward = user_staked
      .checked_mul(&acc_reward_per_share)?
      .checked_sub(&reward_debt)?;
  } else {
    return Err(BRC30Error::UnknownPoolType);
  }

  if reward > Num::zero() {
    //3 update minted of user_info and pool
    user.reward = user_reward.checked_add(&reward)?.truncate_to_u128()?;
    pool.minted = pool_minted.checked_add(&reward)?.truncate_to_u128()?;
  }

  println!(
      "withdraw_user_reward-reward:{}, user.reward_debt:{}, user.staked:{}, pool.acc_reward_per_share:{}",
      reward, user.reward_debt, user.staked, pool.acc_reward_per_share
    );

  return reward.truncate_to_u128();
}

// need to update staked  before, and save pool_info and user_info when call success
pub fn update_user_stake(user: &mut UserInfo, pool: &PoolInfo) -> Result<(), BRC30Error> {
  let user_staked = Into::<Num>::into(user.staked);
  let acc_reward_per_share = Into::<Num>::into(pool.acc_reward_per_share);
  //1 update user's reward_debt
  if pool.ptype == PoolType::Fixed {
    let base = BIGDECIMAL_TEN.checked_powu(STAKED_DECIMAL as u64)?;
    println!("update_user_stake-staked:{}, acc_reward_per_share:{}", user_staked, acc_reward_per_share);
    let a = user_staked.checked_mul(&acc_reward_per_share)?;
    println!("update_user_stake-(stake * per share)):{}", a);
    let b = a.checked_div(&base)?;
    println!("update_user_stake-(stake * per share / base):{}", b);
    user.reward_debt = b.truncate_to_u128()?;
    println!("update_user_stake-reward_debt:{}", user.reward_debt);
  } else if pool.ptype == PoolType::Pool && pool.staked != 0 {
    user.reward_debt = user_staked
      .checked_mul(&acc_reward_per_share)?
      .truncate_to_u128()?;
  } else {
    return Err(BRC30Error::UnknownPoolType);
  }

  println!(
    "update_user_stake--reward_debt:{}, user staked:{}, acc_reward_per_share:{}, pool staked:{}",
    user.reward_debt, user.staked, pool.acc_reward_per_share, pool.staked
  );
  return Ok(());
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::okx::datastore::BRC30::{Pid, PledgedTick, PoolInfo, PoolType, UserInfo};
  use crate::InscriptionId;
  use std::str::FromStr;

  #[test]
  fn test_hello() {
    let base = BIGDECIMAL_TEN
      .checked_powu(STAKED_DECIMAL as u64)
      .unwrap()
      .truncate_to_u128()
      .unwrap();

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let mut pool = new_pool(&pid.clone(), PoolType::Fixed, 10, 100000000000);
    let mut user = new_user(&pid);

    //stake, no reward
    {
      assert_eq!(update_pool(&mut pool, 1), Ok(()));
      assert_eq!(
        withdraw_user_reward(&mut user, &mut pool).expect_err(""),
        BRC30Error::NoStaked("62636131646162636131642331".to_string())
      );
      user.staked += 2 * base;
      pool.staked += 2 * base;
      assert_eq!(update_user_stake(&mut user, &mut pool), Ok(()));
    }

    //withdraw, has reward
    {
      assert_eq!(update_pool(&mut pool, 2), Ok(()));
      assert_eq!(withdraw_user_reward(&mut user, &mut pool).unwrap(), 20);
      user.staked -= 1 * base;
      pool.staked -= 1 * base;
      assert_eq!(update_user_stake(&mut user, &mut pool), Ok(()));
    }

    // query reward
    {
      assert_eq!(query_reward(user, pool, 3).unwrap(), 10);
    }
  }

  struct Case {
    block_mum: u64,
    stake_alter: u128,
    is_add: bool,
    expect_user_new_reward: u128,
    expect_user_remain_reward: u128,
    expert_user_staked: u128,
    expect_pool_staked: u128,
    expect_pool_minted: u128,
    expect_withdraw_reward_result: Result<u128, BRC30Error>,
    expect_update_stake_result: Result<(), BRC30Error>,
  }

  impl Case {
    pub fn new(
      block_mum: u64,
      stake_alter: u128,
      is_add: bool,
      expect_user_new_reward: u128,
      expect_user_remain_reward: u128,
      expert_user_staked: u128,
      expect_pool_staked: u128,
      expect_pool_minted: u128,
      expect_withdraw_reward_result: Result<u128, BRC30Error>,
      expect_update_stake_result: Result<(), BRC30Error>,
    ) -> Self {
      Self {
        block_mum,
        stake_alter,
        is_add,
        expect_user_new_reward,
        expect_user_remain_reward,
        expert_user_staked,
        expect_pool_staked,
        expect_pool_minted,
        expect_withdraw_reward_result,
        expect_update_stake_result,
      }
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

    let mut case;

    // case-1-A deposit 0
    case = Case::new(1, 0, true,
                0, 0, 0,
                0, 0,
                     Err(BRC30Error::NoStaked(user.pid.to_lowercase().hex())),
                     Ok(()));
    do_one_case(&mut user, &mut pool, &case);
    // case-2-A deposit 1
    case = Case::new(2, 1, true,
              0, 0, 1,
              1, 0,
                     Err(BRC30Error::NoStaked(user.pid.to_lowercase().hex())),
                     Err(BRC30Error::InvalidInteger(Num::from_str("0.200").unwrap())));
    do_one_case(&mut user, &mut pool, &case);

    // case-3-A deposit 10
    case = Case::new(3, 9, true,
                     0, 0, 10,
                     10, 0,
                     Ok((0)),
                     Ok(()));
    do_one_case(&mut user, &mut pool, &case);

    //case-4-A same block
    case = Case::new(3, 0, true,
                     0, 0, 10,
                     10, 0,
                     Ok((0)),
                     Ok(()));
    do_one_case(&mut user, &mut pool, &case);


    //case-5-A  jump block
    case = Case::new(10, 0, true,
                     7, 7, 10,
                     10, 7,
                     Ok((7)),
                     Ok(()));
    do_one_case(&mut user, &mut pool, &case);

    //case-6-A deposit 90
    case = Case::new(11, 90, true,
                     1, 8, 100,
                     100, 8,
                     Ok((1)),
                     Ok(()));
    do_one_case(&mut user, &mut pool, &case);

    //case-7-A withdraw 10
    case = Case::new(12, 10, false,
                     10, 18, 90,
                     90, 18,
                     Ok((10)),
                     Ok(()));
    do_one_case(&mut user, &mut pool, &case);

    //case-8-A withdraw 10, jump block
    case = Case::new(20, 10, false,
                     72, 90, 80,
                     80, 90,
                     Ok((72)),
                     Ok(()));
    do_one_case(&mut user, &mut pool, &case);

    //case-9-A withdraw 70
    case = Case::new(21, 70, false,
                     8, 98, 10,
                     10, 98,
                     Ok((8)),
                     Ok(()));
    do_one_case(&mut user, &mut pool, &case);

    //case-10-A ,same block
    case = Case::new(21, 0, false,
                     0, 98, 10,
                     10, 98,
                     Ok((0)),
                     Ok(()));
    do_one_case(&mut user, &mut pool, &case);

    //case-11-A withdraw 9
    case = Case::new(22, 9, false,
                     1, 99, 1,
                     1, 99,
                     Ok((1)),
                     Ok(()));
    do_one_case(&mut user, &mut pool, &case);



  }

  fn do_one_case(
    user: &mut UserInfo,
    pool: &mut PoolInfo,
    case: &Case,
  ) {
    assert_eq!(update_pool(pool, case.block_mum), Ok(()));

    let result = withdraw_user_reward(user, pool);
    match result {
      Ok(value)=>{
        assert_eq!(value, case.expect_withdraw_reward_result.clone().unwrap());
      },
      Err(err) =>{
        println!("err:{:?}", err);
        assert_eq!(err, case.expect_withdraw_reward_result.clone().unwrap_err())
      }
    }

    if case.is_add {
      user.staked += case.stake_alter;
      pool.staked += case.stake_alter;
    } else {
      user.staked -= case.stake_alter;
      pool.staked -= case.stake_alter;
    }
    let u_result = update_user_stake(user, pool);
    match u_result {
      Ok(value)=>{
      },
      Err(err) =>{
        println!("err:{:?}", err);
        assert_eq!(err, case.expect_update_stake_result.clone().unwrap_err())
      }
    }

    assert_eq!(user.reward, case.expect_user_remain_reward);
    assert_eq!(user.staked, case.expert_user_staked);
    assert_eq!(pool.staked, case.expect_pool_staked);
    assert_eq!(pool.minted, case.expect_pool_minted);
    assert_eq!(pool.last_update_block, case.block_mum);
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
