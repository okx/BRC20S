use super::error::RewardError;
use crate::okx::datastore::ScriptKey;
use crate::okx::datastore::BRC30::pool_info::Pid;
use std::collections::HashMap;
use std::time;

struct Pool {}

impl Pool {
  pub fn query_reward(&mut self, pid: &Pid, script_key: &ScriptKey) -> (String, u128) {
    //TODO query_reward
    return (script_key.to_string(), 1u128);
  }

  pub fn update_pool(&mut self, pid: &Pid, block_num: u64) -> Result<(), RewardError> {
    // PoolInfo storage pool = poolInfo[_pid];
    // if (block.number <= pool.lastRewardBlock) {
    //   return;
    // }
    // uint256 lpSupply = pool.lpToken.balanceOf(address(this));
    // if (lpSupply == 0) {
    //   pool.lastRewardBlock = block.number;
    //   return;
    // }
    // uint256 multiplier = getMultiplier(pool.lastRewardBlock, block.number);
    // uint256 cakeReward = multiplier
    //   .mul(cakePerBlock)
    //   .mul(pool.allocPoint)
    //   .div(totalAllocPoint);
    // pool.accCakePerShare = pool.accCakePerShare.add(
    //   cakeReward.mul(1e12).div(lpSupply)
    // );
    // pool.lastRewardBlock = block.number;

    //1. 查询pool是否存在

    //2. 查询pool的block number是否为最新

    //3. 判断池子模式，分别为fixed和pool，使用不同的rewardPerTokenStored，计算 accRewardPerShare

    //4. 更新区块高度

    return Ok(());
  }

  pub fn withdraw_user_reward(
    &mut self,
    pid: &Pid,
    script_key: &ScriptKey,
  ) -> Result<u128, RewardError> {
    // return Err(RewardError::InvalidNum("er".to_string()));
    //1. 判断用户的 staked 是否大于0

    //2. 根据用户的质押数计算收益： staked * accRewardPerShare - user reward_debt，并存储

    return Ok(0u128);
  }

  pub fn update_user_stake(
    &mut self,
    pid: &Pid,
    script_key: &ScriptKey,
    stake_alter: u128,
    is_add: bool,
  ) -> Result<(), RewardError> {
    // return Err(RewardError::InvalidNum("er".to_string()));
    //1.判断是否stake_alter是否大于0

    //2.大于0，则进行相应增加或者减少

    //3.更新用户的 reward_debt

    return Ok(());
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use bitcoin::Address;
  use std::str::FromStr;
  #[test]
  fn test_flow() {
    let mut pool = Pool {};

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let script_key =
      ScriptKey::from_address(Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap());

    // 1.from deposit, withdraw

    // 2.update_pool
    pool.update_pool(&pid, 0u64);

    // 2.if no stake, return 0
    pool.withdraw_user_reward(&pid, &script_key);

    // 3. calculate and update the user's new stake.
    pool.update_user_stake(&pid, &script_key, 0u128, true);
  }
}
