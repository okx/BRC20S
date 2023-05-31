use super::*;

mod index;
mod server;

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
