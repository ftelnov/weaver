use super::{FromRequest, Request};
use std::{collections::HashMap, ops::DerefMut as _};

/// Extracts path parameters from the request.
/// It consumes the request's path parameters, so future invocations will result in an empty map.
///
/// Example:
///
/// ```rust
/// use weaver::frontend::request::path::Path;
/// use weaver::frontend::handler::HandlerFn;
/// use weaver::server::Server;
///
/// fn main() {
///     let mut server = Server::new(Default::default());
///     let handler = HandlerFn::new(handler);
///     server.get("/path/{id}/content/{another_field}/{final_field}", handler);
/// }
///
/// async fn handler(Path(params): Path) -> String {
///     format!("path params: {:#?}", params)
/// }
/// ```
pub struct Path(pub HashMap<String, String>);

impl FromRequest for Path {
    type Rejection = ();

    async fn from_request(request: &mut Request) -> Result<Self, Self::Rejection> {
        let request = request.deref_mut();
        Ok(Self(std::mem::take(&mut request.params)))
    }
}
