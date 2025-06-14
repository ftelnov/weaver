use serde::Serialize;
use weaver::{
    frontend::{
        extras::json::Json, handler::HandlerFn, request::Request, response::ResponsePart,
        routing::Group,
    },
    server::RouteBuilder,
};

/// Test that routing works in bucket manner - even if path clashes, method resolution still works.
pub fn group() -> Group {
    let path = "/methods";
    Group::default()
        .get(path, HandlerFn::new(get_endpoint))
        .post(path, HandlerFn::new(post_endpoint))
        .route(
            RouteBuilder::default()
                .path(path.to_string())
                .method(http::Method::from_bytes(b"VOROJBA").unwrap())
                .build()
                .unwrap(),
            HandlerFn::new(extension_first_endpoint),
        )
        .route(
            RouteBuilder::default()
                .path(path.to_string())
                // Check that heap-allocated extension would be supported as well.
                .method(http::Method::from_bytes(b"ONE_HELL_LONG_VOROJBA_EXTENSION").unwrap())
                .build()
                .unwrap(),
            HandlerFn::new(extension_second_endpoint),
        )
        .take()
}

async fn get_endpoint(request: Request) -> Result<impl ResponsePart, String> {
    Ok(Json(ResponseBody {
        method: request.content.method().to_string(),
        endpoint: "get_endpoint".to_string(),
    }))
}

async fn post_endpoint(request: Request) -> Result<impl ResponsePart, String> {
    Ok(Json(ResponseBody {
        method: request.content.method().to_string(),
        endpoint: "post_endpoint".to_string(),
    }))
}

async fn extension_first_endpoint(request: Request) -> Result<impl ResponsePart, String> {
    Ok(Json(ResponseBody {
        method: request.content.method().to_string(),
        endpoint: "extension_first_endpoint".to_string(),
    }))
}

async fn extension_second_endpoint(request: Request) -> Result<impl ResponsePart, String> {
    Ok(Json(ResponseBody {
        method: request.content.method().to_string(),
        endpoint: "extension_second_endpoint".to_string(),
    }))
}

#[derive(Serialize, Debug)]
struct ResponseBody {
    method: String,
    endpoint: String,
}
