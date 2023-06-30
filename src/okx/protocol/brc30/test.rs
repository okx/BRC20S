pub(crate) use {
  super::*, crate::inscription_id::InscriptionId, crate::okx::datastore::ScriptKey,
  crate::SatPoint, bitcoin::hashes::hex::ToHex, bitcoin::Address, shadow_rs::new,
  std::str::FromStr,
};
pub(crate) fn mock_create_brc30_message(
  from: ScriptKey,
  to: ScriptKey,
  op: BRC30Operation,
) -> BRC30ExecutionMessage {
  let inscription_id =
    InscriptionId::from_str("1111111111111111111111111111111111111111111111111111111111111111i1")
      .unwrap();
  let txid = inscription_id.txid.clone();
  let old_satpoint =
    SatPoint::from_str("1111111111111111111111111111111111111111111111111111111111111111:1:1")
      .unwrap();
  let new_satpoint =
    SatPoint::from_str("1111111111111111111111111111111111111111111111111111111111111111:2:1")
      .unwrap();
  let msg = BRC30ExecutionMessage {
    txid,
    inscription_id,
    inscription_number: 0,
    commit_input_satpoint: None,
    old_satpoint,
    new_satpoint,
    commit_from: Some(from.clone()),
    from: from.clone(),
    to: to.clone(),
    op,
  };
  msg
}

pub(crate) fn mock_deploy_msg(
  pool_type: &str,
  poll_number: &str, //must be hex len == 2, 00 ~ ff: like 01
  stake: &str,
  earn: &str,
  earn_rate: &str,
  dmax: &str,
  supply: &str,
  dec: u8,
  only: bool,
  from: &str,
  to: &str,
) -> (Deploy, BRC30ExecutionMessage) {
  let only = if only { Some("1".to_string()) } else { None };

  let supply_128 = Num::from_str(supply).unwrap().checked_to_u128().unwrap();

  let from_script_key = ScriptKey::from_address(Address::from_str(from).unwrap());
  let to_script_key = ScriptKey::from_address(Address::from_str(to).unwrap());

  let tickid = hash::caculate_tick_id(earn, supply_128, dec, &from_script_key, &to_script_key);
  let pid = tickid.hex().to_string() + "#" + poll_number;
  let msg = Deploy {
    pool_type: pool_type.to_string(),
    pool_id: pid,
    stake: stake.to_string(),
    earn: earn.to_string(),
    earn_rate: earn_rate.to_string(),
    distribution_max: dmax.to_string(),
    total_supply: Some(supply.to_string()),
    decimals: Some(dec.to_string()),
    only,
  };

  let execute_msg = mock_create_brc30_message(
    from_script_key,
    to_script_key,
    BRC30Operation::Deploy(msg.clone()),
  );
  (msg, execute_msg)
}
