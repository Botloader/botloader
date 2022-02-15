use axum::{
    body::{self, BoxBody},
    http::{header, Response, StatusCode},
    response::IntoResponse,
};
use serde_json::json;
use validation::ValidationError;

#[derive(Debug, thiserror::Error)]
pub enum ApiErrorResponse {
    #[error("csrf token expired")]
    BadCsrfToken,

    #[error("Session expired")]
    SessionExpired,

    #[error("validation failed")]
    ValidationFailed(Vec<ValidationError>),

    #[error("Internal server error")]
    InternalError,
}

impl ApiErrorResponse {
    pub fn public_desc(&self) -> (StatusCode, u32, String) {
        match &self {
            Self::SessionExpired => (StatusCode::BAD_REQUEST, 1, self.to_string()),
            Self::BadCsrfToken => (StatusCode::BAD_REQUEST, 2, self.to_string()),
            Self::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, 3, self.to_string()),
            Self::ValidationFailed(verr) => (
                StatusCode::BAD_REQUEST,
                4,
                serde_json::to_string(verr).unwrap_or_default(),
            ),
        }
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response<BoxBody> {
        let (resp_code, err_code, msg) = self.public_desc();

        let body = json!({
            "code": err_code,
            "description": msg,
        })
        .to_string();

        Response::builder()
            .status(resp_code)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body::boxed(body::Full::from(body)))
            .unwrap()
    }
}
