use axum::{
    extract::{MatchedPath, Request},
    response::Response,
};
use futures::future::BoxFuture;
use std::{
    task::{Context, Poll},
    time::Instant,
};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct MetricsLayer {
    pub name_prefix: &'static str,
}

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsMiddleware {
            inner,
            name_prefix: self.name_prefix,
        }
    }
}

#[derive(Clone)]
pub struct MetricsMiddleware<S> {
    pub inner: S,
    name_prefix: &'static str,
}

impl<S> Service<Request> for MetricsMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let path = if let Some(path) = req.extensions().get::<MatchedPath>() {
            path.as_str().to_owned()
        } else {
            "unknown".to_owned()
        };

        let method = req.method().to_string();

        let counter_name = format!("{}.requests_total", self.name_prefix);
        let latency_name = format!("{}.request_duration_seconds", self.name_prefix);

        let future = self.inner.call(req);

        let started = Instant::now();
        Box::pin(async move {
            let result = future.await;

            let elapsed = started.elapsed();

            let status_code = match &result {
                Ok(resp) => resp.status().as_u16(),
                Err(_) => 0,
            };

            let elapsed_seconds = elapsed.as_secs_f64();
            metrics::counter!(counter_name,
                "response_code" => status_code.to_string(),
                "path" => path.clone(),
                "method" => method.clone(),
            )
            .increment(1);
            metrics::histogram!(latency_name,
                "path" => path,
                "method" => method
            )
            .record(elapsed_seconds);

            result
        })
    }
}
