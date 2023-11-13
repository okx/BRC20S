pub(crate) mod brc20;
pub(crate) mod brc20s;
pub(crate) mod execute_manager;
pub(crate) mod message;
pub(crate) mod ord;
pub(crate) mod protocol_manager;
pub(crate) mod resolve_manager;
mod utils;

pub use self::protocol_manager::ProtocolManager;

use {
  self::{
    execute_manager::CallManager,
    message::{Message, Receipt},
    resolve_manager::MsgResolveManager,
  },
  bitcoin::Network,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BlockContext {
  pub network: Network,
  pub blockheight: u64,
  pub blocktime: u32,
}
#[derive(Debug, Clone)]
pub struct Config {
  first_inscription_height: u64,
  first_brc20_height: Option<u64>,
  first_brc20s_height: Option<u64>,
}

#[derive(Default)]
pub struct ConfigBuilder {
  pub first_inscription_height: u64,
  pub first_brc20_height: Option<u64>,
  pub first_brc20s_height: Option<u64>,
}

impl ConfigBuilder {
  pub fn new(first_inscription_height: u64) -> Self {
    Self {
      first_inscription_height,
      ..Default::default()
    }
  }

  pub fn with_brc20(mut self, first_brc20_height: u64) -> Self {
    self.first_brc20_height = Some(first_brc20_height);
    self
  }

  pub fn with_brc20s(mut self, first_brc20s_height: u64) -> Self {
    self.first_brc20s_height = Some(first_brc20s_height);
    self
  }

  pub fn build(self) -> Config {
    Config {
      first_inscription_height: self.first_inscription_height,
      first_brc20_height: self.first_brc20_height,
      first_brc20s_height: self.first_brc20s_height,
    }
  }
}
