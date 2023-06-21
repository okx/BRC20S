use crate::okx::datastore::balance::{
  convert_amount_with_decimal, convert_amount_without_decimal, convert_pledged_tick_with_decimal,
  convert_pledged_tick_without_decimal, get_stake_dec, get_user_common_balance, stake_is_exist,
};
use crate::okx::datastore::brc30::{
  BRC30Event, BRC30Receipt, BRC30Tick, Balance, DeployPoolEvent, DepositEvent,
  InscribeTransferEvent, MintEvent, PassiveWithdrawEvent, Pid, PoolInfo, StakeInfo, TickId,
  TickInfo, TransferEvent, TransferableAsset, UserInfo, WithdrawEvent,
};
use crate::okx::datastore::{BRC20DataStoreReadWrite, BRC30DataStoreReadWrite};
use crate::okx::protocol::brc30::hash::caculate_tick_id;
use crate::okx::protocol::brc30::operation::BRC30Operation;
use crate::okx::protocol::brc30::params::{BIGDECIMAL_TEN, MAX_DECIMAL_WIDTH};
use crate::okx::protocol::brc30::{
  BRC30Error, BRC30Message, Deploy, Error, Mint, Num, PassiveUnStake, Stake, Transfer, UnStake,
};
use crate::okx::reward::reward;
use crate::Result;
use anyhow::anyhow;
use bigdecimal::num_bigint::Sign;
use std::str::FromStr;

pub fn execute<'a, M: BRC20DataStoreReadWrite, N: BRC30DataStoreReadWrite>(
  brc20_store: &'a M,
  brc30_store: &'a N,
  msg: &BRC30Message,
) -> Result<BRC30Receipt> {
  let event = match &msg.op {
    BRC30Operation::Deploy(deploy) => process_deploy(brc20_store, brc30_store, msg, deploy.clone()),
    BRC30Operation::Stake(stake) => process_stake(brc20_store, brc30_store, msg, stake.clone()),
    BRC30Operation::UnStake(unstake) => {
      process_unstake(brc20_store, brc30_store, msg, unstake.clone())
    }
    BRC30Operation::PassiveUnStake(passive_unstake) => {
      process_passive_unstake(brc20_store, brc30_store, msg, passive_unstake.clone())
    }
    BRC30Operation::Mint(mint) => process_mint(brc20_store, brc30_store, msg, mint.clone()),
    BRC30Operation::InscribeTransfer(transfer) => {
      process_inscribe_transfer(brc20_store, brc30_store, msg, transfer.clone())
    }
    BRC30Operation::Transfer => process_transfer(brc20_store, brc30_store, msg),
  };

  let receipt = BRC30Receipt {
    inscription_id: msg.inscription_id,
    inscription_number: msg.inscription_number,
    old_satpoint: msg.old_satpoint,
    new_satpoint: msg.new_satpoint,
    from: msg.from.clone(),
    to: msg.to.clone(),
    op: msg.op.op_type(),
    result: match event {
      Ok(event) => Ok(event),
      Err(Error::BRC30Error(e)) => Err(e),
      Err(e) => {
        return Err(anyhow!(format!(
          "brc30 execute message error: {}",
          e.to_string()
        )))
      }
    },
  };

  brc30_store
    .set_txid_to_receipts(&msg.txid, &receipt)
    .map_err(|e| anyhow!(format!("brc20 execute message error: {}", e.to_string())))?;
  Ok(receipt)
}

pub fn process_deploy<'a, M: BRC20DataStoreReadWrite, N: BRC30DataStoreReadWrite>(
  brc20_store: &'a M,
  brc30_store: &'a N,
  msg: &BRC30Message,
  deploy: Deploy,
) -> Result<BRC30Event, Error<N>> {
  if let Some(iserr) = deploy.validate_basic().err() {
    return Err(Error::BRC30Error(iserr));
  }
  //Prepare the data
  let to_script_key = msg.to.clone();
  let from_script_key = msg.from.clone();
  let tick_id = deploy.get_tick_id();
  let pid = deploy.get_pool_id();
  let ptype = deploy.get_pool_type();

  let stake = deploy.get_stake_id();

  let mut erate = Num::from(0_u128);
  let only = deploy.get_only();
  let name = deploy.get_earn_id();
  let dmax_str = deploy.distribution_max.as_str();
  let mut dmax = 0_u128;

  //check stake
  if !stake_is_exist(&stake, brc30_store, brc20_store) {
    return Err(Error::BRC30Error(BRC30Error::StakeNotFound(
      stake.to_string(),
    )));
  }
  // check pool is exist, if true return error
  if let Some(_) = brc30_store
    .get_pid_to_poolinfo(&pid)
    .map_err(|e| Error::LedgerError(e))?
  {
    return Err(Error::BRC30Error(BRC30Error::PoolAlreadyExist(
      pid.as_str().to_string(),
    )));
  }

  //Get or create the tick
  if let Some(mut temp_tick) = brc30_store
    .get_tick_info(&tick_id)
    .map_err(|e| Error::LedgerError(e))?
  {
    if temp_tick.name != name {
      return Err(Error::BRC30Error(BRC30Error::TickNameNotMatch(
        deploy.earn.clone(),
      )));
    }

    if !temp_tick.deployer.eq(&to_script_key) {
      return Err(Error::BRC30Error(BRC30Error::DeployerNotEqual(
        pid.as_str().to_string(),
        temp_tick.deployer.to_string(),
        to_script_key.to_string(),
      )));
    }

    if !to_script_key.eq(&from_script_key) {
      return Err(Error::BRC30Error(BRC30Error::FromToNotEqual(
        from_script_key.to_string(),
        to_script_key.to_string(),
      )));
    }

    // check stake has exist in tick's pools
    if let Some(_) = brc30_store
      .get_tickid_stake_to_pid(&tick_id, &stake)
      .map_err(|e| Error::LedgerError(e))?
    {
      return Err(Error::BRC30Error(BRC30Error::StakeAlreadyExist(
        stake.to_string(),
        tick_id.to_lowercase().hex(),
      )));
    }

    dmax = convert_amount_with_decimal(dmax_str.clone(), temp_tick.decimal)?.checked_to_u128()?;
    // check dmax
    if temp_tick.supply - temp_tick.allocated < dmax {
      return Err(Error::BRC30Error(BRC30Error::InsufficientTickSupply(
        deploy.distribution_max,
      )));
    }
    temp_tick.allocated = temp_tick.allocated + dmax;
    temp_tick.pids.push(pid.clone());
    brc30_store
      .set_tick_info(&tick_id, &temp_tick)
      .map_err(|e| Error::LedgerError(e))?;

    erate = convert_amount_with_decimal(deploy.earn_rate.as_str(), temp_tick.decimal)?;
  } else {
    let decimal = Num::from_str(&deploy.decimals.map_or(MAX_DECIMAL_WIDTH.to_string(), |v| v))?
      .checked_to_u8()?;
    if decimal > MAX_DECIMAL_WIDTH {
      return Err(Error::BRC30Error(BRC30Error::DecimalsTooLarge(decimal)));
    }

    let supply_str = deploy.total_supply.ok_or(BRC30Error::InternalError(
      "the first deploy must be set total supply".to_string(),
    ))?;
    let total_supply = convert_amount_with_decimal(supply_str.as_str(), decimal)?;
    erate = convert_amount_with_decimal(&deploy.earn_rate.as_str(), decimal)?;

    let supply = total_supply.checked_to_u128()?;
    let c_tick_id = caculate_tick_id(
      convert_amount_without_decimal(supply, decimal)?.checked_to_u128()?,
      decimal,
      &from_script_key,
      &to_script_key,
    );
    if !c_tick_id.to_lowercase().eq(&tick_id) {
      return Err(Error::BRC30Error(BRC30Error::InvalidPoolTickId(
        tick_id.hex(),
        c_tick_id.hex(),
      )));
    }

    let pids = vec![pid.clone()];
    dmax = convert_amount_with_decimal(dmax_str.clone(), decimal)?.checked_to_u128()?;
    let tick = TickInfo::new(
      tick_id,
      &name,
      &msg.inscription_id.clone(),
      dmax,
      decimal,
      0_u128,
      supply,
      &to_script_key,
      msg.block_height,
      msg.block_height,
      pids,
    );
    brc30_store
      .set_tick_info(&tick_id, &tick)
      .map_err(|e| Error::LedgerError(e))?;
  };

  let erate_128 = erate.checked_to_u128()?;
  let pool = PoolInfo::new(
    &pid,
    &ptype,
    &msg.inscription_id.clone(),
    &stake,
    erate_128,
    0,
    0,
    dmax,
    0,
    msg.block_height,
    only,
  );

  brc30_store
    .set_pid_to_poolinfo(&pool.pid, &pool)
    .map_err(|e| Error::LedgerError(e))?;
  brc30_store
    .set_tickid_stake_to_pid(&tick_id, &stake, &pid)
    .map_err(|e| Error::LedgerError(e))?;
  Ok(BRC30Event::DeployPool(DeployPoolEvent {
    pid,
    ptype,
    stake,
    erate: erate_128,
    dmax,
  }))
}

