use crate::brc20::{ledger::Ledger, *};
use crate::InscriptionId;
use bitcoin::Txid;
use redb::{TableDefinition, WriteTransaction};

const BRC20_BALANCES: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC20_BALANCES");
const BRC20_TOKEN: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC20_TOKEN");
const BRC20_TRANSACTION_ID_TO_EVENTS: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC20_TRANSACTION_ID_TO_EVENTS");
const BRC20_ADDRESS_TO_TRANSFERABLE_INSCRIPTIONS: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC20_ADDRESS_TO_TRANSFERABLE_INSCRIPTIONS");

pub struct BRC20Database<'db, 'a> {
  wtx: &'a WriteTransaction<'db>,
}

impl<'db, 'a> BRC20Database<'db, 'a> {
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    Self { wtx }
  }
}

impl<'db, 'a> Ledger for BRC20Database<'db, 'a> {
  type Error = redb::Error;

  /**
   * 查询某个用户下所用的token余额信息
   * 1. 这里统一使用scriptKey当成索引。scriptKey是一个枚举类型，包含Address和ScriptHash，当script可以转换成address,直接使用address。当script不能转换address，则使用scriptHash
   * 3. 存入数据库的Key格式类似于bc1p....._tick 或xxxxxxxxx...._tick方便使用范围去查询
   * 4. 查询某个key下面所有的余额数据，传入key,根据规则1进行解析，并去数据库中使用range方式匹配出一系列key，xxxx_[0,4]。
   */
  fn get_balances(&self, script_key: &ScriptKey) -> Result<Vec<Balance>, Self::Error> {
    todo!("get_balances")
    // Ok(
    //   self
    //     .wtx
    //     .open_table(BRC20_BALANCES)?
    //     .range(address_tick)?
    //     .map(|v| bincode::deserialize::<Balance>(v.value()).unwrap()),
    // )
  }

  /**
   * 查询某个用户下某个token余额信息
   * 1. 与上述规则1方式，生成key格式为bc1p....._tick 或xxxxxxxxx...._tick
   * 2. tick在内部需要转换成小写to_lowercase()
   * 3. 进行数据库查询，返回结果
   */
  fn get_balance(
    &self,
    script_key: &ScriptKey,
    tick: &Tick,
  ) -> Result<Option<Balance>, Self::Error> {
    todo!("get_balancs")
    // Ok(
    //   self
    //     .wtx
    //     .open_table(BRC20_BALANCES)?
    //     .range(address_tick)?
    //     .map(|v| bincode::deserialize::<Balance>(v.value()).unwrap()),
    // )
  }

  /**
   * 更新某个token的balance
   * 1. 与上述规则1方式，生成key格式为bc1p....._tick 或xxxxxxxxx...._tick
   * 2. tick在内部需要转换成小写to_lowercase()
   * 3. 覆盖原值
   */
  fn update_token_balance(
    &self,
    script_key: &ScriptKey,
    tick: &Tick,
    new_balance: Balance,
  ) -> Result<(), Self::Error> {
    todo!("get_balancs")
    // Ok(
    //   self
    //     .wtx
    //     .open_table(BRC20_BALANCES)?
    //     .range(address_tick)?
    //     .map(|v| bincode::deserialize::<Balance>(v.value()).unwrap()),
    // )
  }

  /**
   * 获取token表里的某个数据
   * 1. tick在内部需要转换成小写to_lowercase()
   * 2. TokenInfo内的Tick不需要
   */
  fn get_token_info(&self, tick: &Tick) -> Result<Option<TokenInfo>, Self::Error> {
    todo!("get_balancs")
  }

  /**
   * 获取token表里的某个数据
   */
  fn get_tokens_info(&self) -> Result<Vec<TokenInfo>, Self::Error> {
    todo!("get_balancs")
  }
  /**
   * 直接插入一条token数据
   */
  fn insert_token_info(&self, tick: &Tick, new_info: &TokenInfo) -> Result<(), Self::Error> {
    todo!("get_balancs")
  }

  /**
   * 更新token表里的某个token的的minted数据和区块高度
   * 1. tick在内部需要转换成小写to_lowercase()
   * 2. 根据key查询该token
   * 3. 只更改minted_amt和minted_block_number存入数据库
   */
  fn update_mint_token_info(
    &self,
    tick: &Tick,
    minted_amt: u128,
    minted_block_number: u64,
  ) -> Result<(), Self::Error> {
    todo!("get_balancs")
  }

  // ------event相关------

  // 获取当前交易内的所有events
  fn get_transaction_receipts(&self, txid: Txid) -> Result<Vec<ActionReceipt>, Self::Error> {
    todo!("get_balancs")
  }
  fn save_transaction_receipts(
    &self,
    txid: Txid,
    receipts: &[ActionReceipt],
  ) -> Result<(), Self::Error> {
    todo!("get_balancs")
  }

  // ------transferable inscription相关------
  /**
   * 根据ScriptKey和tick组合成key，查询出所有的TransferableLog
   * 1. tick 小写
   * 2. 没有key或数据返回空数组
   */

  fn get_transferable(&self, script: ScriptKey) -> Result<Vec<TransferableLog>, Self::Error> {
    todo!("get_balancs")
  }

  /**
   * 基于上条原则，从数组中筛出对应的inscription_id，否则返回错误
   */
  fn get_transferable_by_tick(
    &self,
    script: ScriptKey,
    tick: Tick,
  ) -> Result<Vec<TransferableLog>, Self::Error> {
    todo!("get_balancs")
  }

  fn get_transferable_by_id(
    &self,
    script: &ScriptKey,
    inscription_id: InscriptionId,
  ) -> Result<Option<TransferableLog>, Self::Error> {
    todo!("get_balancs")
  }

  /**
   * 根据ScriptKey和tick组合成key，向数据库中的对应key的value中插入一条数据
   * 1. tick 小写
   * 2. 要根据TransferableLog内的inscription_id判重
   * 3. 加入数据项，更新数据库
   */
  fn insert_transferable(
    &self,
    script: &ScriptKey,
    tick: &Tick,
    inscription: &TransferableLog,
  ) -> Result<(), Self::Error> {
    todo!("get_balancs")
  }

  /**
   * 根据ScriptKey和tick组合成key，向数据库中的对应key的value中某条数据的状态
   * 1. tick 小写
   * 2. 查找到这个inscription_id的数据
   * 3. 将此条删除后落库。找不到此项不需要更新，不报错。
   */
  fn remove_transferable(
    &self,
    script: &ScriptKey,
    tick: &Tick,
    inscription_id: InscriptionId,
  ) -> Result<(), Self::Error> {
    todo!("get_balancs")
  }
}
