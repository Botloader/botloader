use axum::{
    body::{self, BoxBody},
    http::{Response, StatusCode},
    response::IntoResponse,
};
use http_body::Empty;

pub struct EmptyResponse;

impl IntoResponse for EmptyResponse {
    fn into_response(self) -> axum::http::Response<BoxBody> {
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(body::boxed(Empty::new()))
            .unwrap()
    }
}
