use super::IntoResponse;
use crate::server::Response;
use bytes::{BufMut as _, BytesMut};
use http::{header, HeaderMap, HeaderValue, StatusCode};

pub struct JsonResponse<T>(pub T);

impl<T: serde::Serialize> IntoResponse for JsonResponse<T> {
    async fn into_response(self) -> Response {
        // Use a small initial capacity of 128 bytes like serde_json::to_vec
        // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();

        match serde_json::to_writer(&mut buf, &self.0) {
            Ok(()) => {
                let mut headers = HeaderMap::new();
                headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                );
                (StatusCode::OK, headers, buf.into_inner().freeze())
                    .into_response()
                    .await
            }
            Err(err) => {
                let mut headers = HeaderMap::new();
                headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                );
                (StatusCode::INTERNAL_SERVER_ERROR, headers, err.to_string())
                    .into_response()
                    .await
            }
        }
    }
}
