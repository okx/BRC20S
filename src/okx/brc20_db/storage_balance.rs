use serde::{Deserialize, Serialize};
use crate::brc20::{Balance, Tick};

#[derive(Serialize, Deserialize)]
pub struct StoreBalance {
  pub tick: Tick,
  pub balance: Balance,
}

