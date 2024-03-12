use axum::{
    body::Body,
    extract::multipart::MultipartError,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
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

    #[error("guild already has this plugin")]
    GuildAlreadyHasPlugin,

    #[error("Script is not a plugin")]
    ScriptNotAPlugin,

    #[error("You're not an botloader admin")]
    NotBlAdmin,

    #[error("Bad multi-part form request: {0}")]
    MultipartFormError(#[from] MultipartError),

    #[error("Bad multi-part form json data: {0}")]
    MultipartFormJson(serde_json::Error),

    #[error("Bad multi-part form, missing field: {0}")]
    MissingMultiPartFormField(String),

    #[error("Uploaded image is too big")]
    ImageTooBig,

    #[error("Unsupported image")]
    ImageNotSupported,

    #[error("Could not find image")]
    ImageNotFound,

    #[error("Reached max images")]
    MaxImagesReached,

    #[error("Script not found")]
    ScriptNotFound,

    #[error("Stripe integration not enabled")]
    StripeNotEnabled,
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
            Self::GuildAlreadyHasPlugin => (StatusCode::BAD_REQUEST, 10, None),
            Self::ScriptNotAPlugin => (StatusCode::BAD_REQUEST, 11, None),
            Self::NotBlAdmin => (StatusCode::FORBIDDEN, 12, None),
            Self::MultipartFormError(_) => (StatusCode::BAD_REQUEST, 13, None),
            Self::MultipartFormJson(_) => (StatusCode::BAD_REQUEST, 14, None),
            Self::MissingMultiPartFormField(_) => (StatusCode::BAD_REQUEST, 15, None),
            Self::ImageTooBig => (StatusCode::BAD_REQUEST, 16, None),
            Self::ImageNotSupported => (StatusCode::BAD_REQUEST, 17, None),
            Self::ImageNotFound => (StatusCode::BAD_REQUEST, 18, None),
            Self::MaxImagesReached => (StatusCode::BAD_REQUEST, 19, None),
            Self::ScriptNotFound => (StatusCode::BAD_REQUEST, 20, None),
            Self::StripeNotEnabled => (StatusCode::INTERNAL_SERVER_ERROR, 21, None),
        }
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
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
            .body(Body::from(body))
            .unwrap()
    }
}
