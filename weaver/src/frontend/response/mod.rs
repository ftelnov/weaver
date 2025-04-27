use crate::server::{Body, Response};
use http::{HeaderMap, StatusCode};
use std::future::Future;

pub mod json;

pub trait IntoResponse {
    fn into_response(self) -> impl Future<Output = Response>;
}

impl IntoResponse for () {
    async fn into_response(self) -> Response {
        Response::new(Body::empty())
    }
}

impl IntoResponse for String {
    async fn into_response(self) -> Response {
        Response::new(Body::from(self))
    }
}

impl IntoResponse for Response {
    async fn into_response(self) -> Response {
        self
    }
}

impl<B: Into<Body>> IntoResponse for (StatusCode, HeaderMap, B) {
    async fn into_response(self) -> Response {
        let mut builder = http::Response::builder().status(self.0);
        *builder.headers_mut().unwrap() = self.1;
        builder.body(self.2.into()).unwrap()
    }
}

impl<R: IntoResponse, E: IntoResponse> IntoResponse for Result<R, E> {
    async fn into_response(self) -> Response {
        match self {
            Ok(r) => r.into_response().await,
            Err(e) => e.into_response().await,
        }
    }
}
