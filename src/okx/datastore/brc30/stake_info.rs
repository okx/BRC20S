use super::*;
use crate::okx::datastore::brc30::PledgedTick;
use crate::okx::protocol::brc30::params::ZERO_NUM;
use crate::okx::protocol::brc30::{BRC30Error, Num};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct StakeInfo {
  pub stake: PledgedTick,
  pub pool_stakes: Vec<(Pid, bool, u128)>,
  pub max_share: u128,
  pub total_only: u128,
}

impl StakeInfo {
  pub fn new(
    pool_stakes: &Vec<(Pid, bool, u128)>,
    stake: &PledgedTick,
    max_share: u128,
    total_only: u128,
  ) -> Self {
    Self {
      stake: stake.clone(),
      pool_stakes: pool_stakes.clone(),
      max_share,
      total_only,
    }
  }

  pub fn calculate_max_share(&self) -> Result<Num, BRC30Error> {
    let mut staked_max_share = Num::from(0_u128);
    for (_, only, pool_stake) in self.pool_stakes.clone() {
      let current_pool_stake = Num::from(pool_stake);
      if !only && current_pool_stake.gt(&staked_max_share) {
        staked_max_share = current_pool_stake;
      }
    }
    Ok(staked_max_share)
  }

  pub fn calculate_withdraw_pools(
    &self,
    stake_alterive: &Num,
  ) -> Result<Vec<(Pid, u128)>, BRC30Error> {
    let mut max_share_alter = Num::from(0_u128); // the max share alter of pools
    let mut total_only = Num::from(0_u128); // the total only alter of pools
    let mut pids: Vec<(Pid, u128)> = Vec::new();
    for (pid, only, pool_stake) in self.pool_stakes.iter() {
      let current_staked = max_share_alter.checked_add(&total_only)?; // the sum stake of share and only pools
      let pool_stake_num = Num::from(*pool_stake);
      if current_staked.ge(&stake_alterive) {
        // if current_stake > stake_alterive, then only change share pool
        if !*only && !max_share_alter.eq(&ZERO_NUM) {
          if max_share_alter.gt(&pool_stake_num) {
            pids.push((pid.clone(), pool_stake_num.checked_to_u128()?))
          } else {
            pids.push((pid.clone(), max_share_alter.checked_to_u128()?))
          }
        }
      } else {
        if *only {
          let remain = stake_alterive.checked_sub(&current_staked)?;
          if remain.gt(&pool_stake_num) {
            total_only = total_only.checked_add(&pool_stake_num)?;
            pids.push((pid.clone(), pool_stake_num.checked_to_u128()?));
          } else {
            total_only = total_only.checked_add(&remain)?;
            pids.push((pid.clone(), remain.checked_to_u128()?));
          }
        } else {
          let remain = stake_alterive.checked_sub(&total_only)?;
          if remain.gt(&pool_stake_num) {
            max_share_alter = Num::max(&max_share_alter, &pool_stake_num);
            pids.push((pid.clone(), pool_stake_num.checked_to_u128()?));
          } else {
            max_share_alter = Num::max(&max_share_alter, &remain);
            pids.push((pid.clone(), remain.checked_to_u128()?));
          }
        }
      }
    }
    Ok(pids)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::okx::protocol::brc30::Num;
  use std::str::FromStr;
  // testcase: the 3 pools can represent all case
  // a        b           c
  //============================(a,b,c:pool 0:share 1:only)
  // 0        0           0
  // 0        0           1
  // 0        1           0
  // 1        0           0
  // 0        1           1
  // 1        1           0
  // 1        0           1
  // 1        1           1
  #[test]
  fn test_calculate_withdraw_pools_000() {
    let mut stake_info = StakeInfo::new(&vec![], &PledgedTick::Unknown, 0, 0);
    let pid1 = Pid::from_str("0000000000#01").unwrap();
    let pid2 = Pid::from_str("0000000000#02").unwrap();
    let pid3 = Pid::from_str("0000000000#03").unwrap();
    stake_info.pool_stakes.push((pid1.clone(), false, 10));
    stake_info.pool_stakes.push((pid2.clone(), false, 20));
    stake_info.pool_stakes.push((pid3.clone(), false, 30));
    {
      //change is less than first pool
      let change = Num::from(5_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 5), (pid2.clone(), 5), (pid3.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than first pool less than second
      let change = Num::from(15_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 15), (pid3.clone(), 15)];
      assert_eq!(result, expect);
    }

    {
      //change is more than second pool less than third
      let change = Num::from(25_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 25)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(35_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 30)];
      assert_eq!(result, expect);
    }
  }

