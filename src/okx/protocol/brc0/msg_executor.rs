use super::*;
use crate::okx::datastore::ord as ord_store;

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
use serde_json::json;
use futures::TryFutureExt;
use hex::FromHex;
use std::convert::TryInto;
use web3::transports::Http;
use web3::types::Bytes;
use web3::{Web3, block_on};

use cosmrs::{
  bank::MsgSend,
  crypto::secp256k1,
  rpc,
  tx::{self, Fee, Msg, SignDoc, SignerInfo, Tx},
  AccountId, Coin,
};

use std::process::Command;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use shadow_rs::pmr::respan_to;


#[derive(Debug, Serialize, Deserialize)]
struct RpcRequest {
  jsonrpc: String,
  id: u64,
  method: String,
  params: RpcParams,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcParams{
  height: String,
  txs: Vec<String>,
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

pub fn execute_msgs(context: BlockContext, msgs: Vec<ExecutionMessage>) -> Result {
  log::debug!("BRC0 execute messages: {:?}", msgs);
  println!("{:?}",msgs);
  let mut txs: Vec<String> = Vec::new();
  for msg in msgs.iter(){
    let _event = match &msg.op {
      Operation::Evm(evm) => {
        txs.push(hex::encode(evm.clone().d.encode_rlp()));
      }
    };
  }

  let client = Client::new();
  let request = RpcRequest {
    jsonrpc: "2.0".to_string(),
    id: 3,
    method: "broadcast_brczero_txs_async".to_string(),
    params: RpcParams{
      height:context.blockheight.to_string(),
      txs,
    },
  };
  println!("Request: {:#?}", request);

  init_tokio_runtime().block_on(async {
    let response = client
        .post("http://localhost:26657")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await;

    println!("Response: {:#?}", response);
  });

  Ok(())
}

fn process_deploy(
  _context: BlockContext,
  _msg: &ExecutionMessage,
  evm: Evm,
) -> Result<Event, Error> {
  // TODO send okbc proposal tx
  // println!("EVM Data-----------{}", evm.d);

  // CLI
  // replace_evm_data(evm.d);
  // let output = Command::new("okbchaincli")
  //     .args(&["tx", "gov", "submit-proposal", "brczero-evm-data"])
  //     .args(&["/Users/oker/lrpData/brczero_evm_data.json"])
  //     .args(&["--from", "captain"])
  //     .args(&["--fees", "0.01okb"])
  //     .args(&["--gas", "3000000"])
  //     .args(&["--chain-id", "okbchain-67"])
  //     .args(&["--node", "http://127.0.0.1:26657"])
  //     .args(&["-b", "block"])
  //     .args(&["-y"])
  //     .output()
  //     .expect("Failed to execute command");
  //
  // if output.status.success() {
  //   let stdout = String::from_utf8_lossy(&output.stdout);
  //   println!("Command output:\n{}", stdout);
  // } else {
  //   let stderr = String::from_utf8_lossy(&output.stderr);
  //   println!("Command failed with error:\n{}", stderr);
  // }

  // send_raw_transaction
  // let http = Http::new("http://localhost:8545").unwrap();
  //
  // let web3 = Web3::new(http);
  //
  // let bytes_tx = Vec::from_hex(evm.d).unwrap();
  // init_tokio_runtime().block_on(async {
  //   let tx_res = web3.eth().send_raw_transaction(Bytes::from(bytes_tx)).await;
  //
  //   match tx_res {
  //     Ok(tx_hash) => {
  //       println!("Transaction sent. Hash: {}", tx_hash.to_string());
  //     }
  //     Err(err) => {
  //       println!("Transaction sent. Errror: {}", err);
  //     }
  //   }
  // });


  //todo: call debug api
  // let client = Client::new();
  // let request = RpcRequest {
  //   jsonrpc: "2.0".to_string(),
  //   method: "eth_submitBrczeroData".to_string(),
  //   params: vec![evm.d],
  //   id: 1,
  // };
  // println!("Request: {:#?}", request);
  //
  // init_tokio_runtime().block_on(async {
  //   let response = client
  //     .post("http://localhost:8545")
  //     .header("Content-Type", "application/json")
  //     .json(&request)
  //     .send()
  //     .await;
  //
  //   //todo: postprocess
  //   println!("Response: {:#?}", response);
  // });

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

fn replace_evm_data(data: String){
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

  let new_line = format!("  \"tx\": \"{}\",",data);
  lines[line_number_to_replace - 1] = Ok(new_line);

  // write to file
  let mut file = fs::File::create(&file_path).unwrap();
  for line in &lines {
    let _ = file.write_all(line.as_ref().unwrap().as_bytes());
    let _ = file.write_all(b"\n");
  }
}
