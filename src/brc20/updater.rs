use std::str::FromStr;

use super::{
  ActionReceipt, BRC20Event, Balance, Deploy, DeployEvent, Error, LedgerRead, LedgerReadWrite,
  Mint, MintEvent, Num, Operation, Tick, TokenInfo, Transfer, TransferPhase1Event,
  TransferPhase2Event, TransferableLog,
};
use crate::{
  brc20::{error::BRC20Error, params::MAX_DECIMAL_WIDTH, ScriptKey},
  InscriptionId, SatPoint, Txid,
};
use bitcoin::{Network, Script};
use rust_decimal::Decimal;

#[derive(Clone)]
pub enum Action {
  Inscribe(InscribeAction),
  Transfer(TransferAction),
}
impl Action {
  pub fn set_to(&mut self, to: Option<Script>) {
    match self {
      Action::Inscribe(inscribe) => inscribe.to_script = to,
      Action::Transfer(transfer) => transfer.to_script = to,
    }
  }
}

#[derive(Clone)]
pub struct InscribeAction {
  pub operation: Operation,
  pub to_script: Option<Script>,
}

#[derive(Clone)]
pub struct TransferAction {
  pub from_script: Script,
  pub to_script: Option<Script>,
}

pub struct InscriptionData {
  pub txid: Txid,
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
  pub action: Action,
}

pub struct BRC20Updater<'a, L: LedgerReadWrite> {
  ledger: &'a L,
  network: Network,
}
impl<'a, L: LedgerReadWrite> BRC20Updater<'a, L> {
  pub fn new(ledger: &'a L, network: Network) -> Self {
    Self { ledger, network }
  }

  pub fn index_transaction(
    &mut self,
    block_number: u64,
    block_time: u32,
    txid: Txid,
    operations: Vec<InscriptionData>,
  ) -> Result<usize, <L as LedgerRead>::Error> {
    let mut receipts = Vec::new();
    for operation in operations {
      let result = match operation.action {
        Action::Inscribe(inscribe) => match inscribe.operation {
          Operation::Deploy(deploy) => self.process_deploy(
            deploy,
            block_number,
            block_time,
            operation.inscription_id,
            inscribe.to_script,
          ),
          Operation::Mint(mint) => self.process_mint(mint, block_number, inscribe.to_script),
          Operation::Transfer(transfer) => {
            self.process_inscribe_transfer(transfer, operation.inscription_id, inscribe.to_script)
          }
        },
        Action::Transfer(transfer) => self.process_transfer(
          operation.inscription_id,
          transfer.from_script,
          transfer.to_script,
        ),
      };

      // 这里只有BRC20Error 认为是协议error，记录到event中
      let result = match result {
        Ok(event) => Ok(event),
        Err(Error::BRC20Error(e)) => Err(e),
        Err(Error::LedgerError(e)) => {
          return Err(e);
        }
      };

      receipts.push(ActionReceipt {
        inscription_id: operation.inscription_id,
        old_satpoint: operation.old_satpoint,
        new_satpoint: operation.new_satpoint,
        result,
      });
    }
    self.ledger.save_transaction_receipts(&txid, &receipts)?;
    Ok(receipts.len())
  }

  fn process_deploy(
    &mut self,
    deploy: Deploy,
    block_number: u64,
    block_time: u32,
    inscription_id: InscriptionId,
    to_script: Option<Script>,
  ) -> Result<BRC20Event, Error<L>> {
    let to_script = to_script.ok_or(BRC20Error::InscribeToCoinbase)?;

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
      return Err(Error::BRC20Error(BRC20Error::InvalidDecimals(dec)));
    }
    let base = Into::<Num>::into(Decimal::TEN).checked_powu(dec as u64)?;

    let supply = Num::from_str(&deploy.max_supply)?;

    if supply > Into::<Num>::into(u64::MAX) {
      return Err(Error::BRC20Error(BRC20Error::InvalidMaxSupply(supply)));
    }

    let limit = Num::from_str(&deploy.mint_limit.map_or(deploy.max_supply, |v| v))?;

