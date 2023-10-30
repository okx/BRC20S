use super::*;

use shadow_rs::shadow;
shadow!(build);
#[derive(Debug, Parser)]
#[command(version(build::CLAP_LONG_VERSION))]
pub(crate) struct Arguments {
  #[command(flatten)]
  pub(crate) options: Options,
  #[command(subcommand)]
  pub(crate) subcommand: Subcommand,
}

impl Arguments {
  pub(crate) fn run(self) -> SubcommandResult {
    self.subcommand.run(self.options)
  }
}
