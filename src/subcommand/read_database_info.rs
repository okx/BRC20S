use super::*;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;

  index.read_database_info();

  Ok(())
}
