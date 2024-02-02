use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct EmptyResponse;

impl IntoResponse for EmptyResponse {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap()
    }
}
