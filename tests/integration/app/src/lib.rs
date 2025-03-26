
use http_body_util::BodyExt as _;
use hyper::{body::Incoming, Request, Response};
use tarantool::log::TarantoolLogger;
use weaver::server::{
    BindParams, Body, RequestHandler, Server, ServerConfigBuilder,
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
    server.route("/echo", MirrorEndpoint).unwrap();
    server.into_fiber().start().unwrap().join().unwrap();
    Ok(())
}

struct MirrorEndpoint;

#[async_trait::async_trait]
impl RequestHandler for MirrorEndpoint {
    type Error = anyhow::Error;

    async fn handle_async(
        &self,
        request: Request<Incoming>,
    ) -> Result<Response<Body>, Self::Error> {
        let content = request.collect().await?.to_bytes();
        let body = Body::from(String::from_utf8(content.to_vec())?);
        Ok(Response::new(body))
    }
}
