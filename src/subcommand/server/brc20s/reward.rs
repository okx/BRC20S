use {
  super::*,
  crate::{
    okx::datastore::{
      brc20,
      brc20s::{self, Pid, PledgedTick},
    },
    subcommand::server::brc20::BRC20Error,
  },
  axum::Json,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
  pub pid: String,
  pub staked: String,
  pub minted: String,
  pub pending_reward: String,
  pub reward_debt: String,
  pub latest_update_block: u64,
}

impl From<&brc20s::UserInfo> for UserInfo {
  fn from(user_info: &brc20s::UserInfo) -> Self {
    Self {
      pid: user_info.pid.as_str().to_string(),
      staked: user_info.staked.to_string(),
      minted: user_info.minted.to_string(),
      pending_reward: user_info.pending_reward.to_string(),
      reward_debt: user_info.reward_debt.to_string(),
      latest_update_block: user_info.latest_updated_block,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserReward {
  #[serde(rename = "pending_reward")]
  pub pending_reward: String,
  #[serde(rename = "block_num")]
  pub block_num: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakedInfo {
  #[serde(rename = "type")]
  pub type_field: String,
  pub tick: String,
  #[serde(rename = "max_share")]
  pub max_share: String,
  #[serde(rename = "total_only")]
  pub total_only: String,
  #[serde(rename = "staked_pids")]
  pub staked_pids: Vec<StakedPid>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakedPid {
  pub pid: String,
  pub only: bool,
  pub stake: String,
}

impl From<&brc20s::StakeInfo> for StakedInfo {
  fn from(stake: &brc20s::StakeInfo) -> Self {
    Self {
      type_field: "BRC20".to_string(),
      tick: "".to_string(),
      max_share: stake.max_share.to_string(),
      total_only: stake.total_only.to_string(),
      staked_pids: stake
        .pool_stakes
        .iter()
        .rev()
        .map(|(a, b, c)| StakedPid {
          pid: a.as_str().to_string(),
          only: *b,
          stake: c.to_string(),
        })
        .collect(),
    }
  }
}
pub(crate) async fn brc20s_user_pending_reward(
  Extension(index): Extension<Arc<Index>>,
  Path((pid, address)): Path<(String, String)>,
) -> ApiResult<UserReward> {
  log::debug!("rpc: get brc20s_user_pending_reward: {}, {}", pid, address);

  let pid =
    Pid::from_str(&pid).map_err(|_| ApiError::bad_request(BRC20SError::IncorrectPidFormat))?;
  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;
  let (user_reward, block) = &index.brc20s_user_pending_reward(&pid, &address)?;

  log::debug!(
    "rpc: get brc20s_user_pending_reward: {:?}, {:?}, {:?}",
    pid.as_str(),
    user_reward,
    block,
  );

  Ok(Json(ApiResponse::ok(UserReward {
    pending_reward: user_reward.clone().unwrap(),
    block_num: block.clone().unwrap(),
  })))
}

// brc20s/pool/:pid/address/:address/userinfo
pub(crate) async fn brc20s_userinfo(
  Extension(index): Extension<Arc<Index>>,
  Path((pid, address)): Path<(String, String)>,
) -> ApiResult<UserInfo> {
  log::debug!("rpc: get brc20s_userinfo: {}, {}", pid, address);

  let pid =
    Pid::from_str(&pid).map_err(|_| ApiError::bad_request(BRC20SError::IncorrectPidFormat))?;

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let user_info = &index
    .brc20s_user_info(&pid, &address)?
    .ok_or_api_not_found(BRC20SError::UserInfoNotFound)?;

  log::debug!(
    "rpc: get brc20s_userinfo: {:?} {:?}",
    pid.as_str(),
    user_info
  );

  if user_info.pid != pid {
    return Err(ApiError::internal("db: not match"));
  }

  Ok(Json(ApiResponse::ok(user_info.into())))
}

// brc20s/debug/pool/:pid/address/:address/userinfo
pub(crate) async fn brc20s_debug_userinfo(
  Extension(index): Extension<Arc<Index>>,
  Path((pid, address)): Path<(String, String)>,
) -> ApiResult<brc20s::UserInfo> {
  log::debug!("rpc: get brc20s_debug_userinfo: {}, {}", pid, address);

  let pid =
    Pid::from_str(&pid).map_err(|_| ApiError::bad_request(BRC20SError::IncorrectPidFormat))?;
  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;
  let user_info = index
    .brc20s_user_info(&pid, &address)?
    .ok_or_api_not_found(BRC20SError::UserInfoNotFound)?;

  log::debug!(
    "rpc: get brc20s_debug_userinfo: {:?} {:?}",
    pid.as_str(),
    user_info
  );

  if user_info.pid != pid {
    return Err(ApiError::internal("db: not match"));
  }

  Ok(Json(ApiResponse::ok(user_info)))
}

// brc20s/stake/:address/:tick?tick_type=0
pub(crate) async fn brc20s_stake_info(
  Extension(index): Extension<Arc<Index>>,
  Path((address, tick)): Path<(String, String)>,
) -> ApiResult<StakedInfo> {
  log::debug!(
    "rpc: get brc20s_stake_info: tick:{}, address:{}",
    tick,
    address
  );

  let tick = brc20::Tick::from_str(&tick)
    .map_err(|_| ApiError::bad_request(BRC20Error::IncorrectTickFormat))?;

  let tick = index
    .brc20_get_tick_info(&tick)?
    .ok_or_api_not_found(BRC20Error::TickNotFound)?
    .tick;

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let stake_info = index
    .brc20s_stake_info(&address, &PledgedTick::BRC20Tick(tick.clone()))?
    .ok_or_api_not_found(BRC20SError::StakeInfoNotFound)?;

  log::debug!("rpc: get brc20s_stake_info: {:?}", stake_info);

  let mut result = StakedInfo::from(&stake_info);
  result.tick = tick.to_string();

  Ok(Json(ApiResponse::ok(result)))
}

pub(crate) async fn brc20s_debug_stake_info(
  Extension(index): Extension<Arc<Index>>,
  Path((address, tick)): Path<(String, String)>,
) -> ApiResult<brc20s::StakeInfo> {
  log::debug!("rpc: get brc20s_debug_stake_info: {},{}", address, tick);

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let stake_info = index
    .brc20s_stake_info(&address, &PledgedTick::from_str(tick.as_str()))?
    .ok_or_api_not_found(BRC20SError::StakeInfoNotFound)?;

  log::debug!("rpc: get brc20s_debug_stake_info: {:?}", stake_info);

  Ok(Json(ApiResponse::ok(stake_info)))
}
