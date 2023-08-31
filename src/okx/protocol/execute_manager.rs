use {
  super::*,
  crate::{
    inscription_id::InscriptionId,
    okx::{
      datastore::{
        balance::convert_pledged_tick_without_decimal, brc20 as brc20_store,
        brc20s as brc20s_store, brc20s::PledgedTick, btc as btc_store, StateRWriter,
      },
      protocol::{
        brc20 as brc20_proto, brc20s as brc20s_proto, brc20s::params::NATIVE_TOKEN,
        btc as btc_proto,
      },
    },
    Result, SatPoint,
  },
  bitcoin::OutPoint,
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
    let receipt = match msg {
      Message::BRC20(msg) => brc20_proto::execute(
        context,
        self.state_store.ord(),
        self.state_store.brc20(),
        &brc20_proto::ExecutionMessage::from_message(self.state_store.ord(), msg, context.network)?,
      )
      .map(|v| v.map(Receipt::BRC20))?,
      Message::BRC20S(msg) => brc20s::execute(
        context,
        brc20s::get_config_by_network(context.network, context.blockheight),
        self.state_store.brc20(),
        self.state_store.brc20s(),
        self.state_store.btc(),
        &brc20s::ExecutionMessage::from_message(self.state_store.ord(), msg, context.network)?,
      )
      .map(|v| v.map(Receipt::BRC20S))?,
      Message::BTC(msg) => btc_proto::gen_receipt(msg).map(|v| v.map(Receipt::BTC))?,
    };

    if receipt.is_none() {
      return Ok(());
    };

    // convert receipt to internal call message
    match receipt.unwrap() {
      Receipt::BRC20(brc20_receipt) => {
        if let Ok(brc20_store::Event::Transfer(brc20_transfer)) = brc20_receipt.result {
          let ptick = brc20s_store::PledgedTick::BRC20Tick(brc20_transfer.tick.clone());
          match convert_pledged_tick_without_decimal(
            &ptick,
            brc20_transfer.amount,
            self.state_store.brc20s(),
            self.state_store.brc20(),
          ) {
            Ok(amt) => {
              let passive_unstake = brc20s_proto::PassiveUnStake {
                stake: brc20_transfer.tick.to_string(),
                amount: amt.to_string(),
              };
              if let Message::BRC20(_) = msg {
                let passive_msg = convert_receipt_to_passive_msg(msg, passive_unstake);
                brc20s::execute(
                  context,
                  brc20s::get_config_by_network(context.network, context.blockheight),
                  self.state_store.brc20(),
                  self.state_store.brc20s(),
                  self.state_store.btc(),
                  &brc20s::ExecutionMessage::from_message(
                    self.state_store.ord(),
                    &passive_msg,
                    context.network,
                  )?,
                )?;
              }
            }
            Err(e) => {
              log::error!("brc20s receipt failed: {e}");
            }
          }
        }
        Ok(())
      }
      Receipt::BRC20S(brc20s_receipt) => {
        if let Ok(events) = brc20s_receipt.result {
          let mut events = events.into_iter();
          while let Some(brc20s_store::Event::Transfer(brc20s_transfer)) = events.next() {
            let ptick = brc20s_store::PledgedTick::BRC20STick(brc20s_transfer.tick_id);
            match convert_pledged_tick_without_decimal(
              &ptick,
              brc20s_transfer.amt,
              self.state_store.brc20s(),
              self.state_store.brc20(),
            ) {
              Ok(amt) => {
                let passive_unstake = brc20s_proto::PassiveUnStake {
                  stake: ptick.to_string(),
                  amount: amt.to_string(),
                };
                if let Message::BRC20S(_) = msg {
                  let passive_msg = convert_receipt_to_passive_msg(msg, passive_unstake);
                  brc20s::execute(
                    context,
                    brc20s::get_config_by_network(context.network, context.blockheight),
                    self.state_store.brc20(),
                    self.state_store.brc20s(),
                    self.state_store.btc(),
                    &brc20s::ExecutionMessage::from_message(
                      self.state_store.ord(),
                      &passive_msg,
                      context.network,
                    )?,
                  )?;
                }
              }
              Err(e) => {
                log::error!("brc20s receipt failed:{}", e);
              }
            }
          }
        }
        Ok(())
      }
      Receipt::BTC(btc_receipt) => {
        match btc_receipt.result {
          btc_store::Event::Transfer(btc_transfer) => {
            match convert_pledged_tick_without_decimal(
              &PledgedTick::Native,
              btc_transfer.amt,
              self.state_store.brc20s(),
              self.state_store.brc20(),
            ) {
              Ok(amt) => {
                let passive_unstake = brc20s_proto::PassiveUnStake {
                  stake: NATIVE_TOKEN.to_string(),
                  amount: amt.to_string(),
                };
                if let Message::BTC(old_btc_msg) = msg {
                  let passive_msg = convert_receipt_to_passive_msg(msg, passive_unstake);
                  brc20s::execute(
                    context,
                    brc20s::get_config_by_network(context.network, context.blockheight),
                    self.state_store.brc20(),
                    self.state_store.brc20s(),
                    self.state_store.btc(),
                    &brc20s::ExecutionMessage::from_btc_message(
                      &passive_msg,
                      old_btc_msg.from.clone(),
                    )?,
                  )?;
                }
              }
              Err(e) => {
                log::error!("brc20s receipt failed: {e}");
              }
            }
          }
        }
        Ok(())
      }
    }
  }
}

fn convert_receipt_to_passive_msg(
  msg: &Message,
  op: brc20s_proto::PassiveUnStake,
) -> brc20s::Message {
  match msg {
    Message::BRC20(msg) => brc20s::Message {
      txid: msg.txid,
      inscription_id: msg.inscription_id,
      commit_input_satpoint: None,
      old_satpoint: msg.old_satpoint,
      new_satpoint: msg.new_satpoint,
      op: brc20s::Operation::PassiveUnStake(op),
      sat_in_outputs: msg.sat_in_outputs,
    },
    Message::BRC20S(msg) => brc20s::Message {
      txid: msg.txid,
      inscription_id: msg.inscription_id,
      commit_input_satpoint: None,
      old_satpoint: msg.old_satpoint,
      new_satpoint: msg.new_satpoint,
      op: brc20s::Operation::PassiveUnStake(op),
      sat_in_outputs: msg.sat_in_outputs,
    },
    Message::BTC(msg) => brc20s::Message {
      txid: msg.txid,
      inscription_id: InscriptionId::from(msg.txid),
      commit_input_satpoint: None,
      old_satpoint: SatPoint {
        outpoint: OutPoint::null(),
        offset: 0,
      },
      new_satpoint: Some(SatPoint {
        outpoint: OutPoint::null(),
        offset: 0,
      }),
      op: brc20s::Operation::PassiveUnStake(op),
      sat_in_outputs: false,
    },
  }
}
