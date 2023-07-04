#![allow(clippy::type_complexity)]

use {
  self::{command_builder::CommandBuilder, expected::Expected},
  bip39::Mnemonic,
  bitcoin::Txid,
  executable_path::executable_path,
  pretty_assertions::assert_eq as pretty_assert_eq,
  regex::Regex,
  reqwest::{StatusCode, Url},
  serde::{de::DeserializeOwned, Deserialize},
  std::{
    fs,
    net::TcpListener,
    path::Path,
    process::{Child, Command, Stdio},
    str::{self},
    thread,
    time::Duration,
  },
  tempfile::TempDir,
};

macro_rules! assert_regex_match {
  ($string:expr, $pattern:expr $(,)?) => {
    let regex = Regex::new(&format!("^(?s){}$", $pattern)).unwrap();
    let string = $string;

    if !regex.is_match(string.as_ref()) {
      panic!(
        "Regex:\n\n{}\n\n…did not match string:\n\n{}",
        regex, string
      );
    }
  };
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct Inscribe {
  commit: Txid,
  inscription: String,
  reveal: Txid,
  fees: u64,
}

#[allow(unused)]
fn inscribe(rpc_server: &test_bitcoincore_rpc::Handle) -> Inscribe {
  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 foo.txt")
    .write("foo.txt", "FOO")
    .rpc_server(rpc_server)
    .run_and_check_output();

  rpc_server.mine_blocks(1);

  output
}

#[allow(unused)]
#[derive(Deserialize)]
struct Create {
  mnemonic: Mnemonic,
}

#[allow(unused)]
fn create_wallet(rpc_server: &test_bitcoincore_rpc::Handle) {
  CommandBuilder::new(format!("--chain {} wallet create", rpc_server.network()))
    .rpc_server(rpc_server)
    .run_and_check_output::<Create>();
}

mod command_builder;
mod core;
mod expected;
mod index;
mod server;
mod test_server;
mod version;
