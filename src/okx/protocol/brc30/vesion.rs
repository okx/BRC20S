use std::collections::HashMap;
use std::string::ToString;

pub const VERSION_KEY_ENABLE_SHARE: &str = "enable_share";

#[derive(Debug, Clone, PartialEq)]
pub struct Version {
  pub name: String,
  pub start_height: u64,
}

pub fn enable_version_by_key(
  versions: &HashMap<String, Version>,
  key: &str,
  current_height: u64,
) -> bool {
  let key = key.to_string();
  match versions.get(&key) {
    None => false,
    Some(v) => current_height >= v.start_height,
  }
}
