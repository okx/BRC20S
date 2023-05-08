use super::error::ApiError;
use super::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ApiResponse<T: Serialize> {
  pub code: i32,
  pub msg: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
  T: Serialize,
{
  pub fn new(code: i32, msg: String, data: Option<T>) -> Self {
    Self { code, msg, data }
  }

  pub fn ok(data: T) -> Self {
    Self::new(0, "ok".to_string(), Some(data))
  }

  pub fn err(code: i32, msg: String) -> Self {
    Self::new(code, msg, None)
  }

  pub fn api_err(err: &ApiError) -> Self {
    match err {
      ApiError::NoError => Self::new(0, "ok".to_string(), None),
      ApiError::Internal(msg) | ApiError::BadRequest(msg) | ApiError::NotFound(msg) => {
        Self::err(err.code(), msg.clone())
      }
    }
  }
}
