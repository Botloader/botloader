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

    #[error("internal server error")]
    InternalError,

    #[error("no active guild")]
    NoActiveGuild,

    #[error("not guild admin")]
    NotGuildAdmin,

    #[error("Plugin does not exist")]
    PluginNotFound,

    #[error("you do not have access to this plugin")]
    NoAccessToPlugin,

    #[error("you have created too many plugins")]
    UserPluginLimitReached,
}

impl ApiErrorResponse {
    pub fn public_desc(&self) -> (StatusCode, u32, Option<serde_json::Value>) {
        match &self {
            Self::SessionExpired => (StatusCode::BAD_REQUEST, 1, None),
            Self::BadCsrfToken => (StatusCode::BAD_REQUEST, 2, None),
            Self::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, 3, None),
            Self::ValidationFailed(v_err) => (
                StatusCode::BAD_REQUEST,
                4,
                Some(serde_json::to_value(v_err).unwrap_or_default()),
            ),
            Self::NoActiveGuild => (StatusCode::BAD_REQUEST, 5, None),
            Self::NotGuildAdmin => (StatusCode::FORBIDDEN, 6, None),
            Self::NoAccessToPlugin => (StatusCode::FORBIDDEN, 7, None),
            Self::UserPluginLimitReached => (StatusCode::BAD_REQUEST, 8, None),
            Self::PluginNotFound => (StatusCode::BAD_REQUEST, 9, None),
        }
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response<BoxBody> {
        let (resp_code, err_code, extra) = self.public_desc();

        let body = json!({
            "code": err_code,
            "description": self.to_string(),
            "extra_data": extra
        })
        .to_string();

        Response::builder()
            .status(resp_code)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body::boxed(body::Full::from(body)))
            .unwrap()
    }
}
