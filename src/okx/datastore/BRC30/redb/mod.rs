mod read_only;
mod read_write;

pub use self::{read_only::BRC30DataStoreReader, read_write::BRC30DataStore};

use crate::okx::datastore::ScriptKey;
use crate::okx::datastore::BRC30::{PledgedTick, TickId};
use crate::InscriptionId;
use bitcoin::Txid;
use redb::TableDefinition;

const TXID_TO_INSCRIPTION_RECEIPTS: TableDefinition<&str, &[u8]> =
  TableDefinition::new("TXID_TO_INSCRIPTION_RECEIPTS");
const BRC30_TICKINFO: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC30_TICKINFO");
const BRC30_PID_TO_POOLINFO: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC30_PID_TO_POOLINFO");
const BRC30_USER_STAKEINFO: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC30_USER_STAKEINFO");
const BRC30_PID_TO_USERINFO: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC30_PID_TO_USERINFO");
const BRC30_STAKE_TICKID_TO_PID: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC30_STAKE_TICKID_TO_PID");
const BRC30_BALANCES: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC30_BALANCE");
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

fn script_pledged_key(script: &ScriptKey, pledged_tick: &PledgedTick) -> String {
  let mut pledged_key: String;
  match pledged_tick {
    PledgedTick::NATIVE => {
      pledged_key = hex::encode("btc").to_string();
    }
    PledgedTick::BRC20Tick(tick) => {
      pledged_key = tick.to_lowercase().hex();
    }
    PledgedTick::BRC30Tick(tick_id) => {
      pledged_key = tick_id.to_lowercase().hex();
    }
  }

  format!("{}_{}", script.to_string(), pledged_key)
}

fn pledgedtick_tickid_key(pledged_tick: &PledgedTick, tick_id: &TickId) -> String {
  let mut pledged_key: String;
  match pledged_tick {
    PledgedTick::NATIVE => {
      pledged_key = hex::encode("btc").to_string();
    }
    PledgedTick::BRC20Tick(tick) => {
      pledged_key = tick.to_lowercase().hex();
    }
    PledgedTick::BRC30Tick(tick_id) => {
      pledged_key = tick_id.to_lowercase().hex();
    }
  }

  format!(
    "{}_{}",
    pledged_key.to_string(),
    tick_id.to_lowercase().hex()
  )
}

fn min_script_tick_id_key(script: &ScriptKey) -> String {
  format!("{}_{}", script.to_string(), TickId::min_hex())
}

fn max_script_tick_id_key(script: &ScriptKey) -> String {
  format!("{}_{}", script.to_string(), TickId::max_hex())
}
