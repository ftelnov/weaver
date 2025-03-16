use http_body_util::BodyExt as _;
use hyper::{body::Incoming, Request, Response};
use weaver::server::{
    BindParams, Body, Error, RequestHandler, Server, ServerConfig, ServerConfigBuilder,
};

#[tarantool::proc]
pub fn run_server(_input: String) -> Result<(), String> {
    _run_server()
}

fn _run_server() -> Result<(), String> {
    let mut server = Server::new(
        ServerConfigBuilder::default()
            .bind(BindParams {
                host: "localhost".into(),
                port: 18989,
            })
            .build()
            .unwrap(),
    );
    server.route("/mirror", MirrorEndpoint).unwrap();
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
