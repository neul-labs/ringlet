//! HTTP error handling and response conversion.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response as AxumResponse},
    Json,
};
use clown_core::rpc::error_codes;
use clown_core::Response;
use serde::Serialize;

/// Standard API response wrapper.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
}

impl ApiResponse<()> {
    pub fn ok() -> Self {
        Self {
            success: true,
            data: None,
            error: None,
        }
    }
}

/// API error details.
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: i32,
    pub message: String,
}

impl ApiError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Map error code to HTTP status.
    pub fn status_code(&self) -> StatusCode {
        match self.code {
            error_codes::AGENT_NOT_FOUND
            | error_codes::PROVIDER_NOT_FOUND
            | error_codes::PROFILE_NOT_FOUND
            | error_codes::ROUTE_NOT_FOUND
            | error_codes::ALIAS_NOT_FOUND => StatusCode::NOT_FOUND,

            error_codes::PROFILE_EXISTS
            | error_codes::PROXY_ALREADY_RUNNING => StatusCode::CONFLICT,

            error_codes::AGENT_NOT_INSTALLED
            | error_codes::INCOMPATIBLE_PROVIDER
            | error_codes::INVALID_ENDPOINT
            | error_codes::HOOKS_NOT_SUPPORTED
            | error_codes::INVALID_HOOK_EVENT
            | error_codes::PROXY_NOT_ENABLED
            | error_codes::PROXY_NOT_RUNNING
            | error_codes::PROXY_NOT_SUPPORTED => StatusCode::BAD_REQUEST,

            error_codes::PROXY_START_FAILED
            | error_codes::SCRIPT_ERROR
            | error_codes::EXECUTION_ERROR
            | error_codes::REGISTRY_ERROR => StatusCode::INTERNAL_SERVER_ERROR,

            error_codes::INTERNAL_ERROR => StatusCode::INTERNAL_SERVER_ERROR,

            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// HTTP error type that implements IntoResponse.
pub struct HttpError {
    pub status: StatusCode,
    pub body: ApiResponse<()>,
}

impl HttpError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        let error = ApiError::new(code, message);
        let status = error.status_code();
        Self {
            status,
            body: ApiResponse {
                success: false,
                data: None,
                error: Some(error),
            },
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(error_codes::INTERNAL_ERROR, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(error_codes::PROFILE_NOT_FOUND, message)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> AxumResponse {
        (self.status, Json(self.body)).into_response()
    }
}

/// Convert clown_core::Response to HTTP response.
pub fn response_to_http<T: Serialize>(
    response: Response,
    extractor: impl FnOnce(Response) -> Result<T, HttpError>,
) -> Result<Json<ApiResponse<T>>, HttpError> {
    match &response {
        Response::Error { code, message } => Err(HttpError::new(*code, message.clone())),
        _ => {
            let data = extractor(response)?;
            Ok(Json(ApiResponse::success(data)))
        }
    }
}

/// Helper macro for extracting specific response variants.
#[macro_export]
macro_rules! extract_response {
    ($response:expr, $variant:ident) => {
        match $response {
            clown_core::Response::$variant(data) => Ok(data),
            clown_core::Response::Error { code, message } => {
                Err($crate::http::error::HttpError::new(code, message))
            }
            _ => Err($crate::http::error::HttpError::internal(
                "Unexpected response type",
            )),
        }
    };
}
