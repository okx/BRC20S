#![allow(clippy::type_complexity)]

use {
  self::{command_builder::CommandBuilder, expected::Expected, test_server::TestServer},
  executable_path::executable_path,
  ord::{inscription_id::InscriptionId, rarity::Rarity, templates::sat::SatJson},
  pretty_assertions::assert_eq as pretty_assert_eq,
  regex::Regex,
  reqwest::{StatusCode, Url},
  serde::de::DeserializeOwned,
  std::{
    fs,
    io::Write,
    net::TcpListener,
    path::Path,
    process::{Child, Command, Stdio},
    str, thread,
    time::Duration,
  },
  tempfile::TempDir,
  test_bitcoincore_rpc::TransactionTemplate,
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
type Inscribe = ord::subcommand::wallet::inscribe::Output;

#[allow(unused)]
fn inscribe(rpc_server: &test_bitcoincore_rpc::Handle) -> Inscribe {
  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 foo.txt")
    .write("foo.txt", "FOO")
    .rpc_server(rpc_server)
    .run_and_deserialize_output();

  rpc_server.mine_blocks(1);

  output
}

#[allow(unused)]
fn envelope(payload: &[&[u8]]) -> bitcoin::Witness {
  let mut builder = bitcoin::script::Builder::new()
    .push_opcode(bitcoin::opcodes::OP_FALSE)
    .push_opcode(bitcoin::opcodes::all::OP_IF);

  for data in payload {
    let mut buf = bitcoin::script::PushBytesBuf::new();
    buf.extend_from_slice(data).unwrap();
    builder = builder.push_slice(buf);
  }

  let script = builder
    .push_opcode(bitcoin::opcodes::all::OP_ENDIF)
    .into_script();

  bitcoin::Witness::from_slice(&[script.into_bytes(), Vec::new()])
}

mod command_builder;
mod expected;
mod test_server;

mod index;
mod json_api;
mod server;
mod version;
