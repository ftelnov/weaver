use super::{FromRequest, Request};
use http_body_util::BodyExt as _;
use serde::de::DeserializeOwned;

pub struct JsonBody<T>(pub T);

impl<T> From<T> for JsonBody<T> {
    fn from(data: T) -> Self {
        Self(data)
    }
}

impl<T: DeserializeOwned> FromRequest for JsonBody<T> {
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
