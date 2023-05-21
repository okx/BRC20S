use super::*;

use shadow_rs::shadow;
shadow!(build);
#[derive(Debug, Parser)]
#[clap(version(build::CLAP_LONG_VERSION))]
pub(crate) struct Arguments {
  #[clap(flatten)]
  pub(crate) options: Options,
  #[clap(subcommand)]
  pub(crate) subcommand: Subcommand,
}

impl Arguments {
  pub(crate) fn run(self) -> Result {
    self.subcommand.run(self.options)
  }
}
