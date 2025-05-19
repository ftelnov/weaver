use super::{error::InternalError, ResponsePart};
use crate::server::Response;
use http::{header::IntoHeaderName, HeaderMap, HeaderName, HeaderValue};

impl ResponsePart for HeaderMap {
    async fn apply(self, response: &mut Response) {
        *response.headers_mut() = self;
    }
}

impl<T: IntoHeaderName> ResponsePart for (T, HeaderValue) {
    async fn apply(self, response: &mut Response) {
        response.headers_mut().insert(self.0, self.1);
    }
}

impl<
        KE: std::error::Error,
        K: TryInto<HeaderName, Error = KE>,
        VE: std::error::Error,
        V: TryInto<HeaderValue, Error = VE>,
        const N: usize,
    > ResponsePart for [(K, V); N]
{
    async fn apply(self, response: &mut Response) {
        let headers = response.headers_mut();
        headers.reserve(N);
        for (k, v) in self {
            let k = match k.try_into().map_err(InternalError::from) {
                Ok(k) => k,
                Err(e) => {
                    e.apply(response).await;
                    return;
                }
            };
            let v = match v.try_into().map_err(InternalError::from) {
                Ok(v) => v,
                Err(e) => {
                    e.apply(response).await;
                    return;
                }
            };
            headers.insert(k, v);
        }
    }
}
