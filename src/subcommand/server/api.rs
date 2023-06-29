use super::error::ApiError;
use super::*;
use axum::Json;

#[derive(Deserialize)]
pub struct Pagination {
  pub start: Option<usize>,
  pub limit: Option<usize>,
}

pub(crate) type ApiResult<T> = Result<axum::Json<ApiResponse<T>>, ApiError>;

pub(super) trait ApiOptionExt<T> {
  fn ok_or_api_err<F: FnOnce() -> ApiError>(self, f: F) -> Result<T, ApiError>;
  fn ok_or_api_not_found<S: Into<String>>(self, s: S) -> Result<T, ApiError>;
}

impl<T> ApiOptionExt<T> for Option<T> {
  fn ok_or_api_err<F: FnOnce() -> ApiError>(self, f: F) -> Result<T, ApiError> {
    match self {
      Some(value) => Ok(value),
      None => Err(f()),
    }
  }
  fn ok_or_api_not_found<S: Into<String>>(self, s: S) -> Result<T, ApiError> {
    match self {
      Some(value) => Ok(value),
      None => Err(ApiError::not_found(s)),
    }
  }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeightInfo<T: Serialize> {
  pub ord_height: Option<u64>,
  pub btc_chain_info: Option<T>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HeightInfoQuery {
  btc: Option<bool>,
}

pub(crate) async fn node_info(
  Extension(index): Extension<Arc<Index>>,
  Query(query): Query<HeightInfoQuery>,
) -> ApiResult<HeightInfo<bitcoincore_rpc::json::GetBlockchainInfoResult>> {
  log::debug!("rpc: get node_info");

  let (ord_height, btc_info) = index.height_btc(query.btc.unwrap_or_default())?;

  let height_info = HeightInfo {
    ord_height: ord_height.map(|h| h.0),
    btc_chain_info: btc_info,
  };

  Ok(Json(ApiResponse::ok(height_info)))
}
