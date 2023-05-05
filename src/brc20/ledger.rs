use crate::brc20::error::BRC20Error;
use crate::brc20::Num;
use crate::InscriptionId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

#[derive(Debug, Deserialize, Serialize)]
pub struct BRC20Balance {
  pub(super) available: Num,
  pub(super) transferable: Num,
}
impl BRC20Balance {
  pub fn new() -> Self {
    Self {
      available: Num::new(Decimal::ZERO),
      transferable: Num::new(Decimal::ZERO),
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BRC20TokenInfo {
  pub(super) tick: String,
  // #[serde(with = "InscriptionIDSerde")]
  pub(super) inscription_id: String,
  pub(super) supply: Num,
  pub(super) minted: Num,
  pub(super) limit_per_mint: Num,
  pub(super) decimal: u8,
  pub(super) deploy_by: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum BRC20Event {
  Deploy {
    event: DeployEvent,
    status: Option<BRC20Error>,
  },
  Mint {
    event: MintEvent,
    status: Option<BRC20Error>,
  },
  Transfer1 {
    event: Transfer1Event,
    status: Option<BRC20Error>,
  },
  Transfer2(Transfer2Event),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeployEvent {
  // #[serde(with = "InscriptionIDSerde")]
  pub inscription_id: String,
  pub supply: Num,
  pub limit_per_mint: Option<Num>,
  pub decimal: u8,
  pub tick: String,
  pub deploy_by: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MintEvent {
  // #[serde(with = "InscriptionIDSerde")]
  pub inscription_id: String,
  pub amount: Num,
  pub tick: String,
  pub mint_to: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Transfer1Event {
  // #[serde(with = "InscriptionIDSerde")]
  pub inscription_id: String,
  pub amount: Num,
  pub tick: String,
  pub owner: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Transfer2Event {
  pub tick: String,
  pub from: String,
  pub to: String,
  pub amount: Num,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inscription {
  // #[serde(with = "InscriptionIDSerde")]
  pub inscription_id: String,
  pub amount: Num,
}

pub struct TransferableInscription {
  pub amount: Num,
  pub tick: String,
  pub owner: String,
}

pub trait Ledger {
  type Error: Debug + Display;

  // balance
  fn get_balance(&self, script_tick: &str) -> Result<Option<BRC20Balance>, Self::Error>;
  fn set_balance(&self, script_tick: &str, new_balance: BRC20Balance) -> Result<(), Self::Error>;

  // token
  fn get_token_info(&self, lower_tick: &str) -> Result<Option<BRC20TokenInfo>, Self::Error>;
  fn set_token_info(&self, lower_tick: &str, new_info: BRC20TokenInfo) -> Result<(), Self::Error>;

  // event
  fn get_events_in_tx(&self, tx_id: &str) -> Result<Option<Vec<BRC20Event>>, Self::Error>;
  fn set_events_in_tx(&self, tx_id: &str, events: &[BRC20Event]) -> Result<(), Self::Error>;

  // inscription
  fn get_inscriptions(&self, address_tick: &str) -> Result<Option<Vec<Inscription>>, Self::Error>;
  fn set_inscriptions(
    &self,
    address_tick: &str,
    inscriptions: &[Inscription],
  ) -> Result<(), Self::Error>;

  // transferable inscription
  fn get_transferable_inscription(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<TransferableInscription>, Self::Error>;
  fn set_transferable_inscription(
    &self,
    inscription_id: InscriptionId,
    transferable_inscription: TransferableInscription,
  ) -> Result<(), Self::Error>;
}
