use crate::okx::datastore::BRC20::{Balance, Tick};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StoreBalance {
  pub tick: Tick,
  pub balance: Balance,
}
