use super::*;

mod index;
mod server;

fn print_json(output: impl Serialize) -> Result {
  serde_json::to_writer_pretty(io::stdout(), &output)?;
  println!();
  Ok(())
}

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  #[clap(about = "Update the index")]
  Index,
  #[clap(about = "Run the explorer server")]
  Server(server::Server),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Index => index::run(options),
      Self::Server(server) => {
        let index = Arc::new(Index::open(&options)?);
        let handle = axum_server::Handle::new();
        LISTENERS.lock().unwrap().push(handle.clone());
        server.run(options, index, handle)
      }
    }
  }
}
