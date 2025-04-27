use http_body_util::BodyExt as _;
use tarantool::log::TarantoolLogger;
use weaver::{
    frontend::{handler::Handler, request::RawRequest},
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
