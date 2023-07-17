use super::*;
use axum::Json;
use shadow_rs::shadow;
shadow!(build);

#[derive(Deserialize)]
pub struct Pagination {
  pub start: Option<usize>,
  pub limit: Option<usize>,
}

pub(crate) type ApiResult<T> = Result<axum::Json<ApiResponse<T>>, ApiError>;

pub(super) trait ApiOptionExt<T> {
  fn ok_or_api_err<F: FnOnce() -> ApiError>(self, f: F) -> Result<T, ApiError>;
  fn ok_or_api_not_found<S: ToString>(self, s: S) -> Result<T, ApiError>;
}

impl<T> ApiOptionExt<T> for Option<T> {
  fn ok_or_api_err<F: FnOnce() -> ApiError>(self, f: F) -> Result<T, ApiError> {
    match self {
      Some(value) => Ok(value),
      None => Err(f()),
    }
  }
  fn ok_or_api_not_found<S: ToString>(self, s: S) -> Result<T, ApiError> {
    match self {
      Some(value) => Ok(value),
      None => Err(ApiError::not_found(s)),
    }
  }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
  pub version: Option<String>,
  pub branch: Option<String>,
  pub commit_hash: Option<String>,
  pub build_time: Option<String>,
  pub chain_info: ChainInfo,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainInfo {
  pub network: Option<String>,
  pub ord_height: Option<u64>,
  pub btc_chain_height: Option<u64>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfoQuery {
  btc: Option<bool>,
}

pub(crate) async fn node_info(
  Extension(index): Extension<Arc<Index>>,
  Query(query): Query<NodeInfoQuery>,
) -> ApiResult<NodeInfo> {
  log::debug!("rpc: get node_info");

  let (ord_height, btc_height) = index.height_btc(query.btc.unwrap_or_default())?;

  let node_info = NodeInfo {
    version: Some(build::PKG_VERSION.into()),
    branch: Some(build::BRANCH.into()),
    commit_hash: Some(build::SHORT_COMMIT.into()),
    build_time: Some(build::BUILD_TIME.into()),
    chain_info: ChainInfo {
      network: Some(index.get_chain_network().to_string()),
      ord_height: ord_height.map(|h| h.0),
      btc_chain_height: btc_height.map(|h| h.0),
    },
  };

  Ok(Json(ApiResponse::ok(node_info)))
}
