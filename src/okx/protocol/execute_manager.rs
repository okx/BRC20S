use super::*;
use crate::okx::datastore::balance::convert_pledged_tick_without_decimal;
use crate::okx::datastore::brc20 as brc20_store;
use crate::okx::datastore::brc20s as brc20s_store;
use crate::okx::datastore::ord as ord_store;
use crate::okx::protocol::brc0 as brc0_proto;
use crate::okx::protocol::brc20 as brc20_proto;
use crate::okx::protocol::brc20s as brc20s_proto;
use crate::rpc::BRCZeroRpcClient;
use crate::Result;
use crate::okx::protocol::brc0::{RpcResponse, BRCZeroTx, RpcRequest, RpcParams};


pub struct CallManager<
  'a,
  O: ord_store::OrdDataStoreReadWrite,
  N: brc20_store::DataStoreReadWrite,
  M: brc20s_store::DataStoreReadWrite,
> {
  ord_store: &'a O,
  brc20_store: &'a N,
  brc20s_store: &'a M,
}

impl<
    'a,
    O: ord_store::OrdDataStoreReadWrite,
    N: brc20_store::DataStoreReadWrite,
    M: brc20s_store::DataStoreReadWrite,
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
      Message::BRC20(msg) => brc20_proto::execute(
        context,
        self.ord_store,
        self.brc20_store,
        &brc20_proto::ExecutionMessage::from_message(self.ord_store, msg, context.network)?,
      )
      .map(|v| v.map(Receipt::BRC20))?,
      Message::BRC20S(msg) => brc20s::execute(
        context,
        self.brc20_store,
        self.brc20s_store,
        &brc20s::ExecutionMessage::from_message(self.ord_store, msg, context.network)?,
      )
      .map(|v| v.map(Receipt::BRC20S))?,
      Message::BRC0(msg) => brc0_proto::execute(
        context,
        &brc0_proto::ExecutionMessage::from_message(self.ord_store, msg, context.network)?,
      )
      .map(|v| v.map(Receipt::BRC0))?,
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
            self.brc20s_store,
            self.brc20_store,
          ) {
            Ok(amt) => {
              let passive_unstake = brc20s_proto::PassiveUnStake {
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
              self.brc20s_store,
              self.brc20_store,
            ) {
              Ok(amt) => {
                let passive_unstake = brc20s_proto::PassiveUnStake {
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
        Ok(())
      }
      Receipt::BRC0(_brc20s_receipt) => Ok(()),
    }
  }

  pub fn execute_block_message(
    &self,
    brc0_client: &BRCZeroRpcClient,
    context: BlockContext,
    messages: Vec<Message>,
  ) -> Result {
    let mut exe_msgs: Vec<brc0_proto::ExecutionMessage> = Vec::new();
    for msg in messages.iter() {
      match msg {
        Message::BRC0(msg) => exe_msgs.push(brc0_proto::ExecutionMessage::from_message(
          self.ord_store,
          msg,
          context.network,
        )?),
        _ => {}
      }
    }

    brc0_proto::execute_msgs(brc0_client, context, exe_msgs)
  }

  pub fn send_to_brc0(
    &self,
    brc0_client: &BRCZeroRpcClient,
    context: BlockContext,
    brc0_msgs: Vec<BrcZeroMsg>,
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
        block_hash: context.blockhash.to_string(),
        is_confirmed: false,
        txs,
      },
    };
    log::debug!("Request: {:#?}", request);

    init_tokio_runtime().block_on(async {
      let response = brc0_client
          .client
          .post(&brc0_client.url)
          .header("Content-Type", "application/json")
          .json(&request)
          .send()
          .await;

      match response {
        Ok(res)=>{
          if res.status().is_success(){
            let body = res.text().await;
            match body {
              Ok(body) => {
                match serde_json::from_str::<RpcResponse>(body.as_str()){
                  Ok(rpc_res) => {
                    if rpc_res.result.len() > 0{
                      for tx_res in rpc_res.result.iter() {
                        log::info!("broadcast brczero txs successes: {}", tx_res.hash);
                      }
                    }else{
                      log::info!("broadcast btc block to brczero successes");
                    }
                  }
                  Err(e) => { log::error!("broadcast brczero txs JSON: {body} failed: {e}");}
                }
              }
              Err(err) => {log::error!("broadcast brczero txs body failed: {err}");}
            }
          }else{
            log::error!("broadcast brczero txs or btc block failed: {}",res.status());
          }
        },
        Err(e)=>{
          log::error!("broadcast brczero txs or btc block failed: {e}");
        }
      }
    });

    Ok(())
  }
}

fn convert_msg_brc20_to_brc20s(
  msg: &brc20_proto::Message,
  op: brc20s_proto::PassiveUnStake,
) -> brc20s::Message {
  brc20s::Message {
    txid: msg.txid,
    inscription_id: msg.inscription_id,
    commit_input_satpoint: None,
    old_satpoint: msg.old_satpoint,
    new_satpoint: msg.new_satpoint,
    op: brc20s::Operation::PassiveUnStake(op),
    sat_in_outputs: msg.sat_in_outputs,
  }
}

fn convert_msg_brc20s(msg: &brc20s::Message, op: brc20s_proto::PassiveUnStake) -> brc20s::Message {
  brc20s::Message {
    txid: msg.txid,
    inscription_id: msg.inscription_id,
    commit_input_satpoint: None,
    old_satpoint: msg.old_satpoint,
    new_satpoint: msg.new_satpoint,
    op: brc20s::Operation::PassiveUnStake(op),
    sat_in_outputs: msg.sat_in_outputs,
  }
}

/// Initialize Tokio runtime
fn init_tokio_runtime() -> tokio::runtime::Runtime {
  tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .unwrap()
}