use super::super::{
    request::{FromRequest, Request},
    response::ResponsePart,
};
use crate::{frontend::response::error::InternalError, server::Response};
use bytes::{BufMut as _, BytesMut};
use http::{header, HeaderValue};
use http_body_util::BodyExt as _;
use serde::de::DeserializeOwned;

pub struct Json<T>(pub T);

impl<T> From<T> for Json<T> {
    fn from(data: T) -> Self {
        Self(data)
    }
}

impl<T: DeserializeOwned> FromRequest for Json<T> {
    type Rejection = String;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Rejection> {
        let body = request
            .body_mut()
            .collect()
            .await
            .map_err(|err| format!("failed to collect body: {}", err))?
            .to_bytes();
        let data = serde_json::from_slice(&body)
            .map_err(|e| format!("failed to deserialize data: {}", e))?;
        Ok(Self(data))
    }
}

impl<T: serde::Serialize> ResponsePart for Json<T> {
    async fn apply(self, response: &mut Response) {
        // Extracted into separate fn so it's only compiled once for all T.
        fn as_response_part(
            buf: BytesMut,
            ser_result: serde_json::Result<()>,
        ) -> impl ResponsePart {
            ser_result
                .map(|_| {
                    (
                        (
                            header::CONTENT_TYPE,
                            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                        ),
                        buf.freeze(),
                    )
                })
                .map_err(InternalError::from)
        }

        // Use a small initial capacity of 128 bytes like serde_json::to_vec
        // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();
        let res = serde_json::to_writer(&mut buf, &self.0);
        as_response_part(buf.into_inner(), res)
            .apply(response)
            .await;
    }
}
