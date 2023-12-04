#![allow(
  clippy::too_many_arguments,
  clippy::type_complexity,
  clippy::result_large_err
)]
#![deny(
  clippy::cast_lossless,
  clippy::cast_possible_truncation,
  clippy::cast_possible_wrap,
  clippy::cast_sign_loss
)]

use {
  self::{
    arguments::Arguments,
    blocktime::Blocktime,
    config::Config,
    decimal::Decimal,
    degree::Degree,
    deserialize_from_str::DeserializeFromStr,
    epoch::Epoch,
    height::Height,
    index::{Index, List},
    inscription_id::InscriptionId,
    media::Media,
    options::Options,
    representation::Representation,
    subcommand::{Subcommand, SubcommandResult},
    tally::Tally,
  },
  anyhow::{anyhow, bail, Context, Error},
  bip39::Mnemonic,
  bitcoin::{
    address::{Address, NetworkUnchecked},
    blockdata::constants::COIN_VALUE,
    consensus::{self, Decodable, Encodable},
    hash_types::BlockHash,
    hashes::Hash,
    Amount, Block, Network, OutPoint, Script, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid,
  },
  bitcoincore_rpc::{Client, RpcApi},
  chain::Chain,
  chrono::{DateTime, TimeZone, Utc},
  clap::{ArgGroup, Parser},
  derive_more::{Display, FromStr},
  html_escaper::{Escape, Trusted},
  lazy_static::lazy_static,
  regex::Regex,
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  std::{
    cmp,
    collections::{BTreeMap, HashSet, VecDeque},
    env,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io,
    net::ToSocketAddrs,
    ops::{Add, AddAssign, Sub},
    path::{Path, PathBuf},
    process,
    str::FromStr,
    sync::{
      atomic::{self, AtomicBool},
      Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime},
  },
  sysinfo::{System, SystemExt},
  tokio::{runtime::Runtime, task},
};

pub use crate::{
  inscription::Inscription, object::Object, rarity::Rarity, sat::Sat, sat_point::SatPoint,
};

#[cfg(test)]
#[macro_use]
mod test;

#[cfg(test)]
use self::test::*;

#[allow(unused_macros)]
macro_rules! tprintln {
    ($($arg:tt)*) => {

      if cfg!(test) {
        eprint!("==> ");
        eprintln!($($arg)*);
      }
    };
}

mod arguments;
mod blocktime;
mod chain;
mod config;
mod decimal;
mod degree;
mod deserialize_from_str;
mod epoch;
mod height;
mod index;
mod inscription;
pub mod inscription_id;
pub mod rpc;
mod logger;
mod media;
mod object;
mod okx;
mod options;
mod page_config;
pub mod rarity;
mod representation;
pub mod sat;
mod sat_point;
pub mod subcommand;
mod tally;
pub mod templates;

type Result<T = (), E = Error> = std::result::Result<T, E>;

const DIFFCHANGE_INTERVAL: u64 = bitcoin::blockdata::constants::DIFFCHANGE_INTERVAL as u64;
const SUBSIDY_HALVING_INTERVAL: u64 =
  bitcoin::blockdata::constants::SUBSIDY_HALVING_INTERVAL as u64;
const CYCLE_EPOCHS: u64 = 6;

static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);
static LISTENERS: Mutex<Vec<axum_server::Handle>> = Mutex::new(Vec::new());
static INDEXER: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(Option::None);

fn integration_test() -> bool {
  env::var_os("ORD_INTEGRATION_TEST")
    .map(|value| value.len() > 0)
    .unwrap_or(false)
}

fn timestamp(seconds: u32) -> DateTime<Utc> {
  Utc.timestamp_opt(seconds.into(), 0).unwrap()
}

fn unbound_outpoint() -> OutPoint {
  OutPoint {
    txid: Hash::all_zeros(),
    vout: 0,
  }
}

fn gracefully_shutdown_indexer() {
  if let Some(indexer) = INDEXER.lock().unwrap().take() {
    // We explicitly set this to true to notify the thread to not take on new work
    SHUTTING_DOWN.store(true, atomic::Ordering::Relaxed);
    log::info!("Waiting for index thread to finish...");
    if indexer.join().is_err() {
      log::warn!("Index thread panicked; join failed");
    }
  }
}

pub fn main() {
  let args = Arguments::parse();
  let log_dir = match args.options.log_dir() {
    Ok(d) => d,
    Err(e) => panic!("get log file error: {}", e),
  };
  if let Err(e) = logger::init(args.options.log_level(), log_dir) {
    panic!("initialize logger error: {}", e);
  }

  ctrlc::set_handler(move || {
    if SHUTTING_DOWN.fetch_or(true, atomic::Ordering::Relaxed) {
      process::exit(1);
    }

    println!("Shutting down gracefully. Press <CTRL-C> again to shutdown immediately.");

    LISTENERS
      .lock()
      .unwrap()
      .iter()
      .for_each(|handle| handle.graceful_shutdown(Some(Duration::from_millis(100))));
  })
  .expect("Error setting <CTRL-C> handler");

  match Arguments::parse().run() {
    Err(err) => {
      eprintln!("error: {err}");
      err
        .chain()
        .skip(1)
        .for_each(|cause| eprintln!("because: {cause}"));
      if env::var_os("RUST_BACKTRACE")
        .map(|val| val == "1")
        .unwrap_or_default()
      {
        eprintln!("{}", err.backtrace());
      }

      gracefully_shutdown_indexer();

      process::exit(1);
    }
    Ok(output) => output.print_json(),
  }

  gracefully_shutdown_indexer();
}
