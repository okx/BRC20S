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
use futures::TryFutureExt;
use hex::FromHex;
use std::convert::TryInto;
use web3::transports::Http;
use web3::types::Bytes;
use web3::Web3;

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

fn process_deploy(
  _context: BlockContext,
  _msg: &ExecutionMessage,
  evm: Evm,
) -> Result<Event, Error> {
  // TODO send okbc proposal tx
  println!("-----------{}", evm.d);

  // CLI
  replace_evm_data(evm.d);
  let output = Command::new("okbchaincli")
      .args(&["tx", "gov", "submit-proposal", "brczero-evm-data"])
      .args(&["/Users/oker/Downloads/brczero_evm_data.json"])
      .args(&["--from", "captain"])
      .args(&["--fees", "0.01okb"])
      .args(&["--gas", "3000000"])
      .args(&["--chain-id", "okbchain-67"])
      .args(&["--node", "http://127.0.0.1:26657"])
      .args(&["-b", "block"])
      .args(&["-y"])
      .output()
      .expect("Failed to execute command");

  if output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Command output:\n{}", stdout);
  } else {
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("Command failed with error:\n{}", stderr);
  }

  // TODO
  // let sender_private_key = secp256k1::SigningKey::random();
  // let sender_public_key = sender_private_key.public_key();
  // let sender_account_id = sender_public_key.account_id("ex").unwrap();
  //
  // let recipient_private_key = secp256k1::SigningKey::random();
  // let recipient_account_id = recipient_private_key.public_key().account_id("ex").unwrap();
  //
  // let amount = Coin {
  //   amount: 1u8.into(),
  //   denom: "okb".parse().unwrap(),
  // };
  //
  // let msg_send = MsgSend {
  //   from_address: sender_account_id.clone(),
  //   to_address: recipient_account_id,
  //   amount: vec![amount.clone()],
  // }
  // .to_any()
  // .unwrap();
  //
  // let chain_id = "okbchain-67".parse().unwrap();
  // let sequence_number = 0;
  // let gas = 100_000u64;
  // let fee = Fee::from_amount_and_gas(amount, gas);
  //
  // let tx_body = tx::BodyBuilder::new().msg(msg_send).memo("MEMO").finish();
  // let auth_info =
  //   SignerInfo::single_direct(Some(sender_public_key), sequence_number).auth_info(fee);
  // let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, 1).unwrap();
  // let tx_signed = sign_doc.sign(&sender_private_key).unwrap();
  // println!("{:?}", tx_signed.to_bytes());
  //
  // init_tokio_runtime().block_on(async {
  //   let rpc_address = "http://localhost:26657";
  //   let rpc_client = rpc::HttpClient::new(rpc_address).unwrap();
  //
  //   let tx_commit_response = tx_signed.broadcast_commit(&rpc_client).await.unwrap();
  //
  //   if tx_commit_response.check_tx.code.is_err() {
  //     panic!("check_tx failed: {:?}", tx_commit_response.check_tx);
  //   }
  //
  //   if tx_commit_response.deliver_tx.code.is_err() {
  //     panic!("deliver_tx failed: {:?}", tx_commit_response.deliver_tx);
  //   }
  // });

  // Serialize the raw transaction as bytes (i.e. `Vec<u8>`).
  // let tx_bytes = tx_signed.to_bytes()?;

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

  // TODO
  // let rpc_url = "http://localhost:26657"; // Replace with your Tendermint RPC URL
  // let client = HttpClient::new(rpc_url).expect("Failed to create HTTP client");
  // // let tendermint_client = Client::new(http_client);
  //
  // let priv_key = "YOUR_PRIVATE_KEY"; // Replace with your private key
  // let from_address = HumanAddr::from("YOUR_SENDER_ADDRESS"); // Replace with your sender address
  // let to_address = HumanAddr::from("GOVERNANCE_CONTRACT_ADDRESS"); // Replace with the governance contract address
  // let amount = Coin::new(100, "okb"); // Replace with the desired amount and token
  //
  // let msg = BankMsg::Send {
  //   to_address: to_address.clone(),
  //   amount: vec![amount.clone()],
  // };
  //
  // let std_msgs = vec![CosmosMsg::Bank(msg)];
  //
  // let fee = Coin::new(1000, "uatom"); // Replace with the desired fee amount and token
  // let gas_limit = 200_000; // Replace with the desired gas limit
  //
  // let tx = StdTx {
  //   msg: std_msgs.clone(),
  //   fee: StdFee {
  //     amount: vec![fee.clone()],
  //     gas: gas_limit.clone(),
  //   },
  //   signatures: vec![],
  //   memo: None,
  // };
  //
  // let key = PrivateKey::from_base64(priv_key).expect("Failed to parse private key");
  //
  // let sign_doc = tx.get_sign_doc(key.chain_id());
  //
  // let signature = key.sign(sign_doc);
  //
  // let signed_tx = tx.clone().with_signature(signature);
  //
  // let tx_bytes = to_binary(&signed_tx).expect("Failed to serialize transaction");
  //
  // let tx_commit_request = Request::new(Binary::from(tx_bytes), None);
  // let tx_commit_result = client.broadcast_tx_commit(&tx_commit_request);
  //
  // match tx_commit_result {
  //   Ok(response) => {
  //     if let Some(tx_result) = response.check_tx {
  //       if let Some(code) = tx_result.code {
  //         if code == 0 {
  //           println!("Transaction successful. Hash: {:?}", tx_result.hash);
  //         } else {
  //           println!("Transaction failed with code: {}", code);
  //         }
  //       } else {
  //         println!("Transaction failed with unknown code");
  //       }
  //     } else {
  //       println!("Transaction check failed");
  //     }
  //   }
  //   Err(err) => {
  //     eprintln!("Transaction error: {:?}", err);
  //   }
  // }

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
  let file_path = "/Users/oker/Downloads/brczero_evm_data.json";
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