mod read_only;
mod read_write;

use super::{LowerTick, ScriptKey, Tick};
use crate::{InscriptionId, Result};

use bitcoin::Txid;
use redb::TableDefinition;

pub use self::{read_only::try_init_tables, read_only::DataStoreReader, read_write::DataStore};

const BRC20_BALANCES: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC20_BALANCES");
const BRC20_TOKEN: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC20_TOKEN");
const BRC20_EVENTS: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC20_EVENTS");
const BRC20_TRANSFERABLELOG: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC20_TRANSFERABLELOG");
const BRC20_INSCRIBE_TRANSFER: TableDefinition<&[u8; 36], &[u8]> =
  TableDefinition::new("BRC20_INSCRIBE_TRANSFER");

fn script_tick_key(script: &ScriptKey, tick: &Tick) -> String {
  format!("{}_{}", script, tick.to_lowercase().hex())
}

fn min_script_tick_key(script: &ScriptKey) -> String {
  format!("{}_{}", script, LowerTick::min_hex())
}

fn max_script_tick_key(script: &ScriptKey) -> String {
  format!("{}_{}", script, LowerTick::max_hex())
}
