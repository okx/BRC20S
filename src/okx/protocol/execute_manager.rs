use {
  super::*,
  crate::{
    okx::{datastore::StateRWriter, protocol::brc20 as brc20_proto},
    Result,
  },
};

pub struct CallManager<'a, RW: StateRWriter> {
  state_store: &'a RW,
}

impl<'a, RW: StateRWriter> CallManager<'a, RW> {
  pub fn new(state_store: &'a RW) -> Self {
    Self { state_store }
  }

  pub fn execute_message(&self, context: BlockContext, msg: &Message) -> Result {
    // execute message
    match msg {
      Message::BRC20(msg) => brc20_proto::execute(
        context,
        self.state_store.ord(),
        self.state_store.brc20(),
        &brc20_proto::ExecutionMessage::from_message(self.state_store.ord(), msg, context.network)?,
      )
      .map(|v| v.map(Receipt::BRC20))?,
    };

    Ok(())
  }
}
