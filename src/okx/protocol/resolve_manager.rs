use bitcoin::BlockHash;
use {
  super::*,
  crate::{
    okx::{
      datastore::{
        ord::{operation::InscriptionOp, DataStoreReadWrite,DataStoreReadOnly,Action},
        StateRWriter,
      },
      protocol::Message,
    },
    Inscription, Result,InscriptionId,Index,sat_point::SatPoint
  },
  anyhow::anyhow,
  bitcoin::{OutPoint, Transaction, TxOut},
  bitcoincore_rpc::Client,
  std::collections::HashMap,
  serde_json::{Value},
};

pub struct MsgResolveManager<'a, RW: StateRWriter> {
  client: &'a Client,
  state_store: &'a RW,
  config: &'a ProtocolConfig,
}

impl<'a, RW: StateRWriter> MsgResolveManager<'a, RW> {
  pub fn new(client: &'a Client, state_store: &'a RW, config: &'a ProtocolConfig) -> Self {
    Self {
      client,
      state_store,
      config,
    }
  }

  pub fn resolve_message(
    &self,
    context: BlockContext,
    tx: &Transaction,
    operations: &[InscriptionOp],
  ) -> Result<Vec<Message>> {
    log::debug!(
      "Resolve Manager indexed transaction {}, operations size: {}, data: {:?}",
      tx.txid(),
      operations.len(),
      operations
    );
    let mut messages = Vec::new();
    let mut operation_iter = operations.iter().peekable();
    let new_inscriptions = Inscription::from_transaction(tx)
      .into_iter()
      .map(|v| v.inscription)
      .collect::<Vec<Inscription>>();

    let mut outpoint_to_txout_cache: HashMap<OutPoint, TxOut> = HashMap::new();
    for input in &tx.input {
      // "operations" is a list of all the operations in the current block, and they are ordered.
      // We just need to find the operation corresponding to the current transaction here.
      while let Some(operation) = operation_iter.peek() {
        if operation.old_satpoint.outpoint != input.previous_output {
          break;
        }
        let operation = operation_iter.next().unwrap();

        // Parse BRC20 message through inscription operation.
        if self
          .config
          .first_brc20_height
          .map(|height| context.blockheight >= height)
          .unwrap_or(false)
        {
          if let Some(msg) =
            brc20::Message::resolve(self.state_store.brc20(), &new_inscriptions, operation)?
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

        // Parse BRC20S message through inscription operation.
        if self
          .config
          .first_brc20s_height
          .map(|height| context.blockheight >= height)
          .unwrap_or(false)
        {
          if let Some(msg) = brc20s::Message::resolve(
            self.client,
            self.state_store.ord(),
            self.state_store.brc20s(),
            &new_inscriptions,
            operation,
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
      }
    }
    self.update_outpoint_to_txout(outpoint_to_txout_cache)?;
    Ok(messages)
  }

  fn update_outpoint_to_txout(&self, outpoint_to_txout_cache: HashMap<OutPoint, TxOut>) -> Result {
    for (outpoint, txout) in outpoint_to_txout_cache {
      self
        .state_store
        .ord()
        .set_outpoint_to_txout(outpoint, &txout)
        .or(Err(anyhow!(
          "failed to get tx out! error: {} not found",
          outpoint
        )))?;
    }
    Ok(())
  }

  pub fn resolve_brczero_inscription(
    &self,
    context: BlockContext,
    tx: &Transaction,
    operations: Vec<InscriptionOp>,
    blockHash: &BlockHash,
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
        let mut inscription_content: String = "".to_string();
        match operation.action {
          // New inscription is not `cursed` or `unbound`.
          Action::New {
            cursed: false,
            unbound: false, ..
          } => {
            let inscription = new_inscriptions.get(usize::try_from(operation.inscription_id.index).unwrap()).unwrap().clone();
            self.state_store.ord().save_inscription_with_id(&operation.inscription_id,&inscription).map_err(|e| {
              anyhow!("failed to set inscription with id in ordinals operations to state! error: {e}")
            })?;
            let des_res = deserialize_inscription(&inscription);
            match des_res {
              Ok(content) => {
                let commit_input_satpoint = get_commit_input_satpoint(
                  self.client,
                  self.state_store.ord(),
                  operation.old_satpoint,
                  &mut outpoint_to_txout_cache,
                )?;
                sender = utils::get_script_key_on_satpoint(commit_input_satpoint, self.state_store.ord(), context.network)?.to_string();
                inscription_content = content;
              },
              Err(err) => {
                continue;
              },
            }
          },
          // Transfer inscription operation.
          Action::Transfer => {
            is_transfer = true;
            let inscription = match get_inscription_by_id(self.client,self.state_store.ord(), &operation.inscription_id) {
              Ok(innnet_inscription) => {innnet_inscription}
              Err(err) => {continue}
            };
            self.state_store.ord().remove_inscription_with_id(&operation.inscription_id).map_err(|e| {
              anyhow!("failed to remove inscription with id in ordinals operations to state! error: {e}")
            })?;
            let des_res = deserialize_inscription(&inscription);
            match des_res {
              Ok(content) => {
                sender = utils::get_script_key_on_satpoint(operation.old_satpoint, self.state_store.ord(), context.network)?.to_string();
                inscription_content = content;
              },
              Err(err) => {
                continue;
              },
            }
          },
          _ => {
            continue;},
        };
        let btc_fee = self.get_btc_transaction_fee(tx);
        messages.push(BrcZeroMsg{
          btc_fee,
          msg: MsgInscription {
            inscription: inscription_content,
            inscription_context: InscriptionContext {
              txid: operation.txid.to_string(),
              inscription_id: operation.inscription_id.to_string(),
              inscription_number: utils::get_inscription_number_by_id(operation.inscription_id, self.state_store.ord())?,
              old_sat_point: operation.old_satpoint.to_string(),
              new_sat_point: operation.new_satpoint.unwrap().to_string(),
              sender,
              receiver: if sat_in_outputs {
                utils::get_script_key_on_satpoint(
                  operation.new_satpoint.unwrap(),
                  self.state_store.ord(),
                  context.network,
                )?.to_string()
              } else {
                "".to_string()
              },
              is_transfer,
              block_height: context.blockheight,
              block_time: context.blocktime,
              block_hash: blockHash.to_string(),
            },
          }
        });
      }
    }
    self.update_outpoint_to_txout(outpoint_to_txout_cache)?;
    Ok(messages)
  }

  fn get_btc_transaction_fee(&self, tx: &Transaction) -> u128 {
    let mut input_value = 0_u128;
    let mut output_value = 0_u128;
    for input in &tx.input {
      let value = self.state_store.ord().get_outpoint_to_txout(input.previous_output);
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


fn get_commit_input_satpoint<O: DataStoreReadWrite>(
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
      ord_store.set_outpoint_to_txout(input.previous_output.clone(), &tx_out.clone())
          .or(Err(anyhow!(
          "failed to get tx out! error: {} not found",
          input.previous_output
        )))?;
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

fn get_inscription_by_id<O: DataStoreReadOnly>(
  client: &Client,
  ord_store: &O,
  inscription_id: &InscriptionId,
) -> Result<Inscription> {

  let inscription = if let Some(inscription) = ord_store.get_inscription_by_id(inscription_id).map_err(|e| {
    anyhow!("failed to get inscription by id ! error: {e}")
  })? {
    inscription
  } else {
    return Err(anyhow!(
        "failed to get tx inscription! error: {} not found",
        inscription_id
      ));
  };
  Ok(inscription)
}
