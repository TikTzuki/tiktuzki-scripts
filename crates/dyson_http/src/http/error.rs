use axum::response::Response;
use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;

/// A default error response for most API errors.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiError {
    /// An error message.
    pub error: String,
    #[serde(skip)]
    pub status: StatusCode,
    /// Optional Additional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_details: Option<Value>,
}

impl ApiError {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
            status: StatusCode::BAD_REQUEST,
            error_details: None,
        }
    }
    pub fn from<E>(error: E) -> Self
    where
        E: core::error::Error,
    {
        Self {
            error: error.to_string(),
            status: StatusCode::BAD_REQUEST,
            error_details: None,
        }
    }

    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.error_details = Some(details);
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut res = serde_json::to_string(&self)
            .unwrap_or_else(|_| {
                serde_json::json!({"error": "Failed to serialize error response"}).to_string()
            })
            .into_response();
        *res.status_mut() = self.status;
        res
    }
}
