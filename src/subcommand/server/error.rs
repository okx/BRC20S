use {
  super::*,
  utoipa::{ToResponse, ToSchema},
};

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
  Internal(String) = 1,
  BadRequest(String) = 2,
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

impl From<ApiError> for axum::Json<ApiErrorResponse> {
  fn from(val: ApiError) -> Self {
    axum::Json(ApiErrorResponse::api_err(&val))
  }
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let status_code = match &self {
      Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
      Self::BadRequest(_) => StatusCode::BAD_REQUEST,
      Self::NotFound(_) => StatusCode::NOT_FOUND,
    };
    let json: axum::Json<ApiErrorResponse> = self.into();

    (status_code, json).into_response()
  }
}

impl From<anyhow::Error> for ApiError {
  fn from(error: anyhow::Error) -> Self {
    Self::internal(error)
  }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema, ToResponse)]
pub(crate) struct ApiErrorResponse {
  pub code: i32,
  /// Error message.
  pub msg: String,
}

impl ApiErrorResponse {
  fn new(code: i32, msg: String) -> Self {
    Self { code, msg }
  }

  pub fn err(code: i32, msg: String) -> Self {
    Self::new(code, msg)
  }

  pub fn api_err(err: &ApiError) -> Self {
    match err {
      ApiError::Internal(msg) | ApiError::BadRequest(msg) | ApiError::NotFound(msg) => {
        Self::err(err.code(), msg.clone())
      }
    }
  }
}
