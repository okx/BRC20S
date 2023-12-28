use {
  self::{
    entry::{BlockHashValue, Entry, InscriptionIdValue, OutPointValue, SatPointValue, SatRange},
    reorg::*,
    updater::Updater,
  },
  super::*,
  bitcoin::block::Header,
  bitcoincore_rpc::{
    json::{GetBlockHeaderResult, GetBlockResult},
    Client,
  },
  chrono::SubsecRound,
  indicatif::{ProgressBar, ProgressStyle},
  log::log_enabled,
  okx::{
    datastore::{
      brc20::{
        self, redb as brc20_db, redb::try_init_tables as try_init_brc20,
        DataStoreReadOnly as BRC20DataStoreReadOnly,
      },
      brc20s::{
        self, redb as brc20s_db, redb::try_init_tables as try_init_brc20s,
        DataStoreReadOnly as BRC20SDataStoreReadOnly, PledgedTick,
      },
      ord::{self, redb::try_init_tables as try_init_ord, DataStoreReadOnly},
      ScriptKey,
    },
    protocol::brc20s::params::NATIVE_TOKEN_DECIMAL,
    reward,
  },
  redb::{
    Database, MultimapTable, MultimapTableDefinition, ReadableMultimapTable, ReadableTable, Table,
    TableDefinition, WriteTransaction,
  },
  std::collections::HashMap,
  std::io::{BufWriter, Read, Write},
};

use crate::okx::datastore::ord::{bitmap::District, collections::CollectionKind};

pub(super) use self::{
  entry::{InscriptionEntry, InscriptionEntryValue},
  updater::BlockData,
};

mod entry;
mod fetcher;
mod reorg;
mod rtx;
mod updater;

const SCHEMA_VERSION: u64 = 6;

macro_rules! define_table {
  ($name:ident, $key:ty, $value:ty) => {
    pub const $name: TableDefinition<$key, $value> = TableDefinition::new(stringify!($name));
  };
}

macro_rules! define_multimap_table {
  ($name:ident, $key:ty, $value:ty) => {
    const $name: MultimapTableDefinition<$key, $value> =
      MultimapTableDefinition::new(stringify!($name));
  };
}

define_multimap_table! { INSCRIPTION_ID_TO_CHILDREN, &InscriptionIdValue, &InscriptionIdValue }
define_multimap_table! { SATPOINT_TO_INSCRIPTION_ID, &SatPointValue, &InscriptionIdValue }
define_multimap_table! { SAT_TO_INSCRIPTION_ID, u64, &InscriptionIdValue }
define_table! { HEIGHT_TO_BLOCK_HASH, u64, &BlockHashValue }
define_table! { HEIGHT_TO_LAST_INSCRIPTION_NUMBER, u64, (i64, i64) }
define_table! { INSCRIPTION_ID_TO_INSCRIPTION_ENTRY, &InscriptionIdValue, InscriptionEntryValue }
define_table! { INSCRIPTION_ID_TO_SATPOINT, &InscriptionIdValue, &SatPointValue }
define_table! { INSCRIPTION_NUMBER_TO_INSCRIPTION_ID, i64, &InscriptionIdValue }
define_table! { OUTPOINT_TO_SAT_RANGES, &OutPointValue, &[u8] }
define_table! { OUTPOINT_TO_ENTRY, &OutPointValue, &[u8] }
define_table! { REINSCRIPTION_ID_TO_SEQUENCE_NUMBER, &InscriptionIdValue, u64 }
define_table! { SAT_TO_SATPOINT, u64, &SatPointValue }
define_table! { STATISTIC_TO_COUNT, u64, u64 }
define_table! { WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP, u64, u128 }

#[derive(Debug, PartialEq)]
pub enum List {
  Spent,
  Unspent(Vec<(u64, u64)>),
}

#[derive(Copy, Clone)]
#[repr(u64)]
pub(crate) enum Statistic {
  Schema = 0,
  Commits = 1,
  LostSats = 2,
  OutputsTraversed = 3,
  SatRanges = 4,
  UnboundInscriptions = 5,
}

impl Statistic {
  fn key(self) -> u64 {
    self.into()
  }
}

impl From<Statistic> for u64 {
  fn from(statistic: Statistic) -> Self {
    statistic as u64
  }
}

#[derive(Serialize)]
pub(crate) struct Info {
  pub(crate) blocks_indexed: u64,
  pub(crate) branch_pages: u64,
  pub(crate) fragmented_bytes: u64,
  pub(crate) index_file_size: u64,
  pub(crate) index_path: PathBuf,
  pub(crate) leaf_pages: u64,
  pub(crate) metadata_bytes: u64,
  pub(crate) outputs_traversed: u64,
  pub(crate) page_size: usize,
  pub(crate) sat_ranges: u64,
  pub(crate) stored_bytes: u64,
  pub(crate) transactions: Vec<TransactionInfo>,
  pub(crate) tree_height: u32,
  pub(crate) utxos_indexed: u64,
}

#[derive(Serialize)]
pub(crate) struct TransactionInfo {
  pub(crate) starting_block_count: u64,
  pub(crate) starting_timestamp: u128,
}

trait BitcoinCoreRpcResultExt<T> {
  fn into_option(self) -> Result<Option<T>>;
}

impl<T> BitcoinCoreRpcResultExt<T> for Result<T, bitcoincore_rpc::Error> {
  fn into_option(self) -> Result<Option<T>> {
    match self {
      Ok(ok) => Ok(Some(ok)),
      Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::error::Error::Rpc(
        bitcoincore_rpc::jsonrpc::error::RpcError { code: -8, .. },
      ))) => Ok(None),
      Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::error::Error::Rpc(
        bitcoincore_rpc::jsonrpc::error::RpcError { message, .. },
      )))
        if message.ends_with("not found") =>
      {
        Ok(None)
      }
      Err(err) => Err(err.into()),
    }
  }
}

pub(crate) struct Index {
  client: Client,
  database: Database,
  database2: Database,
  durability: redb::Durability,
  first_inscription_height: u64,
  genesis_block_coinbase_transaction: Transaction,
  genesis_block_coinbase_txid: Txid,
  height_limit: Option<u64>,
  options: Options,
  path: PathBuf,
  unrecoverably_reorged: AtomicBool,
}

impl Index {
  pub(crate) fn open(options: &Options) -> Result<Self> {
    let client = options.bitcoin_rpc_client()?;

    let path = if let Some(path) = &options.index {
      path.clone()
    } else {
      options.data_dir()?.join("index.redb")
    };

    if let Err(err) = fs::create_dir_all(path.parent().unwrap()) {
      bail!(
        "failed to create data dir `{}`: {err}",
        path.parent().unwrap().display()
      );
    }

    let db_cache_size = match options.db_cache_size {
      Some(db_cache_size) => db_cache_size,
      None => {
        let mut sys = System::new();
        sys.refresh_memory();
        usize::try_from(sys.total_memory() / 4)?
      }
    };

    if let Ok(mut file) = fs::OpenOptions::new().read(true).open(&path) {
      // use cberner's quick hack to check the redb recovery bit
      // https://github.com/cberner/redb/issues/639#issuecomment-1628037591
      const MAGICNUMBER: [u8; 9] = [b'r', b'e', b'd', b'b', 0x1A, 0x0A, 0xA9, 0x0D, 0x0A];
      const RECOVERY_REQUIRED: u8 = 2;

      let mut buffer = [0; MAGICNUMBER.len() + 1];
      file.read_exact(&mut buffer).unwrap();

      if buffer[MAGICNUMBER.len()] & RECOVERY_REQUIRED != 0 {
        println!("Index file {:?} needs recovery. This can take a long time, especially for the --index-sats index.", path);
      }
    }

    log::info!("Setting DB cache size to {} bytes", db_cache_size);

    let durability = if cfg!(test) {
      redb::Durability::None
    } else {
      redb::Durability::Immediate
    };

    let path2 = options.data_dir()?.join("cache.redb");
    let cache_size2 = 1 << 30; // 1GB cache

    let database2 = if path2.exists() {
      Database::builder()
        .set_cache_size(cache_size2)
        .open(&path2)?
    } else {
      Database::builder()
        .set_cache_size(cache_size2)
        .create(&path2)?
    };

    log::info!("database2 created");

    let database = match Database::builder()
      .set_cache_size(db_cache_size)
      .open(&path)
    {
      Ok(database) => {
        let schema_version = database
          .begin_read()?
          .open_table(STATISTIC_TO_COUNT)?
          .get(&Statistic::Schema.key())?
          .map(|x| x.value())
          .unwrap_or(0);

        match schema_version.cmp(&SCHEMA_VERSION) {
          cmp::Ordering::Less =>
            bail!(
              "index at `{}` appears to have been built with an older, incompatible version of ord, consider deleting and rebuilding the index: index schema {schema_version}, ord schema {SCHEMA_VERSION}",
              path.display()
            ),
          cmp::Ordering::Greater =>
            bail!(
              "index at `{}` appears to have been built with a newer, incompatible version of ord, consider updating ord: index schema {schema_version}, ord schema {SCHEMA_VERSION}",
              path.display()
            ),
          cmp::Ordering::Equal => {
          }
        }

        database
      }
      Err(_) => {
        let database = Database::builder()
          .set_cache_size(db_cache_size)
          .create(&path)?;

        let mut tx = database.begin_write()?;

        tx.set_durability(durability);

        tx.open_multimap_table(INSCRIPTION_ID_TO_CHILDREN)?;
        tx.open_multimap_table(SATPOINT_TO_INSCRIPTION_ID)?;
        tx.open_multimap_table(SAT_TO_INSCRIPTION_ID)?;
        tx.open_table(HEIGHT_TO_BLOCK_HASH)?;
        tx.open_table(HEIGHT_TO_LAST_INSCRIPTION_NUMBER)?;
        tx.open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY)?;
        tx.open_table(INSCRIPTION_ID_TO_SATPOINT)?;
        tx.open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?;
        tx.open_table(OUTPOINT_TO_ENTRY)?;
        tx.open_table(REINSCRIPTION_ID_TO_SEQUENCE_NUMBER)?;
        tx.open_table(SAT_TO_SATPOINT)?;
        tx.open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?;

        tx.open_table(STATISTIC_TO_COUNT)?
          .insert(&Statistic::Schema.key(), &SCHEMA_VERSION)?;

        if options.index_sats {
          tx.open_table(OUTPOINT_TO_SAT_RANGES)?
            .insert(&OutPoint::null().store(), [].as_slice())?;
        }

        tx.commit()?;

        database
      }
    };
    {
      let wtx = database.begin_write()?;
      let rtx = database.begin_read()?;
      try_init_ord(&wtx, &rtx)?;
      try_init_brc20(&wtx, &rtx)?;
      try_init_brc20s(&wtx, &rtx)?;
      wtx.commit()?;
      log::info!("Options:\n{:#?}", options);
    }
    let genesis_block_coinbase_transaction =
      options.chain().genesis_block().coinbase().unwrap().clone();

