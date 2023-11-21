use super::*;
use crate::okx::datastore::brc20 as brc20_store;
use crate::okx::datastore::brc20s as brc20s_store;
use crate::okx::datastore::ord as ord_store;
use std::collections::HashMap;

use crate::{
  okx::{datastore::ord::{Action,InscriptionOp,OrdDataStoreReadOnly},
        protocol::{Message}, },
  Inscription, Result, Index,
  sat_point::SatPoint
};
use anyhow::anyhow;
use bitcoin::{OutPoint, Transaction, TxOut};
use bitcoincore_rpc::Client;
use serde_json::{Value};

pub struct MsgResolveManager<
  'a,
  O: ord_store::OrdDataStoreReadWrite,
  N: brc20_store::DataStoreReadWrite,
  M: brc20s_store::DataStoreReadWrite,
> {
  protocol_start_height: HashMap<ProtocolKind, u64>,
  client: &'a Client,
  ord_store: &'a O,
  brc20_store: &'a N,
  brc20s_store: &'a M,
}

impl<
    'a,
    O: ord_store::OrdDataStoreReadWrite,
    N: brc20_store::DataStoreReadWrite,
    M: brc20s_store::DataStoreReadWrite,
  > MsgResolveManager<'a, O, N, M>
{
  pub fn new(
    client: &'a Client,
    ord_store: &'a O,
    brc20_store: &'a N,
    brc20s_store: &'a M,
    first_brc20_height: u64,
    first_brc20s_height: u64,
    first_brczero_height: u64,
  ) -> Self {
    let mut protocol_start_height: HashMap<ProtocolKind, u64> = HashMap::new();
    protocol_start_height.insert(ProtocolKind::BRC20, first_brc20_height);
    protocol_start_height.insert(ProtocolKind::BRC20S, first_brc20s_height);
    protocol_start_height.insert(ProtocolKind::BRC0, first_brczero_height);
    Self {
      client,
      ord_store,
      brc20_store,
      brc20s_store,
      protocol_start_height,
    }
  }

  pub fn resolve_message(
    &self,
    context: BlockContext,
    tx: &Transaction,
    operations: Vec<InscriptionOp>,
  ) -> Result<Vec<Message>> {
    log::debug!(
      "Resolve Manager indexed transaction {}, operations size: {}, data: {:?}",
      tx.txid(),
      operations.len(),
      operations
    );
    let mut messages = Vec::new();
    let mut operation_iter = operations.into_iter().peekable();
    let new_inscriptions = Inscription::from_transaction(tx)
      .into_iter()
      .map(|v| v.inscription)
      .collect::<Vec<Inscription>>();
    let btc_fee = self.get_btc_transaction_fee(tx);

    let mut outpoint_to_txout_cache: HashMap<OutPoint, TxOut> = HashMap::new();
    for input in &tx.input {
      // TODO: BTC transfer to BRC20S passive withdrawal.

      // if self.protocols.contains(&ProtocolKind::BRC20S) {
      //   messages.push(BTC::resolve_message(txid, block_height, block_time, tx));
      // }

      // "operations" is a list of all the operations in the current block, and they are ordered.
      // We just need to find the operation corresponding to the current transaction here.
      while let Some(operation) = operation_iter.peek() {
        if operation.old_satpoint.outpoint != input.previous_output {
          break;
        }
        let operation = operation_iter.next().unwrap();

        // Parse BRC20 message through inscription operation.
        if self
          .protocol_start_height
          .get(&ProtocolKind::BRC20)
          .map(|height| context.blockheight >= *height)
          .unwrap_or(false)
        {
          if let Some(msg) =
            brc20::Message::resolve(self.brc20_store, &new_inscriptions, &operation)?
          {
            log::debug!(
              "BRC20 resolved the message from {:?}, msg {:?}",
              operation,
              msg
            );
            messages.push(Message::BRC20(msg));
            continue;
          }
        }

        // Parse BRC30 message through inscription operation.
        if self
          .protocol_start_height
          .get(&ProtocolKind::BRC20S)
          .map(|height| context.blockheight >= *height)
          .unwrap_or(false)
        {
          if let Some(msg) = brc20s::Message::resolve(
            self.client,
            self.ord_store,
            self.brc20s_store,
            &new_inscriptions,
            &operation,
            &mut outpoint_to_txout_cache,
          )? {
            log::debug!(
              "BRC20S resolved the message from {:?}, msg {:?}",
              operation,
              msg
            );
            messages.push(Message::BRC20S(msg));
            continue;
          }
        }

        // Parse BRC0 message through inscription operation.
        if self
          .protocol_start_height
          .get(&ProtocolKind::BRC0)
          .map(|height| context.blockheight >= *height)
          .unwrap_or(false)
        {
          if let Some(msg) = brc0::Message::resolve(&new_inscriptions, &operation, btc_fee)? {
            log::debug!(
              "BRC0 resolved the message from {:?}, msg {:?}",
              operation,
              msg
            );
            // messages.push(Message::BRC0(msg));
            continue;
          }
        }
      }
    }
    self.update_outpoint_to_txout(outpoint_to_txout_cache)?;
    Ok(messages)
  }

  fn update_outpoint_to_txout(&self, outpoint_to_txout_cache: HashMap<OutPoint, TxOut>) -> Result {
    for (outpoint, txout) in outpoint_to_txout_cache {
      self
        .ord_store
        .set_outpoint_to_txout(outpoint, &txout)
        .or(Err(anyhow!(
          "failed to get tx out! error: {} not found",
          outpoint
        )))?;
    }
    Ok(())
  }

  fn get_btc_transaction_fee(&self, tx: &Transaction) -> u128 {
    let mut input_value = 0_u128;
    let mut output_value = 0_u128;
    for input in &tx.input {
      let value = self.ord_store.get_outpoint_to_txout(input.previous_output);
      match value {
        Ok(op_txout) => match op_txout {
          Some(txout) => input_value = input_value + txout.value as u128,
          None => {
            panic!("get_btc_transaction_fee:  get tx out is none")
          }
        },
        Err(e) => {
          panic!("get_btc_transaction_fee: failed to get tx out from state! error: {}", e)
        }
      }
    }

    for output in &tx.output {
      output_value = output_value + output.value as u128;
    }

    input_value - output_value
  }

  pub fn resolve_brczero_inscription(
    &self,
    context: BlockContext,
    tx: &Transaction,
    operations: Vec<InscriptionOp>,
  ) -> Result<Vec<BrcZeroMsg>> {
    log::debug!(
      "Resolve Inscription indexed transaction {}, operations size: {}, data: {:?}",
      tx.txid(),
      operations.len(),
      operations
    );
    let mut messages = Vec::new();
    let mut operation_iter = operations.into_iter().peekable();
    let new_inscriptions = Inscription::from_transaction(tx)
        .into_iter()
        .map(|v| v.inscription)
        .collect::<Vec<Inscription>>();
    let btc_fee = self.get_btc_transaction_fee(tx);

    let mut outpoint_to_txout_cache: HashMap<OutPoint, TxOut> = HashMap::new();
    for input in &tx.input {
      // "operations" is a list of all the operations in the current block, and they are ordered.
      // We just need to find the operation corresponding to the current transaction here.
      while let Some(operation) = operation_iter.peek() {
        if operation.old_satpoint.outpoint != input.previous_output {
          break;
        }
        let operation = operation_iter.next().unwrap();

        let sat_in_outputs = operation
            .new_satpoint
            .map(|satpoint| satpoint.outpoint.txid == operation.txid)
            .unwrap_or(false);

        let mut is_transfer = false;
        let mut sender = "".to_string();
        match operation.action {
          // New inscription is not `cursed` or `unbound`.
          Action::New {
            cursed: false,
            unbound: false,
          } => {
            let commit_input_satpoint = get_commit_input_satpoint(
                self.client,
                self.ord_store,
                operation.old_satpoint,
                &mut outpoint_to_txout_cache,
              )?;
            sender = utils::get_script_key_on_satpoint(commit_input_satpoint, self.ord_store, context.network)?.to_string();
          },
          // Transfer inscription operation.
          // Attempt to retrieve the `InscribeTransfer` Inscription information from the data store of BRC20S.
          Action::Transfer => match self.brc20_store.get_inscribe_transfer_inscription(operation.inscription_id) {
            // Ignore non-first transfer operations.
            // TODO ignore first?
            Ok(Some(_transfer_info)) if operation.inscription_id.txid == operation.old_satpoint.outpoint.txid => {
              is_transfer = true;
              sender = utils::get_script_key_on_satpoint(operation.old_satpoint, self.ord_store, context.network)?.to_string();
            }
            Err(e) => {
              return Err(anyhow!(
                "failed to get inscribe transfer inscription for {}! error: {}",
                operation.inscription_id, e
              ))
            }
            _ => {}
          },
          _ => {},
        };

        let mut inscription_content: String = "".to_string();
        let des_res = deserialize_inscription(new_inscriptions
            .get(usize::try_from(operation.inscription_id.index).unwrap())
            .unwrap());
        match des_res {
          Ok(content) => {
            inscription_content = content;
          },
          Err(err) => {
            return Err(err);
          },
        }

        messages.push(BrcZeroMsg{
          btc_fee,
          msg: MsgInscription {
            inscription: inscription_content,
            inscription_context: InscriptionContext {
              txid: operation.txid.to_string(),
              inscription_id: operation.inscription_id.to_string(),
              inscription_number: utils::get_inscription_number_by_id(operation.inscription_id, self.ord_store)?,
              old_sat_point: operation.old_satpoint.to_string(),
              new_sat_point: operation.new_satpoint.unwrap().to_string(),
              sender,
              receiver: if sat_in_outputs {
                utils::get_script_key_on_satpoint(
                  operation.new_satpoint.unwrap(),
                  self.ord_store,
                  context.network,
                )?.to_string()
              } else {
                "".to_string()
              },
              is_transfer,
              block_height: context.blockheight,
              block_time: context.blocktime,
              block_hash: context.blockhash.to_string(),
            },
          }
        });
      }
    }
    self.update_outpoint_to_txout(outpoint_to_txout_cache)?;
    Ok(messages)
  }
}

