mod read_only;
mod read_write;

pub use self::{read_only::try_init_tables, read_only::DataStoreReader, read_write::DataStore};
use crate::okx::datastore::ScriptKey;
use redb::TableDefinition;
const BTC_BALANCE: TableDefinition<&str, &[u8]> = TableDefinition::new("BTC_BALANCES");

fn btc_script_key(script: &ScriptKey) -> String {
  format!("{}", script)
}
