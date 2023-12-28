use super::*;
use crate::okx::datastore::ord as ord_store;
use crate::rpc::BRCZeroRpcClient;
use crate::{
  okx::{
    datastore::brc0::Receipt,
    protocol::{
      brc0::{BRCZeroTx, Message, Operation, RpcParams, RpcRequest, RpcResponse},
      utils, BlockContext,
    },
  },
  Result,
};
use anyhow::anyhow;
use bitcoin::Network;

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionMessage {
  pub(self) txid: Txid,
  pub(self) inscription_id: InscriptionId,
  pub(self) inscription_number: i64,
  pub(self) old_satpoint: SatPoint,
  pub(self) new_satpoint: SatPoint,
  pub(self) from: ScriptKey,
  pub(self) to: Option<ScriptKey>,
  pub(self) op: Operation,
  pub(self) btc_fee: u128,
}

impl ExecutionMessage {
  pub fn from_message<O: ord_store::DataStoreReadOnly>(
    ord_store: &O,
    msg: &Message,
    network: Network,
  ) -> Result<Self> {
    Ok(Self {
      txid: msg.txid,
      inscription_id: msg.inscription_id,
      inscription_number: utils::get_inscription_number_by_id(msg.inscription_id, ord_store)?,
      old_satpoint: msg.old_satpoint,
      new_satpoint: msg
        .new_satpoint
        .ok_or(anyhow!("new satpoint cannot be None"))?,
      from: utils::get_script_key_on_satpoint(msg.old_satpoint, ord_store, network)?,
      to: if msg.sat_in_outputs {
        Some(utils::get_script_key_on_satpoint(
          msg.new_satpoint.unwrap(),
          ord_store,
          network,
        )?)
      } else {
        None
      },
      op: msg.op.clone(),
      btc_fee: msg.btc_fee,
    })
  }
}

pub fn execute(_context: BlockContext, msg: &ExecutionMessage) -> Result<Option<Receipt>> {
  log::debug!("BRC0 execute message: {:?}", msg);

  Ok(None)
}

pub fn execute_msgs(
  brc0_client: &BRCZeroRpcClient,
  context: BlockContext,
  msgs: Vec<ExecutionMessage>,
) -> Result {
  log::debug!("BRC0 execute messages: {:?}", msgs);
  println!("{:?}", msgs);
  let mut txs: Vec<BRCZeroTx> = Vec::new();
  for msg in msgs.iter() {
    let _event = match &msg.op {
      Operation::Evm(evm) => {
        let tx = BRCZeroTx {
          hex_rlp_encode_tx: hex::encode(evm.clone().d.encode_rlp()),
          btc_fee: msg.btc_fee.to_string(),
        };
        txs.push(tx);
      }
    };
  }

  let request = RpcRequest {
    jsonrpc: "2.0".to_string(),
    id: 3,
    method: "broadcast_brczero_txs_async".to_string(),
    params: RpcParams {
      height: context.blockheight.to_string(),
      block_hash: "".to_string(),
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
      Ok(res) => {
        if res.status().is_success() {
          let body = res.text().await;
          let rpc_res: RpcResponse = serde_json::from_str(&*body.unwrap()).unwrap();
          if rpc_res.result.len() > 0 {
            for tx_res in rpc_res.result.iter() {
              log::info!("broadcast brczero txs successes: {}", tx_res.hash);
            }
          } else {
            log::info!("broadcast btc block to brczero successes");
          }
          // log::debug!("Response: {:#?}", rpc_res);
        } else {
          log::error!(
            "broadcast brczero txs or btc block failed: {}",
            res.status()
          );
        }
      }
      Err(e) => {
        log::error!("broadcast brczero txs or btc block failed: {e}");
      }
    }
  });

  Ok(())
}

/// Initialize Tokio runtime
fn init_tokio_runtime() -> tokio::runtime::Runtime {
  tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
}
