use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct EmptyResponse;

impl aide::OperationOutput for EmptyResponse {
    type Inner = ();

    fn inferred_responses(
        _ctx: &mut aide::generate::GenContext,
        _operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        vec![(
            Some(StatusCode::NO_CONTENT.as_u16()),
            aide::openapi::Response {
                description: "no content".to_string(),
                ..Default::default()
            },
        )]
    }
}

impl IntoResponse for EmptyResponse {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap()
    }
}