    if limit > supply {
      return Err(Error::BRC20Error(BRC20Error::InvalidMintLimit));
    }

    let supply = supply.checked_mul(base)?.checked_to_u128()?;
    let limit = limit.checked_mul(base)?.checked_to_u128()?;

    let script_key = ScriptKey::from_script(&to_script, self.network);

    let new_info = TokenInfo {
      inscription_id,
      tick,
      decimal: dec,
      supply,
      limit_per_mint: limit,
      minted: 0 as u128,
      deploy_by: script_key,
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
      deploy: new_info.deploy_by,
    }))
  }

  fn process_mint(
    &mut self,
    mint: Mint,
    block_number: u64,
    to_script: Option<Script>,
  ) -> Result<BRC20Event, Error<L>> {
    let to_script = to_script.ok_or(BRC20Error::InscribeToCoinbase)?;
    let tick = mint.tick.parse::<Tick>()?;
    let lower_tick = tick.to_lowercase();

    let token_info = self
      .ledger
      .get_token_info(&lower_tick)
      .map_err(|e| Error::LedgerError(e))?
      .ok_or(BRC20Error::TickNotFound(lower_tick.as_str().to_string()))?;

    let base = Into::<Num>::into(Decimal::TEN).checked_powu(token_info.decimal as u64)?;

    let mut amt = Num::from_str(&mint.amount)?.checked_mul(base)?;

    if amt > Into::<Num>::into(token_info.limit_per_mint) {
      return Err(Error::BRC20Error(BRC20Error::MintAmountExceedLimit(
        token_info.limit_per_mint.to_string(),
      )));
    }
    let minted = Into::<Num>::into(token_info.minted);
    let supply = Into::<Num>::into(token_info.supply);

    if minted >= supply {
      return Err(Error::BRC20Error(BRC20Error::TickMintOut(
        token_info.tick.as_str().to_string(),
      )));
    }

    // cut off any excess.
    let mut msg = None;
    amt = if amt.checked_add(minted)? > supply {
      let new = supply.checked_sub(minted)?;
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
    let script_key = ScriptKey::from_script(&to_script, self.network);
    let mut balance = self
      .ledger
      .get_balance(&script_key, &lower_tick)
      .map_err(|e| Error::LedgerError(e))?
      .map_or(Balance::new(), |v| v);

    // add amount to available balance.
    balance.overall_balance = Into::<Num>::into(balance.overall_balance)
      .checked_add(amt)?
      .checked_to_u128()?;

    // store to database.
    self
      .ledger
      .update_token_balance(&script_key, &lower_tick, balance)
      .map_err(|e| Error::LedgerError(e))?;

    // update token minted.
    let minted = minted.checked_add(amt)?.checked_to_u128()?;
    self
      .ledger
      .update_mint_token_info(&lower_tick, minted, block_number)
      .map_err(|e| Error::LedgerError(e))?;

    Ok(BRC20Event::Mint(MintEvent {
      tick: token_info.tick,
      to: script_key,
      amount: amt.checked_to_u128()?,
      msg,
    }))
  }

  fn process_inscribe_transfer(
    &mut self,
    transfer: Transfer,
    inscription_id: InscriptionId,
    to_script: Option<Script>,
  ) -> Result<BRC20Event, Error<L>> {
    let to_script = to_script.ok_or(BRC20Error::InscribeToCoinbase)?;
    let tick = transfer.tick.parse::<Tick>()?;
    let lower_tick = tick.to_lowercase();

    let token_info = self
      .ledger
      .get_token_info(&lower_tick)
      .map_err(|e| Error::LedgerError(e))?
      .ok_or(BRC20Error::TickNotFound(lower_tick.as_str().to_string()))?;

    let base = Into::<Num>::into(Decimal::TEN).checked_powu(token_info.decimal as u64)?;

    let amt = Num::from_str(&transfer.amount)?.checked_mul(base)?;

    if amt <= Into::<Num>::into(0 as u128) || amt > Into::<Num>::into(token_info.supply) {
      return Err(Error::BRC20Error(BRC20Error::InscribeTransferOverflow(amt)));
    }

    let script_key = ScriptKey::from_script(&to_script, self.network);
    let mut balance = self
      .ledger
      .get_balance(&script_key, &lower_tick)
      .map_err(|e| Error::LedgerError(e))?
      .map_or(Balance::new(), |v| v);

    let overall = Into::<Num>::into(balance.overall_balance);
    let transferable = Into::<Num>::into(balance.transferable_balance);

    if overall.checked_sub(transferable)? < amt {
      return Err(Error::BRC20Error(BRC20Error::InsufficientBalance));
    }

    balance.transferable_balance = transferable.checked_add(amt)?.checked_to_u128()?;

    let amt = amt.checked_to_u128()?;

    self
      .ledger
      .update_token_balance(&script_key, &lower_tick, balance)
      .map_err(|e| Error::LedgerError(e))?;

    let inscription = TransferableLog {
      inscription_id,
      amount: amt,
      tick: token_info.tick,
      owner: script_key,
    };
    self
      .ledger
      .insert_transferable(&inscription.owner, &lower_tick, inscription.clone())
      .map_err(|e| Error::LedgerError(e))?;

    Ok(BRC20Event::TransferPhase1(TransferPhase1Event {
      tick: inscription.tick,
      owner: inscription.owner,
      amount: amt,
    }))
  }

  fn process_transfer(
    &mut self,
    inscription_id: InscriptionId,
    from_script: Script,
    to_script: Option<Script>,
  ) -> Result<BRC20Event, Error<L>> {
    let from_key = ScriptKey::from_script(&from_script, self.network);
    let transferable = self
      .ledger
      .get_transferable_by_id(&from_key, &inscription_id)
      .map_err(|e| Error::LedgerError(e))?
      .ok_or(BRC20Error::TransferableNotFound(inscription_id))?;

    let amt = Into::<Num>::into(transferable.amount);

    if transferable.owner != from_key {
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
      .get_balance(&from_key, &lower_tick)
      .map_err(|e| Error::LedgerError(e))?
      .map_or(Balance::new(), |v| v);

    let from_overall = Into::<Num>::into(from_balance.overall_balance);
    let from_transferable = Into::<Num>::into(from_balance.transferable_balance);

    let from_overall = from_overall.checked_sub(amt)?.checked_to_u128()?;
    let from_transferable = from_transferable.checked_sub(amt)?.checked_to_u128()?;

    from_balance.overall_balance = from_overall;
    from_balance.transferable_balance = from_transferable;

    self
      .ledger
      .update_token_balance(&from_key, &lower_tick, from_balance)
      .map_err(|e| Error::LedgerError(e))?;

    // redirect receiver to sender if transfer to conibase.
    let mut msg = None;
    let to_script = if let None = to_script {
      msg = Some(format!(
        "redirect receiver to sender, reason: transfer inscription to coinbase"
      ));
      from_script
    } else {
      to_script.unwrap()
    };
    // update to key balance.
    let to_key = ScriptKey::from_script(&to_script, self.network);
    let mut to_balance = self
      .ledger
      .get_balance(&to_key, &lower_tick)
      .map_err(|e| Error::LedgerError(e))?
      .map_or(Balance::new(), |v| v);

    let to_overall = Into::<Num>::into(to_balance.overall_balance);
    to_balance.overall_balance = to_overall.checked_add(amt)?.checked_to_u128()?;

    self
      .ledger
      .update_token_balance(&to_key, &lower_tick, to_balance)
      .map_err(|e| Error::LedgerError(e))?;

    self
      .ledger
      .remove_transferable(&from_key, &lower_tick, inscription_id)
      .map_err(|e| Error::LedgerError(e))?;

    Ok(BRC20Event::TransferPhase2(TransferPhase2Event {
      from: from_key,
      to: to_key,
      msg,
      tick: token_info.tick,
      amount: amt.checked_to_u128()?,
    }))
  }
}
