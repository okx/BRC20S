mod read_only;
mod read_write;

pub use self::{read_only::try_init_tables, read_only::DataStoreReader, read_write::DataStore};

use crate::okx::datastore::brc20s::{Pid, PledgedTick, TickId};
use crate::okx::datastore::ScriptKey;
use crate::InscriptionId;
use bitcoin::Txid;
use redb::TableDefinition;

const TXID_TO_INSCRIPTION_RECEIPTS: TableDefinition<&str, &[u8]> =
  TableDefinition::new("TXID_TO_INSCRIPTION_RECEIPTS");
const BRC20S_TICKINFO: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC20S_TICKINFO");
const BRC20S_PID_TO_POOLINFO: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC20S_PID_TO_POOLINFO");
const BRC20S_USER_STAKEINFO: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC20S_USER_STAKEINFO");
const BRC20S_PID_TO_USERINFO: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC20S_PID_TO_USERINFO");
const BRC20S_STAKE_TICKID_TO_PID: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC20S_STAKE_TICKID_TO_PID");
const BRC20S_TICKID_STAKE_TO_PID: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC20S_TICKID_STAKE_TO_PID");
const BRC20S_BALANCES: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC20S_BALANCE");
const BRC20S_TRANSFERABLE_ASSETS: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC20S_TRANSFERABLE_ASSETS");
const BRC20S_TXID_TO_RECEIPTS: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC20S_TXID_TO_RECEIPTS");
const BRC20S_INSCRIBE_TRANSFER: TableDefinition<&[u8; 36], &[u8]> =
  TableDefinition::new("BRC20S_INSCRIBE_TRANSFER");

fn script_tickid_key(script: &ScriptKey, tick_id: &TickId) -> String {
  format!("{}_{}", script, tick_id.hex())
}

fn script_tickid_inscriptionid_key(
  script: &ScriptKey,
  tick_id: &TickId,
  inscriptionid: &InscriptionId,
) -> String {
  format!("{}_{}_{}", script, tick_id.hex(), inscriptionid)
}

fn script_pid_key(script: &ScriptKey, pid: &Pid) -> String {
  format!("{}_{}", script, pid.hex(),)
}

fn script_pledged_key(script: &ScriptKey, pledged_tick: &PledgedTick) -> String {
  let pledged_key = match pledged_tick {
    PledgedTick::Native => hex::encode("btc"),
    PledgedTick::BRC20Tick(tick) => tick.to_lowercase().hex(),
    PledgedTick::BRC20STick(tick_id) => tick_id.hex(),
    PledgedTick::Unknown => hex::encode("!@#$%"),
  };

  format!("{}_{}", script, pledged_key)
}

fn stake_tickid_key(pledged_tick: &PledgedTick, tick_id: &TickId) -> String {
  let pledged_key = match pledged_tick {
    PledgedTick::Native => hex::encode("btc"),
    PledgedTick::BRC20Tick(tick) => tick.to_lowercase().hex(),
    PledgedTick::BRC20STick(tick_id) => tick_id.hex(),
    PledgedTick::Unknown => hex::encode("!@#$%"),
  };

  format!("{}_{}", pledged_key, tick_id.hex())
}

fn tickid_stake_key(pledged_tick: &PledgedTick, tick_id: &TickId) -> String {
  let pledged_key = match pledged_tick {
    PledgedTick::Native => hex::encode("btc"),
    PledgedTick::BRC20Tick(tick) => tick.to_lowercase().hex(),
    PledgedTick::BRC20STick(tick_id) => tick_id.hex(),
    PledgedTick::Unknown => hex::encode("!@#$%"),
  };

  format!("{}_{}", tick_id.hex(), pledged_key)
}

fn min_script_tick_id_key(script: &ScriptKey) -> String {
  format!("{}_{}", script, TickId::min_hex())
}

fn max_script_tick_id_key(script: &ScriptKey) -> String {
  format!("{}_{}", script, TickId::max_hex())
}

fn min_tickid_stake_key(tick_id: &TickId) -> String {
  format!("{}_{}", tick_id.hex(), PledgedTick::min_hex())
}

fn max_tickid_stake_key(tick_id: &TickId) -> String {
  format!("{}_{}", tick_id.hex(), PledgedTick::max_hex())
}

fn min_stake_tickid_key(pledged: &PledgedTick) -> String {
  let pledged_key = match pledged {
    PledgedTick::Native => hex::encode("btc"),
    PledgedTick::BRC20Tick(tick) => tick.to_lowercase().hex(),
    PledgedTick::BRC20STick(tick_id) => tick_id.hex(),
    PledgedTick::Unknown => hex::encode("!@#$%"),
  };

  format!("{}_{}", pledged_key, TickId::min_hex())
}

fn max_stake_tickid_key(pledged: &PledgedTick) -> String {
  let pledged_key = match pledged {
    PledgedTick::Native => hex::encode("btc"),
    PledgedTick::BRC20Tick(tick) => tick.to_lowercase().hex(),
    PledgedTick::BRC20STick(tick_id) => tick_id.hex(),
    PledgedTick::Unknown => hex::encode("!@#$%"),
  };

  format!("{}_{}", pledged_key, TickId::max_hex())
}

fn min_tid_to_pid_key(tick_id: &TickId) -> String {
  format!("{}000000", hex::encode(tick_id.hex()).as_str())
}

fn max_tid_to_pid_key(tick_id: &TickId) -> String {
  format!("{}ffffff", hex::encode(tick_id.hex()).as_str())
}
