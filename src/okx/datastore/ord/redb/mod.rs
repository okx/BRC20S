pub mod read_only;
pub mod read_write;

pub use self::read_only::OrdDbReader;
pub use self::read_write::OrdDbReadWriter;
use redb::TableDefinition;

const ORD_TX_TO_OPERATIONS: TableDefinition<&str, &[u8]> =
  TableDefinition::new("ORD_TX_TO_OPERATIONS");

const ORD_BRCZERO_TO_RPCPARAMS: TableDefinition<u64, &[u8]> =
  TableDefinition::new("ORD_BRCZERO_TO_RPCPARAMS");
