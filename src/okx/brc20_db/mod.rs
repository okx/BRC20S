mod storage_balance;

use self::storage_balance::StoreBalance;
use crate::brc20::{ledger::Ledger, *};
use crate::InscriptionId;
use bitcoin::Txid;
use redb::{ReadableTable, TableDefinition, WriteTransaction};
use std::str::FromStr;

const BRC20_BALANCES: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC20_BALANCES");
const BRC20_TOKEN: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC20_TOKEN");
const BRC20_EVENTS: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC20_EVENTS");
const BRC20_TRANSFERABLELOG: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC20_TRANSFERABLELOG");

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

  fn get_balances(&self, script_key: &ScriptKey) -> Result<Vec<(Tick, Balance)>, Self::Error> {
    Ok(
      self
        .wtx
        .open_table(BRC20_BALANCES)?
        .range(min_script_tick_key(script_key).as_str()..max_script_tick_key(&script_key).as_str())?
        .map(|(_, data)| {
          let bal = bincode::deserialize::<StoreBalance>(data.value()).unwrap();
          (bal.tick, bal.balance)
        })
        .collect(),
    )
  }

  fn get_balance(
    &self,
    script_key: &ScriptKey,
    tick: &Tick,
  ) -> Result<Option<Balance>, Self::Error> {
    Ok(
      self
        .wtx
        .open_table(BRC20_BALANCES)?
        .get(script_tick_key(script_key, tick).as_str())?
        .map(|v| {
          let bal = bincode::deserialize::<StoreBalance>(v.value()).unwrap();
          assert_eq!(&bal.tick, tick);
          bal.balance
        }),
    )
  }

  fn update_token_balance(
    &self,
    script_key: &ScriptKey,
    tick: &Tick,
    new_balance: Balance,
  ) -> Result<(), Self::Error> {
    let bal = StoreBalance {
      tick: tick.clone(),
      balance: new_balance,
    };
    self.wtx.open_table(BRC20_BALANCES)?.insert(
      script_tick_key(script_key, tick).as_str(),
      bincode::serialize(&bal).unwrap().as_slice(),
    )?;
    Ok(())
  }

  fn get_token_info(&self, tick: &Tick) -> Result<Option<TokenInfo>, Self::Error> {
    Ok(
      self
        .wtx
        .open_table(BRC20_TOKEN)?
        .get(tick.to_lowercase().hex().as_str())?
        .map(|v| bincode::deserialize::<TokenInfo>(v.value()).unwrap()),
    )
  }

  fn get_tokens_info(&self) -> Result<Vec<TokenInfo>, Self::Error> {
    Ok(
      self
        .wtx
        .open_table(BRC20_TOKEN)?
        .range::<&str>(..)?
        .map(|(_, data)| bincode::deserialize::<TokenInfo>(data.value()).unwrap())
        .collect(),
    )
  }

  fn insert_token_info(&self, tick: &Tick, new_info: &TokenInfo) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC20_TOKEN)?.insert(
      tick.to_lowercase().hex().as_str(),
      bincode::serialize(new_info).unwrap().as_slice(),
    )?;
    Ok(())
  }

  fn update_mint_token_info(
    &self,
    tick: &Tick,
    minted_amt: u128,
    minted_block_number: u64,
  ) -> Result<(), Self::Error> {
    let mut info = self
      .get_token_info(tick)?
      .expect(&format!("token {} not exist", tick.hex()));

    info.minted += minted_amt;
    info.latest_mint_number = minted_block_number;

    self.wtx.open_table(BRC20_TOKEN)?.insert(
      tick.to_lowercase().hex().as_str(),
      bincode::serialize(&info).unwrap().as_slice(),
    )?;
    Ok(())
  }

  fn get_transaction_receipts(&self, txid: &Txid) -> Result<Vec<ActionReceipt>, Self::Error> {
    Ok(
      self
        .wtx
        .open_table(BRC20_EVENTS)?
        .get(txid.to_string().as_str())?
        .map_or(Vec::new(), |v| {
          bincode::deserialize::<Vec<ActionReceipt>>(v.value()).unwrap()
        }),
    )
  }

  fn save_transaction_receipts(
    &self,
    txid: &Txid,
    receipts: &[ActionReceipt],
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC20_EVENTS)?.insert(
      txid.to_string().as_str(),
      bincode::serialize(receipts).unwrap().as_slice(),
    )?;
    Ok(())
  }

  fn get_transferable(&self, script: &ScriptKey) -> Result<Vec<TransferableLog>, Self::Error> {
    Ok(
      self
        .wtx
        .open_table(BRC20_TRANSFERABLELOG)?
        .range(min_script_tick_key(script).as_str()..max_script_tick_key(script).as_str())?
        .map(|(_, v)| bincode::deserialize::<Vec<TransferableLog>>(v.value()).unwrap())
        .flatten()
        .collect(),
    )
  }

  fn get_transferable_by_tick(
    &self,
    script: &ScriptKey,
    tick: &Tick,
  ) -> Result<Vec<TransferableLog>, Self::Error> {
    Ok(
      self
        .wtx
        .open_table(BRC20_TRANSFERABLELOG)?
        .get(script_tick_key(script, tick).as_str())?
        .map_or(Vec::new(), |v| {
          bincode::deserialize::<Vec<TransferableLog>>(v.value()).unwrap()
        }),
    )
  }

  fn get_transferable_by_id(
    &self,
    script: &ScriptKey,
    inscription_id: &InscriptionId,
  ) -> Result<Option<TransferableLog>, Self::Error> {
    Ok(
      self
        .get_transferable(script)?
        .iter()
        .find(|log| log.inscription_id == *inscription_id)
        .map(|log| log.clone()),
    )
  }

  fn insert_transferable(
    &self,
    script: &ScriptKey,
    tick: &Tick,
    inscription: TransferableLog,
  ) -> Result<(), Self::Error> {
    let mut logs = self.get_transferable_by_tick(script, tick)?;
    if logs
      .iter()
      .find(|log| log.inscription_id == inscription.inscription_id)
      .is_some()
    {
      return Ok(());
    }

    logs.push(inscription);

    self.wtx.open_table(BRC20_TRANSFERABLELOG)?.insert(
      script_tick_key(script, tick).as_str(),
      bincode::serialize(&logs).unwrap().as_slice(),
    )?;
    Ok(())
  }

  fn remove_transferable(
    &self,
    script: &ScriptKey,
    tick: &Tick,
    inscription_id: InscriptionId,
  ) -> Result<(), Self::Error> {
    let mut logs = self.get_transferable_by_tick(script, tick)?;
    let old_len = logs.len();

    logs.retain(|log| log.inscription_id != inscription_id);

    if logs.len() != old_len {
      self.wtx.open_table(BRC20_TRANSFERABLELOG)?.insert(
        script_tick_key(script, tick).as_str(),
        bincode::serialize(&logs).unwrap().as_slice(),
      )?;
    }
    Ok(())
  }
}

