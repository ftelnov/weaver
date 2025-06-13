use http_body_util::BodyExt as _;
use hyper::{header::HeaderValue, HeaderMap, StatusCode};
use std::time::Duration;
use tarantool::{fiber, log::TarantoolLogger};
use weaver::{
    frontend::{
        extras::json::Json,
        handler::HandlerFn,
        request::{path::Path, Request},
        response::{Extend, ResponsePart},
    },
    server::{BindParams, Body, Server, ServerConfigBuilder},
};

pub mod middleware;

#[tarantool::proc]
pub fn run_server(_input: String) -> Result<(), String> {
    tarolog::set_default_logger_format(tarolog::Format::JsonTarantool(None));
    static LOGGER: TarantoolLogger = TarantoolLogger::new();

    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
    _run_server()
}

fn _run_server() -> Result<(), String> {
    let mut server = Server::new(
        ServerConfigBuilder::default()
            .bind(BindParams {
                host: "127.0.0.1".into(),
                port: 18989,
            })
            .build()
            .unwrap(),
    );
    server
        .route("/echo", HandlerFn::new(echo_endpoint))
        .unwrap();
    server
        .route("/json", HandlerFn::new(json_endpoint))
        .unwrap();
    server
        .route("/extend", HandlerFn::new(extend_endpoint))
        .unwrap();
    server
        .route("/long-running", HandlerFn::new(long_running_endpoint))
        .unwrap();
    server
        .route(
            "/path/{id}/content/{another_field}/{final_field}",
            HandlerFn::new(path_endpoint),
        )
        .unwrap();

    server.group(middleware::simple::group()).unwrap();
    server.group(middleware::layer::group()).unwrap();

    server.into_fiber().start().unwrap().join().unwrap();
    Ok(())
}

async fn echo_endpoint(mut request: Request) -> Result<impl ResponsePart, String> {
    let content = request
        .body_mut()
        .collect()
        .await
        .map_err(|err| format!("failed to extract body: {err}"))?
        .to_bytes();
    let body = Body::from(
        String::from_utf8(content.to_vec())
            .map_err(|err| format!("failed to convert body to string: {err}"))?,
    );
    Ok(body)
}

async fn json_endpoint(Json(value): Json<serde_json::Value>) -> impl ResponsePart {
    Json(value)
}

/// Demonstrates how to use [Extend] to merge response parts.
async fn extend_endpoint(
    Json(value): Json<serde_json::Value>,
) -> Result<impl ResponsePart, String> {
    let mut first_headers = HeaderMap::new();
    first_headers.insert("X-Header-1", "header-1".parse().unwrap());
    first_headers.insert("X-Header-2", "header-2".parse().unwrap());

    let mut second_headers = HeaderMap::new();
    second_headers.insert("X-Header-1", "header-1-2".parse().unwrap());
    second_headers.insert("X-Header-3", "header-3".parse().unwrap());
    second_headers.insert("X-Header-4", "header-4".parse().unwrap());

    Ok((
        StatusCode::CREATED,
        first_headers,
        Json(value),
        Extend(second_headers),
        ("X-Header-5", "header-5".parse().unwrap()),
        [
            ("X-Header-6", HeaderValue::from_static("header-6")),
            ("X-Header-4", HeaderValue::from_static("header-4-1")),
        ],
    ))
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct LongRunResponse {
    request: serde_json::Value,
    #[serde(serialize_with = "serialize_datetime")]
    handle_start: chrono::DateTime<chrono::Utc>,
    #[serde(serialize_with = "serialize_datetime")]
    handle_end: chrono::DateTime<chrono::Utc>,
}

fn serialize_datetime<S>(
    datetime: &chrono::DateTime<chrono::Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let timestamp = datetime.timestamp_millis();
    serializer.serialize_i64(timestamp)
}

async fn long_running_endpoint(
    Json(value): Json<serde_json::Value>,
) -> Result<Json<LongRunResponse>, String> {
    let start = chrono::Utc::now();
    fiber::sleep(Duration::from_secs(1));
    let end = chrono::Utc::now();
    Ok(Json(LongRunResponse {
        request: value,
        handle_start: start,
        handle_end: end,
    }))
}

async fn path_endpoint(Path(path): Path) -> Result<Json<serde_json::Value>, String> {
    Ok(Json(serde_json::json!(path)))
}
