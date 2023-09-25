use super::*;
use crate::okx::datastore::{brc0, ord as ord_store};

use crate::rpc::BRCZeroRpcClient;
use crate::{
  okx::{
    datastore::brc0::{Event, EvmEvent, Receipt},
    protocol::{
      brc0::{Message, Operation},
      utils, BlockContext,
    },
  },
  Result,
};
use anyhow::anyhow;
use bitcoin::Network;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

use std::fs;
use std::io::{BufRead, BufReader, Write};

#[derive(Debug, Serialize, Deserialize)]
struct RpcRequest {
  jsonrpc: String,
  id: u64,
  method: String,
  params: RpcParams,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcParams {
  height: String,
  block_hash: String,
  is_confirmed: bool,
  txs: Vec<BRCZeroTx>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BRCZeroTx {
  hex_rlp_encode_tx: String,
  btc_fee: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcResponse {
  jsonrpc: String,
  result: String,
  id: u64,
}

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
  pub fn from_message<O: ord_store::OrdDataStoreReadOnly>(
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

pub fn execute(context: BlockContext, msg: &ExecutionMessage) -> Result<Option<Receipt>> {
  log::debug!("BRC0 execute message: {:?}", msg);
  let _event = match &msg.op {
    Operation::Evm(evm) => process_deploy(context, msg, evm.clone()),
  };

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
      block_hash: context.blockhash.to_string(),
      is_confirmed: false,
      txs,
    },
  };
  println!("Request: {:#?}", request);

  init_tokio_runtime().block_on(async {
    let response = brc0_client
      .client
      .post(&brc0_client.url)
      .header("Content-Type", "application/json")
      .json(&request)
      .send()
      .await;

    println!("Response: {:#?}", response);
  });

  Ok(())
}

fn process_deploy(_context: BlockContext, _msg: &ExecutionMessage, _: Evm) -> Result<Event, Error> {
  Ok(Event::Evm(EvmEvent {
    txhash: "tx_hash".to_string(),
  }))
}

/// Initialize Tokio runtime
fn init_tokio_runtime() -> tokio::runtime::Runtime {
  tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
}

fn replace_evm_data(data: String) {
  // read file
  let file_path = "/Users/oker/lrpData/brczero_evm_data.json";
  let mut lines = vec![];

  let file = fs::File::open(&file_path).unwrap();
  let reader = BufReader::new(file);

  for line in reader.lines() {
    lines.push(line);
  }

  // replacing a specific line
  let line_number_to_replace = 4; // Counting from 1

  let new_line = format!("  \"tx\": \"{}\",", data);
  lines[line_number_to_replace - 1] = Ok(new_line);

  // write to file
  let mut file = fs::File::create(&file_path).unwrap();
  for line in &lines {
    let _ = file.write_all(line.as_ref().unwrap().as_bytes());
    let _ = file.write_all(b"\n");
  }
}
