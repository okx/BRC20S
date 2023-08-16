pub(crate) mod brc20;
pub(crate) mod brc20s;
pub(crate) mod btc;
pub(crate) mod execute_manager;
pub(crate) mod message;
pub(crate) mod ord;
pub(crate) mod protocol_manager;
pub(crate) mod resolve_manager;
mod utils;

pub use self::protocol_manager::{BlockContext, ProtocolManager};
use self::{
  execute_manager::CallManager,
  message::{Message, Receipt},
  resolve_manager::MsgResolveManager,
};

#[derive(Debug, Clone)]
pub struct Config {
  first_inscription_height: u64,
  first_brc20_height: Option<u64>,
  first_brc20s_height: Option<u64>,
  index_btc_balance: bool,
}

#[derive(Default)]
pub struct ConfigBuilder {
  first_inscription_height: u64,
  first_brc20_height: Option<u64>,
  first_brc20s_height: Option<u64>,
  index_btc_balance: bool,
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
    self.index_btc_balance = true;
    self
  }

  pub fn build(self) -> Config {
    Config {
      first_inscription_height: self.first_inscription_height,
      first_brc20_height: self.first_brc20_height,
      first_brc20s_height: self.first_brc20s_height,
      index_btc_balance: self.index_btc_balance,
    }
  }
}
