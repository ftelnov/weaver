use crate::server::{Body, Response};
use http::{Extensions, HeaderMap, StatusCode};
use std::{convert::Infallible, future::Future};

pub mod error;
pub mod headers;

pub trait ResponsePart {
    fn apply(self, response: &mut Response) -> impl Future<Output = ()>;
}

impl ResponsePart for Infallible {
    async fn apply(self, _response: &mut Response) {}
}

impl ResponsePart for () {
    async fn apply(self, _response: &mut Response) {}
}

impl<T: Into<Body>> ResponsePart for T {
    async fn apply(self, response: &mut Response) {
        *response.body_mut() = self.into();
    }
}

impl<L: ResponsePart, R: ResponsePart> ResponsePart for either::Either<L, R> {
    async fn apply(self, response: &mut Response) {
        match self {
            either::Either::Left(l) => l.apply(response).await,
            either::Either::Right(r) => r.apply(response).await,
        }
    }
}

impl<R: ResponsePart, E: ResponsePart> ResponsePart for Result<R, E> {
    async fn apply(self, response: &mut Response) {
        match self {
            Ok(r) => r.apply(response).await,
            Err(e) => e.apply(response).await,
        }
    }
}

impl ResponsePart for Response {
    async fn apply(self, response: &mut Response) {
        *response = self;
    }
}

impl ResponsePart for StatusCode {
    async fn apply(self, response: &mut Response) {
        *response.status_mut() = self;
    }
}

impl ResponsePart for Extensions {
    async fn apply(self, response: &mut Response) {
        *response.extensions_mut() = self;
    }
}

/// [Extend] provides alternative implementations for some response parts
/// to avoid replacing the previously set parts by merging new ones into them.
///
/// Semantics are exactly similar to [std::iter::Extend] trait.
pub struct Extend<T>(pub T);

impl ResponsePart for Extend<HeaderMap> {
    async fn apply(self, response: &mut Response) {
        response.headers_mut().extend(self.0);
    }
}

impl ResponsePart for Extend<Extensions> {
    async fn apply(self, response: &mut Response) {
        response.extensions_mut().extend(self.0);
    }
}

#[macro_export]
macro_rules! impl_response_part_chain {
    ($($part:ident),*) => {
        impl<$($part),*> ResponsePart for ($($part),*)
        where
        $( $part: ResponsePart, )*
         {
            async fn apply(self, response: &mut Response) {
                #[allow(non_snake_case)]
                let ($($part),*) = self;
                $($part.apply(response).await;)*
            }
        }
    };
}

impl_response_part_chain!(A, B);
impl_response_part_chain!(A, B, C);
impl_response_part_chain!(A, B, C, D);
impl_response_part_chain!(A, B, C, D, E);
impl_response_part_chain!(A, B, C, D, E, F);
impl_response_part_chain!(A, B, C, D, E, F, G);
impl_response_part_chain!(A, B, C, D, E, F, G, H);
impl_response_part_chain!(A, B, C, D, E, F, G, H, I);
impl_response_part_chain!(A, B, C, D, E, F, G, H, I, J);
impl_response_part_chain!(A, B, C, D, E, F, G, H, I, J, K);
impl_response_part_chain!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_response_part_chain!(A, B, C, D, E, F, G, H, I, J, K, L, M);
