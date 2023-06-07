use std::str::FromStr;

use super::{
  BRC30Event, BRC30PoolInfo, BRC30Receipt, BRC30TickInfo, Balance, Deploy, Error, EventType,
  InscriptionOperation, LedgerReadWrite, Mint, MintEvent, Num, Operation, Pid, Stake, TickId,
  Transfer, TransferableAsset, UnStake, UserInfo,
};

use crate::brc20::ScriptKey;
use crate::brc30::params::BIGDECIMAL_TEN;
use crate::{
  brc30::{error::BRC30Error, params::MAX_DECIMAL_WIDTH},
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

pub(crate) struct BRC30Updater<'a, 'db, 'tx, L: LedgerReadWrite> {
  ledger: &'a L,
  id_to_entry: &'a Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
}
impl<'a, 'db, 'tx, L: LedgerReadWrite> BRC30Updater<'a, 'db, 'tx, L> {
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
              block_time,
              operation.inscription_id,
              inscription_number,
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
              inscription_number,
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

      receipts.push(BRC30Receipt {
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
    block_time: u32,
    inscription_id: InscriptionId,
    inscription_number: u64,
    to_script_key: Option<ScriptKey>,
  ) -> Result<BRC30Event, Error<L>> {
    return Err(Error::BRC30Error(BRC30Error::InternalError("".to_string())));
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
    return Err(Error::BRC30Error(BRC30Error::InternalError("".to_string())));
  }

  fn process_inscribe_transfer(
    &mut self,
    transfer: Transfer,
    inscription_id: InscriptionId,
    inscription_number: u64,
    to_script_key: Option<ScriptKey>,
  ) -> Result<BRC30Event, Error<L>> {
    return Err(Error::BRC30Error(BRC30Error::InternalError("".to_string())));
  }

  fn process_transfer(
    &mut self,
    inscription_id: InscriptionId,
    from_script_key: ScriptKey,
    to_script_key: Option<ScriptKey>,
  ) -> Result<BRC30Event, Error<L>> {
    return Err(Error::BRC30Error(BRC30Error::InternalError("".to_string())));
  }
}
