use {super::*, crate::command_builder::ToArgs};

#[test]
fn run() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let port = TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port();

  let builder = CommandBuilder::new(format!("server --address 127.0.0.1 --http-port {port}"))
    .rpc_server(&rpc_server);

  let mut command = builder.command();

  let mut child = command.spawn().unwrap();

  for attempt in 0.. {
    if let Ok(response) = reqwest::blocking::get(format!("http://localhost:{port}/status")) {
      if response.status() == 200 {
        assert_eq!(response.text().unwrap(), "OK");
        break;
      }
    }

    if attempt == 100 {
      panic!("Server did not respond to status check",);
    }

    thread::sleep(Duration::from_millis(50));
  }

  child.kill().unwrap();
}

#[test]
fn expected_sat_time_is_rounded() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    "/sat/2099999997689999",
    r".*<dt>timestamp</dt><dd><time>.* \d+:\d+:\d+ UTC</time> \(expected\)</dd>.*",
  );
}

#[test]
fn server_runs_with_rpc_user_and_pass_as_env_vars() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  let tempdir = TempDir::new().unwrap();
  let port = TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port();

  let mut child = Command::new(executable_path("ord"))
    .args(format!(
      "--rpc-url {} --bitcoin-data-dir {} --data-dir {} server --http-port {port} --address 127.0.0.1",
      rpc_server.url(),
      tempdir.path().display(),
      tempdir.path().display()).to_args()
      )
      .env("ORD_BITCOIN_RPC_PASS", "bar")
      .env("ORD_BITCOIN_RPC_USER", "foo")
      .env("ORD_INTEGRATION_TEST", "1")
      .current_dir(&tempdir)
      .spawn().unwrap();

  for i in 0.. {
    match reqwest::blocking::get(format!("http://127.0.0.1:{port}/status")) {
      Ok(_) => break,
      Err(err) => {
        if i == 400 {
          panic!("Server failed to start: {err}");
        }
      }
    }

    thread::sleep(Duration::from_millis(25));
  }

  rpc_server.mine_blocks(1);
  thread::sleep(Duration::from_secs(1));

  let response = reqwest::blocking::get(format!("http://127.0.0.1:{port}/blockcount")).unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(response.text().unwrap(), "2");

  child.kill().unwrap();
}

#[test]
fn missing_credentials() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new("--bitcoin-rpc-user foo server")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: no bitcoind rpc password specified\n")
    .run_and_extract_stdout();

  CommandBuilder::new("--bitcoin-rpc-pass bar server")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: no bitcoind rpc user specified\n")
    .run_and_extract_stdout();
}
