use crate::okx::protocol::{BRC20, BRC30};
use bitcoin::Txid;
use std::collections::VecDeque;

pub enum Protocol {
  BRC20((Txid, Vec<BRC20::InscriptionData>)),
  BRC30((Txid, Vec<BRC30::InscriptionData>)),
}

impl Protocol {
  pub fn inner_conversion() -> Option<Self> {
    todo!("convert operations to Protocol");
  }
}