fn process_stake<'a, M: BRC20DataStoreReadWrite, N: BRC30DataStoreReadWrite>(
  brc20_store: &'a M,
  brc30_store: &'a N,
  msg: &BRC30Message,
  stake_msg: Stake,
) -> Result<BRC30Event, Error<N>> {
  if let Some(iserr) = stake_msg.validate_basics().err() {
    return Err(Error::BRC30Error(iserr));
  }
  let pool_id = stake_msg.get_pool_id();
  let to_script_key = msg.to.clone();

  let mut pool = brc30_store
    .get_pid_to_poolinfo(&pool_id)
    .map_err(|e| Error::LedgerError(e))?
    .ok_or(Error::BRC30Error(BRC30Error::PoolNotExist(
      pool_id.as_str().to_string(),
    )))?;

  let stake_tick = pool.stake.clone();
  let amount = convert_pledged_tick_with_decimal(
    &stake_tick,
    stake_msg.amount.as_str(),
    brc30_store,
    brc20_store,
  )?;

  // check user balance of stake is more than ammount to staked
  let stake_balance =
    get_user_common_balance(&to_script_key, &stake_tick, brc30_store, brc20_store);
  let mut is_first_stake = false;
  let mut userinfo = match brc30_store.get_pid_to_use_info(&to_script_key, &pool_id) {
    Ok(Some(info)) => {
      is_first_stake = false;
      info
    }
    _ => {
      is_first_stake = true;
      UserInfo::default(&pool_id)
    }
  };

  let has_staked = Num::from(userinfo.staked);
  if stake_balance.lt(&has_staked) {
    return Err(Error::BRC30Error(BRC30Error::InValidStakeInfo(
      userinfo.staked,
      stake_balance.checked_to_u128()?,
    )));
  } else if stake_balance.checked_sub(&has_staked)?.lt(&amount) {
    return Err(Error::BRC30Error(BRC30Error::InsufficientBalance(
      amount.clone(),
      stake_balance.checked_sub(&has_staked)?,
    )));
  }
  let dec = get_stake_dec(&stake_tick, brc30_store, brc20_store);
  reward::update_pool(&mut pool, msg.block_height, dec)?;
  let mut reward = 0_128;
  if !is_first_stake {
    reward = reward::withdraw_user_reward(&mut userinfo, &mut pool, dec)?;
  }
  // updated user balance of stakedhehe =
  userinfo.staked = has_staked.checked_add(&amount)?.checked_to_u128()?;
  reward::update_user_stake(&mut userinfo, &mut pool, dec)?;
  brc30_store
    .set_pid_to_use_info(&to_script_key, &pool_id, &userinfo)
    .map_err(|e| Error::LedgerError(e))?;

  //update the stake_info of user
  let mut user_stakeinfo = brc30_store
    .get_user_stakeinfo(&to_script_key, &stake_tick)
    .map_err(|e| Error::LedgerError(e))?
    .map_or(
      StakeInfo::new(
        &vec![(pool_id.clone(), pool.only, userinfo.staked)],
        &stake_tick,
      ),
      |v| v,
    );

  for pool_stake in user_stakeinfo.pool_stakes.iter_mut() {
    if pool_stake.0 == pool_id {
      pool_stake.2 = userinfo.staked;
      break;
    }
  }

  brc30_store
    .set_user_stakeinfo(&to_script_key, &stake_tick, &user_stakeinfo)
    .map_err(|e| Error::LedgerError(e))?;

  // update pool_info for stake
  pool.staked = Num::from(pool.staked)
    .checked_add(&amount)?
    .checked_to_u128()?;
  brc30_store
    .set_pid_to_poolinfo(&pool_id, &pool)
    .map_err(|e| Error::LedgerError(e))?;

  return Ok(BRC30Event::Deposit(DepositEvent {
    pid: pool_id,
    amt: amount.checked_to_u128()?,
    reward,
  }));
}

fn process_unstake<'a, M: BRC20DataStoreReadWrite, N: BRC30DataStoreReadWrite>(
  brc20_store: &'a M,
  brc30_store: &'a N,
  msg: &BRC30Message,
  unstake: UnStake,
) -> Result<BRC30Event, Error<N>> {
  if let Some(iserr) = unstake.validate_basics().err() {
    return Err(Error::BRC30Error(iserr));
  }
  let pool_id = unstake.get_pool_id();
  let to_script_key = msg.to.clone();

  let mut pool = brc30_store
    .get_pid_to_poolinfo(&pool_id)
    .map_err(|e| Error::LedgerError(e))?
    .ok_or(Error::BRC30Error(BRC30Error::PoolNotExist(
      pool_id.as_str().to_string(),
    )))?;

  let stake_tick = pool.stake.clone();

  let amount = convert_pledged_tick_with_decimal(
    &stake_tick,
    unstake.amount.as_str(),
    brc30_store,
    brc20_store,
  )?;

  let mut userinfo = brc30_store
    .get_pid_to_use_info(&to_script_key, &pool_id)
    .map_or(Some(UserInfo::default(&pool_id)), |v| v)
    .unwrap();
  let has_staked = Num::from(userinfo.staked);
  if has_staked.lt(&amount) {
    return Err(Error::BRC30Error(BRC30Error::InsufficientBalance(
      has_staked.clone(),
      amount.clone(),
    )));
  }

  let dec = get_stake_dec(&stake_tick, brc30_store, brc20_store);
  reward::update_pool(&mut pool, msg.block_height, dec)?;
  let reward = reward::withdraw_user_reward(&mut userinfo, &mut pool, dec)?;
  userinfo.staked = has_staked.checked_sub(&amount)?.checked_to_u128()?;
  pool.staked = Num::from(pool.staked)
    .checked_sub(&amount)?
    .checked_to_u128()?;
  reward::update_user_stake(&mut userinfo, &mut pool, dec)?;

  brc30_store
    .set_pid_to_use_info(&to_script_key, &pool_id, &userinfo)
    .map_err(|e| Error::LedgerError(e))?;

  let mut user_stakeinfo = brc30_store
    .get_user_stakeinfo(&to_script_key, &stake_tick)
    .map_err(|e| Error::LedgerError(e))?
    .ok_or(Error::BRC30Error(BRC30Error::InsufficientBalance(
      Num::from(amount.clone()),
      Num::from(0_u128),
    )))?;

  //update pool_stakes
  for pool_stake in user_stakeinfo.pool_stakes.iter_mut() {
    if pool_stake.0 == pool_id {
      pool_stake.2 = userinfo.staked;
      break;
    }
  }

  brc30_store
    .set_pid_to_poolinfo(&pool_id, &pool)
    .map_err(|e| Error::LedgerError(e))?;

  brc30_store
    .set_user_stakeinfo(&to_script_key, &stake_tick, &user_stakeinfo)
    .map_err(|e| Error::LedgerError(e))?;
  return Ok(BRC30Event::Withdraw(WithdrawEvent {
    pid: pool_id,
    amt: amount.checked_to_u128()?,
    initiative: false,
  }));
}

