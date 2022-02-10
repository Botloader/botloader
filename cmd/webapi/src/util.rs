use std::convert::Infallible;

use axum::{
    body::Bytes,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use http_body::Empty;

pub struct EmptyResponse;

impl IntoResponse for EmptyResponse {
    type Body = Empty<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> axum::http::Response<Self::Body> {
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Empty::new())
            .unwrap()
    }
}
