use crate::okx::datastore::ord::DataStoreReadWrite;
use crate::okx::protocol::brc0::{BRCZeroTx, RpcParams, RpcRequest};
use bitcoin::BlockHash;
use {
  super::*,
  crate::{
    okx::{
      datastore::{
        balance::convert_pledged_tick_without_decimal, brc20 as brc20_store,
        brc20s as brc20s_store, StateRWriter,
      },
      protocol::{brc20 as brc20_proto, brc20s as brc20s_proto},
    },
    Result,
  },
  anyhow::anyhow,
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
        &brc20s::ExecutionMessage::from_message(self.state_store.ord(), msg, context.network)?,
      )
      .map(|v| v.map(Receipt::BRC20S))?,
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
    }
  }

  pub fn send_to_brc0(
    &self,
    context: BlockContext,
    brc0_msgs: Vec<BrcZeroMsg>,
    block_hash: &BlockHash,
  ) -> Result {
    let mut txs: Vec<BRCZeroTx> = Vec::new();
    for brc0_msg in brc0_msgs.iter() {
      let tx = BRCZeroTx {
        hex_rlp_encode_tx: serde_json::to_string(&brc0_msg.msg).unwrap(),
        btc_fee: brc0_msg.btc_fee.to_string(),
      };
      txs.push(tx);
    }

    let request = RpcRequest {
      jsonrpc: "2.0".to_string(),
      id: 3,
      method: "broadcast_brczero_txs_async".to_string(),
      params: RpcParams {
        height: context.blockheight.to_string(),
        block_hash: block_hash.to_string(),
        is_confirmed: false,
        txs,
      },
    };
    log::info!("Request: {:#?}", request);

    let err = self
      .state_store
      .ord()
      .save_brczero_to_rpcparams(context.blockheight, &request.params.clone())
      .map_err(|e| anyhow!("failed to set transaction ordinals operations to state! error: {e}"));
    match err {
      Ok(()) => {}
      Err(e) => {
        log::error!("save_brczero_to_rpcparams error: {:#?}", e);
      }
    }

    Ok(())
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
  }
}
