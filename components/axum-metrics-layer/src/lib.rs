use axum::http::{Request, Response};
use metrics::Counter;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct MetricsLayer {
    pub name: &'static str,
}

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        let counter = metrics::counter!(self.name);
        MetricsMiddleware { inner, counter }
    }
}

#[derive(Clone)]
pub struct MetricsMiddleware<S> {
    pub inner: S,
    counter: Counter,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for MetricsMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        self.counter.increment(1);
        self.inner.call(req)
    }
}
