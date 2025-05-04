use super::response::IntoResponse;
use crate::server;
use std::{
    future::Future,
    ops::{Deref, DerefMut},
};

pub mod json;
pub mod path;

pub struct Request(Option<server::Request>);

impl Request {
    const TAKE_ERROR: &str = "Request already taken by other extractor, ensure that only one extractor consumes the request itself - e.g. final one";

    pub fn new(request: server::Request) -> Self {
        Self(Some(request))
    }

    pub fn take(&mut self) -> server::Request {
        self.0.take().expect(Self::TAKE_ERROR)
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
    type Rejection: IntoResponse;

    fn from_request(request: &mut Request) -> impl Future<Output = Result<Self, Self::Rejection>>
    where
        Self: Sized;
}

pub struct RawRequest(pub server::Request);

impl FromRequest for RawRequest {
    type Rejection = ();

    async fn from_request(request: &mut Request) -> Result<Self, Self::Rejection>
    where
        Self: Sized,
    {
        Ok(Self(request.take()))
    }
}
