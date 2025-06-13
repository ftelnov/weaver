//! This example demonstrates how ordering of middlewares works.
//! First middleware unsets a header if needed, second middleware would fail if the header is set.
//! Hence, first middleware "prevents" second middleware from error.
//! Those middlewares also does response postprocessing to demonstrate order.
use hyper::StatusCode;
use std::sync::{
    atomic::{self, AtomicU64},
    Arc,
};
use weaver::{
    frontend::{
        extras::json::Json,
        handler::HandlerFn,
        middleware::{MiddlewareFn, Next},
        request::{FromRequest, Request},
        response::ResponsePart,
        routing::Group,
    },
    server::Response,
};

pub fn group() -> Group {
    Group::default()
        .group(
            Group::default()
                .path("/just_second")
                .middleware(MiddlewareFn::new(second_middleware))
                .route("/echo", HandlerFn::new(handler)),
        )
        .unwrap()
        .group(
            Group::default()
                .path("/combined")
                .middleware(MiddlewareFn::new(first_middleware))
                .group(
                    Group::default()
                        .middleware(MiddlewareFn::new(second_middleware))
                        .route("/echo", HandlerFn::new(handler)),
                )
                .unwrap(),
        )
        .unwrap()
}

const MUST_BE_UNSET_HEADER: &str = "X-Must-Be-Unset";
const WAS_SET_HEADER: &str = "X-Was-Set";

enum Error {
    HeaderMustBeUnset,
}

impl ResponsePart for Error {
    async fn apply(self, response: &mut Response) {
        match self {
            Self::HeaderMustBeUnset => {
                Json(serde_json::json!({
                    "error": "Header must be unset"
                }))
                .apply(response)
                .await;
                *response.status_mut() = StatusCode::BAD_REQUEST;
            }
        }
    }
}

#[derive(Default, Clone)]
struct TransitiveCounter(Arc<AtomicU64>);

impl FromRequest for TransitiveCounter {
    type Rejection = ();
    async fn from_request(request: &mut Request) -> Result<Self, Self::Rejection> {
        let counter: &mut TransitiveCounter = request.extensions_mut().get_or_insert_default();
        Ok(Self(counter.0.clone()))
    }
}

async fn first_middleware(
    TransitiveCounter(counter): TransitiveCounter,
    mut request: Request,
    next: Next,
) -> Result<impl ResponsePart, Error> {
    let was_set = request.headers_mut().remove(MUST_BE_UNSET_HEADER).is_some();
    counter.fetch_add(1, atomic::Ordering::Relaxed);
    let response = next.call(request).await;
    Ok((
        response,
        (WAS_SET_HEADER, was_set.to_string().parse().unwrap()),
    ))
}

async fn second_middleware(
    TransitiveCounter(counter): TransitiveCounter,
    request: Request,
    next: Next,
) -> Result<impl ResponsePart, Error> {
    let is_set = request.headers().get(MUST_BE_UNSET_HEADER).is_some();
    if is_set {
        return Err(Error::HeaderMustBeUnset);
    }
    counter.fetch_add(1, atomic::Ordering::Relaxed);
    let response = next.call(request).await;
    Ok(response)
}

async fn handler(
    TransitiveCounter(counter): TransitiveCounter,
) -> Result<impl ResponsePart, Error> {
    counter.fetch_add(1, atomic::Ordering::Relaxed);
    Ok(Json(serde_json::json!({
        "counter": counter.load(atomic::Ordering::Relaxed)
    })))
}
