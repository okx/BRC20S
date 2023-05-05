use crate::brc20::ledger::{BRC20Balance, BRC20Event, BRC20TokenInfo, Inscription, Ledger};
use redb::{ReadableTable, TableDefinition, WriteTransaction};

const BRC20_BALANCES: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC20_BALANCES");
const BRC20_TOKEN: TableDefinition<&str, &[u8]> = TableDefinition::new("BRC20_TOKEN");
const BRC20_TRANSACTION_ID_TO_EVENTS: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC20_TRANSACTION_ID_TO_EVENTS");
const BRC20_ADDRESS_TO_TRANSFERABLE_INSCRIPTIONS: TableDefinition<&str, &[u8]> =
  TableDefinition::new("BRC20_ADDRESS_TO_TRANSFERABLE_INSCRIPTIONS");

pub struct BRC20Database<'db> {
  wtx: WriteTransaction<'db>,
}

impl<'db> BRC20Database<'db> {
  pub fn new(wtx: WriteTransaction<'db>) -> Self {
    Self { wtx }
  }

  pub fn commit(self) -> Result<(), redb::Error> {
    self.wtx.commit()
  }
}

impl<'db> Ledger for BRC20Database<'db> {
  type Error = redb::Error;

  // balance
  fn get_balance(&self, address_tick: &str) -> Result<Option<BRC20Balance>, Self::Error> {
    Ok(
      self
        .wtx
        .open_table(BRC20_BALANCES)?
        .get(address_tick)?
        .map(|v| bincode::deserialize::<BRC20Balance>(v.value()).unwrap()),
    )
  }

  fn set_balance(&self, address_tick: &str, new_balance: BRC20Balance) -> Result<(), Self::Error> {
    let data = bincode::serialize(&new_balance).unwrap();
    self
      .wtx
      .open_table(BRC20_BALANCES)?
      .insert(address_tick, data.as_slice())?;
    Ok(())
  }

  // token
  fn get_token_info(&self, tick: &str) -> Result<Option<BRC20TokenInfo>, Self::Error> {
    Ok(
      self
        .wtx
        .open_table(BRC20_TOKEN)?
        .get(tick.to_lowercase().as_str())?
        .map(|v| bincode::deserialize::<BRC20TokenInfo>(v.value()).unwrap()),
    )
  }
  fn set_token_info(&self, tick: &str, new_info: BRC20TokenInfo) -> Result<(), Self::Error> {
    let data = bincode::serialize(&new_info).unwrap();
    self
      .wtx
      .open_table(BRC20_TOKEN)?
      .insert(tick.to_lowercase().as_str(), data.as_slice())?;
    Ok(())
  }

  // event
  fn get_events_in_tx(&self, tx_id: &str) -> Result<Option<Vec<BRC20Event>>, Self::Error> {
    Ok(
      self
        .wtx
        .open_table(BRC20_TRANSACTION_ID_TO_EVENTS)?
        .get(tx_id)?
        .map(|v| bincode::deserialize::<Vec<BRC20Event>>(v.value()).unwrap()),
    )
  }
  fn set_events_in_tx(&self, tx_id: &str, events: &[BRC20Event]) -> Result<(), Self::Error> {
    let data = bincode::serialize(events).unwrap();
    self
      .wtx
      .open_table(BRC20_TRANSACTION_ID_TO_EVENTS)?
      .insert(tx_id, data.as_slice())?;
    Ok(())
  }

  // inscription
  fn get_inscriptions(&self, address_tick: &str) -> Result<Option<Vec<Inscription>>, Self::Error> {
    Ok(
      self
        .wtx
        .open_table(BRC20_ADDRESS_TO_TRANSFERABLE_INSCRIPTIONS)?
        .get(address_tick)?
        .map(|v| bincode::deserialize::<Vec<Inscription>>(v.value()).unwrap()),
    )
  }
  fn set_inscriptions(
    &self,
    address_tick: &str,
    inscriptions: &[Inscription],
  ) -> Result<(), Self::Error> {
    let data = bincode::serialize(&inscriptions).unwrap();
    self
      .wtx
      .open_table(BRC20_ADDRESS_TO_TRANSFERABLE_INSCRIPTIONS)?
      .insert(address_tick, data.as_slice())?;
    Ok(())
  }
}
