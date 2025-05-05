use bench_helper::{HealthResponse, TestRequest, TestResponse};
use shors::{
    tarantool::tlua,
    transport::{
        http::{route::Builder, Request, Response},
        Context,
    },
};
use std::{collections::HashMap, error::Error, ffi::c_int, time::Duration};
use tarantool::ffi::lua as ffi_lua;
use tarantool::fiber;

#[tarantool::proc]
pub fn run_server(_input: String) -> Result<(), String> {
    bench_helper::setup_logger();

    let lua = tarantool::lua_state();
    // This forces shors to avoid cartridge dependency and use server defined in _G.pico
    lua.exec(
        r#"
        local port = tonumber(os.getenv("PORT")) or 19000
        rawset(_G, "pico", {httpd = require('http.server').new('127.0.0.1', port, {log_requests = false})})
        "#,
    )
    .unwrap();

    _run_server()?;
    lua.exec(
        r#"
        rawget(_G, "pico").httpd:start()
        "#,
    )
    .unwrap();
    // Sleep forever to keep the server running
    fiber::sleep(Duration::MAX);
    Ok(())
}

fn _run_server() -> Result<(), String> {
    let s = shors::transport::http::server::Server::new();

    let test_endpoint = Builder::new()
        .with_method("POST")
        .with_path("/test/:param_a/subcommand/:param_b")
        .build(
            |_ctx: &mut Context, request: Request| -> Result<_, Box<dyn Error>> {
                let body: TestRequest = serde_json::from_slice(&request.body)?;
                let path = request.stash;

                Ok(Response {
                    status: 200,
                    headers: HashMap::from([(
                        "content-type".to_string(),
                        "application/json".to_string(),
                    )]),
                    body: serde_json::to_vec(&TestResponse {
                        request: body,
                        path,
                    })?,
                })
            },
        );

    let health_endpoint = Builder::new()
        .with_method("GET")
        .with_path("/health")
        .build(
            |_ctx: &mut Context, _request: Request| -> Result<_, Box<dyn Error>> {
                Ok(Response {
                    status: 200,
                    headers: HashMap::new(),
                    body: serde_json::to_vec(&HealthResponse::default())?,
                })
            },
        );

    s.register(Box::new(test_endpoint));
    s.register(Box::new(health_endpoint));

    Ok(())
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn luaopen_libshors_bench(l: *mut ffi_lua::lua_State) -> c_int {
    let lua = tlua::StaticLua::from_static(l);
    shors::init_lua_functions(&lua).unwrap();
    1
}