fn script_tick_key(script: &ScriptKey, tick: &Tick) -> String {
  format!("{}_{}", script.to_string(), tick.to_lowercase().hex())
}

fn min_script_tick_key(script: &ScriptKey) -> String {
  format!("{}_{}", script.to_string(), Tick::min_hex())
}

fn max_script_tick_key(script: &ScriptKey) -> String {
  format!("{}_{}", script.to_string(), Tick::max_hex())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::brc20::BRC20Error;
  use crate::SatPoint;
  use bitcoin::Address;
  use redb::Database;
  use std::str::FromStr;
  use tempfile::NamedTempFile;

  #[test]
  fn test_get_balances() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc20db = BRC20Database::new(&wtx);

    let script = ScriptKey::from_address(
      Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
    );
    let tick1 = Tick::from_str("abcd").unwrap();
    let tick2 = Tick::from_str("1234").unwrap();
    let tick3 = Tick::from_str(";23!").unwrap();
    let expect_balance1 = Balance {
      overall_balance: 10,
      transferable_balance: 10,
    };
    let expect_balance2 = Balance {
      overall_balance: 30,
      transferable_balance: 30,
    };
    let expect_balance3 = Balance {
      overall_balance: 100,
      transferable_balance: 30,
    };
    brc20db
      .update_token_balance(&script, &tick1, expect_balance1.clone())
      .unwrap();
    brc20db
      .update_token_balance(&script, &tick2, expect_balance2.clone())
      .unwrap();
    brc20db
      .update_token_balance(&script, &tick3, expect_balance3.clone())
      .unwrap();

    let script2 =
      ScriptKey::from_address(Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap());
    assert_ne!(script.to_string(), script2.to_string());
    let expect_balance22 = Balance {
      overall_balance: 100,
      transferable_balance: 30,
    };
    brc20db
      .update_token_balance(&script2, &tick1, expect_balance22.clone())
      .unwrap();

    let mut all_balances = brc20db.get_balances(&script).unwrap();
    all_balances.sort_by(|a, b| a.0.hex().cmp(&b.0.hex()));
    let mut expect = vec![
      (tick2, expect_balance2),
      (tick1, expect_balance1),
      (tick3, expect_balance3),
    ];
    expect.sort_by(|a, b| a.0.hex().cmp(&b.0.hex()));
    assert_eq!(all_balances, expect);
  }

  #[test]
  fn test_set_get_balance() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc20db = BRC20Database::new(&wtx);

    let script = ScriptKey::from_address(
      Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
    );
    let lower_tick = Tick::from_str("abcd").unwrap();
    let expect_balance = Balance {
      overall_balance: 30,
      transferable_balance: 30,
    };

    brc20db
      .update_token_balance(&script, &lower_tick, expect_balance.clone())
      .unwrap();

    let upper_tick = Tick::from_str("ABCd").unwrap();
    assert_eq!(
      brc20db.get_balance(&script, &upper_tick).unwrap(),
      Some(expect_balance)
    );
    assert_eq!(
      brc20db
        .get_balance(&script, &Tick::from_str("1111").unwrap())
        .unwrap(),
      None
    );
  }

  #[test]
  fn test_get_set_token_info() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc20db = BRC20Database::new(&wtx);

    let upper_tick = Tick::from_str("ABCD").unwrap();
    let expect = TokenInfo {
      tick: upper_tick.clone(),
      inscription_id: InscriptionId::from_str(
        "1111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      supply: 100,
      minted: 10,
      limit_per_mint: 10,
      decimal: 1,
      deploy_by: ScriptKey::from_address(
        Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
      ),
      deployed_number: 99,
      deployed_timestamp: 11222,
      latest_mint_number: 101,
    };

    brc20db.insert_token_info(&upper_tick, &expect).unwrap();

    let lower_tick = upper_tick.to_lowercase();
    assert_eq!(brc20db.get_token_info(&lower_tick).unwrap(), Some(expect));
  }

  #[test]
  fn test_get_tokens_info() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc20db = BRC20Database::new(&wtx);

    let expect1 = TokenInfo {
      tick: Tick::from_str("abcd").unwrap(),
      inscription_id: InscriptionId::from_str(
        "1111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      supply: 100,
      minted: 10,
      limit_per_mint: 10,
      decimal: 1,
      deploy_by: ScriptKey::from_address(
        Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
      ),
      deployed_number: 99,
      deployed_timestamp: 11222,
      latest_mint_number: 101,
    };
    let expect2 = TokenInfo {
      tick: Tick::from_str("1234").unwrap(),
      inscription_id: InscriptionId::from_str(
        "2111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      supply: 200,
      minted: 20,
      limit_per_mint: 20,
      decimal: 1,
      deploy_by: ScriptKey::from_address(
        Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
      ),
      deployed_number: 299,
      deployed_timestamp: 33222,
      latest_mint_number: 2101,
    };
    let expect3 = TokenInfo {
      tick: Tick::from_str("xyzm").unwrap(),
      inscription_id: InscriptionId::from_str(
        "3111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      supply: 300,
      minted: 30,
      limit_per_mint: 20,
      decimal: 1,
      deploy_by: ScriptKey::from_address(
        Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
      ),
      deployed_number: 399,
      deployed_timestamp: 33222,
      latest_mint_number: 3101,
    };

    brc20db.insert_token_info(&expect1.tick, &expect1).unwrap();
    brc20db.insert_token_info(&expect2.tick, &expect2).unwrap();
    brc20db.insert_token_info(&expect3.tick, &expect3).unwrap();

    let mut infos = brc20db.get_tokens_info().unwrap();
    infos.sort_by(|a, b| a.tick.hex().cmp(&b.tick.hex()));
    let mut expect = vec![expect1, expect2, expect3];
    expect.sort_by(|a, b| a.tick.hex().cmp(&b.tick.hex()));
    assert_eq!(infos, expect);
  }

  #[test]
  fn test_update_mint_token_info() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc20db = BRC20Database::new(&wtx);

    let tick = Tick::from_str("aBcd").unwrap();
    let org_info = TokenInfo {
      tick: tick.clone(),
      inscription_id: InscriptionId::from_str(
        "1111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      supply: 100,
      minted: 10,
      limit_per_mint: 10,
      decimal: 1,
      deploy_by: ScriptKey::from_address(
        Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
      ),
      deployed_number: 99,
      deployed_timestamp: 33222,
      latest_mint_number: 101,
    };

    brc20db.insert_token_info(&tick, &org_info).unwrap();
    let mint_amount = 30;
    let mint_block = 222;
    brc20db
      .update_mint_token_info(&tick, mint_amount, mint_block)
      .unwrap();

    let upper_tick = Tick::from_str("ABcD").unwrap();
    assert_eq!(
      brc20db.get_token_info(&upper_tick).unwrap(),
      Some(TokenInfo {
        minted: org_info.minted + mint_amount,
        latest_mint_number: mint_block,
        ..org_info
      })
    );
  }

  #[test]
  fn test_save_get_transaction_receipts() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc20db = BRC20Database::new(&wtx);

    let txid =
      Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap();
    let receipts = vec![
      ActionReceipt {
        inscription_id: InscriptionId::from_str(
          "1111111111111111111111111111111111111111111111111111111111111111i1",
        )
        .unwrap(),
        old_satpoint: SatPoint::from_str(
          "1111111111111111111111111111111111111111111111111111111111111111:1:1",
        )
        .unwrap(),
        new_satpoint: Some(
          SatPoint::from_str(
            "2111111111111111111111111111111111111111111111111111111111111111:1:1",
          )
          .unwrap(),
        ),
        result: Err(BRC20Error::InvalidTickLen(5)),
      },
      ActionReceipt {
        inscription_id: InscriptionId::from_str(
          "2111111111111111111111111111111111111111111111111111111111111111i1",
        )
        .unwrap(),
        old_satpoint: SatPoint::from_str(
          "2111111111111111111111111111111111111111111111111111111111111111:1:1",
        )
        .unwrap(),
        new_satpoint: Some(
          SatPoint::from_str(
            "3111111111111111111111111111111111111111111111111111111111111111:1:1",
          )
          .unwrap(),
        ),
        result: Ok(BRC20Event::Mint(MintEvent {
          tick: Tick::from_str("maEd").unwrap(),
          to: ScriptKey::from_address(
            Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
          ),
          amount: 30,
          msg: None,
        })),
      },
      ActionReceipt {
        inscription_id: InscriptionId::from_str(
          "3111111111111111111111111111111111111111111111111111111111111111i1",
        )
        .unwrap(),
        old_satpoint: SatPoint::from_str(
          "4111111111111111111111111111111111111111111111111111111111111111:1:1",
        )
        .unwrap(),
        new_satpoint: Some(
          SatPoint::from_str(
            "4111111111111111111111111111111111111111111111111111111111111111:1:1",
          )
          .unwrap(),
        ),
        result: Ok(BRC20Event::TransferPhase2(TransferPhase2Event {
          tick: Tick::from_str("mmmm").unwrap(),
          from: ScriptKey::from_address(
            Address::from_str("1QJVDzdqb1VpbDK7uDeyVXy9mR27CJiyhY").unwrap(),
          ),
          to: ScriptKey::from_address(
            Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
          ),
          amount: 11,
          msg: Some("a msg".to_string()),
        })),
      },
    ];

    brc20db.save_transaction_receipts(&txid, &receipts).unwrap();

    assert_eq!(brc20db.get_transaction_receipts(&txid).unwrap(), receipts);
  }

  #[test]
  fn test_get_transferable_by_tick() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc20db = BRC20Database::new(&wtx);

    let script = ScriptKey::from_address(
      Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
    );
    let tick = Tick::from_str("m23e").unwrap();
    let transferable_log1 = TransferableLog {
      inscription_id: InscriptionId::from_str(
        "3111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      amount: 10,
      tick: tick.clone(),
      owner: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
    };
    let transferable_log2 = TransferableLog {
      inscription_id: InscriptionId::from_str(
        "2111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      amount: 20,
      tick: tick.clone(),
      owner: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
    };

    brc20db
      .insert_transferable(&script, &tick, transferable_log1.clone())
      .unwrap();
    brc20db
      .insert_transferable(&script, &tick, transferable_log2.clone())
      .unwrap();

    let upper_tick = Tick::from_str("M23E").unwrap();
    assert_eq!(
      brc20db
        .get_transferable_by_tick(&script, &upper_tick)
        .unwrap(),
      vec![transferable_log1, transferable_log2]
    );

    // check not exist key
    let not_exist_tick = Tick::from_str("1111").unwrap();
    assert_eq!(
      brc20db
        .get_transferable_by_tick(&script, &not_exist_tick)
        .unwrap(),
      Vec::new()
    );
    let not_exist_script =
      ScriptKey::from_address(Address::from_str("1QJVDzdqb1VpbDK7uDeyVXy9mR27CJiyhY").unwrap());
    assert_eq!(
      brc20db
        .get_transferable_by_tick(&not_exist_script, &tick)
        .unwrap(),
      Vec::new()
    );
  }

  #[test]
  fn test_get_transferable_by_id() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc20db = BRC20Database::new(&wtx);

    let script = ScriptKey::from_address(
      Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
    );
    let tick = Tick::from_str("m23e").unwrap();
    let transferable_log1 = TransferableLog {
      inscription_id: InscriptionId::from_str(
        "3111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      amount: 10,
      tick: tick.clone(),
      owner: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
    };
    let transferable_log2 = TransferableLog {
      inscription_id: InscriptionId::from_str(
        "2111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      amount: 20,
      tick: tick.clone(),
      owner: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
    };

    brc20db
      .insert_transferable(&script, &tick, transferable_log1.clone())
      .unwrap();
    brc20db
      .insert_transferable(&script, &tick, transferable_log2.clone())
      .unwrap();

    assert_eq!(
      brc20db
        .get_transferable_by_id(&script, &transferable_log1.inscription_id)
        .unwrap(),
      Some(transferable_log1.clone())
    );
    assert_eq!(
      brc20db
        .get_transferable_by_id(&script, &transferable_log2.inscription_id)
        .unwrap(),
      Some(transferable_log2.clone())
    );

    // check not exist key
    let not_exist_id =
      InscriptionId::from_str("9991111111111111111111111111111111111111111111111111111111111111i1")
        .unwrap();
    assert_eq!(
      brc20db
        .get_transferable_by_id(&script, &not_exist_id)
        .unwrap(),
      None
    );
    let not_exist_script =
      ScriptKey::from_address(Address::from_str("1QJVDzdqb1VpbDK7uDeyVXy9mR27CJiyhY").unwrap());
    assert_eq!(
      brc20db
        .get_transferable_by_id(&not_exist_script, &transferable_log1.inscription_id)
        .unwrap(),
      None
    );
    assert_eq!(
      brc20db
        .get_transferable_by_id(&not_exist_script, &transferable_log2.inscription_id)
        .unwrap(),
      None
    );
  }

  #[test]
  fn test_insert_transferable_duplicate() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc20db = BRC20Database::new(&wtx);

    let script = ScriptKey::from_address(
      Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
    );
    let tick = Tick::from_str("m23e").unwrap();
    let transferable_log1 = TransferableLog {
      inscription_id: InscriptionId::from_str(
        "1111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      amount: 10,
      tick: tick.clone(),
      owner: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
    };
    let transferable_log2 = TransferableLog {
      inscription_id: InscriptionId::from_str(
        "2111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      amount: 10,
      tick: tick.clone(),
      owner: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
    };

    brc20db
      .insert_transferable(&script, &tick, transferable_log1.clone())
      .unwrap();
    brc20db
      .insert_transferable(&script, &tick, transferable_log2.clone())
      .unwrap();
    brc20db
      .insert_transferable(&script, &tick, transferable_log1.clone())
      .unwrap();

    assert_eq!(
      brc20db.get_transferable_by_tick(&script, &tick).unwrap(),
      vec![transferable_log1, transferable_log2]
    );
  }

  #[test]
  fn test_get_transferable() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc20db = BRC20Database::new(&wtx);

    let script1 = ScriptKey::from_address(
      Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
    );
    let tick1 = Tick::from_str("m23e").unwrap();
    let transferable_log11 = TransferableLog {
      inscription_id: InscriptionId::from_str(
        "1111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      amount: 10,
      tick: tick1.clone(),
      owner: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
    };
    let transferable_log12 = TransferableLog {
      inscription_id: InscriptionId::from_str(
        "1211111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      amount: 30,
      tick: tick1.clone(),
      owner: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
    };
    // insert two logs for script1 and tick1
    brc20db
      .insert_transferable(&script1, &tick1, transferable_log11.clone())
      .unwrap();
    brc20db
      .insert_transferable(&script1, &tick1, transferable_log12.clone())
      .unwrap();

    let tick2 = Tick::from_str("2222").unwrap();
    let transferable_log13 = TransferableLog {
      inscription_id: InscriptionId::from_str(
        "1311111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      amount: 10,
      tick: tick2.clone(),
      owner: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
    };
    let transferable_log14 = TransferableLog {
      inscription_id: InscriptionId::from_str(
        "1411111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      amount: 30,
      tick: tick2.clone(),
      owner: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
    };
    // insert two logs for script1 and tick2
    brc20db
      .insert_transferable(&script1, &tick2, transferable_log13.clone())
      .unwrap();
    brc20db
      .insert_transferable(&script1, &tick2, transferable_log14.clone())
      .unwrap();

    let script2 =
      ScriptKey::from_address(Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap());
    let transferable_log21 = TransferableLog {
      inscription_id: InscriptionId::from_str(
        "2111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      amount: 30,
      tick: Tick::from_str("m333").unwrap(),
      owner: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
    };
    // insert one log for script2 and tick
    assert_ne!(script1.to_string(), script2.to_string());
    brc20db
      .insert_transferable(
        &script2,
        &transferable_log21.tick,
        transferable_log21.clone(),
      )
      .unwrap();

    let mut transferable_logs = brc20db.get_transferable(&script1).unwrap();
    transferable_logs.sort_by(|a, b| a.tick.hex().cmp(&b.tick.hex()));
    let mut expect = vec![
      transferable_log11,
      transferable_log12,
      transferable_log13,
      transferable_log14,
    ]; // there's no transferable_log21
    expect.sort_by(|a, b| a.tick.hex().cmp(&b.tick.hex()));
    assert_eq!(transferable_logs, expect);
  }

  #[test]
  fn test_remove_transferable() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc20db = BRC20Database::new(&wtx);

    let script = ScriptKey::from_address(
      Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
    );
    let tick = Tick::from_str("m23e").unwrap();
    let transferable_log1 = TransferableLog {
      inscription_id: InscriptionId::from_str(
        "1111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      amount: 10,
      tick: tick.clone(),
      owner: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
    };
    let transferable_log2 = TransferableLog {
      inscription_id: InscriptionId::from_str(
        "2111111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      amount: 10,
      tick: tick.clone(),
      owner: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
    };
    brc20db
      .insert_transferable(&script, &tick, transferable_log1.clone())
      .unwrap();
    brc20db
      .insert_transferable(&script, &tick, transferable_log2.clone())
      .unwrap();

    // remove a not exist inscription_id
    let not_exist_id =
      InscriptionId::from_str("9911111111111111111111111111111111111111111111111111111111111111i1")
        .unwrap();
    brc20db
      .remove_transferable(&script, &tick, not_exist_id)
      .unwrap();
    assert_eq!(
      brc20db.get_transferable_by_tick(&script, &tick).unwrap(),
      vec![transferable_log1.clone(), transferable_log2.clone()]
    );

    let upper_tick = Tick::from_str("M23E").unwrap();
    brc20db
      .remove_transferable(&script, &upper_tick, transferable_log1.inscription_id)
      .unwrap();
    assert_eq!(
      brc20db
        .get_transferable_by_tick(&script, &upper_tick)
        .unwrap(),
      vec![transferable_log2]
    );
  }
}
