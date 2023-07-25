use bitcoin::Network;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::string::ToString;

/**
 * enable share pool
 * current is disable
 */
pub const VERSION_KEY_ENABLE_SHARE: &str = "enable_share";

/**
 * update staked pool number limit
 */
pub const VERSION_KEY_STAKED_POOL_NUM_LIMIT_V1: &str = "staked_pool_num_limit_v1";

lazy_static! {

  static ref MAIN_NET_VERSION: HashMap<String, Version> = {
    let mut version: HashMap<String, Version> = HashMap::new();
    version.insert(
      VERSION_KEY_STAKED_POOL_NUM_LIMIT_V1.to_string(),
      Version {
        name: VERSION_KEY_STAKED_POOL_NUM_LIMIT_V1.to_string(),
        start_height: 800310,
      },
    );
    version
  };

  static ref TEST_NET_VERSION: HashMap<String, Version> = {
    HashMap::new()
  };

  static ref SIG_NET_VERSION: HashMap<String, Version> = {
    HashMap::new()
  };

  static ref REGTEST_NET_VERSION: HashMap<String, Version> = {
    let mut version: HashMap<String, Version> = HashMap::new();
    version.insert(
      VERSION_KEY_STAKED_POOL_NUM_LIMIT_V1.to_string(),
      Version {
        name: VERSION_KEY_STAKED_POOL_NUM_LIMIT_V1.to_string(),
        start_height: 2100,
      },
    );
    version
  };

  pub static ref UNIT_TEST_VERSION: HashMap<String, Version> = {
    let mut version: HashMap<String, Version> = HashMap::new();

    //enable share pool
    version.insert(
      VERSION_KEY_ENABLE_SHARE.to_string(),
      Version {
        name: VERSION_KEY_ENABLE_SHARE.to_string(),
        start_height: 0,
      },
    );

    version.insert(
      VERSION_KEY_STAKED_POOL_NUM_LIMIT_V1.to_string(),
      Version {
        name: VERSION_KEY_STAKED_POOL_NUM_LIMIT_V1.to_string(),
        start_height: 20,
      },
    );

    version
  };
}

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

pub fn get_version_by_network(network: Network) -> HashMap<String, Version> {
  match network {
    Network::Bitcoin => MAIN_NET_VERSION.clone(),
    Network::Testnet => TEST_NET_VERSION.clone(),
    Network::Signet => SIG_NET_VERSION.clone(),
    Network::Regtest => REGTEST_NET_VERSION.clone(),
  }
}
