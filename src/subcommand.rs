use super::*;

pub mod decode;
mod index;
mod server;
pub mod wallet;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  #[command(subcommand, about = "Index commands")]
  Index(index::IndexSubcommand),
  #[command(about = "Run the explorer server")]
  Server(server::Server),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    match self {
      Self::Index(index) => index.run(options),
      Self::Server(server) => {
        let index = Arc::new(Index::open(&options)?);
        let handle = axum_server::Handle::new();
        LISTENERS.lock().unwrap().push(handle.clone());
        server.run(options, index, handle)
      }
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct Empty {}

pub(crate) trait Output: Send {
  fn print_json(&self);
}

impl<T> Output for T
where
  T: Serialize + Send,
{
  fn print_json(&self) {
    serde_json::to_writer_pretty(io::stdout(), self).ok();
    println!();
  }
}

pub(crate) type SubcommandResult = Result<Box<dyn Output>>;
