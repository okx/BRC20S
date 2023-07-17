use super::*;

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
pub(crate) enum ApiError {
  #[allow(dead_code)]
  NoError = 0,
  Internal(String) = 1,
  BadRequest(String) = 2,
  NotFound(String) = 3,
}

impl ApiError {
  pub(crate) fn code(&self) -> i32 {
    match self {
      Self::NoError => 0,
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

impl<T> Into<axum::Json<ApiResponse<T>>> for ApiError
where
  T: Serialize,
{
  fn into(self) -> axum::Json<ApiResponse<T>> {
    axum::Json(ApiResponse::api_err(&self))
  }
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let status_code = match &self {
      Self::NoError => StatusCode::OK,
      Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
      Self::BadRequest(_) => StatusCode::BAD_REQUEST,
      Self::NotFound(_) => StatusCode::NOT_FOUND,
    };
    let json: axum::Json<ApiResponse<()>> = self.into();

    (status_code, json).into_response()
  }
}

impl From<anyhow::Error> for ApiError {
  fn from(error: anyhow::Error) -> Self {
    Self::internal(error)
  }
}
