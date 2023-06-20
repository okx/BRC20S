// use crate::okx::datastore::BRC20::BRC20DataStoreReadWrite;
// use crate::okx::datastore::BRC30::{
//   BRC30DataStoreReadWrite, BRC30Event, BRC30Receipt, BRC30Tick, Balance, DeployPoolEvent,
//   DepositEvent, EventType, InscribeTransferEvent, MintEvent, PassiveWithdrawEvent, Pid,
//   PledgedTick, PoolInfo, PoolType, StakeInfo, TickId, TickInfo, TransferEvent, TransferableAsset,
//   UserInfo, WithdrawEvent,
// };
// use crate::okx::protocol::BRC30::{operation::*, BRC30Error, Error, Num};
// use bigdecimal::num_bigint::Sign;
// use std::cmp;
// use std::str::FromStr;
//
// use crate::okx::datastore::balance::{
//   convert_amount_with_decimal, convert_pledged_tick_with_decimal,
//   convert_pledged_tick_without_decimal, get_user_common_balance, stake_is_exist,
// };
// use crate::okx::datastore::ScriptKey;
// use crate::okx::datastore::BRC30::PoolType::Pool;
// use crate::okx::protocol::BRC30::hash::caculate_tick_id;
// use crate::okx::protocol::BRC30::params::{
//   BIGDECIMAL_TEN, MAXIMUM_SUPPLY, MAX_DECIMAL_WIDTH, MAX_SUPPLY_WIDTH,
// };
// use crate::okx::reward::reward;
// use crate::{
//   index::{InscriptionEntryValue, InscriptionIdValue},
//   Index, InscriptionId, SatPoint, Txid,
// };
// use bigdecimal::ToPrimitive;
// use futures::future::ok;
// use redb::Table;
//
// #[derive(Clone)]
// pub enum Action {
//   Inscribe(Operation),
//   Transfer(Transfer),
// }
//
// pub struct InscriptionData {
//   pub txid: Txid,
//   pub inscription_id: InscriptionId,
//   pub old_satpoint: SatPoint,
//   pub new_satpoint: Option<SatPoint>,
//   pub from_script: ScriptKey,
//   pub to_script: Option<ScriptKey>,
//   pub action: Action,
// }
//
// pub(crate) struct BRC30Updater<'a, 'db, 'tx, L: BRC30DataStoreReadWrite, M: BRC20DataStoreReadWrite>
// {
//   ledger: &'a L,
//   brc20ledger: &'a M,
//   id_to_entry: &'a Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
// }
// impl<'a, 'db, 'tx, L: BRC30DataStoreReadWrite, M: BRC20DataStoreReadWrite>
//   BRC30Updater<'a, 'db, 'tx, L, M>
// {
//   pub fn new(
//     ledger: &'a L,
//     brc20ledge: &'a M,
//     id_to_entry: &'a Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
//   ) -> Self {
//     Self {
//       ledger,
//       brc20ledger: brc20ledge,
//       id_to_entry,
//     }
//   }
//
//   pub fn index_transaction(
//     &mut self,
//     block_number: u64,
//     block_time: u32,
//     txid: Txid,
//     operations: Vec<InscriptionData>,
//   ) -> Result<usize, Error<L>> {
//     let mut receipts = Vec::new();
//     for operation in operations {
//       let op: EventType;
//
//       let inscription_number =
//         Index::get_number_by_inscription_id(self.id_to_entry, operation.inscription_id)
//           .map_err(|e| Error::Others(e))?;
//       let result: Result<BRC30Event, Error<L>> = match operation.action {
//         Action::Inscribe(inscribe) => match inscribe {
//           Operation::Deploy(deploy) => {
//             op = EventType::DeployTick;
//
//             self.process_deploy(
//               deploy,
//               block_number,
//               operation.inscription_id,
//               Some(operation.from_script.clone()),
//               operation.to_script.clone(),
//             )
//           }
//           Operation::Stake(stake) => {
//             op = EventType::Deposit;
//             self.process_stake(stake, block_number, operation.to_script.clone())
//           }
//           Operation::Mint(mint) => {
//             op = EventType::Mint;
//             self.process_mint(mint, block_number, operation.to_script.clone())
//           }
//           Operation::UnStake(unstake) => {
//             op = EventType::Withdraw;
//             self.process_unstake(unstake, block_number, operation.to_script.clone())
//           }
//           Operation::PassiveUnStake(passive_unstake) => {
//             op = EventType::PassiveWithdraw;
//             self.process_passive_unstake(passive_unstake, block_number, operation.to_script.clone())
//           }
//           Operation::Transfer(transfer) => {
//             op = EventType::InscribeTransfer;
//             self.process_inscribe_transfer(
//               transfer,
//               operation.inscription_id,
//               inscription_number.to_u64().unwrap(),
//               operation.to_script.clone(),
//             )
//           }
//         },
//         Action::Transfer(_) => {
//           op = EventType::Transfer;
//           self.process_transfer(
//             operation.inscription_id,
//             operation.from_script.clone(),
//             operation.to_script.clone(),
//           )
//         }
//       };
//
//       let result = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => {
//           return Err(e);
//         }
//       };
//
//       receipts.push(BRC30Receipt {
//         inscription_id: operation.inscription_id,
//         result,
//       });
//     }
//     if !receipts.is_empty() {
//       self
//         .ledger
//         .set_txid_to_receipts(&txid, &receipts)
//         .map_err(|e| Error::LedgerError(e))?;
//     }
//     Ok(receipts.len())
//   }
//
//   pub fn process_deploy(
//     &mut self,
//     deploy: Deploy,
//     block_number: u64,
//     inscription_id: InscriptionId,
//     from_script_key: Option<ScriptKey>,
//     to_script_key: Option<ScriptKey>,
//   ) -> Result<BRC30Event, Error<L>> {
//     if let Some(iserr) = deploy.validate_basic().err() {
//       return Err(Error::BRC30Error(iserr));
//     }
//     //Prepare the data
//     let to_script_key = to_script_key.ok_or(BRC30Error::InscribeToCoinbase)?;
//     let from_script_key = from_script_key.ok_or(BRC30Error::InscribeToCoinbase)?;
//     let tick_id = deploy.get_tick_id();
//     let pid = deploy.get_pool_id();
//     let ptype = deploy.get_pool_type();
//     if PoolType::Unknown == ptype {
//       return Err(Error::BRC30Error(BRC30Error::UnknownPoolType));
//     }
//
//     let stake = deploy.get_stake_id();
//     if PledgedTick::UNKNOWN == stake {
//       return Err(Error::BRC30Error(BRC30Error::UnknownStakeType));
//     };
//
//     let erate = deploy.get_earn_rate();
//     let only = deploy.get_only();
//     let name = deploy.get_earn_id();
//     let dmax_str = deploy.distribution_max.as_str();
//     let mut dmax = 0_u128;
//
//     //check stake
//     if !stake_is_exist(&stake, self.ledger, self.brc20ledger) {
//       return Err(Error::BRC30Error(BRC30Error::StakeNotFound(
//         stake.to_string(),
//       )));
//     }
//     // check pool is exist, if true return error
//     if let Some(_) = self
//       .ledger
//       .get_pid_to_poolinfo(&pid)
//       .map_err(|e| Error::LedgerError(e))?
//     {
//       return Err(Error::BRC30Error(BRC30Error::PoolAlreadyExist(
//         pid.as_str().to_string(),
//       )));
//     }
//
//     //Get or create the tick
//     if let Some(mut temp_tick) = self
//       .ledger
//       .get_tick_info(&tick_id)
//       .map_err(|e| Error::LedgerError(e))?
//     {
//       if temp_tick.name != name {
//         return Err(Error::BRC30Error(BRC30Error::TickNameNotMatch(
//           deploy.earn.clone(),
//         )));
//       }
//
//       if !temp_tick.deployer.eq(&to_script_key) {
//         return Err(Error::BRC30Error(BRC30Error::DeployerNotEqual(
//           pid.hex(),
//           temp_tick.deployer.to_string(),
//           to_script_key.to_string(),
//         )));
//       }
//
//       if !to_script_key.eq(&from_script_key) {
//         return Err(Error::BRC30Error(BRC30Error::FromToNotEqual(
//           from_script_key.to_string(),
//           to_script_key.to_string(),
//         )));
//       }
//
//       // check stake has exist in tick's pools
//       if let Some(_) = self
//         .ledger
//         .get_tickid_stake_to_pid(&tick_id, &stake)
//         .map_err(|e| Error::LedgerError(e))?
//       {
//         return Err(Error::BRC30Error(BRC30Error::StakeAlreadyExist(
//           stake.to_string(),
//           tick_id.to_lowercase().hex(),
//         )));
//       }
//
//       dmax = convert_amount_with_decimal(dmax_str.clone(), temp_tick.decimal)?.checked_to_u128()?;
//       // check dmax
//       if temp_tick.supply - temp_tick.allocated < dmax {
//         return Err(Error::BRC30Error(BRC30Error::InsufficientTickSupply(
//           deploy.distribution_max,
//         )));
//       }
//       temp_tick.allocated = temp_tick.allocated + dmax;
//       temp_tick.pids.push(pid.clone());
//       self
//         .ledger
//         .set_tick_info(&tick_id, &temp_tick)
//         .map_err(|e| Error::LedgerError(e))?;
//     } else {
//       let decimal = Num::from_str(&deploy.decimals.map_or(MAX_DECIMAL_WIDTH.to_string(), |v| v))?
//         .checked_to_u8()?;
//       if decimal > MAX_DECIMAL_WIDTH {
//         return Err(Error::BRC30Error(BRC30Error::DecimalsTooLarge(decimal)));
//       }
//       let base = BIGDECIMAL_TEN.checked_powu(decimal as u64)?;
//
//       let total_supply = Num::from_str(&deploy.total_supply.ok_or(Error::BRC30Error(
//         BRC30Error::InvalidSupply(Num::from(0_u128)),
//       ))?)?;
//
//       if total_supply.sign() == Sign::NoSign
//         || total_supply > Into::<Num>::into(u64::MAX)
//         || total_supply.scale() > decimal as i64
//       {
//         return Err(Error::BRC30Error(BRC30Error::InvalidSupply(total_supply)));
//       }
//
//       let c_tick_id = caculate_tick_id(
//         total_supply.checked_to_u128()?,
//         decimal,
//         &from_script_key,
//         &to_script_key,
//       );
//       if !c_tick_id.to_lowercase().eq(&tick_id) {
//         return Err(Error::BRC30Error(BRC30Error::InvalidPoolTickId(
//           tick_id.hex(),
//           c_tick_id.hex(),
//         )));
//       }
//
//       let supply = total_supply.checked_mul(&base)?.checked_to_u128()?;
//       let pids = vec![pid.clone()];
//       dmax = convert_amount_with_decimal(dmax_str.clone(), decimal)?.checked_to_u128()?;
//       let tick = TickInfo::new(
//         tick_id,
//         &name,
//         &inscription_id,
//         dmax,
//         decimal,
//         0_u128,
//         supply,
//         &to_script_key,
//         block_number,
//         block_number,
//         pids,
//       );
//       self
//         .ledger
//         .set_tick_info(&tick_id, &tick)
//         .map_err(|e| Error::LedgerError(e))?;
//     };
//
//     let pool = PoolInfo::new(
//       &pid,
//       &ptype,
//       &inscription_id,
//       &stake,
//       erate,
//       0,
//       0,
//       dmax,
//       0, //TODO need change
//       block_number,
//       only,
//     );
//
//     self
//       .ledger
//       .set_pid_to_poolinfo(&pool.pid, &pool)
//       .map_err(|e| Error::LedgerError(e))?;
//     self
//       .ledger
//       .set_tickid_stake_to_pid(&tick_id, &stake, &pid)
//       .map_err(|e| Error::LedgerError(e))?;
//     Ok(BRC30Event::DeployPool(DeployPoolEvent {
//       pid,
//       ptype,
//       stake,
//       erate,
//       dmax,
//     }))
//   }
//
//   fn process_stake(
//     &mut self,
//     stakeMsg: Stake,
//     block_number: u64,
//     to_script_key: Option<ScriptKey>,
//   ) -> Result<BRC30Event, Error<L>> {
//     if let Some(iserr) = stakeMsg.validate_basics().err() {
//       return Err(Error::BRC30Error(iserr));
//     }
//     let pool_id = stakeMsg.get_pool_id();
//
//     let to_script_key = to_script_key.ok_or(BRC30Error::InscribeToCoinbase)?;
//
//     let mut pool = self
//       .ledger
//       .get_pid_to_poolinfo(&pool_id)
//       .map_err(|e| Error::LedgerError(e))?
//       .ok_or(Error::BRC30Error(BRC30Error::PoolNotExist(pool_id.hex())))?;
//
//     let stake_tick = pool.stake.clone();
//     let amount = convert_pledged_tick_with_decimal(
//       &stake_tick,
//       stakeMsg.amount.as_str(),
//       self.ledger,
//       self.brc20ledger,
//     )?;
//
//     // check user balance of stake is more than ammount to staked
//     let stake_balance =
//       get_user_common_balance(&to_script_key, &stake_tick, self.ledger, self.brc20ledger);
//     let mut userinfo = self
//       .ledger
//       .get_pid_to_use_info(&to_script_key, &pool_id)
//       .map_or(Some(UserInfo::default(&pool_id)), |v| v)
//       .unwrap();
//     let has_staked = Num::from(userinfo.staked);
//     if stake_balance.lt(&has_staked) {
//       return Err(Error::BRC30Error(BRC30Error::InValidStakeInfo(
//         userinfo.staked,
//         stake_balance.checked_to_u128()?,
//       )));
//     } else if stake_balance.checked_sub(&has_staked)?.lt(&amount) {
//       return Err(Error::BRC30Error(BRC30Error::InsufficientBalance(
//         amount.clone(),
//         stake_balance.checked_sub(&has_staked)?,
//       )));
//     }
//     reward::update_pool(&mut pool, block_number)?;
//     let reward = reward::withdraw_user_reward(&mut userinfo, &mut pool)?;
//     reward::update_user_stake(&mut userinfo, &mut pool)?;
//     // updated user balance of stakedhehe =
//     userinfo.staked = has_staked.checked_add(&amount)?.checked_to_u128()?;
//     self
//       .ledger
//       .set_pid_to_use_info(&to_script_key, &pool_id, &userinfo)
//       .map_err(|e| Error::LedgerError(e))?;
//
//     //update the stake_info of user
//     let mut user_stakeinfo = self
//       .ledger
//       .get_user_stakeinfo(&to_script_key, &stake_tick)
//       .map_err(|e| Error::LedgerError(e))?
//       .map_or(
//         StakeInfo::new(
//           &vec![(pool_id.clone(), pool.only, userinfo.staked)],
//           &stake_tick,
//           0,
//           0,
//         ),
//         |v| v,
//       );
//
//     for pool_stake in user_stakeinfo.pool_stakes.iter_mut() {
//       if pool_stake.0 == pool_id {
//         pool_stake.2 = userinfo.staked;
//         break;
//       }
//     }
//
//     if pool.only {
//       user_stakeinfo.total_only = Num::from(user_stakeinfo.total_only)
//         .checked_add(&amount)?
//         .checked_to_u128()?;
//     } else {
//       user_stakeinfo.max_share = cmp::max(user_stakeinfo.max_share, userinfo.staked)
//     }
//     self
//       .ledger
//       .set_user_stakeinfo(&to_script_key, &stake_tick, &user_stakeinfo)
//       .map_err(|e| Error::LedgerError(e))?;
//
//     // update pool_info for stake
//     pool.staked = Num::from(pool.staked)
//       .checked_add(&amount)?
//       .checked_to_u128()?;
//     self
//       .ledger
//       .set_pid_to_poolinfo(&pool_id, &pool)
//       .map_err(|e| Error::LedgerError(e))?;
//
//     return Ok(BRC30Event::Deposit(DepositEvent {
//       pid: pool_id,
//       amt: amount.checked_to_u128()?,
//     }));
//   }
//
//   fn process_unstake(
//     &mut self,
//     unstakeMsg: UnStake,
//     block_number: u64,
//     to_script_key: Option<ScriptKey>,
//   ) -> Result<BRC30Event, Error<L>> {
//     if let Some(iserr) = unstakeMsg.validate_basics().err() {
//       return Err(Error::BRC30Error(iserr));
//     }
//     let pool_id = unstakeMsg.get_pool_id();
//     let to_script_key = to_script_key.ok_or(BRC30Error::InscribeToCoinbase)?;
//
//     let mut pool = self
//       .ledger
//       .get_pid_to_poolinfo(&pool_id)
//       .map_err(|e| Error::LedgerError(e))?
//       .ok_or(Error::BRC30Error(BRC30Error::PoolNotExist(pool_id.hex())))?;
//
//     let stake_tick = pool.stake.clone();
//
//     let amount = convert_pledged_tick_with_decimal(
//       &stake_tick,
//       unstakeMsg.amount.as_str(),
//       self.ledger,
//       self.brc20ledger,
//     )?;
//
//     let mut userinfo = self
//       .ledger
//       .get_pid_to_use_info(&to_script_key, &pool_id)
//       .map_or(Some(UserInfo::default(&pool_id)), |v| v)
//       .unwrap();
//     let has_staked = Num::from(userinfo.staked);
//     if has_staked.lt(&amount) {
//       return Err(Error::BRC30Error(BRC30Error::InsufficientBalance(
//         has_staked.clone(),
//         amount.clone(),
//       )));
//     }
//
//     reward::update_pool(&mut pool, block_number)?;
//     let reward = reward::withdraw_user_reward(&mut userinfo, &mut pool)?;
//     reward::update_user_stake(&mut userinfo, &mut pool)?;
//
//     userinfo.staked = has_staked.checked_sub(&amount)?.checked_to_u128()?;
//     self
//       .ledger
//       .set_pid_to_use_info(&to_script_key, &pool_id, &userinfo)
//       .map_err(|e| Error::LedgerError(e))?;
//
//     let mut user_stakeinfo = self
//       .ledger
//       .get_user_stakeinfo(&to_script_key, &stake_tick)
//       .map_err(|e| Error::LedgerError(e))?
//       .ok_or(Error::BRC30Error(BRC30Error::InsufficientBalance(
//         Num::from(amount.clone()),
//         Num::from(0_u128),
//       )))?;
//
//     if pool.only {
//       user_stakeinfo.total_only = Num::from(user_stakeinfo.total_only)
//         .checked_sub(&amount)?
//         .checked_to_u128()?;
//     } else {
//       //update pool_stakes
//       for pool_stake in user_stakeinfo.pool_stakes.iter_mut() {
//         if pool_stake.0 == pool_id {
//           pool_stake.2 = userinfo.staked;
//           break;
//         }
//       }
//       //search max stake within share pools
//       let max_pool_stakes = user_stakeinfo
//         .pool_stakes.iter()
//         .filter(|(pid, only, staked)| *only)//filter only pool
//         .max_by_key(|(pid, only, stake)| stake.clone())//search max stake
//         .ok_or(Error::BRC30Error(BRC30Error::InternalError("stakes info can not got max_share".to_string())))?;
//       user_stakeinfo.max_share = max_pool_stakes.2
//     }
//     return Ok(BRC30Event::Withdraw(WithdrawEvent {
//       pid: pool_id,
//       amt: amount.checked_to_u128()?,
//       initiative: false,
//     }));
//   }
//
//   fn process_passive_unstake(
//     &mut self,
//     passive_unstake: PassiveUnStake,
//     block_number: u64,
//     script_key: Option<ScriptKey>,
//   ) -> Result<BRC30Event, Error<L>> {
//     if let Some(iserr) = passive_unstake.validate_basics().err() {
//       return Err(Error::BRC30Error(iserr));
//     }
//     let to_script_key = script_key.ok_or(BRC30Error::InscribeToCoinbase)?;
//     let stake_tick = passive_unstake.get_stake_tick();
//     let stake_info = self
//       .ledger
//       .get_user_stakeinfo(&to_script_key, &stake_tick)
//       .map_err(|e| Error::LedgerError(e))?
//       .ok_or(Error::BRC30Error(BRC30Error::InsufficientBalance(
//         Num::from(0_u128),
//         Num::from(0_u128),
//       )))?;
//
//     let mut stake_alterive = convert_pledged_tick_with_decimal(
//       &stake_tick,
//       passive_unstake.amount.as_str(),
//       self.ledger,
//       self.brc20ledger,
//     )?;
//
//     let mut max_share = Num::from(0_u128);
//     let mut total_only = Num::from(0_u128);
//     let mut pids: Vec<(Pid, u128)> = Vec::new();
//     for (pid, only, pool_stake) in stake_info.pool_stakes.iter() {
//       let current = max_share.checked_add(&total_only)?;
//       if current.ge(&stake_alterive) {
//         break;
//       }
//       let pool_stake_num = Num::from(*pool_stake);
//       if *only {
//         let remain = stake_alterive.checked_sub(&current)?;
//         if remain.gt(&pool_stake_num) {
//           total_only = total_only.checked_add(&pool_stake_num)?;
//           pids.push((pid.clone(), pool_stake_num.checked_to_u128()?));
//         } else {
//           total_only = total_only.checked_add(&remain)?;
//           pids.push((pid.clone(), remain.checked_to_u128()?));
//         }
//       } else {
//         let remain = stake_alterive.checked_sub(&total_only)?;
//         if remain.gt(&pool_stake_num) {
//           max_share = Num::max(&max_share, &pool_stake_num);
//           pids.push((pid.clone(), pool_stake_num.checked_to_u128()?));
//         } else {
//           max_share = Num::max(&max_share, &remain);
//           pids.push((pid.clone(), remain.checked_to_u128()?));
//         }
//       }
//     }
//     for (pid, stake) in pids.iter() {
//       let withdraw_stake =
//         convert_pledged_tick_without_decimal(&stake_tick, *stake, self.ledger, self.brc20ledger)?;
//       let stakeMsg = UnStake::new(
//         pid.to_lowercase().as_str(),
//         withdraw_stake.to_string().as_str(),
//       );
//       self.process_unstake(stakeMsg, block_number, Some(to_script_key.clone()))?;
//     }
//
//     Ok(BRC30Event::PassiveWithdraw(PassiveWithdrawEvent {
//       pid: pids,
//     }))
//   }
//   fn process_mint(
//     &mut self,
//     mint: Mint,
//     block_number: u64,
//     to_script_key: Option<ScriptKey>,
//   ) -> Result<BRC30Event, Error<L>> {
//     let to_script_key = to_script_key.ok_or(BRC30Error::InscribeToCoinbase)?;
//     // check tick
//     let tick_id = TickId::from_str(mint.tick_id.as_str())?;
//     let mut tick_info = self
//       .ledger
//       .get_tick_info(&tick_id)
//       .map_err(|e| Error::LedgerError(e))?
//       .ok_or(BRC30Error::TickNotFound(mint.tick.clone()))?;
//
//     let tick_name = BRC30Tick::from_str(mint.tick.as_str())?;
//     if tick_info.name != tick_name {
//       return Err(Error::BRC30Error(BRC30Error::TickNameNotMatch(
//         mint.tick.clone(),
//       )));
//     }
//
//     // check amount
//     let mut amt = Num::from_str(&mint.amount)?;
//     if amt.scale() > tick_info.decimal as i64 {
//       return Err(Error::BRC30Error(BRC30Error::AmountOverflow(amt)));
//     }
//     let base = BIGDECIMAL_TEN.checked_powu(tick_info.decimal as u64)?;
//     amt = amt.checked_mul(&base)?;
//     if amt.sign() == Sign::NoSign {
//       return Err(Error::BRC30Error(BRC30Error::InvalidZeroAmount));
//     }
//     // get all staked pools and calculate total reward
//     let mut staked_pools: Vec<(Pid, u128)> = Vec::new();
//     let mut total_reward = 0;
//     for pid in tick_info.pids.clone() {
//       let user_info = if let Ok(Some(u)) = self.ledger.get_pid_to_use_info(&to_script_key, &pid) {
//         u
//       } else {
//         continue;
//       };
//       let pool_info = if let Ok(Some(p)) = self.ledger.get_pid_to_poolinfo(&pid) {
//         p
//       } else {
//         continue;
//       };
//
//       let reward = if let Ok(r) = reward::query_reward(user_info, pool_info, block_number) {
//         r
//       } else {
//         continue;
//       };
//       if reward > 0 {
//         total_reward += reward;
//         staked_pools.push((pid, reward))
//       }
//     }
//     if amt > total_reward.into() {
//       return Err(Error::BRC30Error(BRC30Error::AmountExceedLimit(amt)));
//     }
//
//     // claim rewards
//     let mut remain_amt = amt.clone();
//     for (pid, reward) in staked_pools {
//       let reward = Num::from(reward);
//       if remain_amt <= Num::zero() {
//         break;
//       }
//       let mut reward = if reward < remain_amt {
//         reward
//       } else {
//         remain_amt.clone()
//       };
//
//       let mut user_info = self
//         .ledger
//         .get_pid_to_use_info(&to_script_key, &pid)
//         .map_err(|e| Error::LedgerError(e))?
//         .ok_or(BRC30Error::InternalError(String::from(
//           "user info not found",
//         )))?;
//       let mut pool_info = self
//         .ledger
//         .get_pid_to_poolinfo(&pid)
//         .map_err(|e| Error::LedgerError(e))?
//         .ok_or(BRC30Error::InternalError(String::from("pool not found")))?;
//
//       let withdraw_reward = reward::withdraw_user_reward(&mut user_info, &mut pool_info)?;
//       if withdraw_reward > reward.checked_to_u128()? {
//         user_info.reward = user_info.reward - withdraw_reward + reward.checked_to_u128()?;
//         pool_info.minted = pool_info.minted - withdraw_reward + reward.checked_to_u128()?;
//       } else {
//         reward = Num::from(withdraw_reward)
//       }
//
//       self
//         .ledger
//         .set_pid_to_use_info(&to_script_key, &pid, &user_info)
//         .map_err(|e| Error::LedgerError(e))?;
//       self
//         .ledger
//         .set_pid_to_poolinfo(&pid, &pool_info)
//         .map_err(|e| Error::LedgerError(e))?;
//
//       remain_amt = remain_amt.checked_sub(&reward)?;
//     }
//
//     // update tick info
//     tick_info.minted += amt.checked_to_u128()?;
//     tick_info.latest_mint_block = block_number;
//     self
//       .ledger
//       .set_tick_info(&tick_id, &tick_info)
//       .map_err(|e| Error::LedgerError(e))?;
//
//     // update user balance
//     let mut user_balance = self
//       .ledger
//       .get_balance(&to_script_key, &tick_id)
//       .map_err(|e| Error::LedgerError(e))?
//       .map_or(Balance::new(tick_id.clone()), |v| v);
//
//     user_balance.overall_balance = Into::<Num>::into(user_balance.overall_balance)
//       .checked_add(&amt)?
//       .checked_to_u128()?;
//
//     self
//       .ledger
//       .set_token_balance(&to_script_key, &tick_id, user_balance)
//       .map_err(|e| Error::LedgerError(e))?;
//
//     Ok(BRC30Event::Mint(MintEvent {
//       tick_id,
//       amt: amt.checked_to_u128()?,
//     }))
//   }
//
//   fn process_inscribe_transfer(
//     &mut self,
//     transfer: Transfer,
//     inscription_id: InscriptionId,
//     inscription_number: u64,
//     to_script_key: Option<ScriptKey>,
//   ) -> Result<BRC30Event, Error<L>> {
//     let to_script_key = to_script_key.ok_or(BRC30Error::InscribeToCoinbase)?;
//     // check tick
//     let tick_id = TickId::from_str(transfer.tick_id.as_str())?;
//     let tick_info = self
//       .ledger
//       .get_tick_info(&tick_id)
//       .map_err(|e| Error::LedgerError(e))?
//       .ok_or(BRC30Error::TickNotFound(tick_id.hex()))?;
//
//     let tick_name = BRC30Tick::from_str(transfer.tick.as_str())?;
//     if tick_info.name != tick_name {
//       return Err(Error::BRC30Error(BRC30Error::TickNameNotMatch(
//         transfer.tick.clone(),
//       )));
//     }
//
//     // check amount
//     let mut amt = Num::from_str(&transfer.amount)?;
//     if amt.scale() > tick_info.decimal as i64 {
//       return Err(Error::BRC30Error(BRC30Error::AmountOverflow(amt)));
//     }
//     let base = BIGDECIMAL_TEN.checked_powu(tick_info.decimal as u64)?;
//     amt = amt.checked_mul(&base)?;
//     if amt.sign() == Sign::NoSign {
//       return Err(Error::BRC30Error(BRC30Error::InvalidZeroAmount));
//     }
//
//     // update balance
//     let mut balance = self
//       .ledger
//       .get_balance(&to_script_key, &tick_id)
//       .map_err(|e| Error::LedgerError(e))?
//       .map_or(Balance::new(tick_id.clone()), |v| v);
//
//     let overall = Into::<Num>::into(balance.overall_balance);
//     let transferable = Into::<Num>::into(balance.transferable_balance);
//     let available = overall.checked_sub(&transferable)?;
//     if available < amt {
//       return Err(Error::BRC30Error(BRC30Error::InsufficientBalance(
//         available, amt,
//       )));
//     }
//     balance.transferable_balance = transferable.checked_add(&amt)?.checked_to_u128()?;
//     self
//       .ledger
//       .set_token_balance(&to_script_key, &tick_id, balance)
//       .map_err(|e| Error::LedgerError(e))?;
//
//     // insert transferable assets
//     let amount = amt.checked_to_u128()?;
//     let transferable_assets = TransferableAsset {
//       inscription_id,
//       amount,
//       tick_id,
//       owner: to_script_key.clone(),
//     };
//     self
//       .ledger
//       .set_transferable_assets(
//         &to_script_key,
//         &tick_id,
//         &inscription_id,
//         &transferable_assets,
//       )
//       .map_err(|e| Error::LedgerError(e))?;
//
//     Ok(BRC30Event::InscribeTransfer(InscribeTransferEvent {
//       tick_id,
//       amt: amount,
//     }))
//   }
//
//   fn process_transfer(
//     &mut self,
//     inscription_id: InscriptionId,
//     from_script_key: ScriptKey,
//     to_script_key: Option<ScriptKey>,
//   ) -> Result<BRC30Event, Error<L>> {
//     let transferable = self
//       .ledger
//       .get_transferable_by_id(&from_script_key, &inscription_id)
//       .map_err(|e| Error::LedgerError(e))?
//       .ok_or(BRC30Error::TransferableNotFound(inscription_id))?;
//
//     let amt = Into::<Num>::into(transferable.amount);
//
//     if transferable.owner != from_script_key {
//       return Err(Error::BRC30Error(BRC30Error::TransferableOwnerNotMatch(
//         inscription_id,
//       )));
//     }
//
//     let tick_info = self
//       .ledger
//       .get_tick_info(&transferable.tick_id)
//       .map_err(|e| Error::LedgerError(e))?
//       .ok_or(BRC30Error::TickNotFound(transferable.tick_id.hex()))?;
//
//     // update from key balance.
//     let mut from_balance = self
//       .ledger
//       .get_balance(&from_script_key, &transferable.tick_id)
//       .map_err(|e| Error::LedgerError(e))?
//       .map_or(Balance::new(transferable.tick_id), |v| v);
//
//     let from_overall = Into::<Num>::into(from_balance.overall_balance);
//     let from_transferable = Into::<Num>::into(from_balance.transferable_balance);
//
//     let from_overall = from_overall.checked_sub(&amt)?.checked_to_u128()?;
//     let from_transferable = from_transferable.checked_sub(&amt)?.checked_to_u128()?;
//
//     from_balance.overall_balance = from_overall;
//     from_balance.transferable_balance = from_transferable;
//
//     self
//       .ledger
//       .set_token_balance(&from_script_key, &transferable.tick_id, from_balance)
//       .map_err(|e| Error::LedgerError(e))?;
//
//     // redirect receiver to sender if transfer to conibase.
//     let to_script_key = if let None = to_script_key.clone() {
//       from_script_key.clone()
//     } else {
//       to_script_key.unwrap()
//     };
//
//     // update to key balance.
//     let mut to_balance = self
//       .ledger
//       .get_balance(&to_script_key, &transferable.tick_id)
//       .map_err(|e| Error::LedgerError(e))?
//       .map_or(Balance::new(transferable.tick_id), |v| v);
//
//     let to_overall = Into::<Num>::into(to_balance.overall_balance);
//     to_balance.overall_balance = to_overall.checked_add(&amt)?.checked_to_u128()?;
//
//     self
//       .ledger
//       .set_token_balance(&to_script_key, &transferable.tick_id, to_balance)
//       .map_err(|e| Error::LedgerError(e))?;
//
//     self
//       .ledger
//       .remove_transferable(&from_script_key, &transferable.tick_id, &inscription_id)
//       .map_err(|e| Error::LedgerError(e))?;
//
//     Ok(BRC30Event::Transfer(TransferEvent {
//       tick_id: transferable.tick_id,
//       amt: amt.checked_to_u128()?,
//     }))
//   }
// }
//
// #[cfg(test)]
// mod tests {
//   use super::super::*;
//   use super::*;
//   use crate::index::INSCRIPTION_ID_TO_INSCRIPTION_ENTRY;
//   use crate::okx::datastore::BRC20::redb::BRC20DataStore;
//   use crate::okx::datastore::BRC20::{Tick, TokenInfo};
//   use crate::okx::datastore::BRC30::redb::BRC30DataStore;
//   use crate::okx::datastore::BRC30::BRC30DataStoreReadOnly;
//   use bech32::ToBase32;
//   use bitcoin::Address;
//   use redb::{Database, WriteTransaction};
//   use std::borrow::Borrow;
//   use tempfile::NamedTempFile;
//
//   #[test]
//   fn test_process_deploy() {
//     let dbfile = NamedTempFile::new().unwrap();
//     let db = Database::create(dbfile.path()).unwrap();
//     let wtx = db.begin_write().unwrap();
//     let mut inscription_id_to_inscription_entry =
//       wtx.open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY).unwrap();
//
//     let brc20_data_store = BRC20DataStore::new(&wtx);
//     let brc30_data_store = BRC30DataStore::new(&wtx);
//     let mut brc30update = BRC30Updater::new(
//       &brc30_data_store,
//       &brc20_data_store,
//       &inscription_id_to_inscription_entry,
//     );
//
//     let deploy = Deploy {
//       pool_type: "pool".to_string(),
//       pool_id: "c8195197bc#1f".to_string(),
//       stake: "btc".to_string(),
//       earn: "ordi1".to_string(),
//       earn_rate: "10".to_string(),
//       distribution_max: "12000000".to_string(),
//       decimals: Some("18".to_string()),
//       total_supply: Some("21000000".to_string()),
//       only: Some("1".to_string()),
//     };
//     let addr1 =
//       Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();
//     let script = ScriptKey::from_address(addr1);
//     let inscruptionId =
//       InscriptionId::from_str("1111111111111111111111111111111111111111111111111111111111111111i1")
//         .unwrap();
//     let result = brc30update.process_deploy(
//       deploy.clone(),
//       0,
//       inscruptionId,
//       Some(script.clone()),
//       Some(script.clone()),
//     );
//
//     let result: Result<BRC30Event, BRC30Error> = match result {
//       Ok(event) => Ok(event),
//       Err(Error::BRC30Error(e)) => Err(e),
//       Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//     };
//
//     match result {
//       Ok(event) => {
//         println!("success:{}", serde_json::to_string_pretty(&event).unwrap());
//       }
//       Err(e) => {
//         assert_eq!("error", e.to_string())
//       }
//     }
//     let tick_id = deploy.get_tick_id();
//     let pid = deploy.get_pool_id();
//     let tickinfo = brc30_data_store.get_tick_info(&tick_id).unwrap().unwrap();
//     let poolinfo = brc30_data_store.get_pid_to_poolinfo(&pid).unwrap().unwrap();
//
//     let expectTickINfo = r##"{"tick_id":"c8195197bc","name":"ordi1","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","allocated":12000000000000000000000000,"decimal":18,"minted":0,"supply":21000000000000000000000000,"deployer":{"Address":"bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e"},"deploy_block":0,"latest_mint_block":0,"pids":["c8195197bc#1f"]}"##;
//     let expectPoolInfo = r##"{"pid":"c8195197bc#1f","ptype":"Pool","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","stake":"NATIVE","erate":10,"minted":0,"staked":0,"dmax":12000000000000000000000000,"acc_reward_per_share":0,"last_update_block":0,"only":true}"##;
//     assert_eq!(expectPoolInfo, serde_json::to_string(&poolinfo).unwrap());
//     assert_eq!(expectTickINfo, serde_json::to_string(&tickinfo).unwrap());
//
//     let result = brc30update.process_deploy(
//       deploy.clone(),
//       0,
//       inscruptionId,
//       Some(script.clone()),
//       Some(script.clone()),
//     );
//
//     let result: Result<BRC30Event, BRC30Error> = match result {
//       Ok(event) => Ok(event),
//       Err(Error::BRC30Error(e)) => Err(e),
//       Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//     };
//
//     assert_eq!(
//       Err(BRC30Error::PoolAlreadyExist(pid.as_str().to_string())),
//       result
//     );
//
//     let token = Tick::from_str("orea".to_string().as_str()).unwrap();
//     let token_info = TokenInfo {
//       tick: token.clone(),
//       inscription_id: inscruptionId.clone(),
//       inscription_number: 0,
//       supply: 0,
//       minted: 0,
//       limit_per_mint: 0,
//       decimal: 0,
//       deploy_by: script.clone(),
//       deployed_number: 0,
//       deployed_timestamp: 0,
//       latest_mint_number: 0,
//     };
//     brc20_data_store.insert_token_info(&token, &token_info);
//
//     let mut secondDeply = deploy.clone();
//     secondDeply.pool_id = "c8195197bc#11".to_string();
//     secondDeply.stake = "orea".to_string();
//     secondDeply.distribution_max = "9000000".to_string();
//     let result = brc30update.process_deploy(
//       secondDeply.clone(),
//       0,
//       inscruptionId,
//       Some(script.clone()),
//       Some(script.clone()),
//     );
//
//     let result: Result<BRC30Event, BRC30Error> = match result {
//       Ok(event) => Ok(event),
//       Err(Error::BRC30Error(e)) => Err(e),
//       Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//     };
//
//     assert_ne!(true, result.is_err());
//     let tick_id = secondDeply.get_tick_id();
//     let pid = secondDeply.get_pool_id();
//     let tickinfo = brc30_data_store.get_tick_info(&tick_id).unwrap().unwrap();
//     let poolinfo = brc30_data_store.get_pid_to_poolinfo(&pid).unwrap().unwrap();
//
//     let expectTickINfo = r##"{"tick_id":"c8195197bc","name":"ordi1","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","allocated":21000000000000000000000000,"decimal":18,"minted":0,"supply":21000000000000000000000000,"deployer":{"Address":"bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e"},"deploy_block":0,"latest_mint_block":0,"pids":["c8195197bc#1f","c8195197bc#11"]}"##;
//     let expectPoolInfo = r##"{"pid":"c8195197bc#11","ptype":"Pool","inscription_id":"1111111111111111111111111111111111111111111111111111111111111111i1","stake":{"BRC20Tick":"orea"},"erate":10,"minted":0,"staked":0,"dmax":9000000000000000000000000,"acc_reward_per_share":0,"last_update_block":0,"only":true}"##;
//     assert_eq!(expectPoolInfo, serde_json::to_string(&poolinfo).unwrap());
//     assert_eq!(expectTickINfo, serde_json::to_string(&tickinfo).unwrap());
//   }
//
//   #[test]
//   fn test_process_error_params() {
//     let dbfile = NamedTempFile::new().unwrap();
//     let db = Database::create(dbfile.path()).unwrap();
//     let wtx = db.begin_write().unwrap();
//     let mut inscription_id_to_inscription_entry =
//       wtx.open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY).unwrap();
//
//     let brc20_data_store = BRC20DataStore::new(&wtx);
//     let brc30_data_store = BRC30DataStore::new(&wtx);
//     let mut brc30update = BRC30Updater::new(
//       &brc30_data_store,
//       &brc20_data_store,
//       &inscription_id_to_inscription_entry,
//     );
//
//     let deploy = Deploy {
//       pool_type: "pool".to_string(),
//       pool_id: "c8195197bc#1f".to_string(),
//       stake: "btc".to_string(),
//       earn: "ordi1".to_string(),
//       earn_rate: "10".to_string(),
//       distribution_max: "12000000".to_string(),
//       decimals: Some("18".to_string()),
//       total_supply: Some("21000000".to_string()),
//       only: Some("1".to_string()),
//     };
//     let addr1 =
//       Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();
//     let script = ScriptKey::from_address(addr1);
//     let inscruptionId =
//       InscriptionId::from_str("1111111111111111111111111111111111111111111111111111111111111111i1")
//         .unwrap();
//
//     //err pool type
//     {
//       let mut err_pool_type = deploy.clone();
//       err_pool_type.pool_type = "errtype".to_string();
//       let result = brc30update.process_deploy(
//         err_pool_type.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(Err(BRC30Error::UnknownPoolType), result);
//     }
//
//     //err pid
//     {
//       let mut err_pid = deploy.clone();
//       err_pid.pool_id = "l8195197bc#1f".to_string();
//       let result = brc30update.process_deploy(
//         err_pid.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidPoolId(
//           err_pid.pool_id.clone(),
//           "the prefix of pool id is not hex".to_string()
//         )),
//         result
//       );
//
//       let mut err_pid = deploy.clone();
//       err_pid.pool_id = "8195197bc#1f".to_string();
//       let result = brc30update.process_deploy(
//         err_pid.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidPoolId(
//           err_pid.pool_id.clone(),
//           "pool id length is not 13".to_string()
//         )),
//         result
//       );
//
//       let mut err_pid = deploy.clone();
//       err_pid.pool_id = "c8195197bc#lf".to_string();
//       let result = brc30update.process_deploy(
//         err_pid.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//
//       assert_eq!(
//         Err(BRC30Error::InvalidPoolId(
//           err_pid.pool_id.clone(),
//           "the suffix of pool id is not hex".to_string()
//         )),
//         result
//       );
//
//       let mut err_pid = deploy.clone();
//       err_pid.pool_id = "c81195197bc#f".to_string();
//       let result = brc30update.process_deploy(
//         err_pid.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidPoolId(
//           err_pid.pool_id.clone(),
//           "the prefix of pool id is not hex".to_string()
//         )),
//         result
//       );
//
//       let mut err_pid = deploy.clone();
//       err_pid.pool_id = "c8195197bc$1f".to_string();
//       let result = brc30update.process_deploy(
//         err_pid.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidPoolId(
//           err_pid.pool_id.clone(),
//           "pool id must contains '#'".to_string()
//         )),
//         result
//       );
//
//       let mut err_pid = deploy.clone();
//       err_pid.pool_id = "c819519#bc#df".to_string();
//       let result = brc30update.process_deploy(
//         err_pid.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidPoolId(
//           err_pid.pool_id.clone(),
//           "pool id must contains only one '#'".to_string()
//         )),
//         result
//       );
//
//       let mut err_pid = deploy.clone();
//       err_pid.pool_id = "c819519#bc#1f".to_string();
//       let result = brc30update.process_deploy(
//         err_pid.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidPoolId(
//           err_pid.pool_id.clone(),
//           "pool id must contains only one '#'".to_string()
//         )),
//         result
//       );
//
//       let mut err_pid = deploy.clone();
//       err_pid.pool_id = "a8195197bc#1f".to_string();
//       let result = brc30update.process_deploy(
//         err_pid.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidPoolTickId(
//           "a8195197bc".to_string(),
//           "c8195197bc".to_string()
//         )),
//         result
//       );
//     }
//
//     //err stake,earn
//     {
//       let mut err_stake = deploy.clone();
//       err_stake.stake = "he".to_string();
//       let result = brc30update.process_deploy(
//         err_stake.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(Err(BRC30Error::UnknownStakeType), result);
//
//       let mut err_stake = deploy.clone();
//       err_stake.stake = "hehehh".to_string();
//       let result = brc30update.process_deploy(
//         err_stake.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(Err(BRC30Error::UnknownStakeType), result);
//
//       let mut err_stake = deploy.clone();
//       err_stake.stake = "test".to_string();
//       let result = brc30update.process_deploy(
//         err_stake.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(Err(BRC30Error::StakeNotFound(err_stake.stake)), result);
//
//       let mut err_earn = deploy.clone();
//       err_earn.earn = "tes".to_string();
//       let result = brc30update.process_deploy(
//         err_earn.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidTickLen(err_earn.earn.to_string())),
//         result
//       );
//
//       let mut err_earn = deploy.clone();
//       err_earn.earn = "test".to_string();
//       let result = brc30update.process_deploy(
//         err_earn.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_ne!(
//         Err(BRC30Error::InvalidTickLen(err_earn.earn.to_string())),
//         result
//       );
//
//       let mut err_earn = deploy.clone();
//       err_earn.earn = "testt".to_string();
//       let result = brc30update.process_deploy(
//         err_earn.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_ne!(
//         Err(BRC30Error::InvalidTickLen(err_earn.earn.to_string())),
//         result
//       );
//
//       let mut err_earn = deploy.clone();
//       err_earn.earn = "testttt".to_string();
//       let result = brc30update.process_deploy(
//         err_earn.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidTickLen(err_earn.earn.to_string())),
//         result
//       );
//     }
//     // err erate
//     {
//       let mut err_erate = deploy.clone();
//       err_erate.earn_rate = "".to_string();
//       let result = brc30update.process_deploy(
//         err_erate.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidNum("invalid number: ".to_string())),
//         result
//       );
//
//       let mut err_erate = deploy.clone();
//       err_erate.earn_rate = "1l".to_string();
//       let result = brc30update.process_deploy(
//         err_erate.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidNum("1linvalid number: 1l".to_string())),
//         result
//       );
//     }
//
//     //err dmax
//     {
//       let mut err_dmax = deploy.clone();
//       err_dmax.distribution_max = "".to_string();
//       let result = brc30update.process_deploy(
//         err_dmax.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidNum("invalid number: ".to_string())),
//         result
//       );
//
//       let mut err_dmax = deploy.clone();
//       err_dmax.distribution_max = "1l".to_string();
//       let result = brc30update.process_deploy(
//         err_dmax.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidNum("1linvalid number: 1l".to_string())),
//         result
//       );
//
//       let mut err_dmax = deploy.clone();
//       err_dmax.distribution_max = "21000001".to_string();
//       let result = brc30update.process_deploy(
//         err_dmax.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::ExceedDmax(
//           "21000001".to_string(),
//           "21000000".to_string()
//         )),
//         result
//       );
//     }
//
//     //err total_supply
//     {
//       let mut err_total = deploy.clone();
//       err_total.total_supply = Some("".to_string());
//       let result = brc30update.process_deploy(
//         err_total.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidNum("invalid number: ".to_string())),
//         result
//       );
//
//       let mut err_dmax = deploy.clone();
//       err_dmax.total_supply = Some("1l".to_string());
//       let result = brc30update.process_deploy(
//         err_dmax.clone(),
//         0,
//         inscruptionId,
//         Some(script.clone()),
//         Some(script.clone()),
//       );
//       let pid = deploy.get_pool_id();
//       let result: Result<BRC30Event, BRC30Error> = match result {
//         Ok(event) => Ok(event),
//         Err(Error::BRC30Error(e)) => Err(e),
//         Err(e) => Err(BRC30Error::InternalError(e.to_string())),
//       };
//       assert_eq!(
//         Err(BRC30Error::InvalidNum("1linvalid number: 1l".to_string())),
//         result
//       );
//     }
//   }
// }