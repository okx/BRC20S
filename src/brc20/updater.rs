use super::{
  deserialize_brc20,
  ledger::{BRC20Balance, BRC20TokenInfo},
  Deploy, Error, Ledger, Mint, Num, Operation, Transfer,
};
use crate::{
  brc20::error::{BRC20Error, JSONError},
  Address, Index, Inscription, InscriptionId, Txid,
};
use bitcoin::{hashes::hex::ToHex, Script};
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub enum Action {
  Inscribe(Script),
  Transfer(Script, Script),
}
impl Action {
  pub fn set_to(&mut self, to: Script) {
    match self {
      Action::Inscribe(to_script) => *to_script = to,
      Action::Transfer(_, to_script) => *to_script = to,
    }
  }
}
pub struct InscriptionData {
  pub(crate) txid: Txid,
  pub(crate) inscription_id: InscriptionId,
  pub(crate) inscription: Inscription,
  pub(crate) action: Action,
}

impl InscriptionData {
  pub fn get_inscription_id(&self) -> InscriptionId {
    self.inscription_id
  }

  pub fn get_action(&self) -> Action {
    self.action.clone()
  }

  pub fn set_action(&mut self, action: Action) {
    self.action = action;
  }
}

pub(super) struct Updater<'a, L: Ledger> {
  ledger: &'a mut L,
  index: &'a Index,
}
impl<'a, L: Ledger> Updater<'a, L>
where
  Error<L>: From<<L as Ledger>::Error>,
{
  pub fn new(ledger: &'a mut L, index: &'a Index) -> Self {
    Self { ledger, index }
  }

  pub fn index_transaction(
    &mut self,
    txid: Txid,
    transactions: Vec<InscriptionData>,
  ) -> Result<(), Error<L>> {
    for transaction in transactions {
      match transaction.get_action() {
        Action::Inscribe(to_script) => {
          let operation = deserialize_brc20(
            std::str::from_utf8(
              transaction
                .inscription
                .body()
                .ok_or(JSONError::InvalidJson)?,
            )
            .ok()
            .ok_or(JSONError::InvalidJson)?,
          )?;

          match operation {
            Operation::Deploy(deploy) => {
              self.process_deploy_operation(deploy, transaction.inscription_id, to_script)?;
              todo!("not implemented")
            }
            Operation::Mint(mint) => {
              self.process_mint_operation(mint, transaction.inscription_id, to_script)?;
              todo!("not implemented")
            }
            Operation::Transfer(transfer) => {
              self.process_inscribe_transfer_operation(
                transfer,
                transaction.inscription_id,
                to_script,
              )?;
              todo!("not implemented")
            }
          }
        }
        Action::Transfer(from_script, to_script) => {
          self.process_transfer_operation(transaction.inscription_id, from_script, to_script)?;
          todo!("not implemented")
        }
      }
    }
    todo!("ssss")
  }

  fn process_deploy_operation(
    &mut self,
    deploy: Deploy,
    inscription_id: InscriptionId,
    to: Script,
  ) -> Result<(), Error<L>> {
    let lower_tick = deploy.tick.to_lowercase();

    if let Some(_) = self.ledger.get_token_info(lower_tick.as_str())? {
      return Err(Error::BRC20Error(BRC20Error::DuplicateTick(lower_tick)));
    }
    let u_supply = deploy
      .max_supply
      .checked_mul(Into::<Num>::into(Decimal::TEN).checked_powu(deploy.decimals as u64)?)?;

    let u_mint_limit = if let Some(mint) = deploy.mint_limit {
      mint.checked_mul(Into::<Num>::into(Decimal::TEN).checked_powu(deploy.decimals as u64)?)?
    } else {
      u_supply.clone()
    };

    if u_mint_limit > u_supply {
      return Err(Error::BRC20Error(BRC20Error::InvalidMintLimit));
    }
    let token_info = BRC20TokenInfo {
      inscription_id: inscription_id.to_string(),
      tick: deploy.tick.to_string(),
      decimal: deploy.decimals,
      supply: u_supply,
      limit_per_mint: u_mint_limit,
      minted: Num::new(Decimal::ZERO),
      deploy_by: to.to_hex(),
    };

    self
      .ledger
      .set_token_info(lower_tick.as_str(), token_info)?;

    todo!("ss")
  }

  fn process_mint_operation(
    &mut self,
    mint: Mint,
    inscription_id: InscriptionId,
    to_script: Script,
  ) -> Result<(), Error<L>> {
    let lower_tick = mint.tick.to_lowercase();

    let mut token_info = self
      .ledger
      .get_token_info(lower_tick.as_str())?
      .ok_or(BRC20Error::TickNotFound(lower_tick.clone()))?;

    if token_info.minted > token_info.supply {
      return Err(Error::BRC20Error(BRC20Error::TickMintOut(token_info.tick)));
    }

    let mut u_amt = mint
      .amount
      .checked_mul(Into::<Num>::into(Decimal::TEN).checked_powu(token_info.decimal as u64)?)?;
    // cut off any excess.
    u_amt = if u_amt.checked_add(token_info.minted)? > token_info.supply {
      token_info.supply.checked_sub(token_info.minted)?
    } else {
      u_amt
    };

    // get or initialize user balance.
    let script_key = Address::from_script(&to_script, self.index.get_chain_network())
      .map_or(to_script.to_hex(), |v| v.to_string());
    let mut script_balance = if let Some(balance) = self.ledger.get_balance(script_key.as_str())? {
      balance
    } else {
      BRC20Balance::new()
    };

    // add amount to available balance. and store to database.
    script_balance.available = script_balance.available.checked_add(u_amt)?;
    self
      .ledger
      .set_balance(script_key.as_str(), script_balance)?;

    // update token minted.
    token_info.minted = token_info.minted.checked_add(u_amt.clone())?;
    self
      .ledger
      .set_token_info(lower_tick.as_str(), token_info)?;

    // BRC20Event::Mint {
    //   event: MintEvent {},
    //   status:
    // }
    todo!("sss")
  }

  fn process_inscribe_transfer_operation(
    &mut self,
    transfer: Transfer,
    inscription_id: InscriptionId,
    to_script: Script,
  ) -> Result<(), Error<L>> {
    let lower_tick = transfer.tick.to_lowercase();

    let token_info = self
      .ledger
      .get_token_info(lower_tick.as_str())?
      .ok_or(BRC20Error::TickNotFound(lower_tick))?;

    let mut u_amt = transfer
      .amount
      .checked_mul(Into::<Num>::into(Decimal::TEN).checked_powu(token_info.decimal as u64)?)?;

    let script_key = Address::from_script(&to_script, self.index.get_chain_network())
      .map_or(to_script.to_hex(), |v| v.to_string());
    let mut script_balance = if let Some(balance) = self.ledger.get_balance(script_key.as_str())? {
      balance
    } else {
      BRC20Balance::new()
    };

    script_balance.available =
      if let Some(balance) = script_balance.available.checked_sub(u_amt).ok() {
        balance
      } else {
        return Err(Error::BRC20Error(BRC20Error::InsufficientBalance));
      };

    script_balance.transferable =
      if let Some(balance) = script_balance.transferable.checked_add(u_amt).ok() {
        balance
      } else {
        return Err(Error::BRC20Error(BRC20Error::InsufficientBalance));
      };

    self
      .ledger
      .set_balance(script_key.as_str(), script_balance)?;
    todo!("ssss")
  }

  fn process_transfer_operation(
    &mut self,
    inscription_id: InscriptionId,
    from_script: Script,
    to_script: Script,
  ) -> Result<(), Error<L>> {
    // let transferable_inscription = self
    //   .ledger
    //   .get_transferable_inscription(inscription_id)?
    //   .ok_or(BRC20Error::InscriptionNotFound(inscription_id.to_string()))?;

    // let lower_tick = transferable_inscription.tick.to_lowercase();

    // let token_info = self
    //   .ledger
    //   .get_token_info(lower_tick.as_str())?
    //   .ok_or(BRC20Error::TickNotFound(lower_tick.clone()))?;

    // let from_script_key = self.parse_script_tick(&from_script, lower_tick.as_str());
    // let mut from_script_balance =
    //   if let Some(balance) = self.ledger.get_balance(from_script_key.as_str())? {
    //     &balance
    //   } else {
    //     &BRC20Balance::new()
    //   };

    // let to_script_key = self.parse_script_tick(&from_script, lower_tick.as_str());
    // let mut to_script_balance = if from_script_key == to_script_key {
    //   from_script_balance
    // } else {
    //   if let Some(balance) = self.ledger.get_balance(to_script_key.as_str())? {
    //     &balance
    //   } else {
    //     &BRC20Balance::new()
    //   }
    // };

    // from_script_balance.transferable = if let Some(balance) = from_script_balance
    //   .transferable
    //   .checked_sub(transferable_inscription.amount)
    //   .ok()
    // {
    //   balance
    // } else {
    //   return Err(Error::BRC20Error(BRC20Error::InsufficientBalance));
    // };

    // to_script_balance.available = if let Some(balance) = to_script_balance
    //   .available
    //   .checked_add(transferable_inscription.amount)
    //   .ok()
    // {
    //   balance
    // } else {
    //   return Err(Error::BRC20Error(BRC20Error::BalanceOverflow));
    // };

    // self
    //   .ledger
    //   .set_balance(from_script_key.as_str(), *from_script_balance)?;

    // self
    //   .ledger
    //   .set_balance(to_script_key.as_str(), *to_script_balance)?;

    todo!("sss")
  }

  fn parse_script_tick(&mut self, script: &Script, tick: &str) -> String {
    let script_key = Address::from_script(script, self.index.get_chain_network())
      .map_or(script.to_hex(), |v| v.to_string());
    format!("{}_{}", script_key, tick.to_lowercase())
  }
}
