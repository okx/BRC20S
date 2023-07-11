use super::*;
use crate::okx::datastore::balance::convert_pledged_tick_without_decimal;
use crate::okx::datastore::brc20 as store_brc20;
use crate::okx::datastore::brc20s as store_brc20s;
use crate::okx::datastore::ord as store_ord;
use crate::okx::protocol::brc20 as proto_brc20;
use crate::okx::protocol::brc20s as proto_brc20s;
use crate::Result;

pub struct CallManager<
  'a,
  O: store_ord::OrdDataStoreReadWrite,
  N: store_brc20::BRC20DataStoreReadWrite,
  M: store_brc20s::DataStoreReadWrite,
> {
  ord_store: &'a O,
  brc20_store: &'a N,
  brc20s_store: &'a M,
}

impl<
    'a,
    O: store_ord::OrdDataStoreReadWrite,
    N: store_brc20::BRC20DataStoreReadWrite,
    M: store_brc20s::DataStoreReadWrite,
  > CallManager<'a, O, N, M>
{
  pub fn new(ord_store: &'a O, brc20_store: &'a N, brc20s_store: &'a M) -> Self {
    Self {
      ord_store,
      brc20_store,
      brc20s_store,
    }
  }

  pub fn execute_message(&self, context: BlockContext, msg: &Message) -> Result {
    // execute message
    let receipt = match msg {
      Message::BRC20(msg) => proto_brc20::execute(
        context,
        self.ord_store,
        self.brc20_store,
        &proto_brc20::BRC20ExecutionMessage::from_message(self.ord_store, &msg, context.network)?,
      )
      .map(|v| v.map(Receipt::BRC20))?,
      Message::BRC20S(msg) => brc20s::execute(
        context,
        self.brc20_store,
        self.brc20s_store,
        &brc20s::ExecutionMessage::from_message(self.ord_store, &msg, context.network)?,
      )
      .map(|v| v.map(Receipt::BRC20S))?,
    };

    if receipt.is_none() {
      return Ok(());
    };

    // convert receipt to internal call message
    match receipt.unwrap() {
      Receipt::BRC20(brc20_receipt) => {
        match brc20_receipt.result {
          Ok(store_brc20::BRC20Event::Transfer(brc20_transfer)) => {
            let ptick = store_brc20s::PledgedTick::BRC20Tick(brc20_transfer.tick.clone());
            match convert_pledged_tick_without_decimal(
              &ptick,
              brc20_transfer.amount,
              self.brc20s_store,
              self.brc20_store,
            ) {
              Ok(amt) => {
                let passive_unstake = proto_brc20s::PassiveUnStake {
                  stake: brc20_transfer.tick.as_str().to_string(),
                  amount: amt.to_string(),
                };
                if let Message::BRC20(old_brc20_msg) = msg {
                  let passive_msg = convert_msg_brc20_to_brc20s(old_brc20_msg, passive_unstake);
                  brc20s::execute(
                    context,
                    self.brc20_store,
                    self.brc20s_store,
                    &brc20s::ExecutionMessage::from_message(
                      self.ord_store,
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
          _ => { /*others operation ,we we do nothing */ }
        }
        Ok(())
      }
      Receipt::BRC20S(brc20s_receipt) => {
        match brc20s_receipt.result {
          Ok(events) => {
            let mut events = events.into_iter();
            while let Some(store_brc20s::Event::Transfer(brc20s_transfer)) = events.next() {
              let ptick = store_brc20s::PledgedTick::BRC20STick(brc20s_transfer.tick_id.clone());
              match convert_pledged_tick_without_decimal(
                &ptick,
                brc20s_transfer.amt,
                self.brc20s_store,
                self.brc20_store,
              ) {
                Ok(amt) => {
                  let passive_unstake = proto_brc20s::PassiveUnStake {
                    stake: ptick.to_string(),
                    amount: amt.to_string(),
                  };
                  if let Message::BRC20S(old_brc20s_msg) = msg {
                    let passive_msg = convert_msg_brc20s(old_brc20s_msg, passive_unstake);
                    brc20s::execute(
                      context,
                      self.brc20_store,
                      self.brc20s_store,
                      &brc20s::ExecutionMessage::from_message(
                        self.ord_store,
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
          _ => { /*others operation ,we we do nothing */ }
        }
        Ok(())
      }
    }
  }
}

fn convert_msg_brc20_to_brc20s(
  msg: &proto_brc20::BRC20Message,
  op: proto_brc20s::PassiveUnStake,
) -> brc20s::Message {
  brc20s::Message {
    txid: msg.txid.clone(),
    inscription_id: msg.inscription_id.clone(),
    commit_input_satpoint: None,
    old_satpoint: msg.old_satpoint.clone(),
    new_satpoint: msg.new_satpoint.clone(),
    op: proto_brc20s::Operation::PassiveUnStake(op),
  }
}

fn convert_msg_brc20s(msg: &brc20s::Message, op: proto_brc20s::PassiveUnStake) -> brc20s::Message {
  brc20s::Message {
    txid: msg.txid.clone(),
    inscription_id: msg.inscription_id.clone(),
    commit_input_satpoint: None,
    old_satpoint: msg.old_satpoint.clone(),
    new_satpoint: msg.new_satpoint.clone(),
    op: proto_brc20s::Operation::PassiveUnStake(op),
  }
}
