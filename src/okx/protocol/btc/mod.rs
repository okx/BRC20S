use {
  crate::{
    okx::{
      datastore::{
        btc::{Balance, DataStoreReadOnly, DataStoreReadWrite, Event, Receipt, TransferEvent},
        ord::DataStoreReadOnly as OrdDataStoreReadOnly,
        ScriptKey, StateRWriter,
      },
      protocol::BlockContext,
    },
    Result,
  },
  anyhow::anyhow,
  bitcoin::{Transaction, Txid},
  std::collections::HashMap,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
  pub txid: Txid,
  pub from: ScriptKey,
  pub amt: u128,
}

pub fn gen_receipt(msg: &Message) -> Result<Option<Receipt>> {
  let event = Event::Transfer(TransferEvent {
    amt: msg.amt,
    msg: None,
  });

  let receipt = Receipt {
    from: msg.from.clone(),
    result: Ok(event),
  };

  log::debug!("BTC message receipt: {:?}", receipt);
  Ok(Some(receipt))
}

/// index transaction and update balance.
pub fn index_transaction_balance<RW: StateRWriter>(
  context: BlockContext,
  state_store: &RW,
  tx: &Transaction,
) -> Result<Vec<Message>> {
  // update address btc balance by input
  let mut balance_change_map: HashMap<ScriptKey, i64> = HashMap::new();
  for tx_in in &tx.input {
    if tx_in.previous_output.is_null() {
      // ingore coinbase input.
      continue;
    }

    let prev_output = state_store
      .ord()
      .get_outpoint_to_txout(tx_in.previous_output)
      .map_err(|e| anyhow!("failed to get tx out from state! error: {e}"))?
      .unwrap();

    *balance_change_map
      .entry(ScriptKey::from_script(
        &prev_output.script_pubkey,
        context.network,
      ))
      .or_insert(0) -= i64::try_from(prev_output.value).unwrap();
  }

  // update address btc balance by output
  for tx_out in &tx.output {
    *balance_change_map
      .entry(ScriptKey::from_script(
        &tx_out.script_pubkey,
        context.network,
      ))
      .or_insert(0) += i64::try_from(tx_out.value).unwrap();
  }

  let mut messages = Vec::new();
  // Passive withdrawal is triggered by the amount of balance change,
  // and <sk, balance> is saved to redb.
  for (sk, diff) in balance_change_map.into_iter() {
    let mut btc_balance = state_store
      .btc()
      .get_balance(&sk)
      .map_err(|e| anyhow!("failed to get balance from state! error: {e}"))?
      .map_or(Balance::new(), |v| v);

    if diff < 0i64 {
      // BTC transfer to BRC20S passive withdrawal
      messages.push(Message {
        txid: tx.txid(),
        from: sk.clone(),
        amt: u128::try_from(diff.abs()).unwrap(),
      });
    }

    btc_balance.balance = btc_balance.balance.checked_add_signed(diff).ok_or(anyhow!(
      "balance overflow! {} {} {}",
      btc_balance.balance,
      if diff >= 0 { "+" } else { "-" },
      diff
    ))?;

    // store to database.

    state_store
      .btc()
      .update_balance(&sk, btc_balance)
      .map_err(|e| anyhow!("failed to update balance to state! error: {e}"))?;
  }
  Ok(messages)
}
