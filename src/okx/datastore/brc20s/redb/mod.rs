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
  format!("{}_{}", script.to_string(), tick_id.hex())
}

fn script_tickid_inscriptionid_key(
  script: &ScriptKey,
  tick_id: &TickId,
  inscriptionid: &InscriptionId,
) -> String {
  format!(
    "{}_{}_{}",
    script.to_string(),
    tick_id.hex(),
    inscriptionid.to_string()
  )
}

fn script_pid_key(script: &ScriptKey, pid: &Pid) -> String {
  format!("{}_{}", script.to_string(), pid.hex(),)
}

fn script_pledged_key(script: &ScriptKey, pledged_tick: &PledgedTick) -> String {
  let pledged_key: String;
  match pledged_tick {
    PledgedTick::Native => {
      pledged_key = hex::encode("btc").to_string();
    }
    PledgedTick::BRC20Tick(tick) => {
      pledged_key = tick.to_lowercase().hex();
    }
    PledgedTick::BRC20STick(tick_id) => {
      pledged_key = tick_id.hex();
    }
    PledgedTick::Unknown => {
      pledged_key = hex::encode("!@#$%").to_string();
    }
  }

  format!("{}_{}", script.to_string(), pledged_key)
}

fn stake_tickid_key(pledged_tick: &PledgedTick, tick_id: &TickId) -> String {
  let pledged_key: String;
  match pledged_tick {
    PledgedTick::Native => {
      pledged_key = hex::encode("btc").to_string();
    }
    PledgedTick::BRC20Tick(tick) => {
      pledged_key = tick.to_lowercase().hex();
    }
    PledgedTick::BRC20STick(tick_id) => {
      pledged_key = tick_id.hex();
    }
    PledgedTick::Unknown => {
      pledged_key = hex::encode("!@#$%").to_string();
    }
  }

  format!("{}_{}", pledged_key.to_string(), tick_id.hex())
}

fn tickid_stake_key(pledged_tick: &PledgedTick, tick_id: &TickId) -> String {
  let pledged_key: String;
  match pledged_tick {
    PledgedTick::Native => {
      pledged_key = hex::encode("btc").to_string();
    }
    PledgedTick::BRC20Tick(tick) => {
      pledged_key = tick.to_lowercase().hex();
    }
    PledgedTick::BRC20STick(tick_id) => {
      pledged_key = tick_id.hex();
    }
    PledgedTick::Unknown => {
      pledged_key = hex::encode("!@#$%").to_string();
    }
  }

  format!("{}_{}", tick_id.hex(), pledged_key.to_string())
}

fn min_script_tick_id_key(script: &ScriptKey) -> String {
  format!("{}_{}", script.to_string(), TickId::min_hex())
}

fn max_script_tick_id_key(script: &ScriptKey) -> String {
  format!("{}_{}", script.to_string(), TickId::max_hex())
}

fn min_tickid_stake_key(tick_id: &TickId) -> String {
  format!("{}_{}", tick_id.hex(), PledgedTick::min_hex())
}

fn max_tickid_stake_key(tick_id: &TickId) -> String {
  format!("{}_{}", tick_id.hex(), PledgedTick::max_hex())
}

fn min_stake_tickid_key(pledged: &PledgedTick) -> String {
  let pledged_key: String;
  match pledged {
    PledgedTick::Native => {
      pledged_key = hex::encode("btc").to_string();
    }
    PledgedTick::BRC20Tick(tick) => {
      pledged_key = tick.to_lowercase().hex();
    }
    PledgedTick::BRC20STick(tick_id) => {
      pledged_key = tick_id.hex();
    }
    PledgedTick::Unknown => {
      pledged_key = hex::encode("!@#$%").to_string();
    }
  }

  format!("{}_{}", pledged_key.to_string(), TickId::min_hex())
}

fn max_stake_tickid_key(pledged: &PledgedTick) -> String {
  let pledged_key: String;
  match pledged {
    PledgedTick::Native => {
      pledged_key = hex::encode("btc").to_string();
    }
    PledgedTick::BRC20Tick(tick) => {
      pledged_key = tick.to_lowercase().hex();
    }
    PledgedTick::BRC20STick(tick_id) => {
      pledged_key = tick_id.hex();
    }
    PledgedTick::Unknown => {
      pledged_key = hex::encode("!@#$%").to_string();
    }
  }

  format!("{}_{}", pledged_key.to_string(), TickId::max_hex())
}
