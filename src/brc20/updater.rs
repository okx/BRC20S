use std::str::FromStr;

use super::{
  ActionReceipt, BRC20Event, Balance, Deploy, DeployEvent, Error, EventType, LedgerReadWrite, Mint,
  MintEvent, Num, Operation, Tick, TokenInfo, Transfer, TransferPhase1Event, TransferPhase2Event,
  TransferableLog,
};
use crate::brc20::params::BIGDECIMAL_TEN;
use crate::{
  brc20::{error::BRC20Error, params::MAX_DECIMAL_WIDTH, ScriptKey},
  index::{InscriptionEntryValue, InscriptionIdValue},
  Index, InscriptionId, SatPoint, Txid,
};
use bigdecimal::num_bigint::Sign;
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

pub(crate) struct BRC20Updater<'a, 'db, 'tx, L: LedgerReadWrite> {
  ledger: &'a L,
  id_to_entry: &'a Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
}
impl<'a, 'db, 'tx, L: LedgerReadWrite> BRC20Updater<'a, 'db, 'tx, L> {
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
      let result: Result<BRC20Event, Error<L>> = match operation.action {
        Action::Inscribe(inscribe) => match inscribe {
          Operation::Deploy(deploy) => {
            op = EventType::Deploy;
            self.process_deploy(
              deploy,
              block_number,
              block_time,
              operation.inscription_id,
              inscription_number,
              operation.to_script.clone(),
            )
          }
          Operation::Mint(mint) => {
            op = EventType::Mint;
            self.process_mint(mint, block_number, operation.to_script.clone())
          }
          Operation::Transfer(transfer) => {
            op = EventType::TransferPhase1;
            self.process_inscribe_transfer(
              transfer,
              operation.inscription_id,
              inscription_number,
              operation.to_script.clone(),
            )
          }
        },
        Action::Transfer(_) => {
          op = EventType::TransferPhase2;
          self.process_transfer(
            operation.inscription_id,
            operation.from_script.clone(),
            operation.to_script.clone(),
          )
        }
      };

