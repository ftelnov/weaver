use http_body_util::BodyExt as _;
use std::time::Duration;
use tarantool::{fiber, log::TarantoolLogger};
use weaver::{
    frontend::{
        handler::Handler,
        request::{json::JsonBody, path::Path, RawRequest},
        response::json::JsonResponse,
    },
    server::{BindParams, Body, Response, Server, ServerConfigBuilder},
};

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
        .route("/echo", Handler::new(mirror_endpoint))
        .unwrap();
    server.route("/json", Handler::new(json_endpoint)).unwrap();
    server
        .route("/long-running", Handler::new(long_running_endpoint))
        .unwrap();
    server
        .route(
            "/path/{id}/content/{another_field}/{final_field}",
            Handler::new(path_endpoint),
        )
        .unwrap();
    server.into_fiber().start().unwrap().join().unwrap();
    Ok(())
}

async fn mirror_endpoint(RawRequest(mut request): RawRequest) -> Result<Response, String> {
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
    Ok(Response::new(body))
}

async fn json_endpoint(
    JsonBody(value): JsonBody<serde_json::Value>,
) -> Result<JsonResponse<serde_json::Value>, String> {
    Ok(JsonResponse(value))
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
    JsonBody(value): JsonBody<serde_json::Value>,
) -> Result<JsonResponse<LongRunResponse>, String> {
    let start = chrono::Utc::now();
    fiber::sleep(Duration::from_secs(1));
    let end = chrono::Utc::now();
    Ok(JsonResponse(LongRunResponse {
        request: value,
        handle_start: start,
        handle_end: end,
    }))
}

async fn path_endpoint(Path(path): Path) -> Result<JsonResponse<serde_json::Value>, String> {
    Ok(JsonResponse(serde_json::json!(path)))
}