fn process_passive_unstake<'a, M: BRC20DataStoreReadWrite, N: BRC30DataStoreReadWrite>(
  brc20_store: &'a M,
  brc30_store: &'a N,
  msg: &BRC30Message,
  passive_unstake: PassiveUnStake,
) -> Result<BRC30Event, Error<N>> {
  if let Some(iserr) = passive_unstake.validate_basics().err() {
    return Err(Error::BRC30Error(iserr));
  }
  let to_script_key = msg.to.clone();
  let stake_tick = passive_unstake.get_stake_tick();
  let stake_info = brc30_store
    .get_user_stakeinfo(&to_script_key, &stake_tick)
    .map_err(|e| Error::LedgerError(e))?
    .ok_or(Error::BRC30Error(BRC30Error::InsufficientBalance(
      Num::from(0_u128),
      Num::from(0_u128),
    )))?;

  let stake_alterive = convert_pledged_tick_with_decimal(
    &stake_tick,
    passive_unstake.amount.as_str(),
    brc30_store,
    brc20_store,
  )?;

  let mut max_share = Num::from(0_u128);
  let mut total_only = Num::from(0_u128);
  let mut pids: Vec<(Pid, u128)> = Vec::new();
  for (pid, only, pool_stake) in stake_info.pool_stakes.iter() {
    let current = max_share.checked_add(&total_only)?;
    if current.ge(&stake_alterive) {
      break;
    }
    let pool_stake_num = Num::from(*pool_stake);
    if *only {
      let remain = stake_alterive.checked_sub(&current)?;
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
        max_share = Num::max(&max_share, &pool_stake_num);
        pids.push((pid.clone(), pool_stake_num.checked_to_u128()?));
      } else {
        max_share = Num::max(&max_share, &remain);
        pids.push((pid.clone(), remain.checked_to_u128()?));
      }
    }
  }
  for (pid, stake) in pids.iter() {
    let withdraw_stake =
      convert_pledged_tick_without_decimal(&stake_tick, *stake, brc30_store, brc20_store)?;
    let stake_msg = UnStake::new(
      pid.to_lowercase().as_str(),
      withdraw_stake.to_string().as_str(),
    );
    process_unstake(brc20_store, brc30_store, &msg, stake_msg)?;
  }

  Ok(BRC30Event::PassiveWithdraw(PassiveWithdrawEvent {
    pid: pids,
  }))
}
fn process_mint<'a, M: BRC20DataStoreReadWrite, N: BRC30DataStoreReadWrite>(
  _brc20_store: &'a M,
  brc30_store: &'a N,
  msg: &BRC30Message,
  mint: Mint,
) -> Result<BRC30Event, Error<N>> {
  let to_script_key = msg.to.clone();
  // check tick
  let tick_id = TickId::from_str(mint.tick_id.as_str())?;
  let mut tick_info = brc30_store
    .get_tick_info(&tick_id)
    .map_err(|e| Error::LedgerError(e))?
    .ok_or(BRC30Error::TickNotFound(mint.tick.clone()))?;

  let tick_name = BRC30Tick::from_str(mint.tick.as_str())?;
  if tick_info.name != tick_name {
    return Err(Error::BRC30Error(BRC30Error::TickNameNotMatch(
      mint.tick.clone(),
    )));
  }

  // check amount
  let mut amt = Num::from_str(&mint.amount)?;
  if amt.scale() > tick_info.decimal as i64 {
    return Err(Error::BRC30Error(BRC30Error::AmountOverflow(amt)));
  }
  let base = BIGDECIMAL_TEN.checked_powu(tick_info.decimal as u64)?;
  amt = amt.checked_mul(&base)?;
  if amt.sign() == Sign::NoSign {
    return Err(Error::BRC30Error(BRC30Error::InvalidZeroAmount));
  }
  // get all staked pools and calculate total reward
  let mut staked_pools: Vec<(Pid, u128)> = Vec::new();
  let mut total_reward = 0;
  for pid in tick_info.pids.clone() {
    let user_info = if let Ok(Some(u)) = brc30_store.get_pid_to_use_info(&to_script_key, &pid) {
      u
    } else {
      continue;
    };
    let pool_info = if let Ok(Some(p)) = brc30_store.get_pid_to_poolinfo(&pid) {
      p
    } else {
      continue;
    };
    let dec = get_stake_dec(&pool_info.stake, brc30_store, _brc20_store);
    let reward = if let Ok(r) = reward::query_reward(user_info, pool_info, msg.block_height, dec) {
      r
    } else {
      continue;
    };
    if reward > 0 {
      total_reward += reward;
      staked_pools.push((pid, reward))
    }
  }
  if amt > total_reward.into() {
    return Err(Error::BRC30Error(BRC30Error::AmountExceedLimit(amt)));
  }

  // claim rewards
  let mut remain_amt = amt.clone();
  for (pid, reward) in staked_pools {
    let reward = Num::from(reward);
    if remain_amt <= Num::zero() {
      break;
    }
    let mut reward = if reward < remain_amt {
      reward
    } else {
      remain_amt.clone()
    };

    let mut user_info = brc30_store
      .get_pid_to_use_info(&to_script_key, &pid)
      .map_err(|e| Error::LedgerError(e))?
      .ok_or(BRC30Error::InternalError(String::from(
        "user info not found",
      )))?;
    let mut pool_info = brc30_store
      .get_pid_to_poolinfo(&pid)
      .map_err(|e| Error::LedgerError(e))?
      .ok_or(BRC30Error::InternalError(String::from("pool not found")))?;

    let dec = get_stake_dec(&pool_info.stake, brc30_store, _brc20_store);
    reward::update_pool(&mut pool_info, msg.block_height, dec)?;
    let withdraw_reward = reward::withdraw_user_reward(&mut user_info, &mut pool_info, dec)?;
    reward::update_user_stake(&mut user_info, &mut pool_info, dec)?;

    if withdraw_reward > reward.checked_to_u128()? {
      user_info.reward = user_info.reward - withdraw_reward + reward.checked_to_u128()?;
      //pool_info.minted = pool_info.minted - withdraw_reward + reward.checked_to_u128()?;
    } else {
      reward = Num::from(withdraw_reward)
    }

    brc30_store
      .set_pid_to_use_info(&to_script_key, &pid, &user_info)
      .map_err(|e| Error::LedgerError(e))?;
    brc30_store
      .set_pid_to_poolinfo(&pid, &pool_info)
      .map_err(|e| Error::LedgerError(e))?;

    remain_amt = remain_amt.checked_sub(&reward)?;
  }

  // update tick info
  tick_info.minted += amt.checked_to_u128()?;
  tick_info.latest_mint_block = msg.block_height;
  brc30_store
    .set_tick_info(&tick_id, &tick_info)
    .map_err(|e| Error::LedgerError(e))?;

  // update user balance
  let mut user_balance = brc30_store
    .get_balance(&to_script_key, &tick_id)
    .map_err(|e| Error::LedgerError(e))?
    .map_or(Balance::new(tick_id.clone()), |v| v);

  user_balance.overall_balance = Into::<Num>::into(user_balance.overall_balance)
    .checked_add(&amt)?
    .checked_to_u128()?;

  brc30_store
    .set_token_balance(&to_script_key, &tick_id, user_balance)
    .map_err(|e| Error::LedgerError(e))?;

  Ok(BRC30Event::Mint(MintEvent {
    tick_id,
    amt: amt.checked_to_u128()?,
  }))
}

fn process_inscribe_transfer<'a, M: BRC20DataStoreReadWrite, N: BRC30DataStoreReadWrite>(
  _brc20_store: &'a M,
  brc30_store: &'a N,
  msg: &BRC30Message,
  transfer: Transfer,
) -> Result<BRC30Event, Error<N>> {
  let to_script_key = msg.to.clone();
  // check tick
  let tick_id = TickId::from_str(transfer.tick_id.as_str())?;
  let tick_info = brc30_store
    .get_tick_info(&tick_id)
    .map_err(|e| Error::LedgerError(e))?
    .ok_or(BRC30Error::TickNotFound(tick_id.hex()))?;

  let tick_name = BRC30Tick::from_str(transfer.tick.as_str())?;
  if tick_info.name != tick_name {
    return Err(Error::BRC30Error(BRC30Error::TickNameNotMatch(
      transfer.tick.clone(),
    )));
  }

  // check amount
  let mut amt = Num::from_str(&transfer.amount)?;
  if amt.scale() > tick_info.decimal as i64 {
    return Err(Error::BRC30Error(BRC30Error::AmountOverflow(amt)));
  }
  let base = BIGDECIMAL_TEN.checked_powu(tick_info.decimal as u64)?;
  amt = amt.checked_mul(&base)?;
  if amt.sign() == Sign::NoSign {
    return Err(Error::BRC30Error(BRC30Error::InvalidZeroAmount));
  }

  // update balance
  let mut balance = brc30_store
    .get_balance(&to_script_key, &tick_id)
    .map_err(|e| Error::LedgerError(e))?
    .map_or(Balance::new(tick_id.clone()), |v| v);

  let overall = Into::<Num>::into(balance.overall_balance);
  let transferable = Into::<Num>::into(balance.transferable_balance);
  let available = overall.checked_sub(&transferable)?;
  if available < amt {
    return Err(Error::BRC30Error(BRC30Error::InsufficientBalance(
      available, amt,
    )));
  }
  balance.transferable_balance = transferable.checked_add(&amt)?.checked_to_u128()?;
  brc30_store
    .set_token_balance(&to_script_key, &tick_id, balance)
    .map_err(|e| Error::LedgerError(e))?;

  // insert transferable assets
  let amount = amt.checked_to_u128()?;
  let transferable_assets = TransferableAsset {
    inscription_id: msg.inscription_id,
    amount,
    tick_id,
    owner: to_script_key.clone(),
  };
  brc30_store
    .set_transferable_assets(
      &to_script_key,
      &tick_id,
      &msg.inscription_id,
      &transferable_assets,
    )
    .map_err(|e| Error::LedgerError(e))?;

  Ok(BRC30Event::InscribeTransfer(InscribeTransferEvent {
    tick_id,
    amt: amount,
  }))
}