    Ok(Self {
      genesis_block_coinbase_txid: genesis_block_coinbase_transaction.txid(),
      client,
      database,
      database2,
      durability,
      first_inscription_height: options.first_inscription_height(),
      genesis_block_coinbase_transaction,
      height_limit: options.height_limit,
      options: options.clone(),
      path,
      unrecoverably_reorged: AtomicBool::new(false),
    })
  }

  pub(crate) fn get_chain_network(&self) -> Network {
    self.options.chain().network()
  }

  #[cfg(test)]
  fn set_durability(&mut self, durability: redb::Durability) {
    self.durability = durability;
  }

  pub(crate) fn get_unspent_outputs(&self) -> Result<BTreeMap<OutPoint, Amount>> {
    let mut utxos = BTreeMap::new();
    utxos.extend(
      self
        .client
        .list_unspent(None, None, None, None, None)?
        .into_iter()
        .map(|utxo| {
          let outpoint = OutPoint::new(utxo.txid, utxo.vout);
          let amount = utxo.amount;

          (outpoint, amount)
        }),
    );

    #[derive(Deserialize)]
    pub(crate) struct JsonOutPoint {
      txid: bitcoin::Txid,
      vout: u32,
    }

    for JsonOutPoint { txid, vout } in self
      .client
      .call::<Vec<JsonOutPoint>>("listlockunspent", &[])?
    {
      utxos.insert(
        OutPoint { txid, vout },
        Amount::from_sat(self.client.get_raw_transaction(&txid, None)?.output[vout as usize].value),
      );
    }
    for outpoint in utxos.keys() {
      if self.get_outpoint_entry(*outpoint)?.is_none() {
        return Err(anyhow!(
          "output in Bitcoin Core wallet but not in ord index: {outpoint}"
        ));
      }
    }

    Ok(utxos)
  }
  pub(crate) fn get_outpoint_entry(&self, outpoint: OutPoint) -> Result<Option<TxOut>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(OUTPOINT_TO_ENTRY)?
        .get(&outpoint.store())?
        .map(|x| Decodable::consensus_decode(&mut io::Cursor::new(x.value())).unwrap()),
    )
  }

  #[allow(unused)]
  pub(crate) fn get_unspent_output_ranges(&self) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
    self
      .get_unspent_outputs()?
      .into_keys()
      .map(|outpoint| match self.list(outpoint)? {
        Some(List::Unspent(sat_ranges)) => Ok((outpoint, sat_ranges)),
        Some(List::Spent) => bail!("output {outpoint} in wallet but is spent according to index"),
        None => bail!("index has not seen {outpoint}"),
      })
      .collect()
  }

  pub(crate) fn has_sat_index(&self) -> Result<bool> {
    match self.begin_read()?.0.open_table(OUTPOINT_TO_SAT_RANGES) {
      Ok(_) => Ok(true),
      Err(redb::TableError::TableDoesNotExist(_)) => Ok(false),
      Err(err) => Err(err.into()),
    }
  }

  fn require_sat_index(&self, feature: &str) -> Result {
    if !self.has_sat_index()? {
      bail!("{feature} requires index created with `--index-sats` flag")
    }

    Ok(())
  }

  #[allow(unused)]
  pub(crate) fn info(&self) -> Result<Info> {
    let wtx = self.begin_write()?;

    let stats = wtx.stats()?;

    let info = {
      let statistic_to_count = wtx.open_table(STATISTIC_TO_COUNT)?;
      let sat_ranges = statistic_to_count
        .get(&Statistic::SatRanges.key())?
        .map(|x| x.value())
        .unwrap_or(0);
      let outputs_traversed = statistic_to_count
        .get(&Statistic::OutputsTraversed.key())?
        .map(|x| x.value())
        .unwrap_or(0);
      Info {
        index_path: self.path.clone(),
        blocks_indexed: wtx
          .open_table(HEIGHT_TO_BLOCK_HASH)?
          .range(0..)?
          .next_back()
          .and_then(|result| result.ok())
          .map(|(height, _hash)| height.value() + 1)
          .unwrap_or(0),
        branch_pages: stats.branch_pages(),
        fragmented_bytes: stats.fragmented_bytes(),
        index_file_size: fs::metadata(&self.path)?.len(),
        leaf_pages: stats.leaf_pages(),
        metadata_bytes: stats.metadata_bytes(),
        sat_ranges,
        outputs_traversed,
        page_size: stats.page_size(),
        stored_bytes: stats.stored_bytes(),
        transactions: wtx
          .open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?
          .range(0..)?
          .flat_map(|result| {
            result.map(
              |(starting_block_count, starting_timestamp)| TransactionInfo {
                starting_block_count: starting_block_count.value(),
                starting_timestamp: starting_timestamp.value(),
              },
            )
          })
          .collect(),
        tree_height: stats.tree_height(),
        utxos_indexed: wtx.open_table(OUTPOINT_TO_SAT_RANGES)?.len()?,
      }
    };

    Ok(info)
  }

  pub(crate) fn update(&self) -> Result {
    let mut updater = Updater::new(self)?;

    loop {
      match updater.update_index() {
        Ok(ok) => return Ok(ok),
        Err(err) => {
          log::info!("{}", err.to_string());

          match err.downcast_ref() {
            Some(&ReorgError::Recoverable { height, depth }) => {
              Reorg::handle_reorg(self, height, depth)?;

              updater = Updater::new(self)?;
            }
            Some(&ReorgError::Unrecoverable) => {
              self
                .unrecoverably_reorged
                .store(true, atomic::Ordering::Relaxed);
              return Err(anyhow!(ReorgError::Unrecoverable));
            }
            _ => return Err(err),
          };
        }
      }
    }
  }

  pub(crate) fn export(&self, filename: &String, include_addresses: bool) -> Result {
    let mut writer = BufWriter::new(File::create(filename)?);
    let rtx = self.database.begin_read()?;

    let blocks_indexed = rtx
      .open_table(HEIGHT_TO_BLOCK_HASH)?
      .range(0..)?
      .next_back()
      .and_then(|result| result.ok())
      .map(|(height, _hash)| height.value() + 1)
      .unwrap_or(0);

    writeln!(writer, "# export at block height {}", blocks_indexed)?;

    log::info!("exporting database tables to {filename}");

    for result in rtx
      .open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?
      .iter()?
    {
      let (number, id) = result?;
      let inscription_id = InscriptionId::load(*id.value());

      let satpoint = self
        .get_inscription_satpoint_by_id(inscription_id)?
        .unwrap();

      write!(
        writer,
        "{}\t{}\t{}",
        number.value(),
        inscription_id,
        satpoint
      )?;

      if include_addresses {
        let address = if satpoint.outpoint == unbound_outpoint() {
          "unbound".to_string()
        } else {
          let output = self
            .get_transaction(satpoint.outpoint.txid)?
            .unwrap()
            .output
            .into_iter()
            .nth(satpoint.outpoint.vout.try_into().unwrap())
            .unwrap();
          self
            .options
            .chain()
            .address_from_script(&output.script_pubkey)
            .map(|address| address.to_string())
            .unwrap_or_else(|e| e.to_string())
        };
        write!(writer, "\t{}", address)?;
      }
      writeln!(writer)?;

      if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
        break;
      }
    }
    writer.flush()?;
    Ok(())
  }

  pub(crate) fn is_unrecoverably_reorged(&self) -> bool {
    self.unrecoverably_reorged.load(atomic::Ordering::Relaxed)
  }

  pub(crate) fn is_json_api_enabled(&self) -> bool {
    self.options.enable_json_api
  }

  pub(crate) fn begin_read(&self) -> Result<rtx::Rtx> {
    Ok(rtx::Rtx(self.database.begin_read()?))
  }

  fn begin_write(&self) -> Result<WriteTransaction> {
    let mut tx = self.database.begin_write()?;
    tx.set_durability(self.durability);
    Ok(tx)
  }

  fn begin_write2(&self) -> Result<WriteTransaction> {
    let mut tx = self.database2.begin_write()?;
    tx.set_durability(self.durability);
    Ok(tx)
  }

  fn increment_statistic(wtx: &WriteTransaction, statistic: Statistic, n: u64) -> Result {
    let mut statistic_to_count = wtx.open_table(STATISTIC_TO_COUNT)?;
    let value = statistic_to_count
      .get(&(statistic.key()))?
      .map(|x| x.value())
      .unwrap_or(0)
      + n;
    statistic_to_count.insert(&statistic.key(), &value)?;
    Ok(())
  }

  #[cfg(test)]
  pub(crate) fn statistic(&self, statistic: Statistic) -> u64 {
    self
      .database
      .begin_read()
      .unwrap()
      .open_table(STATISTIC_TO_COUNT)
      .unwrap()
      .get(&statistic.key())
      .unwrap()
      .map(|x| x.value())
      .unwrap_or(0)
  }

  pub(crate) fn height(&self) -> Result<Option<Height>> {
    self.begin_read()?.block_height()
  }

  pub(crate) fn height_btc(&self, query_btc: bool) -> Result<(Option<Height>, Option<Height>)> {
    let ord_height = self.begin_read()?.block_height()?;
    if let Some(height) = ord_height {
      if query_btc {
        let btc_height = match self.client.get_blockchain_info() {
          Ok(info) => Height(info.headers),
          Err(e) => {
            return Err(anyhow!(
              "failed to get blockchain info from bitcoin node: {}",
              e.to_string()
            ));
          }
        };
        return Ok((Some(height), Some(btc_height)));
      }
      Ok((Some(height), None))
    } else {
      Ok((None, None))
    }
  }

  pub(crate) fn block_count(&self) -> Result<u64> {
    self.begin_read()?.block_count()
  }

  pub(crate) fn block_height(&self) -> Result<Option<Height>> {
    self.begin_read()?.block_height()
  }

  pub(crate) fn block_hash(&self, height: Option<u64>) -> Result<Option<BlockHash>> {
    self.begin_read()?.block_hash(height)
  }

  pub(crate) fn latest_block(&self) -> Result<Option<(Height, BlockHash)>> {
    self.begin_read()?.latest_block()
  }

  pub(crate) fn blocks(&self, take: usize) -> Result<Vec<(u64, BlockHash)>> {
    let rtx = self.begin_read()?;

    let block_count = rtx.block_count()?;

    let height_to_block_hash = rtx.0.open_table(HEIGHT_TO_BLOCK_HASH)?;

    let mut blocks = Vec::with_capacity(block_count.try_into().unwrap());

    for next in height_to_block_hash.range(0..block_count)?.rev().take(take) {
      let next = next?;
      blocks.push((next.0.value(), Entry::load(*next.1.value())));
    }

    Ok(blocks)
  }

  pub(crate) fn rare_sat_satpoints(&self) -> Result<Option<Vec<(Sat, SatPoint)>>> {
    if self.has_sat_index()? {
      let mut result = Vec::new();

      let rtx = self.database.begin_read()?;

      let sat_to_satpoint = rtx.open_table(SAT_TO_SATPOINT)?;

      for range in sat_to_satpoint.range(0..)? {
        let (sat, satpoint) = range?;
        result.push((Sat(sat.value()), Entry::load(*satpoint.value())));
      }

      Ok(Some(result))
    } else {
      Ok(None)
    }
  }

  pub(crate) fn rare_sat_satpoint(&self, sat: Sat) -> Result<Option<SatPoint>> {
    if self.has_sat_index()? {
      Ok(
        self
          .database
          .begin_read()?
          .open_table(SAT_TO_SATPOINT)?
          .get(&sat.n())?
          .map(|satpoint| Entry::load(*satpoint.value())),
      )
    } else {
      Ok(None)
    }
  }

  pub(crate) fn block_header(&self, hash: BlockHash) -> Result<Option<Header>> {
    self.client.get_block_header(&hash).into_option()
  }

  pub(crate) fn block_header_info(&self, hash: BlockHash) -> Result<Option<GetBlockHeaderResult>> {
    self.client.get_block_header_info(&hash).into_option()
  }

  pub(crate) fn get_block_by_height(&self, height: u64) -> Result<Option<Block>> {
    Ok(
      self
        .client
        .get_block_hash(height)
        .into_option()?
        .map(|hash| self.client.get_block(&hash))
        .transpose()?,
    )
  }

  pub(crate) fn get_block_by_hash(&self, hash: BlockHash) -> Result<Option<Block>> {
    self.client.get_block(&hash).into_option()
  }

  pub(crate) fn get_block_info_by_hash(&self, hash: BlockHash) -> Result<Option<GetBlockResult>> {
    self.client.get_block_info(&hash).into_option()
  }

  pub(crate) fn get_children_by_inscription_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Vec<InscriptionId>> {
    self
      .database
      .begin_read()?
      .open_multimap_table(INSCRIPTION_ID_TO_CHILDREN)?
      .get(&inscription_id.store())?
      .map(|result| {
        result
          .map(|inscription_id| InscriptionId::load(*inscription_id.value()))
          .map_err(|err| err.into())
      })
      .collect()
  }

  pub(crate) fn get_inscription_ids_by_sat(&self, sat: Sat) -> Result<Vec<InscriptionId>> {
    let rtx = &self.database.begin_read()?;

    let mut ids = rtx
      .open_multimap_table(SAT_TO_INSCRIPTION_ID)?
      .get(&sat.n())?
      .map(|result| {
        result
          .map(|inscription_id| InscriptionId::load(*inscription_id.value()))
          .map_err(|err| err.into())
      })
      .collect::<Result<Vec<InscriptionId>>>()?;

    if ids.len() > 1 {
      let re_id_to_seq_num = rtx.open_table(REINSCRIPTION_ID_TO_SEQUENCE_NUMBER)?;
      ids.sort_by_key(
        |inscription_id| match re_id_to_seq_num.get(&inscription_id.store()) {
          Ok(Some(num)) => num.value() + 1, // remove at next index refactor
          _ => 0,
        },
      );
    }

    Ok(ids)
  }

  pub(crate) fn get_inscription_id_by_inscription_number(
    &self,
    n: i64,
  ) -> Result<Option<InscriptionId>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?
        .get(&n)?
        .map(|id| Entry::load(*id.value())),
    )
  }

  pub(crate) fn get_inscription_satpoint_by_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<SatPoint>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(INSCRIPTION_ID_TO_SATPOINT)?
        .get(&inscription_id.store())?
        .map(|satpoint| Entry::load(*satpoint.value())),
    )
  }

  pub(crate) fn ord_get_collections_by_inscription_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<Vec<CollectionKind>>> {
    Ok(
      ord::OrdDbReader::new(&self.database.begin_read()?)
        .get_collections_of_inscription(inscription_id)?,
    )
  }

  pub(crate) fn ord_get_district_inscription_id(
    &self,
    number: u64,
  ) -> Result<Option<InscriptionId>> {
    let district = District { number };
    Ok(
      ord::OrdDbReader::new(&self.database.begin_read()?)
        .get_collection_inscription_id(&district.to_collection_key())?,
    )
  }

  pub(crate) fn get_inscription_by_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<Inscription>> {
    if self
      .database
      .begin_read()?
      .open_table(INSCRIPTION_ID_TO_SATPOINT)?
      .get(&inscription_id.store())?
      .is_none()
    {
      return Ok(None);
    }

    Ok(self.get_transaction(inscription_id.txid)?.and_then(|tx| {
      Inscription::from_transaction(&tx)
        .get(inscription_id.index as usize)
        .map(|transaction_inscription| transaction_inscription.inscription.clone())
    }))
  }

  pub(crate) fn get_inscriptions_on_output_with_satpoints(
    &self,
    outpoint: OutPoint,
  ) -> Result<Vec<(SatPoint, InscriptionId)>> {
    let rtx = &self.database.begin_read()?;
    let sat_to_id = rtx.open_multimap_table(SATPOINT_TO_INSCRIPTION_ID)?;
    let re_id_to_seq_num = rtx.open_table(REINSCRIPTION_ID_TO_SEQUENCE_NUMBER)?;

    Self::inscriptions_on_output_ordered(&re_id_to_seq_num, &sat_to_id, outpoint)
  }

  pub(crate) fn get_inscriptions_on_output(
    &self,
    outpoint: OutPoint,
  ) -> Result<Vec<InscriptionId>> {
    Ok(
      self
        .get_inscriptions_on_output_with_satpoints(outpoint)?
        .iter()
        .map(|(_satpoint, inscription_id)| *inscription_id)
        .collect(),
    )
  }

  pub(crate) fn get_inscriptions_with_satpoint_on_output(
    &self,
    outpoint: OutPoint,
  ) -> Result<Vec<(SatPoint, InscriptionId)>> {
    Self::inscriptions_on_output_ordered(
      &self
        .database
        .begin_read()?
        .open_table(REINSCRIPTION_ID_TO_SEQUENCE_NUMBER)?,
      &self
        .database
        .begin_read()?
        .open_multimap_table(SATPOINT_TO_INSCRIPTION_ID)?,
      outpoint,
    )
  }

  pub(crate) fn get_transaction_output_by_outpoint(
    &self,
    outpoint: OutPoint,
  ) -> Result<Option<TxOut>> {
    Self::transaction_output_by_outpoint(
      &self.database.begin_read()?.open_table(OUTPOINT_TO_ENTRY)?,
      &outpoint,
    )
  }

  pub(crate) fn get_transaction(&self, txid: Txid) -> Result<Option<Transaction>> {
    if txid == self.genesis_block_coinbase_txid {
      Ok(Some(self.genesis_block_coinbase_transaction.clone()))
    } else {
      self.client.get_raw_transaction(&txid, None).into_option()
    }
  }

  pub(crate) fn get_transaction_with_retries(&self, txid: Txid) -> Result<Option<Transaction>> {
    Self::get_transaction_retries(&self.client, txid)
  }

  pub(crate) fn get_transaction_retries(
    client: &Client,
    txid: Txid,
  ) -> Result<Option<Transaction>> {
    let mut errors = 0;
    loop {
      match client.get_raw_transaction(&txid, None).into_option() {
        Err(err) => {
          if cfg!(test) {
            return Err(err);
          }
          errors += 1;
          let seconds = 1 << errors;
          log::warn!("failed to fetch transaction {txid}, retrying in {seconds}s: {err}");

          if seconds > 120 {
            log::error!("would sleep for more than 120s, giving up");
            return Err(err);
          }

          thread::sleep(Duration::from_secs(seconds));
        }
        Ok(result) => return Ok(result),
      }
    }
  }

  pub(crate) fn get_transaction_blockhash(&self, txid: Txid) -> Result<Option<BlockHash>> {
    Ok(
      self
        .client
        .get_raw_transaction_info(&txid, None)
        .into_option()?
        .and_then(|info| {
          if info.in_active_chain.unwrap_or_default() {
            info.blockhash
          } else {
            None
          }
        }),
    )
  }

  pub(crate) fn is_transaction_in_active_chain(&self, txid: Txid) -> Result<bool> {
    Ok(
      self
        .client
        .get_raw_transaction_info(&txid, None)
        .into_option()?
        .and_then(|info| info.in_active_chain)
        .unwrap_or(false),
    )
  }

  #[allow(unused)]
  pub(crate) fn find(&self, sat: u64) -> Result<Option<SatPoint>> {
    self.require_sat_index("find")?;

    let rtx = self.begin_read()?;

    if rtx.block_count()? <= Sat(sat).height().n() {
      return Ok(None);
    }

    let outpoint_to_sat_ranges = rtx.0.open_table(OUTPOINT_TO_SAT_RANGES)?;

    for range in outpoint_to_sat_ranges.range::<&[u8; 36]>(&[0; 36]..)? {
      let (key, value) = range?;
      let mut offset = 0;
      for chunk in value.value().chunks_exact(11) {
        let (start, end) = SatRange::load(chunk.try_into().unwrap());
        if start <= sat && sat < end {
          return Ok(Some(SatPoint {
            outpoint: Entry::load(*key.value()),
            offset: offset + sat - start,
          }));
        }
        offset += end - start;
      }
    }

    Ok(None)
  }

  fn list_inner(&self, outpoint: OutPointValue) -> Result<Option<Vec<u8>>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(OUTPOINT_TO_SAT_RANGES)?
        .get(&outpoint)?
        .map(|outpoint| outpoint.value().to_vec()),
    )
  }

  pub(crate) fn list(&self, outpoint: OutPoint) -> Result<Option<List>> {
    self.require_sat_index("list")?;

    let array = outpoint.store();

    let sat_ranges = self.list_inner(array)?;

    match sat_ranges {
      Some(sat_ranges) => Ok(Some(List::Unspent(
        sat_ranges
          .chunks_exact(11)
          .map(|chunk| SatRange::load(chunk.try_into().unwrap()))
          .collect(),
      ))),
      None => {
        if self.is_transaction_in_active_chain(outpoint.txid)? {
          Ok(Some(List::Spent))
        } else {
          Ok(None)
        }
      }
    }
  }

  pub(crate) fn block_time(&self, height: Height) -> Result<Blocktime> {
    let height = height.n();

    match self.get_block_by_height(height)? {
      Some(block) => Ok(Blocktime::confirmed(block.header.time)),
      None => {
        let tx = self.database.begin_read()?;

        let current = tx
          .open_table(HEIGHT_TO_BLOCK_HASH)?
          .range(0..)?
          .next_back()
          .and_then(|result| result.ok())
          .map(|(height, _hash)| height)
          .map(|x| x.value())
          .unwrap_or(0);

        let expected_blocks = height.checked_sub(current).with_context(|| {
          format!("current {current} height is greater than sat height {height}")
        })?;

        Ok(Blocktime::Expected(
          Utc::now()
            .round_subsecs(0)
            .checked_add_signed(chrono::Duration::seconds(
              10 * 60 * i64::try_from(expected_blocks)?,
            ))
            .ok_or_else(|| anyhow!("block timestamp out of range"))?,
        ))
      }
    }
  }

  #[allow(unused)]
  pub(crate) fn get_inscriptions(
    &self,
    utxos: BTreeMap<OutPoint, Amount>,
  ) -> Result<BTreeMap<SatPoint, InscriptionId>> {
    let rtx = self.database.begin_read()?;

    let mut result = BTreeMap::new();

    let table = rtx.open_multimap_table(SATPOINT_TO_INSCRIPTION_ID)?;
    for utxo in utxos.keys() {
      result.extend(Self::inscriptions_on_output_unordered(&table, *utxo)?);
    }

    Ok(result)
  }

  pub(crate) fn get_latest_inscriptions_with_prev_and_next(
    &self,
    n: usize,
    from: Option<i64>,
  ) -> Result<(Vec<InscriptionId>, Option<i64>, Option<i64>, i64, i64)> {
    let rtx = self.database.begin_read()?;

    let inscription_number_to_inscription_id =
      rtx.open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?;

    let highest = match inscription_number_to_inscription_id.iter()?.next_back() {
      Some(Ok((number, _id))) => number.value(),
      Some(Err(_)) | None => return Ok(Default::default()),
    };

    let lowest = match inscription_number_to_inscription_id.iter()?.next() {
      Some(Ok((number, _id))) => number.value(),
      Some(Err(_)) | None => return Ok(Default::default()),
    };

    let from = from.unwrap_or(highest);

    let prev = if let Some(prev) = from.checked_sub(n.try_into()?) {
      inscription_number_to_inscription_id
        .get(&prev)?
        .map(|_| prev)
    } else {
      None
    };

    let next = if from < highest {
      Some(
        from
          .checked_add(n.try_into()?)
          .unwrap_or(highest)
          .min(highest),
      )
    } else {
      None
    };

    let inscriptions = inscription_number_to_inscription_id
      .range(..=from)?
      .rev()
      .take(n)
      .flat_map(|result| result.map(|(_number, id)| Entry::load(*id.value())))
      .collect();

    Ok((inscriptions, prev, next, lowest, highest))
  }

  pub(crate) fn get_inscriptions_in_block(&self, block_height: u64) -> Result<Vec<InscriptionId>> {
    let rtx = self.database.begin_read()?;

    let height_to_last_inscription_number = rtx.open_table(HEIGHT_TO_LAST_INSCRIPTION_NUMBER)?;
    let inscription_id_by_number = rtx.open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?;

    let block_inscriptions = match (
      height_to_last_inscription_number
        .get(block_height.saturating_sub(1))?
        .map(|ag| ag.value())
        .unwrap_or((0, -1)),
      height_to_last_inscription_number
        .get(&block_height)?
        .map(|ag| ag.value()),
    ) {
      ((oldest_blessed, oldest_cursed), Some((newest_blessed, newest_cursed))) => {
        ((newest_cursed + 1)..=oldest_cursed)
          .chain(oldest_blessed..newest_blessed)
          .map(|num| match inscription_id_by_number.get(&num) {
            Ok(Some(inscription_id)) => Ok(InscriptionId::load(*inscription_id.value())),
            Ok(None) => Err(anyhow!(
              "could not find inscription for inscription number {num}"
            )),
            Err(err) => Err(anyhow!(err)),
          })
          .collect::<Result<Vec<InscriptionId>>>()?
      }
      _ => Vec::new(),
    };

    Ok(block_inscriptions)
  }

  pub(crate) fn get_highest_paying_inscriptions_in_block(
    &self,
    block_height: u64,
    n: usize,
  ) -> Result<(Vec<InscriptionId>, usize)> {
    let inscription_ids = self.get_inscriptions_in_block(block_height)?;

    let mut inscription_to_fee: Vec<(InscriptionId, u64)> = Vec::new();
    for id in &inscription_ids {
      inscription_to_fee.push((
        *id,
        self
          .get_inscription_entry(*id)?
          .ok_or_else(|| anyhow!("could not get entry for inscription {id}"))?
          .fee,
      ));
    }

    inscription_to_fee.sort_by_key(|(_, fee)| *fee);

    Ok((
      inscription_to_fee
        .iter()
        .map(|(id, _)| *id)
        .rev()
        .take(n)
        .collect(),
      inscription_ids.len(),
    ))
  }

  pub(crate) fn get_feed_inscriptions(&self, n: usize) -> Result<Vec<(i64, InscriptionId)>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?
        .iter()?
        .rev()
        .take(n)
        .flat_map(|result| result.map(|(number, id)| (number.value(), Entry::load(*id.value()))))
        .collect(),
    )
  }

  pub(crate) fn get_inscription_entry(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<InscriptionEntry>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY)?
        .get(&inscription_id.store())?
        .map(|value| InscriptionEntry::load(value.value())),
    )
  }

  #[cfg(test)]
  fn assert_inscription_location(
    &self,
    inscription_id: InscriptionId,
    satpoint: SatPoint,
    sat: Option<u64>,
  ) {
    let rtx = self.database.begin_read().unwrap();

    let satpoint_to_inscription_id = rtx.open_multimap_table(SATPOINT_TO_INSCRIPTION_ID).unwrap();

    let inscription_id_to_satpoint = rtx.open_table(INSCRIPTION_ID_TO_SATPOINT).unwrap();

    assert_eq!(
      satpoint_to_inscription_id.len().unwrap(),
      inscription_id_to_satpoint.len().unwrap(),
    );

    assert_eq!(
      SatPoint::load(
        *inscription_id_to_satpoint
          .get(&inscription_id.store())
          .unwrap()
          .unwrap()
          .value()
      ),
      satpoint,
    );

    assert!(satpoint_to_inscription_id
      .get(&satpoint.store())
      .unwrap()
      .any(|id| InscriptionId::load(*id.unwrap().value()) == inscription_id));

    match sat {
      Some(sat) => {
        if self.has_sat_index().unwrap() {
          // unbound inscriptions should not be assigned to a sat
          assert!(satpoint.outpoint != unbound_outpoint());
          assert!(rtx
            .open_multimap_table(SAT_TO_INSCRIPTION_ID)
            .unwrap()
            .get(&sat)
            .unwrap()
            .any(|id| InscriptionId::load(*id.unwrap().value()) == inscription_id));

          // we do not track common sats (only the sat ranges)
          if !Sat(sat).is_common() {
            assert_eq!(
              SatPoint::load(
                *rtx
                  .open_table(SAT_TO_SATPOINT)
                  .unwrap()
                  .get(&sat)
                  .unwrap()
                  .unwrap()
                  .value()
              ),
              satpoint,
            );
          }
        }
      }
      None => {
        if self.has_sat_index().unwrap() {
          assert!(satpoint.outpoint == unbound_outpoint())
        }
      }
    }
  }

  #[cfg(test)]
  fn assert_non_existence_of_inscription(&self, inscription_id: InscriptionId) {
    let rtx = self.database.begin_read().unwrap();

    let inscription_id_to_satpoint = rtx.open_table(INSCRIPTION_ID_TO_SATPOINT).unwrap();
    assert!(inscription_id_to_satpoint
      .get(&inscription_id.store())
      .unwrap()
      .is_none());

    let inscription_id_to_entry = rtx.open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY).unwrap();
    assert!(inscription_id_to_entry
      .get(&inscription_id.store())
      .unwrap()
      .is_none());

    for range in rtx
      .open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)
      .unwrap()
      .iter()
      .into_iter()
    {
      for entry in range.into_iter() {
        let (_number, id) = entry.unwrap();
        assert!(InscriptionId::load(*id.value()) != inscription_id);
      }
    }

    for range in rtx
      .open_multimap_table(SATPOINT_TO_INSCRIPTION_ID)
      .unwrap()
      .iter()
      .into_iter()
    {
      for entry in range.into_iter() {
        let (_satpoint, ids) = entry.unwrap();
        assert!(!ids
          .into_iter()
          .any(|id| InscriptionId::load(*id.unwrap().value()) == inscription_id))
      }
    }

    if self.has_sat_index().unwrap() {
      for range in rtx
        .open_multimap_table(SAT_TO_INSCRIPTION_ID)
        .unwrap()
        .iter()
        .into_iter()
      {
        for entry in range.into_iter() {
          let (_sat, ids) = entry.unwrap();
          assert!(!ids
            .into_iter()
            .any(|id| InscriptionId::load(*id.unwrap().value()) == inscription_id))
        }
      }
    }
  }

  #[inline(never)]
  fn inscriptions_on_output_unordered<'a: 'tx, 'tx>(
    satpoint_to_id: &'a impl ReadableMultimapTable<&'static SatPointValue, &'static InscriptionIdValue>,
    outpoint: OutPoint,
  ) -> Result<impl Iterator<Item = (SatPoint, InscriptionId)> + 'tx> {
    let start = SatPoint {
      outpoint,
      offset: 0,
    }
    .store();

    let end = SatPoint {
      outpoint,
      offset: u64::MAX,
    }
    .store();

    let mut inscriptions = Vec::new();

    for range in satpoint_to_id.range::<&[u8; 44]>(&start..=&end)? {
      let (satpoint, ids) = range?;
      for id_result in ids {
        let id = id_result?;
        inscriptions.push((Entry::load(*satpoint.value()), Entry::load(*id.value())));
      }
    }

    Ok(inscriptions.into_iter())
  }

  fn inscriptions_on_output_ordered<'a: 'tx, 'tx>(
    re_id_to_seq_num: &'a impl ReadableTable<&'static InscriptionIdValue, u64>,
    satpoint_to_id: &'a impl ReadableMultimapTable<&'static SatPointValue, &'static InscriptionIdValue>,
    outpoint: OutPoint,
  ) -> Result<Vec<(SatPoint, InscriptionId)>> {
    let mut result = Self::inscriptions_on_output_unordered(satpoint_to_id, outpoint)?
      .collect::<Vec<(SatPoint, InscriptionId)>>();

    if result.len() <= 1 {
      return Ok(result);
    }

    result.sort_by_key(|(_satpoint, inscription_id)| {
      match re_id_to_seq_num.get(&inscription_id.store()) {
        Ok(Some(num)) => num.value() + 1, // remove at next index refactor
        Ok(None) => 0,
        _ => 0,
      }
    });

    Ok(result)
  }

  fn inscriptions_on_output_ordered2<'a: 'tx, 'tx>(
    remap: &'a HashMap<InscriptionId, u64>,
    satpoint_to_id: &'a impl ReadableMultimapTable<&'static SatPointValue, &'static InscriptionIdValue>,
    outpoint: OutPoint,
  ) -> Result<Vec<(SatPoint, InscriptionId)>> {
    let mut result = Self::inscriptions_on_output_unordered(satpoint_to_id, outpoint)?
      .collect::<Vec<(SatPoint, InscriptionId)>>();

    if result.len() <= 1 {
      return Ok(result);
    }

    result
      .sort_by_key(|(_satpoint, inscription_id)| remap.get(inscription_id).map_or(0, |n| n + 1));

    Ok(result)
  }

  #[inline(never)]
  pub(crate) fn transaction_output_by_outpoint(
    outpoint_to_entry: &impl ReadableTable<&'static OutPointValue, &'static [u8]>,
    outpoint: &OutPoint,
  ) -> Result<Option<TxOut>> {
    Ok(
      outpoint_to_entry
        .get(&outpoint.store())?
        .map(|x| Decodable::consensus_decode(&mut io::Cursor::new(x.value())).unwrap()),
    )
  }

  pub(crate) fn brc20_get_tick_info(&self, name: &brc20::Tick) -> Result<Option<brc20::TokenInfo>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20_db = brc20_db::DataStoreReader::new(&rtx);
    let info = brc20_db.get_token_info(name)?;
    Ok(info)
  }

  pub(crate) fn brc20_get_all_tick_info(&self) -> Result<Vec<brc20::TokenInfo>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20_db = brc20_db::DataStoreReader::new(&rtx);
    let info = brc20_db.get_tokens_info()?;
    Ok(info)
  }

  pub(crate) fn brc20_get_balance_by_address(
    &self,
    tick: &brc20::Tick,
    address: &bitcoin::Address,
  ) -> Result<Option<brc20::Balance>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20_db = brc20_db::DataStoreReader::new(&rtx);
    let bal = brc20_db.get_balance(&ScriptKey::from_address(address.clone()), tick)?;
    Ok(bal)
  }

  pub(crate) fn brc20_get_all_balance_by_address(
    &self,
    address: &bitcoin::Address,
  ) -> Result<Vec<brc20::Balance>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20_db = brc20_db::DataStoreReader::new(&rtx);
    Ok(brc20_db.get_balances(&ScriptKey::from_address(address.clone()))?)
  }

  pub(crate) fn get_transaction_info(
    &self,
    txid: &bitcoin::Txid,
  ) -> Result<Option<bitcoincore_rpc::json::GetRawTransactionResult>> {
    if *txid == self.genesis_block_coinbase_txid {
      Ok(None)
    } else {
      self
        .client
        .get_raw_transaction_info(txid, None)
        .into_option()
    }
  }

  pub(crate) fn brc20_get_tx_events_by_txid(
    &self,
    txid: &bitcoin::Txid,
  ) -> Result<Option<Vec<brc20::Receipt>>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20_db = brc20_db::DataStoreReader::new(&rtx);
    let res = brc20_db.get_transaction_receipts(txid)?;

    if res.is_empty() {
      let tx = self.client.get_raw_transaction_info(txid, None)?;
      if let Some(tx_blockhash) = tx.blockhash {
        let tx_bh = self.client.get_block_header_info(&tx_blockhash)?;
        let parsed_height = self.height()?;
        if parsed_height.is_none() || tx_bh.height as u64 > parsed_height.unwrap().0 {
          return Ok(None);
        }
      } else {
        return Err(anyhow!("can't get tx block hash: {txid}"));
      }
    }

    Ok(Some(res))
  }

  pub(crate) fn brc20_get_txs_events(
    &self,
    txs: &Vec<Txid>,
  ) -> Result<Vec<(bitcoin::Txid, Vec<brc20::Receipt>)>> {
    let rtx = self.database.begin_read()?;
    let brc20_db = brc20_db::DataStoreReader::new(&rtx);
    let mut result = Vec::new();
    for txid in txs {
      let tx_events = brc20_db.get_transaction_receipts(txid)?;
      if tx_events.is_empty() {
        continue;
      }
      result.push((*txid, tx_events));
    }
    Ok(result)
  }

  pub(crate) fn brc20_get_tick_transferable_by_address(
    &self,
    tick: &brc20::Tick,
    address: &bitcoin::Address,
  ) -> Result<Vec<brc20::TransferableLog>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20_db = brc20_db::DataStoreReader::new(&rtx);
    let res = brc20_db.get_transferable_by_tick(&ScriptKey::from_address(address.clone()), tick)?;

    Ok(res)
  }

  pub(crate) fn brc20_get_all_transferable_by_address(
    &self,
    address: &bitcoin::Address,
  ) -> Result<Vec<brc20::TransferableLog>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20_db = brc20_db::DataStoreReader::new(&rtx);
    let res = brc20_db.get_transferable(&ScriptKey::from_address(address.clone()))?;

    Ok(res)
  }

  pub(crate) fn brc20s_all_tick_info(
    &self,
    start: usize,
    limit: Option<usize>,
  ) -> Result<(Vec<brc20s::TickInfo>, usize)> {
    let rtx = self.database.begin_read().unwrap();
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);
    let all_tick = brc20s_db.get_all_tick_info(start, limit)?;
    Ok(all_tick)
  }

  pub(crate) fn brc20s_tick_info(
    &self,
    tick_id: &brc20s::TickId,
  ) -> Result<Option<brc20s::TickInfo>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);
    let info = brc20s_db.get_tick_info(tick_id)?;
    Ok(info)
  }

  pub(crate) fn brc20s_pool_info(&self, pid: &brc20s::Pid) -> Result<Option<brc20s::PoolInfo>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);
    let info = brc20s_db.get_pid_to_poolinfo(pid)?;
    Ok(info)
  }
  pub(crate) fn brc20s_stake_info(
    &self,
    address: &bitcoin::Address,
    pledged_tick: &PledgedTick,
  ) -> Result<Option<brc20s::StakeInfo>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);

    let info =
      brc20s_db.get_user_stakeinfo(&ScriptKey::from_address(address.clone()), pledged_tick)?;
    Ok(info)
  }

  pub(crate) fn brc20s_all_pool_info(
    &self,
    start: usize,
    limit: Option<usize>,
  ) -> Result<(Vec<brc20s::PoolInfo>, usize)> {
    let rtx = self.database.begin_read().unwrap();
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);
    let all_pool = brc20s_db.get_all_poolinfo(start, limit)?;
    Ok(all_pool)
  }

  pub(crate) fn brc20s_all_pools_by_tid(
    &self,
    tick_id: &brc20s::TickId,
  ) -> Result<Vec<brc20s::PoolInfo>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);
    let all_pool = brc20s_db.get_all_pools_by_tid(tick_id)?;
    Ok(all_pool)
  }

  pub(crate) fn brc20s_user_info(
    &self,
    pid: &brc20s::Pid,
    address: &bitcoin::Address,
  ) -> Result<Option<brc20s::UserInfo>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);
    let info = brc20s_db.get_pid_to_use_info(&ScriptKey::from_address(address.clone()), pid)?;
    Ok(info)
  }

  pub(crate) fn brc20s_user_pending_reward(
    &self,
    pid: &brc20s::Pid,
    address: &bitcoin::Address,
  ) -> Result<(Option<String>, Option<String>)> {
    let rtx = self.database.begin_read().unwrap();
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);
    let brc20_db = brc20_db::DataStoreReader::new(&rtx);
    let user_info = brc20s_db
      .get_pid_to_use_info(&ScriptKey::from_address(address.clone()), pid)?
      .ok_or(anyhow!("user info not found from state!"))?;

    let pool_info = brc20s_db
      .get_pid_to_poolinfo(pid)?
      .ok_or(anyhow!("pool info not found from state!"))?;

    let dec = match pool_info.clone().stake {
      PledgedTick::Native => NATIVE_TOKEN_DECIMAL,
      PledgedTick::BRC20STick(tickid) => brc20s_db.get_tick_info(&tickid).unwrap().unwrap().decimal,
      PledgedTick::BRC20Tick(tick) => brc20_db.get_token_info(&tick).unwrap().unwrap().decimal,
      PledgedTick::Unknown => 0_u8,
    };

    let block = self.height().unwrap().unwrap_or(Height(0)).n();

    let result = reward::query_reward(
      user_info,
      pool_info,
      self.height().unwrap().unwrap_or(Height(0)).n(),
      dec,
    )?;

    Ok((Some(result.to_string()), Some(block.to_string())))
  }

  pub(crate) fn brc20s_balance(
    &self,
    tick_id: &brc20s::TickId,
    address: &bitcoin::Address,
  ) -> Result<Option<brc20s::Balance>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);
    let info = brc20s_db.get_balance(&ScriptKey::from_address(address.clone()), tick_id)?;
    Ok(info)
  }

  pub(crate) fn brc20s_all_balance(
    &self,
    address: &bitcoin::Address,
  ) -> Result<Vec<(brc20s::TickId, brc20s::Balance)>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);
    let all_balance = brc20s_db.get_balances(&ScriptKey::from_address(address.clone()))?;
    Ok(all_balance)
  }

  pub(crate) fn brc20s_tickid_transferable(
    &self,
    tick_id: &brc20s::TickId,
    address: &bitcoin::Address,
  ) -> Result<Vec<brc20s::TransferableAsset>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);

    let result =
      brc20s_db.get_transferable_by_tickid(&ScriptKey::from_address(address.clone()), tick_id)?;
    Ok(result)
  }

  pub(crate) fn brc20s_all_transferable(
    &self,
    address: &bitcoin::Address,
  ) -> Result<Vec<brc20s::TransferableAsset>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);
    let info = brc20s_db.get_transferable(&ScriptKey::from_address(address.clone()))?;
    Ok(info)
  }

  pub(crate) fn brc20s_txid_receipts(&self, txid: &Txid) -> Result<Option<Vec<brc20s::Receipt>>> {
    let rtx = self.database.begin_read().unwrap();
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);
    let res = brc20s_db.get_txid_to_receipts(txid)?;

    if res.is_empty() {
      let tx = self.client.get_raw_transaction_info(txid, None)?;
      if let Some(tx_blockhash) = tx.blockhash {
        let tx_bh = self.client.get_block_header_info(&tx_blockhash)?;
        let parsed_height = self.height()?;
        if parsed_height.is_none() || tx_bh.height as u64 > parsed_height.unwrap().0 {
          return Ok(None);
        }
      } else {
        return Err(anyhow!("can't get tx block hash: {txid}"));
      }
    }

    Ok(Some(res))
  }

  pub(crate) fn brc20s_txs_receipts(
    &self,
    txs: &Vec<Txid>,
  ) -> Result<Vec<(bitcoin::Txid, Vec<brc20s::Receipt>)>> {
    let rtx = self.database.begin_read()?;
    let brc20s_db = brc20s_db::DataStoreReader::new(&rtx);
    let mut result = Vec::new();
    for txid in txs {
      let tx_events = brc20s_db.get_txid_to_receipts(txid)?;
      if tx_events.is_empty() {
        continue;
      }
      result.push((*txid, tx_events));
    }
    Ok(result)
  }

  pub(crate) fn ord_txid_inscriptions(
    &self,
    txid: &Txid,
  ) -> Result<Option<Vec<ord::InscriptionOp>>> {
    let rtx = self.database.begin_read().unwrap();
    let ord_db = ord::OrdDbReader::new(&rtx);
    let res = ord_db.get_transaction_operations(txid)?;

    if res.is_empty() {
      let tx = self.client.get_raw_transaction_info(txid, None)?;
      if let Some(tx_blockhash) = tx.blockhash {
        let tx_bh = self.client.get_block_header_info(&tx_blockhash)?;
        let parsed_height = self.height()?;
        if parsed_height.is_none() || tx_bh.height as u64 > parsed_height.unwrap().0 {
          return Ok(None);
        }
      } else {
        return Err(anyhow!("can't get tx block hash: {txid}"));
      }
    }

    Ok(Some(res))
  }
  pub(crate) fn ord_get_txs_inscriptions(
    &self,
    txs: &Vec<Txid>,
  ) -> Result<Vec<(bitcoin::Txid, Vec<ord::InscriptionOp>)>> {
    let rtx = self.database.begin_read()?;
    let ord_db = ord::OrdDbReader::new(&rtx);
    let mut result = Vec::new();
    for txid in txs {
      let inscriptions = ord_db.get_transaction_operations(txid)?;
      if inscriptions.is_empty() {
        continue;
      }
      result.push((*txid, inscriptions));
    }
    Ok(result)
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    bitcoin::secp256k1::rand::{self, RngCore},
    std::ffi::OsString,
    tempfile::TempDir,
  };

  struct ContextBuilder {
    args: Vec<OsString>,
    tempdir: Option<TempDir>,
  }

  impl ContextBuilder {
    fn build(self) -> Context {
      self.try_build().unwrap()
    }

    fn try_build(self) -> Result<Context> {
      let rpc_server = test_bitcoincore_rpc::builder()
        .network(Network::Regtest)
        .build();

      let tempdir = self.tempdir.unwrap_or_else(|| TempDir::new().unwrap());
      let cookie_file = tempdir.path().join("cookie");
      fs::write(&cookie_file, "username:password").unwrap();

      let command: Vec<OsString> = vec![
        "ord".into(),
        "--rpc-url".into(),
        rpc_server.url().into(),
        "--data-dir".into(),
        tempdir.path().into(),
        "--cookie-file".into(),
        cookie_file.into(),
        "--regtest".into(),
      ];

      let options = Options::try_parse_from(command.into_iter().chain(self.args)).unwrap();
      let index = Index::open(&options)?;
      index.update().unwrap();

      Ok(Context {
        options,
        rpc_server,
        tempdir,
        index,
      })
    }

    fn arg(mut self, arg: impl Into<OsString>) -> Self {
      self.args.push(arg.into());
      self
    }

    fn args<T: Into<OsString>, I: IntoIterator<Item = T>>(mut self, args: I) -> Self {
      self.args.extend(args.into_iter().map(|arg| arg.into()));
      self
    }

    fn tempdir(mut self, tempdir: TempDir) -> Self {
      self.tempdir = Some(tempdir);
      self
    }
  }

  #[allow(unused)]
  struct Context {
    options: Options,
    rpc_server: test_bitcoincore_rpc::Handle,
    #[allow(unused)]
    tempdir: TempDir,
    index: Index,
  }

  impl Context {
    fn builder() -> ContextBuilder {
      ContextBuilder {
        args: Vec::new(),
        tempdir: None,
      }
    }

    fn mine_blocks(&self, n: u64) -> Vec<Block> {
      let blocks = self.rpc_server.mine_blocks(n);
      self.index.update().unwrap();
      blocks
    }

    fn mine_blocks_with_subsidy(&self, n: u64, subsidy: u64) -> Vec<Block> {
      let blocks = self.rpc_server.mine_blocks_with_subsidy(n, subsidy);
      self.index.update().unwrap();
      blocks
    }

    fn configurations() -> Vec<Context> {
      vec![
        Context::builder().build(),
        Context::builder().arg("--index-sats").build(),
      ]
    }
  }

  #[test]
  fn height_limit() {
    {
      let context = Context::builder().args(["--height-limit", "0"]).build();
      context.mine_blocks(1);
      assert_eq!(context.index.block_height().unwrap(), None);
      assert_eq!(context.index.block_count().unwrap(), 0);
    }

    {
      let context = Context::builder().args(["--height-limit", "1"]).build();
      context.mine_blocks(1);
      assert_eq!(context.index.block_height().unwrap(), Some(Height(0)));
      assert_eq!(context.index.block_count().unwrap(), 1);
    }

    {
      let context = Context::builder().args(["--height-limit", "2"]).build();
      context.mine_blocks(2);
      assert_eq!(context.index.block_height().unwrap(), Some(Height(1)));
      assert_eq!(context.index.block_count().unwrap(), 2);
    }
  }

  #[test]
  fn inscriptions_below_first_inscription_height_are_skipped() {
    let inscription = inscription("text/plain;charset=utf-8", "hello");
    let template = TransactionTemplate {
      inputs: &[(1, 0, 0, inscription.to_witness())],
      ..Default::default()
    };

    {
      let context = Context::builder().build();
      context.mine_blocks(1);
      let txid = context.rpc_server.broadcast_tx(template.clone());
      let inscription_id = InscriptionId { txid, index: 0 };
      context.mine_blocks(1);

      assert_eq!(
        context.index.get_inscription_by_id(inscription_id).unwrap(),
        Some(inscription)
      );

      assert_eq!(
        context
          .index
          .get_inscription_satpoint_by_id(inscription_id)
          .unwrap(),
        Some(SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        })
      );
    }

    {
      let context = Context::builder()
        .arg("--first-inscription-height=3")
        .build();
      context.mine_blocks(1);
      let txid = context.rpc_server.broadcast_tx(template);
      let inscription_id = InscriptionId { txid, index: 0 };
      context.mine_blocks(1);

      assert_eq!(
        context
          .index
          .get_inscription_satpoint_by_id(inscription_id)
          .unwrap(),
        None,
      );
    }
  }

  #[test]
  fn list_first_coinbase_transaction() {
    let context = Context::builder().arg("--index-sats").build();
    assert_eq!(
      context
        .index
        .list(
          "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0"
            .parse()
            .unwrap()
        )
        .unwrap()
        .unwrap(),
      List::Unspent(vec![(0, 50 * COIN_VALUE)])
    )
  }

  #[test]
  fn list_second_coinbase_transaction() {
    let context = Context::builder().arg("--index-sats").build();
    let txid = context.mine_blocks(1)[0].txdata[0].txid();
    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![(50 * COIN_VALUE, 100 * COIN_VALUE)])
    )
  }

  #[test]
  fn list_split_ranges_are_tracked_correctly() {
    let context = Context::builder().arg("--index-sats").build();

    context.mine_blocks(1);
    let split_coinbase_output = TransactionTemplate {
      inputs: &[(1, 0, 0, Default::default())],
      outputs: 2,
      fee: 0,
      ..Default::default()
    };
    let txid = context.rpc_server.broadcast_tx(split_coinbase_output);

    context.mine_blocks(1);

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![(50 * COIN_VALUE, 75 * COIN_VALUE)])
    );

    assert_eq!(
      context.index.list(OutPoint::new(txid, 1)).unwrap().unwrap(),
      List::Unspent(vec![(75 * COIN_VALUE, 100 * COIN_VALUE)])
    );
  }

  #[test]
  fn list_merge_ranges_are_tracked_correctly() {
    let context = Context::builder().arg("--index-sats").build();

    context.mine_blocks(2);
    let merge_coinbase_outputs = TransactionTemplate {
      inputs: &[(1, 0, 0, Default::default()), (2, 0, 0, Default::default())],
      fee: 0,
      ..Default::default()
    };

    let txid = context.rpc_server.broadcast_tx(merge_coinbase_outputs);
    context.mine_blocks(1);

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![
        (50 * COIN_VALUE, 100 * COIN_VALUE),
        (100 * COIN_VALUE, 150 * COIN_VALUE)
      ]),
    );
  }

  #[test]
  fn list_fee_paying_transaction_range() {
    let context = Context::builder().arg("--index-sats").build();

    context.mine_blocks(1);
    let fee_paying_tx = TransactionTemplate {
      inputs: &[(1, 0, 0, Default::default())],
      outputs: 2,
      fee: 10,
      ..Default::default()
    };
    let txid = context.rpc_server.broadcast_tx(fee_paying_tx);
    let coinbase_txid = context.mine_blocks(1)[0].txdata[0].txid();

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![(50 * COIN_VALUE, 7499999995)]),
    );

    assert_eq!(
      context.index.list(OutPoint::new(txid, 1)).unwrap().unwrap(),
      List::Unspent(vec![(7499999995, 9999999990)]),
    );

    assert_eq!(
      context
        .index
        .list(OutPoint::new(coinbase_txid, 0))
        .unwrap()
        .unwrap(),
      List::Unspent(vec![(10000000000, 15000000000), (9999999990, 10000000000)])
    );
  }

  #[test]
  fn list_two_fee_paying_transaction_range() {
    let context = Context::builder().arg("--index-sats").build();

    context.mine_blocks(2);
    let first_fee_paying_tx = TransactionTemplate {
      inputs: &[(1, 0, 0, Default::default())],
      fee: 10,
      ..Default::default()
    };
    let second_fee_paying_tx = TransactionTemplate {
      inputs: &[(2, 0, 0, Default::default())],
      fee: 10,
      ..Default::default()
    };
    context.rpc_server.broadcast_tx(first_fee_paying_tx);
    context.rpc_server.broadcast_tx(second_fee_paying_tx);

    let coinbase_txid = context.mine_blocks(1)[0].txdata[0].txid();

    assert_eq!(
      context
        .index
        .list(OutPoint::new(coinbase_txid, 0))
        .unwrap()
        .unwrap(),
      List::Unspent(vec![
        (15000000000, 20000000000),
        (9999999990, 10000000000),
        (14999999990, 15000000000)
      ])
    );
  }

  #[test]
  fn list_null_output() {
    let context = Context::builder().arg("--index-sats").build();

    context.mine_blocks(1);
    let no_value_output = TransactionTemplate {
      inputs: &[(1, 0, 0, Default::default())],
      fee: 50 * COIN_VALUE,
      ..Default::default()
    };
    let txid = context.rpc_server.broadcast_tx(no_value_output);
    context.mine_blocks(1);

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(Vec::new())
    );
  }

  #[test]
  fn list_null_input() {
    let context = Context::builder().arg("--index-sats").build();

    context.mine_blocks(1);
    let no_value_output = TransactionTemplate {
      inputs: &[(1, 0, 0, Default::default())],
      fee: 50 * COIN_VALUE,
      ..Default::default()
    };
    context.rpc_server.broadcast_tx(no_value_output);
    context.mine_blocks(1);

    let no_value_input = TransactionTemplate {
      inputs: &[(2, 1, 0, Default::default())],
      fee: 0,
      ..Default::default()
    };
    let txid = context.rpc_server.broadcast_tx(no_value_input);
    context.mine_blocks(1);

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(Vec::new())
    );
  }

  #[test]
  fn list_spent_output() {
    let context = Context::builder().arg("--index-sats").build();
    context.mine_blocks(1);
    context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Default::default())],
      fee: 0,
      ..Default::default()
    });
    context.mine_blocks(1);
    let txid = context.rpc_server.tx(1, 0).txid();
    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Spent,
    );
  }

  #[test]
  fn list_unknown_output() {
    let context = Context::builder().arg("--index-sats").build();

    assert_eq!(
      context
        .index
        .list(
          "0000000000000000000000000000000000000000000000000000000000000000:0"
            .parse()
            .unwrap()
        )
        .unwrap(),
      None
    );
  }

  #[test]
  fn find_first_sat() {
    let context = Context::builder().arg("--index-sats").build();
    assert_eq!(
      context.index.find(0).unwrap().unwrap(),
      SatPoint {
        outpoint: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0"
          .parse()
          .unwrap(),
        offset: 0,
      }
    )
  }

  #[test]
  fn find_second_sat() {
    let context = Context::builder().arg("--index-sats").build();
    assert_eq!(
      context.index.find(1).unwrap().unwrap(),
      SatPoint {
        outpoint: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0"
          .parse()
          .unwrap(),
        offset: 1,
      }
    )
  }

  #[test]
  fn find_first_sat_of_second_block() {
    let context = Context::builder().arg("--index-sats").build();
    context.mine_blocks(1);
    assert_eq!(
      context.index.find(50 * COIN_VALUE).unwrap().unwrap(),
      SatPoint {
        outpoint: "30f2f037629c6a21c1f40ed39b9bd6278df39762d68d07f49582b23bcb23386a:0"
          .parse()
          .unwrap(),
        offset: 0,
      }
    )
  }

  #[test]
  fn find_unmined_sat() {
    let context = Context::builder().arg("--index-sats").build();
    assert_eq!(context.index.find(50 * COIN_VALUE).unwrap(), None);
  }

  #[test]
  fn find_first_sat_spent_in_second_block() {
    let context = Context::builder().arg("--index-sats").build();
    context.mine_blocks(1);
    let spend_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Default::default())],
      fee: 0,
      ..Default::default()
    });
    context.mine_blocks(1);
    assert_eq!(
      context.index.find(50 * COIN_VALUE).unwrap().unwrap(),
      SatPoint {
        outpoint: OutPoint::new(spend_txid, 0),
        offset: 0,
      }
    )
  }

  #[test]
  fn inscriptions_are_tracked_correctly() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn inscriptions_without_sats_are_unbound() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, Default::default())],
        fee: 50 * 100_000_000,
        ..Default::default()
      });

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 0,
        },
        None,
      );

      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(4, 0, 0, Default::default())],
        fee: 50 * 100_000_000,
        ..Default::default()
      });

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(5, 1, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 1,
        },
        None,
      );
    }
  }

  #[test]
  fn unaligned_inscriptions_are_tracked_correctly() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      let send_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0, Default::default()), (2, 1, 0, Default::default())],
        ..Default::default()
      });

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: send_txid,
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn merged_inscriptions_are_tracked_correctly() {
    for context in Context::configurations() {
      context.mine_blocks(2);

      let first_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      let first_inscription_id = InscriptionId {
        txid: first_txid,
        index: 0,
      };

      let second_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0, inscription("text/png", [1; 100]).to_witness())],
        ..Default::default()
      });
      let second_inscription_id = InscriptionId {
        txid: second_txid,
        index: 0,
      };

      context.mine_blocks(1);

      let merged_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 1, 0, Default::default()), (3, 2, 0, Default::default())],
        ..Default::default()
      });

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        first_inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: merged_txid,
            vout: 0,
          },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        second_inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: merged_txid,
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(100 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn inscriptions_that_are_sent_to_second_output_are_are_tracked_correctly() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      let send_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0, Default::default()), (2, 1, 0, Default::default())],
        outputs: 2,
        ..Default::default()
      });

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: send_txid,
            vout: 1,
          },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn missing_inputs_are_fetched_from_bitcoin_core() {
    for args in [
      ["--first-inscription-height", "2"].as_slice(),
      ["--first-inscription-height", "2", "--index-sats"].as_slice(),
    ] {
      let context = Context::builder().args(args).build();
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      let send_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0, Default::default()), (2, 1, 0, Default::default())],
        ..Default::default()
      });

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: send_txid,
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn one_input_fee_spent_inscriptions_are_tracked_correctly() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0, Default::default())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });

      let coinbase_tx = context.mine_blocks(1)[0].txdata[0].txid();

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: coinbase_tx,
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn two_input_fee_spent_inscriptions_are_tracked_correctly() {
    for context in Context::configurations() {
      context.mine_blocks(2);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0, Default::default()), (3, 1, 0, Default::default())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });

      let coinbase_tx = context.mine_blocks(1)[0].txdata[0].txid();

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: coinbase_tx,
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn inscription_can_be_fee_spent_in_first_transaction() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

      let coinbase_tx = context.mine_blocks(1)[0].txdata[0].txid();

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: coinbase_tx,
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn lost_inscriptions() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks_with_subsidy(1, 0);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint::null(),
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn multiple_inscriptions_can_be_lost() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let first_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });
      let first_inscription_id = InscriptionId {
        txid: first_txid,
        index: 0,
      };

      context.mine_blocks_with_subsidy(1, 0);
      context.mine_blocks(1);

      let second_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 0, 0, inscription("text/plain", "hello").to_witness())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });
      let second_inscription_id = InscriptionId {
        txid: second_txid,
        index: 0,
      };

      context.mine_blocks_with_subsidy(1, 0);

      context.index.assert_inscription_location(
        first_inscription_id,
        SatPoint {
          outpoint: OutPoint::null(),
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        second_inscription_id,
        SatPoint {
          outpoint: OutPoint::null(),
          offset: 50 * COIN_VALUE,
        },
        Some(150 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn lost_sats_are_tracked_correctly() {
    let context = Context::builder().arg("--index-sats").build();
    assert_eq!(context.index.statistic(Statistic::LostSats), 0);

    context.mine_blocks(1);
    assert_eq!(context.index.statistic(Statistic::LostSats), 0);

    context.mine_blocks_with_subsidy(1, 0);
    assert_eq!(
      context.index.statistic(Statistic::LostSats),
      50 * COIN_VALUE
    );

    context.mine_blocks_with_subsidy(1, 0);
    assert_eq!(
      context.index.statistic(Statistic::LostSats),
      100 * COIN_VALUE
    );

    context.mine_blocks(1);
    assert_eq!(
      context.index.statistic(Statistic::LostSats),
      100 * COIN_VALUE
    );
  }

  #[test]
  fn lost_sat_ranges_are_tracked_correctly() {
    let context = Context::builder().arg("--index-sats").build();

    let null_ranges = || match context.index.list(OutPoint::null()).unwrap().unwrap() {
      List::Unspent(ranges) => ranges,
      _ => panic!(),
    };

    assert!(null_ranges().is_empty());

    context.mine_blocks(1);

    assert!(null_ranges().is_empty());

    context.mine_blocks_with_subsidy(1, 0);

    assert_eq!(null_ranges(), [(100 * COIN_VALUE, 150 * COIN_VALUE)]);

    context.mine_blocks_with_subsidy(1, 0);

    assert_eq!(
      null_ranges(),
      [
        (100 * COIN_VALUE, 150 * COIN_VALUE),
        (150 * COIN_VALUE, 200 * COIN_VALUE)
      ]
    );

    context.mine_blocks(1);

    assert_eq!(
      null_ranges(),
      [
        (100 * COIN_VALUE, 150 * COIN_VALUE),
        (150 * COIN_VALUE, 200 * COIN_VALUE)
      ]
    );

    context.mine_blocks_with_subsidy(1, 0);

    assert_eq!(
      null_ranges(),
      [
        (100 * COIN_VALUE, 150 * COIN_VALUE),
        (150 * COIN_VALUE, 200 * COIN_VALUE),
        (250 * COIN_VALUE, 300 * COIN_VALUE)
      ]
    );
  }

  #[test]
  fn lost_inscriptions_get_lost_satpoints() {
    for context in Context::configurations() {
      context.mine_blocks_with_subsidy(1, 0);
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0, inscription("text/plain", "hello").to_witness())],
        outputs: 2,
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };
      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 1, 1, Default::default()), (3, 1, 0, Default::default())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });
      context.mine_blocks_with_subsidy(1, 0);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint::null(),
          offset: 75 * COIN_VALUE,
        },
        Some(100 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn inscription_skips_zero_value_first_output_of_inscribe_transaction() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        outputs: 2,
        output_values: &[0, 50 * COIN_VALUE],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };
      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 1 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn inscription_can_be_lost_in_first_transaction() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };
      context.mine_blocks_with_subsidy(1, 0);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint::null(),
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn lost_rare_sats_are_tracked() {
    let context = Context::builder().arg("--index-sats").build();
    context.mine_blocks_with_subsidy(1, 0);
    context.mine_blocks_with_subsidy(1, 0);

    assert_eq!(
      context
        .index
        .rare_sat_satpoint(Sat(50 * COIN_VALUE))
        .unwrap()
        .unwrap(),
      SatPoint {
        outpoint: OutPoint::null(),
        offset: 0,
      },
    );

    assert_eq!(
      context
        .index
        .rare_sat_satpoint(Sat(100 * COIN_VALUE))
        .unwrap()
        .unwrap(),
      SatPoint {
        outpoint: OutPoint::null(),
        offset: 50 * COIN_VALUE,
      },
    );
  }

  #[test]
  fn old_schema_gives_correct_error() {
    let tempdir = {
      let context = Context::builder().build();

      let wtx = context.index.database.begin_write().unwrap();

      wtx
        .open_table(STATISTIC_TO_COUNT)
        .unwrap()
        .insert(&Statistic::Schema.key(), &0)
        .unwrap();

      wtx.commit().unwrap();

      context.tempdir
    };

    let path = tempdir.path().to_owned();

    let delimiter = if cfg!(windows) { '\\' } else { '/' };

    assert_eq!(
      Context::builder().tempdir(tempdir).try_build().err().unwrap().to_string(),
      format!("index at `{}{delimiter}regtest{delimiter}index.redb` appears to have been built with an older, incompatible version of ord, consider deleting and rebuilding the index: index schema 0, ord schema {SCHEMA_VERSION}", path.display()));
  }

  #[test]
  fn new_schema_gives_correct_error() {
    let tempdir = {
      let context = Context::builder().build();

      let wtx = context.index.database.begin_write().unwrap();

      wtx
        .open_table(STATISTIC_TO_COUNT)
        .unwrap()
        .insert(&Statistic::Schema.key(), &u64::MAX)
        .unwrap();

      wtx.commit().unwrap();

      context.tempdir
    };

    let path = tempdir.path().to_owned();

    let delimiter = if cfg!(windows) { '\\' } else { '/' };

    assert_eq!(
      Context::builder().tempdir(tempdir).try_build().err().unwrap().to_string(),
      format!("index at `{}{delimiter}regtest{delimiter}index.redb` appears to have been built with a newer, incompatible version of ord, consider updating ord: index schema {}, ord schema {SCHEMA_VERSION}", path.display(), u64::MAX));
  }

  #[test]
  fn inscriptions_on_output() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

      assert_eq!(
        context
          .index
          .get_inscriptions_on_output(OutPoint { txid, vout: 0 })
          .unwrap(),
        []
      );

      context.mine_blocks(1);

      assert_eq!(
        context
          .index
          .get_inscriptions_on_output(OutPoint { txid, vout: 0 })
          .unwrap(),
        [inscription_id]
      );

      let send_id = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0, Default::default())],
        ..Default::default()
      });

      context.mine_blocks(1);

      assert_eq!(
        context
          .index
          .get_inscriptions_on_output(OutPoint { txid, vout: 0 })
          .unwrap(),
        []
      );

      assert_eq!(
        context
          .index
          .get_inscriptions_on_output(OutPoint {
            txid: send_id,
            vout: 0,
          })
          .unwrap(),
        [inscription_id]
      );
    }
  }

  #[test]
  fn inscriptions_on_same_sat_after_the_first_are_not_unbound() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let first = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId {
        txid: first,
        index: 0,
      };

      assert_eq!(
        context
          .index
          .get_inscriptions_on_output(OutPoint {
            txid: first,
            vout: 0
          })
          .unwrap(),
        [inscription_id]
      );

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: first,
            vout: 0,
          },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      let second = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      let inscription_id = InscriptionId {
        txid: second,
        index: 0,
      };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: second,
            vout: 0,
          },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      assert!(context
        .index
        .get_inscription_by_id(InscriptionId {
          txid: second,
          index: 0
        })
        .unwrap()
        .is_some());

      assert!(context
        .index
        .get_inscription_by_id(InscriptionId {
          txid: second,
          index: 0
        })
        .unwrap()
        .is_some());
    }
  }

  #[test]
  fn get_latest_inscriptions_with_no_prev_and_next() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      let (inscriptions, prev, next, _, _) = context
        .index
        .get_latest_inscriptions_with_prev_and_next(100, None)
        .unwrap();
      assert_eq!(inscriptions, &[inscription_id]);
      assert_eq!(prev, None);
      assert_eq!(next, None);
    }
  }

  #[test]
  fn get_latest_inscriptions_with_prev_and_next() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let mut ids = Vec::new();

      for i in 0..103 {
        let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
          inputs: &[(i + 1, 0, 0, inscription("text/plain", "hello").to_witness())],
          ..Default::default()
        });
        ids.push(InscriptionId { txid, index: 0 });
        context.mine_blocks(1);
      }

      ids.reverse();

      let (inscriptions, prev, next, lowest, highest) = context
        .index
        .get_latest_inscriptions_with_prev_and_next(100, None)
        .unwrap();
      assert_eq!(inscriptions, &ids[..100]);
      assert_eq!(prev, Some(2));
      assert_eq!(next, None);
      assert_eq!(highest, 102);
      assert_eq!(lowest, 0);

      let (inscriptions, prev, next, _lowest, _highest) = context
        .index
        .get_latest_inscriptions_with_prev_and_next(100, Some(101))
        .unwrap();
      assert_eq!(inscriptions, &ids[1..101]);
      assert_eq!(prev, Some(1));
      assert_eq!(next, Some(102));

      let (inscriptions, prev, next, _lowest, _highest) = context
        .index
        .get_latest_inscriptions_with_prev_and_next(100, Some(0))
        .unwrap();
      assert_eq!(inscriptions, &ids[102..103]);
      assert_eq!(prev, None);
      assert_eq!(next, Some(100));
    }
  }

  #[test]
  fn unsynced_index_fails() {
    for context in Context::configurations() {
      let mut entropy = [0; 16];
      rand::thread_rng().fill_bytes(&mut entropy);
      context.rpc_server.mine_blocks(1);
      assert_regex_match!(
        context.index.get_unspent_outputs().unwrap_err().to_string(),
        r"output in Bitcoin Core wallet but not in ord index: [[:xdigit:]]{64}:\d+"
      );
    }
  }

  #[test]
  fn unrecognized_even_field_inscriptions_are_cursed_and_unbound() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let witness = envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[2],
        b"bar",
        &[4],
        b"ord",
      ]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 0,
        },
        None,
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .number,
        -1
      );
    }
  }

  #[test]
  // https://github.com/ordinals/ord/issues/2062
  fn zero_value_transaction_inscription_not_cursed_but_unbound() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, Default::default())],
        fee: 50 * 100_000_000,
        ..Default::default()
      });

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 0,
        },
        None,
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .number,
        0
      );
    }
  }

  #[test]
  fn transaction_with_inscription_inside_zero_value_2nd_input_should_be_unbound_and_cursed() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      // create zero value input
      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, Default::default())],
        fee: 50 * 100_000_000,
        ..Default::default()
      });

      context.mine_blocks(1);

      let witness = inscription("text/plain", "hello").to_witness();

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0, witness.clone()), (2, 1, 0, witness.clone())],
        ..Default::default()
      });

      let second_inscription_id = InscriptionId { txid, index: 1 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        second_inscription_id,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 0,
        },
        None,
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(second_inscription_id)
          .unwrap()
          .unwrap()
          .number,
        -1
      );
    }
  }

  #[test]
  fn multiple_inscriptions_in_same_tx_all_but_first_input_are_cursed() {
    for context in Context::configurations() {
      context.mine_blocks(1);
      context.mine_blocks(1);
      context.mine_blocks(1);

      let witness = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[
          (1, 0, 0, witness.clone()),
          (2, 0, 0, witness.clone()),
          (3, 0, 0, witness.clone()),
        ],
        ..Default::default()
      });

      let first = InscriptionId { txid, index: 0 };
      let second = InscriptionId { txid, index: 1 };
      let third = InscriptionId { txid, index: 2 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        first,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        second,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 50 * COIN_VALUE,
        },
        Some(100 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        third,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 100 * COIN_VALUE,
        },
        Some(150 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(first)
          .unwrap()
          .unwrap()
          .number,
        0
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(second)
          .unwrap()
          .unwrap()
          .number,
        -1
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(third)
          .unwrap()
          .unwrap()
          .number,
        -2
      );
    }
  }

  #[test]
  fn multiple_inscriptions_same_input_all_but_first_are_cursed_and_unbound() {
    for context in Context::configurations() {
      context.rpc_server.mine_blocks(1);

      let script = script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"foo")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"bar")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"qix")
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

      let witness = Witness::from_slice(&[script.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],

        ..Default::default()
      });

      let first = InscriptionId { txid, index: 0 };
      let second = InscriptionId { txid, index: 1 };
      let third = InscriptionId { txid, index: 2 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        first,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        second,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 0,
        },
        None,
      );

      context.index.assert_inscription_location(
        third,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 1,
        },
        None,
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(first)
          .unwrap()
          .unwrap()
          .number,
        0
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(second)
          .unwrap()
          .unwrap()
          .number,
        -1
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(third)
          .unwrap()
          .unwrap()
          .number,
        -2
      );
    }
  }

  #[test]
  fn multiple_inscriptions_different_inputs_and_same_inputs() {
    for context in Context::configurations() {
      context.rpc_server.mine_blocks(1);
      context.rpc_server.mine_blocks(1);
      context.rpc_server.mine_blocks(1);

      let script = script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"foo")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"bar")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"qix")
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

      let witness = Witness::from_slice(&[script.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[
          (1, 0, 0, witness.clone()),
          (2, 0, 0, witness.clone()),
          (3, 0, 0, witness.clone()),
        ],
        ..Default::default()
      });

      let first = InscriptionId { txid, index: 0 }; // normal
      let fourth = InscriptionId { txid, index: 3 }; // cursed but bound
      let ninth = InscriptionId { txid, index: 8 }; // cursed and unbound

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        first,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        fourth,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 50 * COIN_VALUE,
        },
        Some(100 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        ninth,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 5,
        },
        None,
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(first)
          .unwrap()
          .unwrap()
          .number,
        0
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(fourth)
          .unwrap()
          .unwrap()
          .number,
        -3
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(ninth)
          .unwrap()
          .unwrap()
          .number,
        -8
      );
    }
  }

  #[test]
  fn genesis_fee_distributed_evenly() {
    for context in Context::configurations() {
      context.rpc_server.mine_blocks(1);

      let script = script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"foo")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"bar")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"qix")
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

      let witness = Witness::from_slice(&[script.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
        fee: 33,
        ..Default::default()
      });

      let first = InscriptionId { txid, index: 0 };
      let second = InscriptionId { txid, index: 1 };

      context.mine_blocks(1);

      assert_eq!(
        context
          .index
          .get_inscription_entry(first)
          .unwrap()
          .unwrap()
          .fee,
        11
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(second)
          .unwrap()
          .unwrap()
          .fee,
        11
      );
    }
  }

  #[test]
  fn reinscription_on_cursed_inscription_is_not_cursed() {
    for context in Context::configurations() {
      context.mine_blocks(1);
      context.mine_blocks(1);

      let witness = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

      let cursed_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness.clone()), (2, 0, 0, witness.clone())],
        outputs: 2,
        ..Default::default()
      });

      let cursed = InscriptionId {
        txid: cursed_txid,
        index: 1,
      };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        cursed,
        SatPoint {
          outpoint: OutPoint {
            txid: cursed_txid,
            vout: 1,
          },
          offset: 0,
        },
        Some(100 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(cursed)
          .unwrap()
          .unwrap()
          .number,
        -1
      );

      let witness = envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        b"reinscription on cursed",
      ]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 1, 1, witness)],
        ..Default::default()
      });

      let reinscription_on_cursed = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        reinscription_on_cursed,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(100 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(reinscription_on_cursed)
          .unwrap()
          .unwrap()
          .number,
        1
      );
    }
  }

  #[test]
  fn second_reinscription_on_cursed_inscription_is_cursed() {
    for context in Context::configurations() {
      context.mine_blocks(1);
      context.mine_blocks(1);

      let witness = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

      let cursed_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness.clone()), (2, 0, 0, witness.clone())],
        outputs: 2,
        ..Default::default()
      });

      let cursed = InscriptionId {
        txid: cursed_txid,
        index: 1,
      };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        cursed,
        SatPoint {
          outpoint: OutPoint {
            txid: cursed_txid,
            vout: 1,
          },
          offset: 0,
        },
        Some(100 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(cursed)
          .unwrap()
          .unwrap()
          .number,
        -1
      );

      let witness = envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        b"reinscription on cursed",
      ]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 1, 1, witness)],
        ..Default::default()
      });

      let reinscription_on_cursed = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        reinscription_on_cursed,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(100 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(reinscription_on_cursed)
          .unwrap()
          .unwrap()
          .number,
        1
      );

      let witness = envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        b"second reinscription on cursed",
      ]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(4, 1, 0, witness)],
        ..Default::default()
      });

      let second_reinscription_on_cursed = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        second_reinscription_on_cursed,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(100 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(second_reinscription_on_cursed)
          .unwrap()
          .unwrap()
          .number,
        -2
      );

      assert_eq!(
        vec![
          cursed,
          reinscription_on_cursed,
          second_reinscription_on_cursed
        ],
        context
          .index
          .get_inscriptions_on_output_with_satpoints(OutPoint { txid, vout: 0 })
          .unwrap()
          .iter()
          .map(|(_satpoint, inscription_id)| *inscription_id)
          .collect::<Vec<InscriptionId>>()
      )
    }
  }

  #[test]
  fn reinscriptions_on_output_correctly_ordered_and_transferred() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          1,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });

      let first = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          1,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });

      let second = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);
      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          3,
          1,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });

      let third = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      let location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      assert_eq!(
        vec![(location, first), (location, second), (location, third)],
        context
          .index
          .get_inscriptions_on_output_with_satpoints(OutPoint { txid, vout: 0 })
          .unwrap()
      )
    }
  }

  #[test]
  fn reinscriptions_sequence_numbers_are_tracked_correctly() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let mut inscription_ids = vec![];
      for i in 1..=5 {
        let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
          inputs: &[(
            i,
            if i == 1 { 0 } else { 1 },
            0,
            inscription("text/plain;charset=utf-8", &format!("hello {}", i)).to_witness(),
          )], // for the first inscription use coinbase, otherwise use the previous tx
          ..Default::default()
        });

        inscription_ids.push(InscriptionId { txid, index: 0 });

        context.mine_blocks(1);
      }

      let rtx = context.index.database.begin_read().unwrap();
      let re_id_to_seq_num = rtx.open_table(REINSCRIPTION_ID_TO_SEQUENCE_NUMBER).unwrap();

      for (index, id) in inscription_ids.iter().enumerate() {
        match re_id_to_seq_num.get(&id.store()) {
          Ok(Some(num)) => {
            let sequence_number = num.value() + 1; // remove at next index refactor
            assert_eq!(
              index as u64, sequence_number,
              "sequence number mismatch for {:?}",
              id
            );
          }
          Ok(None) => assert_eq!(
            index, 0,
            "only first inscription should not have sequence number for {:?}",
            id
          ),
          Err(error) => panic!("Error retrieving sequence number: {}", error),
        }
      }
    }
  }

  #[test]
  fn reinscriptions_are_ordered_correctly_for_many_outpoints() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let mut inscription_ids = vec![];
      for i in 1..=21 {
        let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
          inputs: &[(
            i,
            if i == 1 { 0 } else { 1 },
            0,
            inscription("text/plain;charset=utf-8", &format!("hello {}", i)).to_witness(),
          )], // for the first inscription use coinbase, otherwise use the previous tx
          ..Default::default()
        });

        inscription_ids.push(InscriptionId { txid, index: 0 });

        context.mine_blocks(1);
      }

      let final_txid = inscription_ids.last().unwrap().txid;
      let location = SatPoint {
        outpoint: OutPoint {
          txid: final_txid,
          vout: 0,
        },
        offset: 0,
      };

      let expected_result = inscription_ids
        .iter()
        .map(|id| (location, *id))
        .collect::<Vec<(SatPoint, InscriptionId)>>();

      assert_eq!(
        expected_result,
        context
          .index
          .get_inscriptions_on_output_with_satpoints(OutPoint {
            txid: final_txid,
            vout: 0
          })
          .unwrap()
      )
    }
  }

  #[test]
  fn recover_from_reorg() {
    for mut context in Context::configurations() {
      context.index.set_durability(redb::Durability::Immediate);

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          1,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });
      let first_id = InscriptionId { txid, index: 0 };
      let first_location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      context.mine_blocks(6);

      context
        .index
        .assert_inscription_location(first_id, first_location, Some(50 * COIN_VALUE));

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });
      let second_id = InscriptionId { txid, index: 0 };
      let second_location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      context.mine_blocks(1);

      context
        .index
        .assert_inscription_location(second_id, second_location, Some(100 * COIN_VALUE));

      context.rpc_server.invalidate_tip();
      context.mine_blocks(2);

      context
        .index
        .assert_inscription_location(first_id, first_location, Some(50 * COIN_VALUE));

      context.index.assert_non_existence_of_inscription(second_id);
    }
  }

  #[test]
  fn recover_from_3_block_deep_and_consecutive_reorg() {
    for mut context in Context::configurations() {
      context.index.set_durability(redb::Durability::Immediate);

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          1,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });
      let first_id = InscriptionId { txid, index: 0 };
      let first_location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      context.mine_blocks(10);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });
      let second_id = InscriptionId { txid, index: 0 };
      let second_location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      context.mine_blocks(1);

      context
        .index
        .assert_inscription_location(second_id, second_location, Some(100 * COIN_VALUE));

      context.rpc_server.invalidate_tip();
      context.rpc_server.invalidate_tip();
      context.rpc_server.invalidate_tip();

      context.mine_blocks(4);

      context.index.assert_non_existence_of_inscription(second_id);

      context.rpc_server.invalidate_tip();

      context.mine_blocks(2);

      context
        .index
        .assert_inscription_location(first_id, first_location, Some(50 * COIN_VALUE));
    }
  }

  #[test]
  fn recover_from_very_unlikely_7_block_deep_reorg() {
    for mut context in Context::configurations() {
      context.index.set_durability(redb::Durability::Immediate);

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          1,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });

      context.mine_blocks(11);

      let first_id = InscriptionId { txid, index: 0 };
      let first_location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });

      let second_id = InscriptionId { txid, index: 0 };
      let second_location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      context.mine_blocks(7);

      context
        .index
        .assert_inscription_location(second_id, second_location, Some(100 * COIN_VALUE));

      for _ in 0..7 {
        context.rpc_server.invalidate_tip();
      }

      context.mine_blocks(9);

      context.index.assert_non_existence_of_inscription(second_id);

      context
        .index
        .assert_inscription_location(first_id, first_location, Some(50 * COIN_VALUE));
    }
  }

  #[test]
  fn inscription_without_parent_tag_has_no_parent_entry() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      assert!(context
        .index
        .get_inscription_entry(inscription_id)
        .unwrap()
        .unwrap()
        .parent
        .is_none());
    }
  }

  #[test]
  fn inscription_with_parent_tag_without_parent_has_no_parent_entry() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let parent_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let parent_inscription_id = InscriptionId {
        txid: parent_txid,
        index: 0,
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          0,
          0,
          Inscription {
            content_type: Some("text/plain".into()),
            body: Some("hello".into()),
            parent: Some(parent_inscription_id.parent_value()),
            unrecognized_even_field: false,
          }
          .to_witness(),
        )],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      assert!(context
        .index
        .get_inscription_entry(inscription_id)
        .unwrap()
        .unwrap()
        .parent
        .is_none());
    }
  }

  #[test]
  fn inscription_with_parent_tag_and_parent_has_parent_entry() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let parent_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let parent_inscription_id = InscriptionId {
        txid: parent_txid,
        index: 0,
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          1,
          0,
          Inscription {
            content_type: Some("text/plain".into()),
            body: Some("hello".into()),
            parent: Some(parent_inscription_id.parent_value()),
            unrecognized_even_field: false,
          }
          .to_witness(),
        )],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .parent
          .unwrap(),
        parent_inscription_id
      );

      assert_eq!(
        context
          .index
          .get_children_by_inscription_id(parent_inscription_id)
          .unwrap(),
        vec![inscription_id]
      );
    }
  }

  #[test]
  fn parents_can_be_in_preceding_input() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let parent_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(2);

      let parent_inscription_id = InscriptionId {
        txid: parent_txid,
        index: 0,
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[
          (2, 1, 0, Default::default()),
          (
            3,
            0,
            0,
            Inscription {
              content_type: Some("text/plain".into()),
              body: Some("hello".into()),
              parent: Some(parent_inscription_id.parent_value()),
              unrecognized_even_field: false,
            }
            .to_witness(),
          ),
        ],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .parent
          .unwrap(),
        parent_inscription_id
      );

      assert_eq!(
        context
          .index
          .get_children_by_inscription_id(parent_inscription_id)
          .unwrap(),
        vec![inscription_id]
      );
    }
  }

  #[test]
  fn parents_can_be_in_following_input() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let parent_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(2);

      let parent_inscription_id = InscriptionId {
        txid: parent_txid,
        index: 0,
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[
          (
            3,
            0,
            0,
            Inscription {
              content_type: Some("text/plain".into()),
              body: Some("hello".into()),
              parent: Some(parent_inscription_id.parent_value()),
              unrecognized_even_field: false,
            }
            .to_witness(),
          ),
          (2, 1, 0, Default::default()),
        ],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .parent
          .unwrap(),
        parent_inscription_id
      );

      assert_eq!(
        context
          .index
          .get_children_by_inscription_id(parent_inscription_id)
          .unwrap(),
        vec![inscription_id]
      );
    }
  }

  #[test]
  fn inscription_with_invalid_parent_tag_and_parent_has_no_parent_entry() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let parent_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let parent_inscription_id = InscriptionId {
        txid: parent_txid,
        index: 0,
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          1,
          0,
          Inscription {
            content_type: Some("text/plain".into()),
            body: Some("hello".into()),
            parent: Some(
              parent_inscription_id
                .parent_value()
                .into_iter()
                .chain(iter::once(0))
                .collect(),
            ),
            unrecognized_even_field: false,
          }
          .to_witness(),
        )],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      assert!(context
        .index
        .get_inscription_entry(inscription_id)
        .unwrap()
        .unwrap()
        .parent
        .is_none());
    }
  }
}
