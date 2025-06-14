use super::response::ResponsePart;
use crate::server::Request;
use http::{HeaderMap, HeaderValue};
use std::future::Future;

pub mod path;

pub trait FromRequest {
    type Rejection: ResponsePart;

    fn from_request(request: &mut Request) -> impl Future<Output = Result<Self, Self::Rejection>>
    where
        Self: Sized;
}

impl FromRequest for HeaderMap<HeaderValue> {
    type Rejection = ();

    async fn from_request(request: &mut Request) -> Result<Self, Self::Rejection> {
        Ok(std::mem::take(request.headers_mut()))
    }
}

pub struct Headers(pub HeaderMap<HeaderValue>);

impl FromRequest for Headers {
    type Rejection = ();

    async fn from_request(request: &mut Request) -> Result<Self, Self::Rejection> {
        Ok(Self(std::mem::take(request.headers_mut())))
    }
}

impl FromRequest for http::Extensions {
    type Rejection = ();

    async fn from_request(request: &mut Request) -> Result<Self, Self::Rejection> {
        Ok(std::mem::take(request.extensions_mut()))
    }
}

pub struct Extensions(pub http::Extensions);

impl FromRequest for Extensions {
    type Rejection = ();

    async fn from_request(request: &mut Request) -> Result<Self, Self::Rejection> {
        Ok(Self(std::mem::take(request.extensions_mut())))
    }
}
