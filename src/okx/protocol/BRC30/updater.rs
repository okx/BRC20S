use crate::okx::datastore::BRC30::{
  BRC30DataStoreReadWrite, BRC30Event, BRC30Tick, Balance, DeployPoolEvent, EventType,
  InscribeTransferEvent, MintEvent, Pid, PledgedTick, PoolInfo, PoolType, Receipt, TickId,
  TickInfo, TransferEvent, TransferableAsset,
};
use crate::okx::protocol::BRC30::{operation::*, BRC30Error, Error, Num};
use bigdecimal::num_bigint::Sign;
use std::str::FromStr;

use crate::okx::datastore::ScriptKey;

use crate::okx::datastore::BRC30::PoolType::Pool;
use crate::okx::protocol::BRC30::hash::caculate_tick_id;
use crate::okx::protocol::BRC30::params::{
  BIGDECIMAL_TEN, MAXIMUM_SUPPLY, MAX_DECIMAL_WIDTH, MAX_SUPPLY_WIDTH,
};
use crate::okx::reward::reward;
use crate::{
  index::{InscriptionEntryValue, InscriptionIdValue},
  Index, InscriptionId, SatPoint, Txid,
};
use bigdecimal::ToPrimitive;
use futures::future::ok;
use redb::Table;

#[derive(Clone)]
pub enum Action {
  Inscribe(Operation),
  Transfer(Transfer),
}

pub struct InscriptionData {
  pub txid: Txid,
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
  pub from_script: ScriptKey,
  pub to_script: Option<ScriptKey>,
  pub action: Action,
}

pub(crate) struct BRC30Updater<'a, 'db, 'tx, L: BRC30DataStoreReadWrite> {
  ledger: &'a L,
  id_to_entry: &'a Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
}
impl<'a, 'db, 'tx, L: BRC30DataStoreReadWrite> BRC30Updater<'a, 'db, 'tx, L> {
  pub fn new(
    ledger: &'a L,
    id_to_entry: &'a Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
  ) -> Self {
    Self {
      ledger,
      id_to_entry,
    }
  }

  pub fn index_transaction(
    &mut self,
    block_number: u64,
    block_time: u32,
    txid: Txid,
    operations: Vec<InscriptionData>,
  ) -> Result<usize, Error<L>> {
    let mut receipts = Vec::new();
    for operation in operations {
      let op: EventType;

      let inscription_number =
        Index::get_number_by_inscription_id(self.id_to_entry, operation.inscription_id)
          .map_err(|e| Error::Others(e))?;
      let result: Result<BRC30Event, Error<L>> = match operation.action {
        Action::Inscribe(inscribe) => match inscribe {
          Operation::Deploy(deploy) => {
            op = EventType::DeployTick;

            self.process_deploy(
              deploy,
              block_number,
              operation.inscription_id,
              Some(operation.from_script.clone()),
              operation.to_script.clone(),
            )
          }
          Operation::Stake(stake) => {
            op = EventType::Deposit;
            self.process_stake(stake, block_number, operation.to_script.clone())
          }
          Operation::Mint(mint) => {
            op = EventType::Mint;
            self.process_mint(mint, block_number, operation.to_script.clone())
          }
          Operation::UnStake(unstake) => {
            op = EventType::Withdraw;
            self.process_unstake(unstake, block_number, operation.to_script.clone())
          }
          Operation::Transfer(transfer) => {
            op = EventType::InscribeTransfer;
            self.process_inscribe_transfer(
              transfer,
              operation.inscription_id,
              inscription_number.to_u64().unwrap(),
              operation.to_script.clone(),
            )
          }
        },
        Action::Transfer(_) => {
          op = EventType::Transfer;
          self.process_transfer(
            operation.inscription_id,
            operation.from_script.clone(),
            operation.to_script.clone(),
          )
        }
      };

      let result = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC30Error(e)) => Err(e),
        Err(e) => {
          return Err(e);
        }
      };

