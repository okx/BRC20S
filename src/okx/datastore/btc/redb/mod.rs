mod read_only;
mod read_write;

use crate::okx::datastore::ScriptKey;
use redb::TableDefinition;
pub use self::{read_only::try_init_tables, read_only::DataStoreReader, read_write::DataStore};
const BTC_BALANCE: TableDefinition<&str, &[u8]> = TableDefinition::new("BTC_BALANCES");

fn btc_script_key(script: &ScriptKey) -> String {
    format!("{}", script.to_string())
}


