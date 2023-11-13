use crate::okx::datastore::brc20s::PledgedTick;
use bitcoin::Network;

#[derive(Debug, Clone)]
pub struct Config {
  pub allow_share_pool: bool,
  pub allow_btc_staking: bool,
  pub allow_brc20_staking: bool,
  pub allow_brc20s_staking: bool,
  pub max_staked_pool_num: u64,
}

// start at block 798108
pub const fn zebra() -> Config {
  Config {
    allow_share_pool: true,
    allow_btc_staking: false,
    allow_brc20_staking: true,
    allow_brc20s_staking: false,
    max_staked_pool_num: 5,
  }
}
// start at block 800310
pub const fn koala() -> Config {
  Config {
    allow_share_pool: true,
    allow_btc_staking: false,
    allow_brc20_staking: true,
    allow_brc20s_staking: false,
    max_staked_pool_num: 128,
  }
}

pub fn get_config_by_network(network: Network, blockheight: u64) -> Config {
  match network {
    Network::Bitcoin => match blockheight {
      n if n >= 800310 => koala(),
      _ => zebra(),
    },
    Network::Testnet => match blockheight {
      n if n >= 2468142 => koala(),
      _ => zebra(),
    },
    Network::Signet => match blockheight {
      n if n >= 153382 => koala(),
      _ => zebra(),
    },
    Network::Regtest => koala(),
    _ => panic!("not support network"),
  }
}

pub fn tick_can_staked(token: &PledgedTick, config: &Config) -> bool {
  match token {
    PledgedTick::Native => config.allow_btc_staking,
    PledgedTick::BRC20STick(_) => config.allow_brc20s_staking,
    PledgedTick::BRC20Tick(_) => config.allow_brc20_staking,
    PledgedTick::Unknown => false,
  }
}
