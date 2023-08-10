use serde::ser::SerializeStruct;

use {super::*, utoipa::ToSchema};

pub(super) enum ServerError {
  Internal(Error),
  BadRequest(String),
  NotFound(String),
}

pub(super) type ServerResult<T> = Result<T, ServerError>;

impl IntoResponse for ServerError {
  fn into_response(self) -> Response {
    match self {
      Self::Internal(error) => {
        eprintln!("error serving request: {error}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          StatusCode::INTERNAL_SERVER_ERROR
            .canonical_reason()
            .unwrap_or_default(),
        )
          .into_response()
      }
      Self::NotFound(message) => (StatusCode::NOT_FOUND, message).into_response(),
      Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
    }
  }
}

pub(super) trait OptionExt<T> {
  fn ok_or_not_found<F: FnOnce() -> S, S: Into<String>>(self, f: F) -> ServerResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
  fn ok_or_not_found<F: FnOnce() -> S, S: Into<String>>(self, f: F) -> ServerResult<T> {
    match self {
      Some(value) => Ok(value),
      None => Err(ServerError::NotFound(f().into() + " not found")),
    }
  }
}

impl From<Error> for ServerError {
  fn from(error: Error) -> Self {
    Self::Internal(error)
  }
}

#[repr(i32)]
#[derive(ToSchema)]
pub(crate) enum ApiError {
  /// Internal server error.
  #[schema(example = json!(&ApiError::internal("internal error")))]
  Internal(String) = 1,

  /// Bad request.
  #[schema(example = json!(&ApiError::internal("bad request")))]
  BadRequest(String) = 2,

  /// Resource not found.
  #[schema(example = json!(&ApiError::internal("not found")))]
  NotFound(String) = 3,
}

impl ApiError {
  pub(crate) fn code(&self) -> i32 {
    match self {
      Self::Internal(_) => 1,
      Self::BadRequest(_) => 2,
      Self::NotFound(_) => 3,
    }
  }

  pub(crate) fn not_found<S: ToString>(message: S) -> Self {
    Self::NotFound(message.to_string())
  }

  pub(crate) fn internal<S: ToString>(message: S) -> Self {
    Self::Internal(message.to_string())
  }

  pub(crate) fn bad_request<S: ToString>(message: S) -> Self {
    Self::BadRequest(message.to_string())
  }
}
impl Serialize for ApiError {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let mut state = serializer.serialize_struct("ApiError", 2)?;
    match self {
      ApiError::Internal(msg) | ApiError::BadRequest(msg) | ApiError::NotFound(msg) => {
        state.serialize_field("code", &self.code())?;
        state.serialize_field("message", &msg)?;
        state.end()
      }
    }
  }
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let status_code = match &self {
      Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
      Self::BadRequest(_) => StatusCode::BAD_REQUEST,
      Self::NotFound(_) => StatusCode::NOT_FOUND,
    };

    (status_code, axum::Json(self)).into_response()
  }
}

impl From<anyhow::Error> for ApiError {
  fn from(error: anyhow::Error) -> Self {
    Self::internal(error)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_serialize_api_error() {
    let api_error = ApiError::internal("internal error");
    let json = serde_json::to_string(&api_error).unwrap();
    assert_eq!(json, r#"{"code":1,"message":"internal error"}"#);

    let api_error = ApiError::bad_request("bad request");
    let json = serde_json::to_string(&api_error).unwrap();
    assert_eq!(json, r#"{"code":2,"message":"bad request"}"#);

    let api_error = ApiError::not_found("not found");
    let json = serde_json::to_string(&api_error).unwrap();
    assert_eq!(json, r#"{"code":3,"message":"not found"}"#);
  }
}
