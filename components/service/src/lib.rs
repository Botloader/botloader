use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

use axum::response::{IntoResponse, Sse, sse::Event};
use futures::StreamExt;
use futures::{Stream, stream::BoxStream};
use reqwest::StatusCode;
use serde::{Serialize, de::DeserializeOwned};

pub use axum::extract::Json;

use crate::sse::SseError;

pub mod sse;

#[macro_export]
macro_rules! service {
    (
        $server_name:ident, $client_name:ident,
        $(
            $method:tt $name:ident: ( $($arg:ty)? ) -> $ret:ty
        ),+ $(,)?
    ) => {
        pub trait $server_name: Send + Sync + 'static {
            $(
                fn $name(
                    &self,
                    $(payload: $arg,)?
                ) -> impl ::std::future::Future<Output = Result<$ret, ::reqwest::StatusCode>> + Send + Sync;
            )+
        }

        $(
            service!(@handler $server_name $method $name ( $($arg)? ) -> $ret);
        )+

        pub fn router<T: $server_name>(service: ::std::sync::Arc<T>) -> Router {
            Router::new()
                $(
                    .route(concat!("/", stringify!($name)), ::axum::routing::on(::axum::routing::MethodFilter::$method, $name::<T>))
                )+
                .layer(::axum::extract::Extension(service))
        }

        pub trait $client_name {

            fn request<BodyT>(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder;

            $(
                service!(@client_method $server_name $method $name ( $($arg)? ) -> $ret);
            )+
        }
    };

    (@handler $service_name:ident $method:tt $name:ident ( $arg:ty ) -> $ret:ty) => {
        async fn $name<T: $service_name>(
            service: ::axum::extract::Extension<::std::sync::Arc<T>>,
            payload: service!(@arg_binding $method $arg)
        ) -> Result<$ret, ::reqwest::StatusCode> {
            let resp = service.0.$name(payload.0).await?;
            Ok(resp)
        }
    };

    (@handler $service_name:ident $method:tt $name:ident () -> $ret:ty) => {
        async fn $name<T: $service_name>(
            service: ::axum::extract::Extension<::std::sync::Arc<T>>,
        ) -> Result<$ret, ::reqwest::StatusCode> {
            let resp = service.0.$name().await?;
            Ok(resp)
        }
    };

    (@arg_binding GET $arg:ty) => {
        ::axum::extract::Query<$arg>
    };

    (@arg_binding $other:tt $arg:ty) => {
        ::axum::extract::Json<$arg>
    };

    (@client_method $service_name:ident $method:tt $name:ident ( $($arg:ty)? ) -> $ret:ty) => {
         fn $name(
            &self,
            $(payload: $arg,)?
        ) -> impl ::std::future::Future<Output = Result<<$ret as $crate::GetClientResponse>::OutputType, $crate::ClientError>> + Send + Sync{
            let req = self.request::<$ret>(::reqwest::Method::$method, stringify!($name));
            let req = service!(@client_apply_arg $method payload req $( $arg )? );

            async move {
                let resp = req.send().await.map_err($crate::ClientError::Reqwest)?;
                if resp.status().is_success() {
                    let data = <$ret>::get_response_body(resp)
                        .await
                        .map_err($crate::ClientError::Reqwest)?;

                    Ok(data)
                } else {
                    Err($crate::ClientError::Other(resp.status()))
                }
            }
        }
    };

    (@client_apply_arg $method:tt $payload:ident $req:ident) => {
        $req
    };

    (@client_apply_arg GET $payload:ident $req:ident $arg:ty) => {
        $req.query(&$payload)
    };

    (@client_apply_arg $method:tt $payload:ident $req:ident $arg:ty) => {
        $req.json(&$payload)
    };
}

pub struct SseStream<T> {
    sse: Sse<BoxStream<'static, Result<Event, String>>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Serialize + 'static> SseStream<T> {
    pub fn new(s: impl Stream<Item = Result<T, String>> + Send + 'static) -> SseStream<T> {
        let s = s
            .map(|item| match item {
                Ok(item) => {
                    let data = serde_json::to_string(&item).map_err(|e| {
                        format!("{{\"error\": \"failed to serialize log item: {}\"}}", e)
                    })?;

                    Ok(Event::default().data(data))
                }
                Err(e) => Err(e),
            })
            .boxed();

        SseStream {
            sse: Sse::new(Box::pin(s)),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> IntoResponse for SseStream<T> {
    fn into_response(self) -> axum::response::Response {
        self.sse.into_response()
    }
}

impl<T: DeserializeOwned + 'static> GetClientResponse for SseStream<T> {
    type OutputType = Pin<Box<dyn Stream<Item = Result<T, SseError>> + Send + 'static>>;

    async fn get_response_body(response: reqwest::Response) -> reqwest::Result<Self::OutputType> {
        let bytes_stream = response.bytes_stream();
        return Ok(sse::SseClientStream::<T>::new(bytes_stream).boxed());
    }
}

impl IntoResponse for Empty {
    fn into_response(self) -> axum::response::Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

pub trait GetClientResponse {
    type OutputType;

    fn get_response_body(
        response: reqwest::Response,
    ) -> impl Future<Output = reqwest::Result<Self::OutputType>> + Send + 'static;
}

impl<T: DeserializeOwned + 'static> GetClientResponse for Json<T> {
    type OutputType = T;

    fn get_response_body(
        response: reqwest::Response,
    ) -> impl Future<Output = reqwest::Result<Self::OutputType>> + Send + 'static {
        response.json::<T>()
    }
}

impl GetClientResponse for Empty {
    type OutputType = Empty;

    fn get_response_body(
        _response: reqwest::Response,
    ) -> impl Future<Output = reqwest::Result<Empty>> + Send + 'static {
        async { Ok(Empty) }
    }
}
pub struct Empty;

#[derive(Debug)]
pub enum ClientError {
    Reqwest(reqwest::Error),
    Other(reqwest::StatusCode),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Reqwest(e) => write!(f, "reqwest error: {}", e),
            ClientError::Other(code) => write!(f, "request failed with status code: {}", code),
        }
    }
}

impl std::error::Error for ClientError {}
