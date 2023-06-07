use crate::brc30::{Balance, TickId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StoreBalance {
  pub tick_id: TickId,
  pub balance: Balance,
}