      let result = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC20Error(e)) => Err(e),
        Err(e) => {
          return Err(e);
        }
      };

      receipts.push(ActionReceipt {
        inscription_id: operation.inscription_id,
        inscription_number,
        op,
        old_satpoint: operation.old_satpoint,
        new_satpoint: operation.new_satpoint,
        from: operation.from_script.clone(),
        to: operation.to_script.map_or(operation.from_script, |v| v),
        result,
      });
    }
    if !receipts.is_empty() {
      self
        .ledger
        .save_transaction_receipts(&txid, &receipts)
        .map_err(|e| Error::LedgerError(e))?;
    }
    Ok(receipts.len())
  }

  fn process_deploy(
    &mut self,
    deploy: Deploy,
    block_number: u64,
    block_time: u32,
    inscription_id: InscriptionId,
    inscription_number: u64,
    to_script_key: Option<ScriptKey>,
  ) -> Result<BRC20Event, Error<L>> {
    let to_script_key = to_script_key.ok_or(BRC20Error::InscribeToCoinbase)?;

    let tick = deploy.tick.parse::<Tick>()?;
    let lower_tick = tick.to_lowercase();

    if let Some(_) = self
      .ledger
      .get_token_info(&lower_tick)
      .map_err(|e| Error::LedgerError(e))?
    {
      return Err(Error::BRC20Error(BRC20Error::DuplicateTick(
        lower_tick.as_str().to_string(),
      )));
    }

    let dec = Num::from_str(&deploy.decimals.map_or(MAX_DECIMAL_WIDTH.to_string(), |v| v))?
      .checked_to_u8()?;
    if dec > MAX_DECIMAL_WIDTH {
      return Err(Error::BRC20Error(BRC20Error::DecimalsTooLarge(dec)));
    }
    let base = BIGDECIMAL_TEN.checked_powu(dec as u64)?;

    let supply = Num::from_str(&deploy.max_supply)?;

    if supply.sign() == Sign::NoSign
      || supply > Into::<Num>::into(u64::MAX)
      || supply.scale() > dec as i64
    {
      return Err(Error::BRC20Error(BRC20Error::InvalidSupply(supply)));
    }

    let limit = Num::from_str(&deploy.mint_limit.map_or(deploy.max_supply, |v| v))?;

    if limit.sign() == Sign::NoSign
      || limit > Into::<Num>::into(u64::MAX)
      || limit.scale() > dec as i64
    {
      return Err(Error::BRC20Error(BRC20Error::MintLimitOutOfRange(
        lower_tick.as_str().to_string(),
        limit,
      )));
    }

    let supply = supply.checked_mul(&base)?.checked_to_u128()?;
    let limit = limit.checked_mul(&base)?.checked_to_u128()?;

    let new_info = TokenInfo {
      inscription_id,
      inscription_number,
      tick,
      decimal: dec,
      supply,
      limit_per_mint: limit,
      minted: 0 as u128,
      deploy_by: to_script_key,
      deployed_number: block_number,
      deployed_timestamp: block_time,
      latest_mint_number: 0 as u64,
    };
    self
      .ledger
      .insert_token_info(&lower_tick, &new_info)
      .map_err(|e| Error::LedgerError(e))?;

    Ok(BRC20Event::Deploy(DeployEvent {
      supply,
      limit_per_mint: limit,
      decimal: dec,
      tick: new_info.tick,
    }))
  }

  fn process_mint(
    &mut self,
    mint: Mint,
    block_number: u64,
    to_script_key: Option<ScriptKey>,
  ) -> Result<BRC20Event, Error<L>> {
    let to_script_key = to_script_key.ok_or(BRC20Error::InscribeToCoinbase)?;
    let tick = mint.tick.parse::<Tick>()?;
    let lower_tick = tick.to_lowercase();

    let token_info = self
      .ledger
      .get_token_info(&lower_tick)
      .map_err(|e| Error::LedgerError(e))?
      .ok_or(BRC20Error::TickNotFound(lower_tick.as_str().to_string()))?;

    let base = BIGDECIMAL_TEN.checked_powu(token_info.decimal as u64)?;

    let mut amt = Num::from_str(&mint.amount)?;

    if amt.scale() > token_info.decimal as i64 {
      return Err(Error::BRC20Error(BRC20Error::AmountOverflow(amt)));
    }

    amt = amt.checked_mul(&base)?;
    if amt.sign() == Sign::NoSign {
      return Err(Error::BRC20Error(BRC20Error::InvalidZeroAmount));
    }
    if amt > Into::<Num>::into(token_info.limit_per_mint) {
      return Err(Error::BRC20Error(BRC20Error::AmountExceedLimit(amt)));
    }
    let minted = Into::<Num>::into(token_info.minted);
    let supply = Into::<Num>::into(token_info.supply);

    if minted >= supply {
      return Err(Error::BRC20Error(BRC20Error::TickMinted(
        token_info.tick.as_str().to_string(),
      )));
    }

    // cut off any excess.
    let mut msg = None;
    amt = if amt.checked_add(&minted)? > supply {
      let new = supply.checked_sub(&minted)?;
      msg = Some(format!(
        "amt has been cut off to fit the supply! origin: {}, now: {}",
        amt.to_string(),
        new.to_string()
      ));
      new
    } else {
      amt
    };

    // get or initialize user balance.
    let mut balance = self
      .ledger
      .get_balance(&to_script_key, &lower_tick)
      .map_err(|e| Error::LedgerError(e))?
      .map_or(Balance::new(), |v| v);

    // add amount to available balance.
    balance.overall_balance = Into::<Num>::into(balance.overall_balance)
      .checked_add(&amt)?
      .checked_to_u128()?;

    // store to database.
    self
      .ledger
      .update_token_balance(&to_script_key, &lower_tick, balance)
      .map_err(|e| Error::LedgerError(e))?;

    // update token minted.
    let minted = minted.checked_add(&amt)?.checked_to_u128()?;
    self
      .ledger
      .update_mint_token_info(&lower_tick, minted, block_number)
      .map_err(|e| Error::LedgerError(e))?;

    Ok(BRC20Event::Mint(MintEvent {
      tick: token_info.tick,
      amount: amt.checked_to_u128()?,
      msg,
    }))
  }

  fn process_inscribe_transfer(
    &mut self,
    transfer: Transfer,
    inscription_id: InscriptionId,
    inscription_number: u64,
    to_script_key: Option<ScriptKey>,
  ) -> Result<BRC20Event, Error<L>> {
    let to_script_key = to_script_key.ok_or(BRC20Error::InscribeToCoinbase)?;
    let tick = transfer.tick.parse::<Tick>()?;
    let lower_tick = tick.to_lowercase();

    let token_info = self
      .ledger
      .get_token_info(&lower_tick)
      .map_err(|e| Error::LedgerError(e))?
      .ok_or(BRC20Error::TickNotFound(lower_tick.as_str().to_string()))?;

    let base = BIGDECIMAL_TEN.checked_powu(token_info.decimal as u64)?;

    let mut amt = Num::from_str(&transfer.amount)?;

    if amt.scale() > token_info.decimal as i64 {
      return Err(Error::BRC20Error(BRC20Error::AmountOverflow(amt)));
    }

    amt = amt.checked_mul(&base)?;
    if amt.sign() == Sign::NoSign || amt > Into::<Num>::into(token_info.supply) {
      return Err(Error::BRC20Error(BRC20Error::AmountOverflow(amt)));
    }

    let mut balance = self
      .ledger
      .get_balance(&to_script_key, &lower_tick)
      .map_err(|e| Error::LedgerError(e))?
      .map_or(Balance::new(), |v| v);

    let overall = Into::<Num>::into(balance.overall_balance);
    let transferable = Into::<Num>::into(balance.transferable_balance);
    let available = overall.checked_sub(&transferable)?;
    if available < amt {
      return Err(Error::BRC20Error(BRC20Error::InsufficientBalance(
        available, amt,
      )));
    }

    balance.transferable_balance = transferable.checked_add(&amt)?.checked_to_u128()?;

    let amt = amt.checked_to_u128()?;

    self
      .ledger
      .update_token_balance(&to_script_key, &lower_tick, balance)
      .map_err(|e| Error::LedgerError(e))?;

    let inscription = TransferableLog {
      inscription_id,
      inscription_number,
      amount: amt,
      tick: token_info.tick,
      owner: to_script_key,
    };
    self
      .ledger
      .insert_transferable(&inscription.owner, &lower_tick, inscription.clone())
      .map_err(|e| Error::LedgerError(e))?;

    Ok(BRC20Event::TransferPhase1(TransferPhase1Event {
      tick: inscription.tick,
      amount: amt,
    }))
  }

  fn process_transfer(
    &mut self,
    inscription_id: InscriptionId,
    from_script_key: ScriptKey,
    to_script_key: Option<ScriptKey>,
  ) -> Result<BRC20Event, Error<L>> {
    let transferable = self
      .ledger
      .get_transferable_by_id(&from_script_key, &inscription_id)
      .map_err(|e| Error::LedgerError(e))?
      .ok_or(BRC20Error::TransferableNotFound(inscription_id))?;

    let amt = Into::<Num>::into(transferable.amount);

    if transferable.owner != from_script_key {
      return Err(Error::BRC20Error(BRC20Error::TransferableOwnerNotMatch(
        inscription_id,
      )));
    }

    let lower_tick = transferable.tick.to_lowercase();

    let token_info = self
      .ledger
      .get_token_info(&lower_tick)
      .map_err(|e| Error::LedgerError(e))?
      .ok_or(BRC20Error::TickNotFound(lower_tick.as_str().to_string()))?;

    // update from key balance.
    let mut from_balance = self
      .ledger
      .get_balance(&from_script_key, &lower_tick)
      .map_err(|e| Error::LedgerError(e))?
      .map_or(Balance::new(), |v| v);

    let from_overall = Into::<Num>::into(from_balance.overall_balance);
    let from_transferable = Into::<Num>::into(from_balance.transferable_balance);

    let from_overall = from_overall.checked_sub(&amt)?.checked_to_u128()?;
    let from_transferable = from_transferable.checked_sub(&amt)?.checked_to_u128()?;

    from_balance.overall_balance = from_overall;
    from_balance.transferable_balance = from_transferable;

    self
      .ledger
      .update_token_balance(&from_script_key, &lower_tick, from_balance)
      .map_err(|e| Error::LedgerError(e))?;

    // redirect receiver to sender if transfer to conibase.
    let mut msg = None;
    let to_script_key = if let None = to_script_key.clone() {
      msg = Some(format!(
        "redirect receiver to sender, reason: transfer inscription to coinbase"
      ));
      from_script_key.clone()
    } else {
      to_script_key.unwrap()
    };
    // update to key balance.
    let mut to_balance = self
      .ledger
      .get_balance(&to_script_key, &lower_tick)
      .map_err(|e| Error::LedgerError(e))?
      .map_or(Balance::new(), |v| v);

    let to_overall = Into::<Num>::into(to_balance.overall_balance);
    to_balance.overall_balance = to_overall.checked_add(&amt)?.checked_to_u128()?;

    self
      .ledger
      .update_token_balance(&to_script_key, &lower_tick, to_balance)
      .map_err(|e| Error::LedgerError(e))?;

    self
      .ledger
      .remove_transferable(&from_script_key, &lower_tick, inscription_id)
      .map_err(|e| Error::LedgerError(e))?;

    Ok(BRC20Event::TransferPhase2(TransferPhase2Event {
      msg,
      tick: token_info.tick,
      amount: amt.checked_to_u128()?,
    }))
  }
}
