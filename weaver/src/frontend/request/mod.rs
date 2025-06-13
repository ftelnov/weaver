use super::response::ResponsePart;
use crate::server;
use http::{HeaderMap, HeaderValue};
use std::{
    future::Future,
    ops::{Deref, DerefMut},
};

pub mod path;

#[derive(Default)]
pub struct Request(Option<server::Request>);

impl Request {
    const TAKE_ERROR: &str = r#"
    Request already taken by other extractor,
    ensure that exactly one extractor consumes the request fully - the last one in list of arguments.
    "#;

    pub fn new(request: server::Request) -> Self {
        Self(Some(request))
    }

    pub fn take(&mut self) -> server::Request {
        self.0.take().expect(Self::TAKE_ERROR)
    }
}

impl From<server::Request> for Request {
    fn from(request: server::Request) -> Self {
        Self(Some(request))
    }
}

impl Deref for Request {
    type Target = server::Request;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect(Self::TAKE_ERROR)
    }
}

impl DerefMut for Request {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect(Self::TAKE_ERROR)
    }
}

pub trait FromRequest {
    type Rejection: ResponsePart;

    fn from_request(request: &mut Request) -> impl Future<Output = Result<Self, Self::Rejection>>
    where
        Self: Sized;
}

impl FromRequest for Request {
    type Rejection = ();

    async fn from_request(request: &mut Request) -> Result<Self, Self::Rejection> {
        Ok(std::mem::take(request))
    }
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