pub(crate) fn deserialize_inscription(
  inscription: &Inscription,
) -> Result<String> {
  let content_body = std::str::from_utf8(inscription.body().ok_or(JSONError::InvalidJson)?)?;
  if content_body.len() == 0 {
    return Err(JSONError::InvalidJson.into());
  }

  let content_type = inscription
      .content_type()
      .ok_or(JSONError::InvalidContentType)?;

  if content_type != "text/plain"
      && content_type != "text/plain;charset=utf-8"
      && content_type != "text/plain;charset=UTF-8"
      && content_type != "application/json"
      && !content_type.starts_with("text/plain;")
  {
    return Err(JSONError::UnSupportContentType.into());
  }

  let value: Value = serde_json::from_str(content_body).map_err(|_| JSONError::InvalidJson)?;
  if value.get("p") == None || !value["p"].is_string(){
    return Err(JSONError::InvalidJson.into());
  }

  return Ok(serde_json::to_string(&value).unwrap())
}


fn get_commit_input_satpoint<O: OrdDataStoreReadOnly>(
  client: &Client,
  ord_store: &O,
  satpoint: SatPoint,
  outpoint_to_txout_cache: &mut HashMap<OutPoint, TxOut>,
) -> Result<SatPoint> {
  let commit_transaction =
      &Index::get_transaction_retries(client, satpoint.outpoint.txid)?.ok_or(anyhow!(
      "failed to BRC20S message commit transaction! error: {} not found",
      satpoint.outpoint.txid
    ))?;

  // get satoshi offset
  let mut offset = 0;
  for (vout, output) in commit_transaction.output.iter().enumerate() {
    if vout < usize::try_from(satpoint.outpoint.vout).unwrap() {
      offset += output.value;
      continue;
    }
    offset += satpoint.offset;
    break;
  }

  let mut input_value = 0;
  for input in &commit_transaction.input {
    let value = if let Some(tx_out) = ord_store
        .get_outpoint_to_txout(input.previous_output)
        .map_err(|e| anyhow!("failed to get tx out from state! error: {e}"))?
    {
      tx_out.value
    } else if let Some(tx_out) = Index::get_transaction_retries(client, input.previous_output.txid)?
        .map(|tx| {
          tx.output
              .get(usize::try_from(input.previous_output.vout).unwrap())
              .unwrap()
              .clone()
        })
    {
      outpoint_to_txout_cache.insert(input.previous_output, tx_out.clone());
      tx_out.value
    } else {
      return Err(anyhow!(
        "failed to get tx out! error: {} not found",
        input.previous_output
      ));
    };

    input_value += value;
    if input_value >= offset {
      return Ok(SatPoint {
        outpoint: input.previous_output,
        offset: value - input_value + offset,
      });
    }
  }
  Err(anyhow!("no match found for the commit offset!"))
}