  #[test]
  fn test_calculate_withdraw_pools_001() {
    let mut stake_info = StakeInfo::new(&vec![], &PledgedTick::Unknown, 0, 0);
    let pid1 = Pid::from_str("0000000000#01").unwrap();
    let pid2 = Pid::from_str("0000000000#02").unwrap();
    let pid3 = Pid::from_str("0000000000#03").unwrap();
    stake_info.pool_stakes.push((pid1.clone(), false, 10));
    stake_info.pool_stakes.push((pid2.clone(), false, 20));
    stake_info.pool_stakes.push((pid3.clone(), true, 30));
    {
      //change is less than first pool
      let change = Num::from(5_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 5), (pid2.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than first pool less than second
      let change = Num::from(15_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 10), (pid2.clone(), 15)];
      assert_eq!(result, expect);
    }

    {
      //change is more than second pool less than third
      let change = Num::from(25_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(35_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 15)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(45_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 25)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(55_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 30)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(65_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 30)];
      assert_eq!(result, expect);
    }
  }

  #[test]
  fn test_calculate_withdraw_pools_010() {
    let mut stake_info = StakeInfo::new(&vec![], &PledgedTick::Unknown, 0, 0);
    let pid1 = Pid::from_str("0000000000#01").unwrap();
    let pid2 = Pid::from_str("0000000000#02").unwrap();
    let pid3 = Pid::from_str("0000000000#03").unwrap();
    stake_info.pool_stakes.push((pid1.clone(), false, 10));
    stake_info.pool_stakes.push((pid2.clone(), true, 20));
    stake_info.pool_stakes.push((pid3.clone(), false, 30));
    {
      //change is less than first pool
      let change = Num::from(5_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 5), (pid3.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than first pool less than second
      let change = Num::from(15_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 5), (pid3.clone(), 10)];
      assert_eq!(result, expect);
    }

    {
      //change is more than second pool less than third
      let change = Num::from(25_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 15), (pid3.clone(), 10)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(35_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 15)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(45_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 25)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(55_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 30)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(65_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 30)];
      assert_eq!(result, expect);
    }
  }

  #[test]
  fn test_calculate_withdraw_pools_100() {
    let mut stake_info = StakeInfo::new(&vec![], &PledgedTick::Unknown, 0, 0);
    let pid1 = Pid::from_str("0000000000#01").unwrap();
    let pid2 = Pid::from_str("0000000000#02").unwrap();
    let pid3 = Pid::from_str("0000000000#03").unwrap();
    stake_info.pool_stakes.push((pid1.clone(), true, 10));
    stake_info.pool_stakes.push((pid2.clone(), false, 20));
    stake_info.pool_stakes.push((pid3.clone(), false, 30));
    {
      //change is less than first pool
      let change = Num::from(5_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than first pool less than second
      let change = Num::from(15_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 10), (pid2.clone(), 5), (pid3.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than second pool less than third
      let change = Num::from(25_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 15), (pid3.clone(), 15)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(35_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 25)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(45_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 30)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(55_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 30)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(65_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 30)];
      assert_eq!(result, expect);
    }
  }

  #[test]
  fn test_calculate_withdraw_pools_011() {
    let mut stake_info = StakeInfo::new(&vec![], &PledgedTick::Unknown, 0, 0);
    let pid1 = Pid::from_str("0000000000#01").unwrap();
    let pid2 = Pid::from_str("0000000000#02").unwrap();
    let pid3 = Pid::from_str("0000000000#03").unwrap();
    stake_info.pool_stakes.push((pid1.clone(), false, 10));
    stake_info.pool_stakes.push((pid2.clone(), true, 20));
    stake_info.pool_stakes.push((pid3.clone(), true, 30));
    {
      //change is less than first pool
      let change = Num::from(5_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than first pool less than second
      let change = Num::from(15_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 10), (pid2.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than second pool less than third
      let change = Num::from(25_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 10), (pid2.clone(), 15)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(35_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(45_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 15)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(55_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 25)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(65_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 30)];
      assert_eq!(result, expect);
    }
  }

  #[test]
  fn test_calculate_withdraw_pools_110() {
    let mut stake_info = StakeInfo::new(&vec![], &PledgedTick::Unknown, 0, 0);
    let pid1 = Pid::from_str("0000000000#01").unwrap();
    let pid2 = Pid::from_str("0000000000#02").unwrap();
    let pid3 = Pid::from_str("0000000000#03").unwrap();
    stake_info.pool_stakes.push((pid1.clone(), true, 10));
    stake_info.pool_stakes.push((pid2.clone(), true, 20));
    stake_info.pool_stakes.push((pid3.clone(), false, 30));
    {
      //change is less than first pool
      let change = Num::from(5_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than first pool less than second
      let change = Num::from(15_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 10), (pid2.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than second pool less than third
      let change = Num::from(25_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 10), (pid2.clone(), 15)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(35_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(45_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 15)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(55_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 25)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(65_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 30)];
      assert_eq!(result, expect);
    }
  }

  #[test]
  fn test_calculate_withdraw_pools_101() {
    let mut stake_info = StakeInfo::new(&vec![], &PledgedTick::Unknown, 0, 0);
    let pid1 = Pid::from_str("0000000000#01").unwrap();
    let pid2 = Pid::from_str("0000000000#02").unwrap();
    let pid3 = Pid::from_str("0000000000#03").unwrap();
    stake_info.pool_stakes.push((pid1.clone(), true, 10));
    stake_info.pool_stakes.push((pid2.clone(), false, 20));
    stake_info.pool_stakes.push((pid3.clone(), true, 30));
    {
      //change is less than first pool
      let change = Num::from(5_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than first pool less than second
      let change = Num::from(15_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 10), (pid2.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than second pool less than third
      let change = Num::from(25_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 10), (pid2.clone(), 15)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(35_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(45_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 15)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(55_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 25)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(65_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 30)];
      assert_eq!(result, expect);
    }
  }

  #[test]
  fn test_calculate_withdraw_pools_111() {
    let mut stake_info = StakeInfo::new(&vec![], &PledgedTick::Unknown, 0, 0);
    let pid1 = Pid::from_str("0000000000#01").unwrap();
    let pid2 = Pid::from_str("0000000000#02").unwrap();
    let pid3 = Pid::from_str("0000000000#03").unwrap();
    stake_info.pool_stakes.push((pid1.clone(), true, 10));
    stake_info.pool_stakes.push((pid2.clone(), false, 20));
    stake_info.pool_stakes.push((pid3.clone(), true, 30));
    {
      //change is less than first pool
      let change = Num::from(5_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than first pool less than second
      let change = Num::from(15_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 10), (pid2.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than second pool less than third
      let change = Num::from(25_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> = vec![(pid1.clone(), 10), (pid2.clone(), 15)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(35_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 5)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(45_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 15)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(55_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 25)];
      assert_eq!(result, expect);
    }

    {
      //change is more than third pool
      let change = Num::from(65_u128);
      let result: Vec<(Pid, u128)> = stake_info.calculate_withdraw_pools(&change).unwrap();
      let expect: Vec<(Pid, u128)> =
        vec![(pid1.clone(), 10), (pid2.clone(), 20), (pid3.clone(), 30)];
      assert_eq!(result, expect);
    }
  }
  #[test]
  fn test_remove_withdraw_pools() {
    let mut stake_info = StakeInfo::new(&vec![], &PledgedTick::Unknown, 0, 0);
    let pid1 = Pid::from_str("0000000000#01").unwrap();
    let pid2 = Pid::from_str("0000000000#02").unwrap();
    let pid3 = Pid::from_str("0000000000#03").unwrap();
    stake_info.pool_stakes.push((pid1.clone(), false, 10));
    stake_info.pool_stakes.push((pid2.clone(), false, 0));
    stake_info.pool_stakes.push((pid3.clone(), false, 30));

    for pool_stake in stake_info.pool_stakes.iter_mut() {
      if pool_stake.0 == pid1 {
        pool_stake.2 = 5;
        break;
      }
    }
    stake_info
      .pool_stakes
      .retain(|pool_stake| pool_stake.2 != 0);
    println!("stake_info:{}", serde_json::to_string(&stake_info).unwrap())
  }
}
