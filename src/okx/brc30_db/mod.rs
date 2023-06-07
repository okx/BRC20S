mod read_only;
mod read_write;

pub use self::{read_only::BRC30DatabaseReader, read_write::BRC30Database};

use crate::brc20::ScriptKey;
use crate::brc30::TickId;
use crate::InscriptionId;
use bitcoin::Txid;
use redb::TableDefinition;

const OUTPOINT_TO_SCRIPT: TableDefinition<&str, &[u8]> = TableDefinition::new("OUTPOINT_TO_SCRIPT");
const TXID_TO_INSCRIPTION_RECEIPTS: TableDefinition<&str, &[u8]> =
  TableDefinition::new("TXID_TO_INSCRIPTION_RECEIPTS");
const BRC30_TICKINFO: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC30_TICKINFO");
const BRC30_PID_TO_POOLINFO: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC30_PID_TO_POOLINFO");
const BRC30_PID_TO_USERINFO: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC30_PID_TO_USERINFO");
const BRC30_STAKE_TICKID_TO_PID: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC30_STAKE_TICKID_TO_PID");
const BRC30_BALANCE: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC30_BALANCE");
const BRC30_TRANSFERABLE_ASSETS: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC30_TRANSFERABLE_ASSETS");
const BRC30_TXID_TO_RECEIPTS: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC30_TXID_TO_RECEIPTS");

fn script_tickid_key(script: &ScriptKey, tick_id: &TickId) -> String {
  format!("{}_{}", script.to_string(), tick_id.to_lowercase().hex())
}

fn script_tickid_inscriptionid_key(
  script: &ScriptKey,
  tick_id: &TickId,
  inscriptionid: &InscriptionId,
) -> String {
  format!(
    "{}_{}_{}",
    script.to_string(),
    tick_id.to_lowercase().hex(),
    inscriptionid.to_string()
  )
}

fn min_script_tick_id_key(script: &ScriptKey) -> String {
  format!("{}_{}", script.to_string(), TickId::min_hex())
}

fn max_script_tick_id_key(script: &ScriptKey) -> String {
  format!("{}_{}", script.to_string(), TickId::max_hex())
}
