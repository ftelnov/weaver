use crate::{echo_endpoint, json_endpoint};
use hyper::{HeaderMap, StatusCode};
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
        .path("/counter_protected")
        .middleware(MiddlewareFn::new(counter_middleware))
        .route("/echo", HandlerFn::new(echo_endpoint))
        .route("/json", HandlerFn::new(json_endpoint))
}

#[derive(Default)]
struct AddValue(u64);

impl AddValue {
    const HEADER_NAME: &'static str = "X-Add-Value";

    fn from_headers(headers: &HeaderMap) -> Option<Self> {
        let value = headers
            .get(Self::HEADER_NAME)?
            .to_str()
            .unwrap()
            .parse()
            .ok()?;
        Some(Self(value))
    }
}

impl FromRequest for AddValue {
    type Rejection = ();

    async fn from_request(request: &mut Request) -> Result<Self, Self::Rejection> {
        let value = Self::from_headers(request.headers()).unwrap_or_default();
        Ok(value)
    }
}

enum CounterError {
    CounterLimitExceeded,
}

impl ResponsePart for CounterError {
    async fn apply(self, response: &mut Response) {
        match self {
            Self::CounterLimitExceeded => {
                Json(serde_json::json!({
                    "error": "Counter limit exceeded"
                }))
                .apply(response)
                .await;
                *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
            }
        }
    }
}

async fn counter_middleware(
    AddValue(value): AddValue,
    request: Request,
    next: Next,
) -> Result<impl ResponsePart, CounterError> {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    let previous = COUNTER.fetch_add(value, std::sync::atomic::Ordering::Relaxed);
    if previous + value > 100 {
        return Err(CounterError::CounterLimitExceeded);
    }
    let response = next.call(request).await;
    Ok(response)
}
