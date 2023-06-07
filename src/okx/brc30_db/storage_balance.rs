use crate::brc30::{Balance, TickId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StoreBalance {
  pub tick: TickId,
  pub balance: Balance,
}