fn process_transfer<'a, M: BRC20DataStoreReadWrite, N: BRC30DataStoreReadWrite>(
  _brc20_store: &'a M,
  brc30_store: &'a N,
  msg: &BRC30Message,
) -> Result<BRC30Event, Error<N>> {
  let from_script_key = msg.from.clone();
  let to_script_key = msg.to.clone();
  let transferable = brc30_store
    .get_transferable_by_id(&from_script_key, &msg.inscription_id)
    .map_err(|e| Error::LedgerError(e))?
    .ok_or(BRC30Error::TransferableNotFound(msg.inscription_id))?;

  let amt = Into::<Num>::into(transferable.amount);

  if transferable.owner != from_script_key {
    return Err(Error::BRC30Error(BRC30Error::TransferableOwnerNotMatch(
      msg.inscription_id,
    )));
  }

  let tick_info = brc30_store
    .get_tick_info(&transferable.tick_id)
    .map_err(|e| Error::LedgerError(e))?
    .ok_or(BRC30Error::TickNotFound(transferable.tick_id.hex()))?;

  // update from key balance.
  let mut from_balance = brc30_store
    .get_balance(&from_script_key, &transferable.tick_id)
    .map_err(|e| Error::LedgerError(e))?
    .map_or(Balance::new(transferable.tick_id), |v| v);

  let from_overall = Into::<Num>::into(from_balance.overall_balance);
  let from_transferable = Into::<Num>::into(from_balance.transferable_balance);

  let from_overall = from_overall.checked_sub(&amt)?.checked_to_u128()?;
  let from_transferable = from_transferable.checked_sub(&amt)?.checked_to_u128()?;

  from_balance.overall_balance = from_overall;
  from_balance.transferable_balance = from_transferable;

  brc30_store
    .set_token_balance(&from_script_key, &transferable.tick_id, from_balance)
    .map_err(|e| Error::LedgerError(e))?;

  // redirect receiver to sender if transfer to conibase.
  // let to_script_key = if let None = to_script_key.clone() {
  //   from_script_key.clone()
  // } else {
  //   to_script_key.unwrap()
  // };

  // update to key balance.
  let mut to_balance = brc30_store
    .get_balance(&to_script_key, &transferable.tick_id)
    .map_err(|e| Error::LedgerError(e))?
    .map_or(Balance::new(transferable.tick_id), |v| v);

  let to_overall = Into::<Num>::into(to_balance.overall_balance);
  to_balance.overall_balance = to_overall.checked_add(&amt)?.checked_to_u128()?;

  brc30_store
    .set_token_balance(&to_script_key, &transferable.tick_id, to_balance)
    .map_err(|e| Error::LedgerError(e))?;

  brc30_store
    .remove_transferable(&from_script_key, &transferable.tick_id, &msg.inscription_id)
    .map_err(|e| Error::LedgerError(e))?;

  Ok(BRC30Event::Transfer(TransferEvent {
    tick_id: transferable.tick_id,
    amt: amt.checked_to_u128()?,
  }))
}

#[cfg(test)]
mod tests {
  use super::super::*;
  use super::*;
  use crate::index::INSCRIPTION_ID_TO_INSCRIPTION_ENTRY;
  use crate::okx::datastore::BRC20::redb::BRC20DataStore;
  use crate::okx::datastore::BRC20::{Balance as BRC20Banalce, Tick, TokenInfo};
  use crate::okx::datastore::BRC30::redb::BRC30DataStore;
  use crate::okx::datastore::BRC30::BRC30DataStoreReadOnly;
  use crate::test::Hash;
  use bech32::ToBase32;
  use bitcoin::Address;
  use redb::{Database, WriteTransaction};
  use std::borrow::Borrow;
  use tempfile::NamedTempFile;

