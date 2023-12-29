pub(crate) mod brc20;
pub(crate) mod context;
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
  crate::Options,
  bitcoin::Network,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BlockContext {
  pub network: Network,
  pub blockheight: u64,
  pub blocktime: u32,
}
#[derive(Debug, Clone)]
pub struct ProtocolConfig {
  first_inscription_height: u64,
  first_brc20_height: Option<u64>,
  first_brc20s_height: Option<u64>,
  enable_ord_receipts: bool,
  enable_index_bitmap: bool,
}

impl ProtocolConfig {
  pub(crate) fn new_with_options(options: &Options) -> Self {
    let mut config = Self {
      first_inscription_height: options.first_inscription_height(),
      first_brc20_height: if options.enable_index_brc20 {
        Some(options.first_brc20_height())
      } else {
        None
      },
      first_brc20s_height: if options.enable_index_brc20s {
        Some(options.first_brc20s_height())
      } else {
        None
      },
      enable_ord_receipts: options.enable_save_ord_receipts,
      enable_index_bitmap: options.enable_index_bitmap,
    };

    if config.first_brc20s_height.is_some() && config.first_brc20_height.is_none() {
      config.first_brc20_height = Some(options.first_brc20_height());
    }
    config
  }
}