      receipts.push(Receipt {
        inscription_id: operation.inscription_id,
        result,
      });
    }
    if !receipts.is_empty() {
      self
        .ledger
        .set_txid_to_receipts(&txid, &receipts)
        .map_err(|e| Error::LedgerError(e))?;
    }
    Ok(receipts.len())
  }

  fn process_deploy(
    &mut self,
    deploy: Deploy,
    block_number: u64,
    inscription_id: InscriptionId,
    from_script_key: Option<ScriptKey>,
    to_script_key: Option<ScriptKey>,
  ) -> Result<BRC30Event, Error<L>> {
    if let Some(iserr) = deploy.validate_basic().err() {
      return Err(Error::BRC30Error(iserr));
    }
    //Prepare the data
    let to_script_key = to_script_key.ok_or(BRC30Error::InscribeToCoinbase)?;
    let from_script_key = from_script_key.ok_or(BRC30Error::InscribeToCoinbase)?;
    let tick_id = deploy.get_tick_id();
    let pid = deploy.get_pool_id();
    let ptype = deploy.get_pool_type();
    if PoolType::Unknown == ptype {
      return Err(Error::BRC30Error(BRC30Error::UnknownPoolType));
    }

    let stake = deploy.get_stake_id();
    if PledgedTick::UNKNOWN == stake {
      return Err(Error::BRC30Error(BRC30Error::UnknownStakeType));
    };

    let erate = deploy.get_earn_rate();
    let only = deploy.get_only();
    let name = deploy.get_earn_id();
    let dmax = deploy.get_distribution_max();

    // check pool is exist, if true return error
    if let Some(_) = self
      .ledger
      .get_pid_to_poolinfo(&pid)
      .map_err(|e| Error::LedgerError(e))?
    {
      return Err(Error::BRC30Error(BRC30Error::PoolAlreadyExist(pid.hex())));
    }

    //Get or create the tick
    if let Some(mut temp_tick) = self
      .ledger
      .get_tick_info(&tick_id)
      .map_err(|e| Error::LedgerError(e))?
    {
      if temp_tick.name != name {
        return Err(Error::BRC30Error(BRC30Error::TickNameNotMatch(
          deploy.earn.clone(),
        )));
      }

      // check stake has exist in tick's pools
      if let Some(_) = self
        .ledger
        .get_tickid_stake_to_pid(&tick_id, &stake)
        .map_err(|e| Error::LedgerError(e))?
      {
        return Err(Error::BRC30Error(BRC30Error::StakeAlreadyExist(
          stake.to_string(),
          tick_id.to_lowercase().hex(),
        )));
      }

      // check dmax
      if temp_tick.supply - temp_tick.allocated < dmax {
        return Err(Error::BRC30Error(BRC30Error::InsufficientTickSupply(
          deploy.distribution_max,
        )));
      }
      let new_allocated = temp_tick.allocated + dmax;
      temp_tick.allocated = new_allocated;
      temp_tick.pids.push(pid.clone());
      self
        .ledger
        .set_tick_info(&tick_id, &temp_tick)
        .map_err(|e| Error::LedgerError(e))?;
    } else {
      let decimal = Num::from_str(&deploy.decimals.map_or(MAX_DECIMAL_WIDTH.to_string(), |v| v))?
        .checked_to_u8()?;
      if decimal > MAX_DECIMAL_WIDTH {
        return Err(Error::BRC30Error(BRC30Error::DecimalsTooLarge(decimal)));
      }
      let base = BIGDECIMAL_TEN.checked_powu(decimal as u64)?;

      let total_supply = Num::from_str(&deploy.total_supply.ok_or(Error::BRC30Error(
        BRC30Error::InvalidSupply(Num::from(0_u128)),
      ))?)?;

      if total_supply.sign() == Sign::NoSign
        || total_supply > Into::<Num>::into(u64::MAX)
        || total_supply.scale() > decimal as i64
      {
        return Err(Error::BRC30Error(BRC30Error::InvalidSupply(total_supply)));
      }

      if tick_id
        != caculate_tick_id(
          total_supply.checked_to_u128()?,
          decimal,
          &from_script_key,
          &to_script_key,
        )
      {
        return Err(Error::BRC30Error(BRC30Error::InvalidTickId(tick_id.hex())));
      }

      let supply = total_supply.checked_mul(&base)?.checked_to_u128()?;
      let pids = vec![pid.clone()];
      let tick = TickInfo::new(
        tick_id,
        &name,
        &inscription_id,
        0_u128,
        decimal,
        0_u128,
        supply,
        &to_script_key,
        block_number,
        block_number,
        pids,
      );
      self
        .ledger
        .set_tick_info(&tick_id, &tick)
        .map_err(|e| Error::LedgerError(e))?;
    };

    let pool = PoolInfo::new(
      &pid,
      &ptype,
      &inscription_id,
      &stake,
      erate,
      0,
      0,
      dmax,
      0, //TODO need change
      block_number,
      only,
    );

    self
      .ledger
      .set_pid_to_poolinfo(&pool.pid, &pool)
      .map_err(|e| Error::LedgerError(e))?;
    Ok(BRC30Event::DeployPool(DeployPoolEvent {
      pid,
      ptype,
      stake,
      erate,
      dmax,
    }))
  }

  fn process_stake(
    &mut self,
    stake: Stake,
    block_number: u64,
    to_script_key: Option<ScriptKey>,
  ) -> Result<BRC30Event, Error<L>> {
    return Err(Error::BRC30Error(BRC30Error::InternalError("".to_string())));
  }

  fn process_unstake(
    &mut self,
    unstake: UnStake,
    block_number: u64,
    to_script_key: Option<ScriptKey>,
  ) -> Result<BRC30Event, Error<L>> {
    return Err(Error::BRC30Error(BRC30Error::InternalError("".to_string())));
  }

  fn process_mint(
    &mut self,
    mint: Mint,
    block_number: u64,
    to_script_key: Option<ScriptKey>,
  ) -> Result<BRC30Event, Error<L>> {
    let to_script_key = to_script_key.ok_or(BRC30Error::InscribeToCoinbase)?;
    // check tick
    let tick_id = TickId::from_str(mint.tick_id.as_str())?;
    let mut tick_info = self
      .ledger
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
      let user_info = if let Ok(Some(u)) = self.ledger.get_pid_to_use_info(&to_script_key, &pid) {
        u
      } else {
        continue;
      };
      let pool_info = if let Ok(Some(p)) = self.ledger.get_pid_to_poolinfo(&pid) {
        p
      } else {
        continue;
      };

      let reward = if let Ok(r) = reward::query_reward(user_info, pool_info, block_number) {
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

      let mut user_info = self
        .ledger
        .get_pid_to_use_info(&to_script_key, &pid)
        .map_err(|e| Error::LedgerError(e))?
        .ok_or(BRC30Error::InternalError(String::from(
          "user info not found",
        )))?;
      let mut pool_info = self
        .ledger
        .get_pid_to_poolinfo(&pid)
        .map_err(|e| Error::LedgerError(e))?
        .ok_or(BRC30Error::InternalError(String::from("pool not found")))?;

      let withdraw_reward = reward::withdraw_user_reward(&mut user_info, &mut pool_info)
        .map_err(|e| Error::LedgerError(e))?;
      if withdraw_reward > reward.checked_to_u128()? {
        user_info.reward = user_info.reward - withdraw_reward + reward.checked_to_u128()?;
        pool_info.minted = pool_info.minted - withdraw_reward + reward.checked_to_u128()?;
      } else {
        reward = Num::from(withdraw_reward)
      }

      remain_amt = remain_amt.checked_sub(&reward)?;
    }

    // update tick info
    tick_info.minted += amt.checked_to_u128()?;
    tick_info.latest_mint_block = block_number;
    self
      .ledger
      .set_tick_info(&tick_id, &tick_info)
      .map_err(|e| Error::LedgerError(e))?;

    // update user balance
    let mut user_balance = self
      .ledger
      .get_balance(&to_script_key, &tick_id)
      .map_err(|e| Error::LedgerError(e))?
      .map_or(Balance::new(tick_id.clone()), |v| v);

    user_balance.overall_balance = Into::<Num>::into(user_balance.overall_balance)
      .checked_add(&amt)?
      .checked_to_u128()?;

    self
      .ledger
      .set_token_balance(&to_script_key, &tick_id, user_balance)
      .map_err(|e| Error::LedgerError(e))?;

    Ok(BRC30Event::Mint(MintEvent {
      tick_id,
      amt: amt.checked_to_u128()?,
    }))
  }

  fn process_inscribe_transfer(
    &mut self,
    transfer: Transfer,
    inscription_id: InscriptionId,
    inscription_number: u64,
    to_script_key: Option<ScriptKey>,
  ) -> Result<BRC30Event, Error<L>> {
    let to_script_key = to_script_key.ok_or(BRC30Error::InscribeToCoinbase)?;
    // check tick
    let tick_id = TickId::from_str(transfer.tick_id.as_str())?;
    let tick_info = self
      .ledger
      .get_tick_info(&tick_id)
      .map_err(|e| Error::LedgerError(e))?
      .ok_or(BRC30Error::TickNotFound(tick_id.as_str().to_string()))?;

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
    let mut balance = self
      .ledger
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
    self
      .ledger
      .set_token_balance(&to_script_key, &tick_id, balance)
      .map_err(|e| Error::LedgerError(e))?;

    // insert transferable assets
    let amount = amt.checked_to_u128()?;
    let transferable_assets = TransferableAsset {
      inscription_id,
      amount,
      tick_id,
      owner: to_script_key.clone(),
    };
    self
      .ledger
      .set_transferable_assets(
        &to_script_key,
        &tick_id,
        &inscription_id,
        &transferable_assets,
      )
      .map_err(|e| Error::LedgerError(e))?;

    Ok(BRC30Event::InscribeTransfer(InscribeTransferEvent {
      tick_id,
      amt: amount,
    }))
  }

  fn process_transfer(
    &mut self,
    inscription_id: InscriptionId,
    from_script_key: ScriptKey,
    to_script_key: Option<ScriptKey>,
  ) -> Result<BRC30Event, Error<L>> {
    let transferable = self
      .ledger
      .get_transferable_by_id(&from_script_key, &inscription_id)
      .map_err(|e| Error::LedgerError(e))?
      .ok_or(BRC30Error::TransferableNotFound(inscription_id))?;

    let amt = Into::<Num>::into(transferable.amount);

    if transferable.owner != from_script_key {
      return Err(Error::BRC30Error(BRC30Error::TransferableOwnerNotMatch(
        inscription_id,
      )));
    }

    let tick_info = self
      .ledger
      .get_tick_info(&transferable.tick_id)
      .map_err(|e| Error::LedgerError(e))?
      .ok_or(BRC30Error::TickNotFound(
        transferable.tick_id.as_str().to_string(),
      ))?;

    // update from key balance.
    let mut from_balance = self
      .ledger
      .get_balance(&from_script_key, &transferable.tick_id)
      .map_err(|e| Error::LedgerError(e))?
      .map_or(Balance::new(transferable.tick_id), |v| v);

    let from_overall = Into::<Num>::into(from_balance.overall_balance);
    let from_transferable = Into::<Num>::into(from_balance.transferable_balance);

    let from_overall = from_overall.checked_sub(&amt)?.checked_to_u128()?;
    let from_transferable = from_transferable.checked_sub(&amt)?.checked_to_u128()?;

    from_balance.overall_balance = from_overall;
    from_balance.transferable_balance = from_transferable;

    self
      .ledger
      .set_token_balance(&from_script_key, &transferable.tick_id, from_balance)
      .map_err(|e| Error::LedgerError(e))?;

    // redirect receiver to sender if transfer to conibase.
    let to_script_key = if let None = to_script_key.clone() {
      from_script_key.clone()
    } else {
      to_script_key.unwrap()
    };

    // update to key balance.
    let mut to_balance = self
      .ledger
      .get_balance(&to_script_key, &transferable.tick_id)
      .map_err(|e| Error::LedgerError(e))?
      .map_or(Balance::new(transferable.tick_id), |v| v);

    let to_overall = Into::<Num>::into(to_balance.overall_balance);
    to_balance.overall_balance = to_overall.checked_add(&amt)?.checked_to_u128()?;

    self
      .ledger
      .set_token_balance(&to_script_key, &transferable.tick_id, to_balance)
      .map_err(|e| Error::LedgerError(e))?;

    self
      .ledger
      .remove_transferable(&from_script_key, &transferable.tick_id, &inscription_id)
      .map_err(|e| Error::LedgerError(e))?;

    Ok(BRC30Event::Transfer(TransferEvent {
      tick_id: transferable.tick_id,
      amt: amt.checked_to_u128()?,
    }))
  }
}