  fn create_brc30_message(
    inscription_id: InscriptionId,
    from: ScriptKey,
    to: ScriptKey,
    op: BRC30Operation,
  ) -> BRC30Message {
    BRC30Message {
      txid: Txid::all_zeros(),
      block_height: 0,
      block_time: 1687245485,
      inscription_id,
      inscription_number: 0,
      from: from.clone(),
      to: to.clone(),
      old_satpoint: SatPoint {
        outpoint: "1111111111111111111111111111111111111111111111111111111111111111:1"
          .parse()
          .unwrap(),
        offset: 1,
      },
      new_satpoint: SatPoint {
        outpoint: "1111111111111111111111111111111111111111111111111111111111111111:2"
          .parse()
          .unwrap(),
        offset: 1,
      },
      op,
    }
  }
  #[test]
  fn test_process_deploy() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let mut inscription_id_to_inscription_entry =
      wtx.open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY).unwrap();

    let brc20_data_store = BRC20DataStore::new(&wtx);
    let brc30_data_store = BRC30DataStore::new(&wtx);

    let deploy = Deploy {
      pool_type: "pool".to_string(),
      pool_id: "c8195197bc#1f".to_string(),
      stake: "btc".to_string(),
      earn: "ordi1".to_string(),
      earn_rate: "10".to_string(),
      distribution_max: "12000000".to_string(),
      decimals: Some("18".to_string()),
      total_supply: Some("21000000".to_string()),
      only: Some("1".to_string()),
    };
    let addr1 =
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();
    let script = ScriptKey::from_address(addr1);
    let inscruptionId =
      InscriptionId::from_str("1111111111111111111111111111111111111111111111111111111111111111i1")
        .unwrap();
    let msg = create_brc30_message(
      inscruptionId,
      script.clone(),
      script.clone(),
      BRC30Operation::Deploy(deploy.clone()),
    );
    let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, deploy.clone());

    let result: Result<BRC30Event, BRC30Error> = match result {
      Ok(event) => Ok(event),
      Err(Error::BRC30Error(e)) => Err(e),
      Err(e) => Err(BRC30Error::InternalError(e.to_string())),
    };

    match result {
      Ok(event) => {
        println!("success:{}", serde_json::to_string_pretty(&event).unwrap());
      }
      Err(e) => {
        assert_eq!("error", e.to_string())
      }
    }
    let tick_id = deploy.get_tick_id();
    let pid = deploy.get_pool_id();
    let tickinfo = brc30_data_store.get_tick_info(&tick_id).unwrap().unwrap();
    let poolinfo = brc30_data_store.get_pid_to_poolinfo(&pid).unwrap().unwrap();

    let expectTickINfo = r##"{"tick_id":"c8195197bc","name":"ordi1","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","allocated":12000000000000000000000000,"decimal":18,"minted":0,"supply":21000000000000000000000000,"deployer":{"Address":"bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e"},"deploy_block":0,"latest_mint_block":0,"pids":["c8195197bc#1f"]}"##;
    let expectPoolInfo = r##"{"pid":"c8195197bc#1f","ptype":"Pool","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","stake":"NATIVE","erate":10000000000000000000,"minted":0,"staked":0,"dmax":12000000000000000000000000,"acc_reward_per_share":0,"last_update_block":0,"only":true}"##;
    assert_eq!(expectPoolInfo, serde_json::to_string(&poolinfo).unwrap());
    assert_eq!(expectTickINfo, serde_json::to_string(&tickinfo).unwrap());

    let msg = create_brc30_message(
      inscruptionId,
      script.clone(),
      script.clone(),
      BRC30Operation::Deploy(deploy.clone()),
    );
    let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, deploy.clone());

    let result: Result<BRC30Event, BRC30Error> = match result {
      Ok(event) => Ok(event),
      Err(Error::BRC30Error(e)) => Err(e),
      Err(e) => Err(BRC30Error::InternalError(e.to_string())),
    };

    assert_eq!(
      Err(BRC30Error::PoolAlreadyExist(pid.as_str().to_string())),
      result
    );

    let token = Tick::from_str("orea".to_string().as_str()).unwrap();
    let token_info = TokenInfo {
      tick: token.clone(),
      inscription_id: inscruptionId.clone(),
      inscription_number: 0,
      supply: 0,
      minted: 0,
      limit_per_mint: 0,
      decimal: 0,
      deploy_by: script.clone(),
      deployed_number: 0,
      deployed_timestamp: 0,
      latest_mint_number: 0,
    };
    brc20_data_store.insert_token_info(&token, &token_info);

    let mut secondDeply = deploy.clone();
    secondDeply.pool_id = "c8195197bc#11".to_string();
    secondDeply.stake = "orea".to_string();
    secondDeply.distribution_max = "9000000".to_string();
    let msg = create_brc30_message(
      inscruptionId,
      script.clone(),
      script.clone(),
      BRC30Operation::Deploy(secondDeply.clone()),
    );
    let result = process_deploy(
      &brc20_data_store,
      &brc30_data_store,
      &msg,
      secondDeply.clone(),
    );

    let result: Result<BRC30Event, BRC30Error> = match result {
      Ok(event) => Ok(event),
      Err(Error::BRC30Error(e)) => Err(e),
      Err(e) => Err(BRC30Error::InternalError(e.to_string())),
    };

    assert_ne!(true, result.is_err());
    let tick_id = secondDeply.get_tick_id();
    let pid = secondDeply.get_pool_id();
    let tickinfo = brc30_data_store.get_tick_info(&tick_id).unwrap().unwrap();
    let poolinfo = brc30_data_store.get_pid_to_poolinfo(&pid).unwrap().unwrap();

    let expectTickINfo = r##"{"tick_id":"c8195197bc","name":"ordi1","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","allocated":21000000000000000000000000,"decimal":18,"minted":0,"supply":21000000000000000000000000,"deployer":{"Address":"bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e"},"deploy_block":0,"latest_mint_block":0,"pids":["c8195197bc#1f","c8195197bc#11"]}"##;
    let expectPoolInfo = r##"{"pid":"c8195197bc#11","ptype":"Pool","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","stake":{"BRC20Tick":"orea"},"erate":10000000000000000000,"minted":0,"staked":0,"dmax":9000000000000000000000000,"acc_reward_per_share":0,"last_update_block":0,"only":true}"##;
    assert_eq!(expectPoolInfo, serde_json::to_string(&poolinfo).unwrap());
    assert_eq!(expectTickINfo, serde_json::to_string(&tickinfo).unwrap());
  }

  #[test]
  fn test_process_error_params() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let mut inscription_id_to_inscription_entry =
      wtx.open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY).unwrap();

    let brc20_data_store = BRC20DataStore::new(&wtx);
    let brc30_data_store = BRC30DataStore::new(&wtx);

    let deploy = Deploy {
      pool_type: "pool".to_string(),
      pool_id: "c8195197bc#1f".to_string(),
      stake: "btc".to_string(),
      earn: "ordi1".to_string(),
      earn_rate: "10".to_string(),
      distribution_max: "12000000".to_string(),
      decimals: Some("18".to_string()),
      total_supply: Some("21000000".to_string()),
      only: Some("1".to_string()),
    };
    let addr1 =
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();
    let script = ScriptKey::from_address(addr1);
    let inscruptionId =
      InscriptionId::from_str("1111111111111111111111111111111111111111111111111111111111111111i1")
        .unwrap();

    //err pool type
    {
      let mut err_pool_type = deploy.clone();
      err_pool_type.pool_type = "errtype".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_pool_type.clone()),
      );
      let result = process_deploy(
        &brc20_data_store,
        &brc30_data_store,
        &msg,
        err_pool_type.clone(),
      );

      let pid = deploy.get_pool_id();

      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(Err(BRC30Error::UnknownPoolType), result);
    }

    //err pid
    {
      let mut err_pid = deploy.clone();
      err_pid.pool_id = "l8195197bc#1f".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_pid.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_pid.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidPoolId(
          err_pid.pool_id.clone(),
          "the prefix of pool id is not hex".to_string()
        )),
        result
      );

      let mut err_pid = deploy.clone();
      err_pid.pool_id = "8195197bc#1f".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_pid.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_pid.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidPoolId(
          err_pid.pool_id.clone(),
          "pool id length is not 13".to_string()
        )),
        result
      );

      let mut err_pid = deploy.clone();
      err_pid.pool_id = "c8195197bc#lf".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_pid.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_pid.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };

      assert_eq!(
        Err(BRC30Error::InvalidPoolId(
          err_pid.pool_id.clone(),
          "the suffix of pool id is not hex".to_string()
        )),
        result
      );

      let mut err_pid = deploy.clone();
      err_pid.pool_id = "c81195197bc#f".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_pid.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_pid.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidPoolId(
          err_pid.pool_id.clone(),
          "the prefix of pool id is not hex".to_string()
        )),
        result
      );

      let mut err_pid = deploy.clone();
      err_pid.pool_id = "c8195197bc$1f".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_pid.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_pid.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidPoolId(
          err_pid.pool_id.clone(),
          "pool id must contains '#'".to_string()
        )),
        result
      );

      let mut err_pid = deploy.clone();
      err_pid.pool_id = "c819519#bc#df".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_pid.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_pid.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidPoolId(
          err_pid.pool_id.clone(),
          "pool id must contains only one '#'".to_string()
        )),
        result
      );

      let mut err_pid = deploy.clone();
      err_pid.pool_id = "c819519#bc#1f".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_pid.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_pid.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidPoolId(
          err_pid.pool_id.clone(),
          "pool id must contains only one '#'".to_string()
        )),
        result
      );

      let mut err_pid = deploy.clone();
      err_pid.pool_id = "a8195197bc#1f".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_pid.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_pid.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidPoolTickId(
          "a8195197bc".to_string(),
          "c8195197bc".to_string()
        )),
        result
      );
    }

    //err stake,earn
    {
      let mut err_stake = deploy.clone();
      err_stake.stake = "he".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_stake.clone()),
      );
      let result = process_deploy(
        &brc20_data_store,
        &brc30_data_store,
        &msg,
        err_stake.clone(),
      );

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(Err(BRC30Error::UnknownStakeType), result);

      let mut err_stake = deploy.clone();
      err_stake.stake = "hehehh".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_stake.clone()),
      );
      let result = process_deploy(
        &brc20_data_store,
        &brc30_data_store,
        &msg,
        err_stake.clone(),
      );

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(Err(BRC30Error::UnknownStakeType), result);

      let mut err_stake = deploy.clone();
      err_stake.stake = "test".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_stake.clone()),
      );
      let result = process_deploy(
        &brc20_data_store,
        &brc30_data_store,
        &msg,
        err_stake.clone(),
      );

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(Err(BRC30Error::StakeNotFound(err_stake.stake)), result);

      let mut err_earn = deploy.clone();
      err_earn.earn = "tes".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_earn.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_earn.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidTickLen(err_earn.earn.to_string())),
        result
      );

      let mut err_earn = deploy.clone();
      err_earn.earn = "test".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_earn.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_earn.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_ne!(
        Err(BRC30Error::InvalidTickLen(err_earn.earn.to_string())),
        result
      );

      let mut err_earn = deploy.clone();
      err_earn.earn = "testt".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_earn.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_earn.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_ne!(
        Err(BRC30Error::InvalidTickLen(err_earn.earn.to_string())),
        result
      );

      let mut err_earn = deploy.clone();
      err_earn.earn = "testttt".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_earn.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_earn.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidTickLen(err_earn.earn.to_string())),
        result
      );

      let mut err_earn = deploy.clone();
      err_earn.earn = "test".to_string();
      err_earn.stake = "test".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_earn.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_earn.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::StakeEqualEarn(
          err_earn.stake.to_string(),
          err_earn.earn.to_string()
        )),
        result
      );
    }
    // err erate
    {
      let mut err_erate = deploy.clone();
      err_erate.earn_rate = "".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_erate.clone()),
      );
      let result = process_deploy(
        &brc20_data_store,
        &brc30_data_store,
        &msg,
        err_erate.clone(),
      );

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidNum("invalid number: ".to_string())),
        result
      );

      let mut err_erate = deploy.clone();
      err_erate.earn_rate = "1l".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_erate.clone()),
      );
      let result = process_deploy(
        &brc20_data_store,
        &brc30_data_store,
        &msg,
        err_erate.clone(),
      );

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidNum("1linvalid number: 1l".to_string())),
        result
      );
    }

    //err dmax
    {
      let mut err_dmax = deploy.clone();
      err_dmax.distribution_max = "".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_dmax.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_dmax.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidNum("invalid number: ".to_string())),
        result
      );

      let mut err_dmax = deploy.clone();
      err_dmax.distribution_max = "1l".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_dmax.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_dmax.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidNum("1linvalid number: 1l".to_string())),
        result
      );

      let mut err_dmax = deploy.clone();
      err_dmax.distribution_max = "21000001".to_string();
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_dmax.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_dmax.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::ExceedDmax(
          "21000001".to_string(),
          "21000000".to_string()
        )),
        result
      );
    }

    //err total_supply
    {
      let mut err_total = deploy.clone();
      err_total.total_supply = Some("".to_string());
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_total.clone()),
      );
      let result = process_deploy(
        &brc20_data_store,
        &brc30_data_store,
        &msg,
        err_total.clone(),
      );

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidNum("invalid number: ".to_string())),
        result
      );

      let mut err_dmax = deploy.clone();
      err_dmax.total_supply = Some("1l".to_string());
      let msg = create_brc30_message(
        inscruptionId,
        script.clone(),
        script.clone(),
        BRC30Operation::Deploy(err_dmax.clone()),
      );
      let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, err_dmax.clone());

      let pid = deploy.get_pool_id();
      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };
      assert_eq!(
        Err(BRC30Error::InvalidNum("1linvalid number: 1l".to_string())),
        result
      );
    }
  }

  #[test]
  fn test_process_stake() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let mut inscription_id_to_inscription_entry =
      wtx.open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY).unwrap();

    let brc20_data_store = BRC20DataStore::new(&wtx);
    let brc30_data_store = BRC30DataStore::new(&wtx);

    let deploy = Deploy {
      pool_type: "pool".to_string(),
      pool_id: "c7f75082ae#1f".to_string(),
      stake: "orea".to_string(),
      earn: "ordi".to_string(),
      earn_rate: "1000".to_string(),
      distribution_max: "12000000".to_string(),
      decimals: Some("2".to_string()),
      total_supply: Some("21000000".to_string()),
      only: Some("1".to_string()),
    };
    let addr1 =
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();
    let script = ScriptKey::from_address(addr1);
    let inscruptionId =
      InscriptionId::from_str("1111111111111111111111111111111111111111111111111111111111111111i1")
        .unwrap();

    let token = Tick::from_str("orea".to_string().as_str()).unwrap();
    let token_info = TokenInfo {
      tick: token.clone(),
      inscription_id: inscruptionId.clone(),
      inscription_number: 0,
      supply: 21000000000_u128,
      minted: 2000000000_u128,
      limit_per_mint: 0,
      decimal: 3,
      deploy_by: script.clone(),
      deployed_number: 0,
      deployed_timestamp: 0,
      latest_mint_number: 0,
    };
    brc20_data_store.insert_token_info(&token, &token_info);
    let balance = BRC20Banalce {
      overall_balance: 2000000000_u128,
      transferable_balance: 0_u128,
    };
    let result = brc20_data_store.update_token_balance(&script, &token.to_lowercase(), balance);
    match result {
      Err(error) => {
        panic!("update_token_balance err: {}", error)
      }
      _ => {}
    }

    let msg = create_brc30_message(
      inscruptionId.clone(),
      script.clone(),
      script.clone(),
      BRC30Operation::Deploy(deploy.clone()),
    );
    let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, deploy.clone());

    let result: Result<BRC30Event, BRC30Error> = match result {
      Ok(event) => Ok(event),
      Err(Error::BRC30Error(e)) => Err(e),
      Err(e) => Err(BRC30Error::InternalError(e.to_string())),
    };

    match result {
      Ok(event) => {
        println!("success:{}", serde_json::to_string_pretty(&event).unwrap());
      }
      Err(e) => {
        assert_eq!("error", e.to_string())
      }
    }
    let tick_id = deploy.get_tick_id();
    let pid = deploy.get_pool_id();
    let tickinfo = brc30_data_store.get_tick_info(&tick_id).unwrap().unwrap();
    let poolinfo = brc30_data_store.get_pid_to_poolinfo(&pid).unwrap().unwrap();

    let expectTickINfo = r##"{"tick_id":"c7f75082ae","name":"ordi","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","allocated":1200000000,"decimal":2,"minted":0,"supply":2100000000,"deployer":{"Address":"bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e"},"deploy_block":0,"latest_mint_block":0,"pids":["c7f75082ae#1f"]}"##;
    let expectPoolInfo = r##"{"pid":"c7f75082ae#1f","ptype":"Pool","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","stake":{"BRC20Tick":"orea"},"erate":100000,"minted":0,"staked":0,"dmax":1200000000,"acc_reward_per_share":0,"last_update_block":0,"only":true}"##;
    assert_eq!(expectPoolInfo, serde_json::to_string(&poolinfo).unwrap());
    assert_eq!(expectTickINfo, serde_json::to_string(&tickinfo).unwrap());

    let stakeTick = PledgedTick::BRC20Tick(token.clone());
    let stakeMsg = Stake {
      pool_id: pid.as_str().to_string(),
      amount: "1000000".to_string(),
    };

    let msg = create_brc30_message(
      inscruptionId.clone(),
      script.clone(),
      script.clone(),
      BRC30Operation::Stake(stakeMsg.clone()),
    );
    let result = process_stake(&brc20_data_store, &brc30_data_store, &msg, stakeMsg.clone());

    let result: Result<BRC30Event, BRC30Error> = match result {
      Ok(event) => Ok(event),
      Err(Error::BRC30Error(e)) => Err(e),
      Err(e) => Err(BRC30Error::InternalError(e.to_string())),
    };

    match result {
      Ok(event) => {
        println!("success:{}", serde_json::to_string_pretty(&event).unwrap());
      }
      Err(e) => {
        assert_eq!("error", e.to_string())
      }
    }
    let stakeinfo = brc30_data_store
      .get_user_stakeinfo(&script, &stakeTick)
      .unwrap();

    let userinfo = brc30_data_store.get_pid_to_use_info(&script, &pid).unwrap();
    let poolinfo = brc30_data_store.get_pid_to_poolinfo(&pid).unwrap();
    let expect_stakeinfo =
      r##"{"stake":{"BRC20Tick":"orea"},"pool_stakes":[["c7f75082ae#1f",true,1000000000]]}"##;
    let expect_userinfo = r##"{"pid":"c7f75082ae#1f","staked":1000000000,"reward":0,"reward_debt":0,"latest_updated_block":0}"##;
    let expect_poolinfo = r##"{"pid":"c7f75082ae#1f","ptype":"Pool","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","stake":{"BRC20Tick":"orea"},"erate":100000,"minted":0,"staked":1000000000,"dmax":1200000000,"acc_reward_per_share":0,"last_update_block":0,"only":true}"##;

    assert_eq!(expect_poolinfo, serde_json::to_string(&poolinfo).unwrap());
    assert_eq!(expect_stakeinfo, serde_json::to_string(&stakeinfo).unwrap());
    assert_eq!(expect_userinfo, serde_json::to_string(&userinfo).unwrap());
    {
      let stakeTick = PledgedTick::BRC20Tick(token.clone());
      let stakeMsg = Stake {
        pool_id: pid.as_str().to_string(),
        amount: "1000000".to_string(),
      };

      let mut msg = create_brc30_message(
        inscruptionId.clone(),
        script.clone(),
        script.clone(),
        BRC30Operation::Stake(stakeMsg.clone()),
      );
      msg.block_height = 1;
      let result = process_stake(&brc20_data_store, &brc30_data_store, &msg, stakeMsg.clone());

      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };

      match result {
        Ok(event) => {
          println!("success:{}", serde_json::to_string_pretty(&event).unwrap());
        }
        Err(e) => {
          assert_eq!("error", e.to_string())
        }
      }
      let stakeinfo = brc30_data_store
        .get_user_stakeinfo(&script, &stakeTick)
        .unwrap();

      let userinfo = brc30_data_store.get_pid_to_use_info(&script, &pid).unwrap();
      let poolinfo = brc30_data_store.get_pid_to_poolinfo(&pid).unwrap();
      let expect_stakeinfo =
        r##"{"stake":{"BRC20Tick":"orea"},"pool_stakes":[["c7f75082ae#1f",true,2000000000]]}"##;
      let expect_userinfo = r##"{"pid":"c7f75082ae#1f","staked":2000000000,"reward":100000,"reward_debt":2000000000,"latest_updated_block":0}"##;
      let expect_poolinfo = r##"{"pid":"c7f75082ae#1f","ptype":"Pool","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","stake":{"BRC20Tick":"orea"},"erate":100000,"minted":100000,"staked":2000000000,"dmax":1200000000,"acc_reward_per_share":1,"last_update_block":1,"only":true}"##;
      println!(
        "expect_poolinfo:{}",
        serde_json::to_string(&poolinfo).unwrap()
      );
      println!(
        "expect_stakeinfo:{}",
        serde_json::to_string(&stakeinfo).unwrap()
      );
      println!(
        "expect_userinfo:{}",
        serde_json::to_string(&userinfo).unwrap()
      );

      assert_eq!(expect_poolinfo, serde_json::to_string(&poolinfo).unwrap());
      assert_eq!(expect_stakeinfo, serde_json::to_string(&stakeinfo).unwrap());
      assert_eq!(expect_userinfo, serde_json::to_string(&userinfo).unwrap());
    }
  }

  #[test]
  fn test_process_unstake() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let mut inscription_id_to_inscription_entry =
      wtx.open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY).unwrap();

    let brc20_data_store = BRC20DataStore::new(&wtx);
    let brc30_data_store = BRC30DataStore::new(&wtx);

    let deploy = Deploy {
      pool_type: "pool".to_string(),
      pool_id: "c7f75082ae#1f".to_string(),
      stake: "orea".to_string(),
      earn: "ordi".to_string(),
      earn_rate: "1000".to_string(),
      distribution_max: "12000000".to_string(),
      decimals: Some("2".to_string()),
      total_supply: Some("21000000".to_string()),
      only: Some("1".to_string()),
    };
    let addr1 =
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();
    let script = ScriptKey::from_address(addr1);
    let inscruptionId =
      InscriptionId::from_str("1111111111111111111111111111111111111111111111111111111111111111i1")
        .unwrap();

    let token = Tick::from_str("orea".to_string().as_str()).unwrap();
    let token_info = TokenInfo {
      tick: token.clone(),
      inscription_id: inscruptionId.clone(),
      inscription_number: 0,
      supply: 21000000000_u128,
      minted: 2000000000_u128,
      limit_per_mint: 0,
      decimal: 3,
      deploy_by: script.clone(),
      deployed_number: 0,
      deployed_timestamp: 0,
      latest_mint_number: 0,
    };
    brc20_data_store.insert_token_info(&token, &token_info);
    let balance = BRC20Banalce {
      overall_balance: 2000000000_u128,
      transferable_balance: 1000000000_u128,
    };
    let result = brc20_data_store.update_token_balance(&script, &token.to_lowercase(), balance);
    match result {
      Err(error) => {
        panic!("update_token_balance err: {}", error)
      }
      _ => {}
    }

    let msg = create_brc30_message(
      inscruptionId.clone(),
      script.clone(),
      script.clone(),
      BRC30Operation::Deploy(deploy.clone()),
    );
    let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, deploy.clone());

    let result: Result<BRC30Event, BRC30Error> = match result {
      Ok(event) => Ok(event),
      Err(Error::BRC30Error(e)) => Err(e),
      Err(e) => Err(BRC30Error::InternalError(e.to_string())),
    };

    match result {
      Ok(event) => {
        println!("success:{}", serde_json::to_string_pretty(&event).unwrap());
      }
      Err(e) => {
        assert_eq!("error", e.to_string())
      }
    }
    let tick_id = deploy.get_tick_id();
    let pid = deploy.get_pool_id();
    let tickinfo = brc30_data_store.get_tick_info(&tick_id).unwrap().unwrap();
    let poolinfo = brc30_data_store.get_pid_to_poolinfo(&pid).unwrap().unwrap();

    let expectTickINfo = r##"{"tick_id":"c7f75082ae","name":"ordi","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","allocated":1200000000,"decimal":2,"minted":0,"supply":2100000000,"deployer":{"Address":"bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e"},"deploy_block":0,"latest_mint_block":0,"pids":["c7f75082ae#1f"]}"##;
    let expectPoolInfo = r##"{"pid":"c7f75082ae#1f","ptype":"Pool","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","stake":{"BRC20Tick":"orea"},"erate":100000,"minted":0,"staked":0,"dmax":1200000000,"acc_reward_per_share":0,"last_update_block":0,"only":true}"##;
    assert_eq!(expectPoolInfo, serde_json::to_string(&poolinfo).unwrap());
    assert_eq!(expectTickINfo, serde_json::to_string(&tickinfo).unwrap());

    let stakeTick = PledgedTick::BRC20Tick(token.clone());
    let stakeMsg = Stake {
      pool_id: pid.as_str().to_string(),
      amount: "1000000".to_string(),
    };

    let msg = create_brc30_message(
      inscruptionId.clone(),
      script.clone(),
      script.clone(),
      BRC30Operation::Stake(stakeMsg.clone()),
    );
    let result = process_stake(&brc20_data_store, &brc30_data_store, &msg, stakeMsg.clone());

    let result: Result<BRC30Event, BRC30Error> = match result {
      Ok(event) => Ok(event),
      Err(Error::BRC30Error(e)) => Err(e),
      Err(e) => Err(BRC30Error::InternalError(e.to_string())),
    };

    match result {
      Ok(event) => {
        println!("success:{}", serde_json::to_string_pretty(&event).unwrap());
      }
      Err(e) => {
        assert_eq!("error", e.to_string())
      }
    }
    let stakeinfo = brc30_data_store
      .get_user_stakeinfo(&script, &stakeTick)
      .unwrap();

    let userinfo = brc30_data_store.get_pid_to_use_info(&script, &pid).unwrap();
    let poolinfo = brc30_data_store.get_pid_to_poolinfo(&pid).unwrap();
    let expect_stakeinfo =
      r##"{"stake":{"BRC20Tick":"orea"},"pool_stakes":[["c7f75082ae#1f",true,1000000000]]}"##;
    let expect_userinfo = r##"{"pid":"c7f75082ae#1f","staked":1000000000,"reward":0,"reward_debt":0,"latest_updated_block":0}"##;
    let expect_poolinfo = r##"{"pid":"c7f75082ae#1f","ptype":"Pool","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","stake":{"BRC20Tick":"orea"},"erate":100000,"minted":0,"staked":1000000000,"dmax":1200000000,"acc_reward_per_share":0,"last_update_block":0,"only":true}"##;

    assert_eq!(expect_poolinfo, serde_json::to_string(&poolinfo).unwrap());
    assert_eq!(expect_stakeinfo, serde_json::to_string(&stakeinfo).unwrap());
    assert_eq!(expect_userinfo, serde_json::to_string(&userinfo).unwrap());
    {
      let stakeTick = PledgedTick::BRC20Tick(token.clone());
      let unstakeMsg = UnStake {
        pool_id: pid.as_str().to_string(),
        amount: "1000000".to_string(),
      };

      let mut msg = create_brc30_message(
        inscruptionId.clone(),
        script.clone(),
        script.clone(),
        BRC30Operation::UnStake(unstakeMsg.clone()),
      );
      msg.block_height = 1;
      let result = process_unstake(
        &brc20_data_store,
        &brc30_data_store,
        &msg,
        unstakeMsg.clone(),
      );

      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };

      match result {
        Ok(event) => {
          println!("success:{}", serde_json::to_string_pretty(&event).unwrap());
        }
        Err(e) => {
          assert_eq!("error", e.to_string())
        }
      }
      let stakeinfo = brc30_data_store
        .get_user_stakeinfo(&script, &stakeTick)
        .unwrap();

      let userinfo = brc30_data_store.get_pid_to_use_info(&script, &pid).unwrap();
      let poolinfo = brc30_data_store.get_pid_to_poolinfo(&pid).unwrap();
      let expect_stakeinfo =
        r##"{"stake":{"BRC20Tick":"orea"},"pool_stakes":[["c7f75082ae#1f",true,0]]}"##;
      let expect_userinfo = r##"{"pid":"c7f75082ae#1f","staked":0,"reward":100000,"reward_debt":0,"latest_updated_block":0}"##;
      let expect_poolinfo = r##"{"pid":"c7f75082ae#1f","ptype":"Pool","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","stake":{"BRC20Tick":"orea"},"erate":100000,"minted":100000,"staked":0,"dmax":1200000000,"acc_reward_per_share":1,"last_update_block":1,"only":true}"##;
      println!(
        "expect_poolinfo:{}",
        serde_json::to_string(&poolinfo).unwrap()
      );
      println!(
        "expect_stakeinfo:{}",
        serde_json::to_string(&stakeinfo).unwrap()
      );
      println!(
        "expect_userinfo:{}",
        serde_json::to_string(&userinfo).unwrap()
      );

      assert_eq!(expect_poolinfo, serde_json::to_string(&poolinfo).unwrap());
      assert_eq!(expect_stakeinfo, serde_json::to_string(&stakeinfo).unwrap());
      assert_eq!(expect_userinfo, serde_json::to_string(&userinfo).unwrap());
    }
  }

  #[test]
  fn test_process_passive_unstake() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let mut inscription_id_to_inscription_entry =
      wtx.open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY).unwrap();

    let brc20_data_store = BRC20DataStore::new(&wtx);
    let brc30_data_store = BRC30DataStore::new(&wtx);

    let deploy = Deploy {
      pool_type: "pool".to_string(),
      pool_id: "c7f75082ae#1f".to_string(),
      stake: "orea".to_string(),
      earn: "ordi".to_string(),
      earn_rate: "1000".to_string(),
      distribution_max: "12000000".to_string(),
      decimals: Some("2".to_string()),
      total_supply: Some("21000000".to_string()),
      only: Some("1".to_string()),
    };
    let addr1 =
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();
    let script = ScriptKey::from_address(addr1);
    let inscruptionId =
      InscriptionId::from_str("1111111111111111111111111111111111111111111111111111111111111111i1")
        .unwrap();

    let token = Tick::from_str("orea".to_string().as_str()).unwrap();
    let token_info = TokenInfo {
      tick: token.clone(),
      inscription_id: inscruptionId.clone(),
      inscription_number: 0,
      supply: 21000000000_u128,
      minted: 2000000000_u128,
      limit_per_mint: 0,
      decimal: 3,
      deploy_by: script.clone(),
      deployed_number: 0,
      deployed_timestamp: 0,
      latest_mint_number: 0,
    };
    brc20_data_store.insert_token_info(&token, &token_info);
    let balance = BRC20Banalce {
      overall_balance: 2000000000_u128,
      transferable_balance: 1000000000_u128,
    };
    let result = brc20_data_store.update_token_balance(&script, &token.to_lowercase(), balance);
    match result {
      Err(error) => {
        panic!("update_token_balance err: {}", error)
      }
      _ => {}
    }

    let msg = create_brc30_message(
      inscruptionId.clone(),
      script.clone(),
      script.clone(),
      BRC30Operation::Deploy(deploy.clone()),
    );
    let result = process_deploy(&brc20_data_store, &brc30_data_store, &msg, deploy.clone());

    let result: Result<BRC30Event, BRC30Error> = match result {
      Ok(event) => Ok(event),
      Err(Error::BRC30Error(e)) => Err(e),
      Err(e) => Err(BRC30Error::InternalError(e.to_string())),
    };

    match result {
      Ok(event) => {
        println!("success:{}", serde_json::to_string_pretty(&event).unwrap());
      }
      Err(e) => {
        assert_eq!("error", e.to_string())
      }
    }
    let tick_id = deploy.get_tick_id();
    let pid = deploy.get_pool_id();
    let tickinfo = brc30_data_store.get_tick_info(&tick_id).unwrap().unwrap();
    let poolinfo = brc30_data_store.get_pid_to_poolinfo(&pid).unwrap().unwrap();

    let expectTickINfo = r##"{"tick_id":"c7f75082ae","name":"ordi","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","allocated":1200000000,"decimal":2,"minted":0,"supply":2100000000,"deployer":{"Address":"bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e"},"deploy_block":0,"latest_mint_block":0,"pids":["c7f75082ae#1f"]}"##;
    let expectPoolInfo = r##"{"pid":"c7f75082ae#1f","ptype":"Pool","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","stake":{"BRC20Tick":"orea"},"erate":100000,"minted":0,"staked":0,"dmax":1200000000,"acc_reward_per_share":0,"last_update_block":0,"only":true}"##;
    assert_eq!(expectPoolInfo, serde_json::to_string(&poolinfo).unwrap());
    assert_eq!(expectTickINfo, serde_json::to_string(&tickinfo).unwrap());

    let stakeTick = PledgedTick::BRC20Tick(token.clone());
    let stakeMsg = Stake {
      pool_id: pid.as_str().to_string(),
      amount: "1000000".to_string(),
    };

    let msg = create_brc30_message(
      inscruptionId.clone(),
      script.clone(),
      script.clone(),
      BRC30Operation::Stake(stakeMsg.clone()),
    );
    let result = process_stake(&brc20_data_store, &brc30_data_store, &msg, stakeMsg.clone());

    let result: Result<BRC30Event, BRC30Error> = match result {
      Ok(event) => Ok(event),
      Err(Error::BRC30Error(e)) => Err(e),
      Err(e) => Err(BRC30Error::InternalError(e.to_string())),
    };

    match result {
      Ok(event) => {
        println!("success:{}", serde_json::to_string_pretty(&event).unwrap());
      }
      Err(e) => {
        assert_eq!("error", e.to_string())
      }
    }
    let stakeinfo = brc30_data_store
      .get_user_stakeinfo(&script, &stakeTick)
      .unwrap();

    let userinfo = brc30_data_store.get_pid_to_use_info(&script, &pid).unwrap();
    let poolinfo = brc30_data_store.get_pid_to_poolinfo(&pid).unwrap();
    let expect_stakeinfo =
      r##"{"stake":{"BRC20Tick":"orea"},"pool_stakes":[["c7f75082ae#1f",true,1000000000]]}"##;
    let expect_userinfo = r##"{"pid":"c7f75082ae#1f","staked":1000000000,"reward":0,"reward_debt":0,"latest_updated_block":0}"##;
    let expect_poolinfo = r##"{"pid":"c7f75082ae#1f","ptype":"Pool","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","stake":{"BRC20Tick":"orea"},"erate":100000,"minted":0,"staked":1000000000,"dmax":1200000000,"acc_reward_per_share":0,"last_update_block":0,"only":true}"##;

    assert_eq!(expect_poolinfo, serde_json::to_string(&poolinfo).unwrap());
    assert_eq!(expect_stakeinfo, serde_json::to_string(&stakeinfo).unwrap());
    assert_eq!(expect_userinfo, serde_json::to_string(&userinfo).unwrap());
    {
      let stakeTick = PledgedTick::BRC20Tick(token.clone());
      let passive_unstakeMsg = PassiveUnStake {
        stake: stakeTick.to_string(),
        amount: "1000000".to_string(),
      };
      let mut msg = create_brc30_message(
        inscruptionId.clone(),
        script.clone(),
        script.clone(),
        BRC30Operation::PassiveUnStake(passive_unstakeMsg.clone()),
      );
      msg.block_height = 1;
      let result = process_passive_unstake(
        &brc20_data_store,
        &brc30_data_store,
        &msg,
        passive_unstakeMsg.clone(),
      );

      let result: Result<BRC30Event, BRC30Error> = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => Err(BRC30Error::InternalError(e.to_string())),
      };

      match result {
        Ok(event) => {
          println!("success:{}", serde_json::to_string_pretty(&event).unwrap());
        }
        Err(e) => {
          assert_eq!("error", e.to_string())
        }
      }
      let stakeinfo = brc30_data_store
        .get_user_stakeinfo(&script, &stakeTick)
        .unwrap();

      let userinfo = brc30_data_store.get_pid_to_use_info(&script, &pid).unwrap();
      let poolinfo = brc30_data_store.get_pid_to_poolinfo(&pid).unwrap();
      let expect_stakeinfo =
        r##"{"stake":{"BRC20Tick":"orea"},"pool_stakes":[["c7f75082ae#1f",true,0]]}"##;
      let expect_userinfo = r##"{"pid":"c7f75082ae#1f","staked":0,"reward":100000,"reward_debt":0,"latest_updated_block":0}"##;
      let expect_poolinfo = r##"{"pid":"c7f75082ae#1f","ptype":"Pool","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","stake":{"BRC20Tick":"orea"},"erate":100000,"minted":100000,"staked":0,"dmax":1200000000,"acc_reward_per_share":1,"last_update_block":1,"only":true}"##;
      println!(
        "expect_poolinfo:{}",
        serde_json::to_string(&poolinfo).unwrap()
      );
      println!(
        "expect_stakeinfo:{}",
        serde_json::to_string(&stakeinfo).unwrap()
      );
      println!(
        "expect_userinfo:{}",
        serde_json::to_string(&userinfo).unwrap()
      );

      assert_eq!(expect_poolinfo, serde_json::to_string(&poolinfo).unwrap());
      assert_eq!(expect_stakeinfo, serde_json::to_string(&stakeinfo).unwrap());
      assert_eq!(expect_userinfo, serde_json::to_string(&userinfo).unwrap());
    }
  }
}
