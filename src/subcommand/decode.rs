use super::*;

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct Output {
  pub inscriptions: Vec<Inscription>,
}

#[derive(Debug, Parser)]
pub(crate) struct Decode {
  transaction: Option<PathBuf>,
}
