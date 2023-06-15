use super::error::RewardError;
use crate::okx::datastore::ScriptKey;
use crate::okx::datastore::BRC30::{
  BRC30DataStoreReadOnly, BRC30DataStoreReadWrite, Balance, InscriptionOperation, Pid, PledgedTick,
  PoolInfo, Receipt, StakeInfo, TickId, TickInfo, TransferableAsset, UserInfo,
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
    //1. 判断用户的 staked 是否大于0
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

    //2. 根据用户的质押数计算收益： staked * accRewardPerShare - user reward_debt，并存储
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
      //3 更新 user_info 和pool 的 minted，TODO check overflow
      user.reward += reward;
      pool.minted += reward;
      self
        .brc30db
        .set_pid_to_use_info(&script_key.clone(), &pid.clone(), &user);
      self.brc30db.set_pid_to_poolinfo(&pid.clone(), &pool);
    }

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
  use redb::Database;
  use std::str::FromStr;
  use tempfile::NamedTempFile;
  #[test]
  fn test_flow() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();

    let mut pool = Pool::new(&wtx);

    let pid = Pid::from_str("Bca1DaBca1D#1").unwrap();
    let script_key =
      ScriptKey::from_address(Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap());

    // 1.from deposit, withdraw

    // 2.update_pool
    // pool.update_pool(&pid, 0u64);

    // 2.if no stake, return 0
    // pool.withdraw_user_reward(&pid, &script_key);

    // 3. calculate and update the user's new stake.
    // pool.update_user_stake(&pid, &script_key, 0u128, true);
  }
}
