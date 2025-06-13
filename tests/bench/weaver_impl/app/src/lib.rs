use bench_helper::{HealthResponse, TestRequest, TestResponse};
use weaver::{
    frontend::{extras::json::Json, handler::HandlerFn, request::path::Path},
    server::{BindParams, Server, ServerConfigBuilder},
};

#[tarantool::proc]
pub fn run_server(_input: String) -> Result<(), String> {
    bench_helper::setup_logger();
    _run_server()
}

fn _run_server() -> Result<(), String> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "19000".to_string());
    let mut server = Server::new(
        ServerConfigBuilder::default()
            .bind(BindParams {
                host: "127.0.0.1".into(),
                port: port.parse().unwrap(),
            })
            .build()
            .unwrap(),
    );
    server
        .route(
            "/test/{param_a}/subcommand/{param_b}",
            HandlerFn::new(test_endpoint),
        )
        .unwrap();
    server
        .route("/health", HandlerFn::new(health_endpoint))
        .unwrap();
    server.into_fiber().start().unwrap().join().unwrap();
    Ok(())
}

async fn health_endpoint() -> Result<Json<HealthResponse>, String> {
    Ok(Json(HealthResponse::default()))
}

async fn test_endpoint(
    Json(value): Json<TestRequest>,
    Path(path): Path,
) -> Result<Json<TestResponse>, String> {
    Ok(Json(TestResponse {
        request: value,
        path,
    }))
}
