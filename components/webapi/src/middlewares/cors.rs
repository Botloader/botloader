use axum::http::{header, HeaderMap, HeaderValue, Method};
use axum::{body::Body, extract::Request, response::Response};
use common::config::RunConfig;
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct CorsLayer {
    pub run_config: RunConfig,
}

impl<S> Layer<S> for CorsLayer {
    type Service = CorsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CorsMiddleware {
            run_config: self.run_config.clone(),
            inner,
        }
    }
}

#[derive(Clone)]
pub struct CorsMiddleware<S> {
    inner: S,
    run_config: RunConfig,
}

impl<S> Service<Request> for CorsMiddleware<S>
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
        let prod_host_base = self.run_config.frontend_host_base.clone();
        let origin = req
            .headers()
            .get(header::ORIGIN)
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_owned();

        let is_valid_origin = is_origin_allowed(&origin, &prod_host_base);

        if matches!(req.method(), &Method::OPTIONS) {
            Box::pin(async move {
                let mut resp = Response::new(Body::empty());

                if is_valid_origin {
                    insert_headers(&origin, resp.headers_mut());
                }

                Ok(resp)
            })
        } else {
            let future = self.inner.call(req);
            Box::pin(async move {
                match future.await {
                    Ok(mut resp) => {
                        if is_valid_origin {
                            insert_headers(&origin, resp.headers_mut());
                        }
                        Ok(resp)
                    }
                    Err(err) => Err(err),
                }
            })
        }
    }
}

fn is_origin_allowed(origin: &str, prod_origin: &str) -> bool {
    if origin == prod_origin {
        return true;
    }

    if origin == "http://localhost"
        || origin.starts_with("http://localhost:")
        || origin == "https://localhost"
        || origin.starts_with("https://localhost:")
    {
        // accept localhost domains, the user would have to install something locally for this to
        // be abused
        //
        // this helps a ton as we can develop the frontend against the prod API without the need
        // for running a local version of the stack
        return true;
    }

    false
}

fn insert_headers(host_base: &str, headers: &mut HeaderMap) {
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_str(host_base).unwrap(),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        HeaderValue::from_static("true"),
    );
}

#[cfg(test)]
mod tests {

    macro_rules! origin_allowed_test_case {
        ($actual_origin:literal, $allowed_origin:literal, allow) => {
            assert_eq!(
                super::is_origin_allowed($actual_origin, $allowed_origin),
                true
            );
        };

        ($actual_origin:literal, $allowed_origin:literal, deny) => {
            assert_eq!(
                super::is_origin_allowed($actual_origin, $allowed_origin),
                false
            );
        };
    }

    #[test]
    fn test_is_origin_allowed() {
        origin_allowed_test_case!("https://botloader.io", "https://botloader.io", allow);
        origin_allowed_test_case!("https://banana.io", "https://botloader.io", deny);
        origin_allowed_test_case!("https://localhost:3000", "https://botloader.io", allow);
        origin_allowed_test_case!("http://localhost", "https://botloader.io", allow);
    }
}
