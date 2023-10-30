use super::*;

#[test]
fn get_sat_without_sat_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let response = TestServer::spawn_with_args(&rpc_server, &["--enable-json-api"])
    .json_request("/sat/2099999997689999");

  assert_eq!(response.status(), StatusCode::OK);

  let mut sat_json: SatJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  // this is a hack to ignore the timestamp, since it changes for every request
  sat_json.timestamp = 0;

  pretty_assert_eq!(
    sat_json,
    SatJson {
      number: 2099999997689999,
      decimal: "6929999.0".into(),
      degree: "5°209999′1007″0‴".into(),
      name: "a".into(),
      block: 6929999,
      cycle: 5,
      epoch: 32,
      period: 3437,
      offset: 0,
      rarity: Rarity::Uncommon,
      percentile: "100%".into(),
      satpoint: None,
      timestamp: 0,
      inscriptions: vec![],
    }
  )
}

#[allow(unused)]
fn create_210_inscriptions(
  rpc_server: &test_bitcoincore_rpc::Handle,
) -> (Vec<InscriptionId>, Vec<InscriptionId>) {
  let witness = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let mut blessed_inscriptions = Vec::new();
  let mut cursed_inscriptions = Vec::new();

  // Create 150 inscriptions, 50 non-cursed and 100 cursed
  for i in 0..50 {
    rpc_server.mine_blocks(1);
    rpc_server.mine_blocks(1);
    rpc_server.mine_blocks(1);

    let txid = rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[
        (i * 3 + 1, 0, 0, witness.clone()),
        (i * 3 + 2, 0, 0, witness.clone()),
        (i * 3 + 3, 0, 0, witness.clone()),
      ],
      ..Default::default()
    });

    blessed_inscriptions.push(InscriptionId { txid, index: 0 });
    cursed_inscriptions.push(InscriptionId { txid, index: 1 });
    cursed_inscriptions.push(InscriptionId { txid, index: 2 });
  }

  rpc_server.mine_blocks(1);

  // Create another 60 non cursed
  for _ in 0..60 {
    let Inscribe { reveal, .. } = CommandBuilder::new("wallet inscribe --fee-rate 1 foo.txt")
      .write("foo.txt", "FOO")
      .rpc_server(rpc_server)
      .run_and_deserialize_output();
    rpc_server.mine_blocks(1);
    blessed_inscriptions.push(InscriptionId {
      txid: reveal,
      index: 0,
    });
  }

  rpc_server.mine_blocks(1);

  (blessed_inscriptions, cursed_inscriptions)
}

#[test]
fn json_request_fails_when_not_enabled() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let response =
    TestServer::spawn_with_args(&rpc_server, &[]).json_request("/sat/2099999997689999");

  assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
}
