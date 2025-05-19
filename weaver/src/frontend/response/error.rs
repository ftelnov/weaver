use super::ResponsePart;
use crate::server::Response;
use http::{header, HeaderValue, StatusCode};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct InternalError<E>(pub E);

impl<E: std::error::Error> From<E> for InternalError<E> {
    fn from(err: E) -> Self {
        Self(err)
    }
}

impl<E: Display> ResponsePart for InternalError<E> {
    async fn apply(self, response: &mut Response) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
            ),
            self.0.to_string(),
        )
            .apply(response)
            .await;
    }
}
