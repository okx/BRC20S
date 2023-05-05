use crate::brc20::custom_serde::InscriptionIDSerde;
use crate::brc20::error::BRC20Error;
use crate::brc20::Num;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

#[derive(Debug, Deserialize, Serialize)]
pub struct BRC20Balance {
  available: Num,
  transferable: Num,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BRC20TokenInfo {
  #[serde(with = "InscriptionIDSerde")]
  inscription_id: [u8; 36],
  supply: Num,
  minted: Num,
  limit_per_mint: Option<Num>,
  decimal: u32,
  deploy_by: String,
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
  #[serde(with = "InscriptionIDSerde")]
  pub inscription_id: [u8; 36],
  pub supply: Num,
  pub limit_per_mint: Option<Num>,
  pub decimal: u32,
  pub tick: String,
  pub deploy_by: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MintEvent {
  #[serde(with = "InscriptionIDSerde")]
  pub inscription_id: [u8; 36],
  pub amount: Num,
  pub tick: String,
  pub mint_to: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Transfer1Event {
  #[serde(with = "InscriptionIDSerde")]
  pub inscription_id: [u8; 36],
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
  #[serde(with = "InscriptionIDSerde")]
  pub inscription_id: [u8; 36],
  pub amount: Num,
}

pub trait Ledger {
  type Error: Debug + Display;

  // balance
  fn get_balance(&self, address_tick: &str) -> Result<Option<BRC20Balance>, Self::Error>;
  fn set_balance(&self, address_tick: &str, new_balance: BRC20Balance) -> Result<(), Self::Error>;

  // token
  fn get_token_info(&self, tick: &str) -> Result<Option<BRC20TokenInfo>, Self::Error>;
  fn set_token_info(&self, tick: &str, new_info: BRC20TokenInfo) -> Result<(), Self::Error>;

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
}
