# Weaver

Modern HTTP-server for [Picodata](https://docs.picodata.io/picodata) (Rust in-memory DBMS).

You can use it to define HTTP API for your [Picodata plugins](https://lib.rs/crates/picodata-plugin).

It's API is inspired by [`axum`](https://github.com/tokio-rs/axum) and features declarative style.

## Quick start

Add `weaver` to your plugin's `Cargo.toml`:

```toml
weaver = { git = "https://github.com/ftelnov/weaver.git", features = ["json"] }
```

Define your API endpoints and launch a server instance:

```rust
fn run_server() {
    let mut server = Server::new(
        ServerConfigBuilder::default()
            .bind(BindParams {
                host: "127.0.0.1".into(),
                port: 8000,
            })
            .build()
            .unwrap(),
    );
    server.post("/echo/json", HandlerFn::new(echo_json_endpoint)).unwrap();
    server.into_fiber().start().unwrap().join().unwrap();
}

async fn echo_json_endpoint(
    Json(body): Json<serde_json::Value>
) -> impl ResponsePart {
    (Json(body), http::StatusCode::CREATED)
}
```

Endpoint logic is simple: parse request's body as a `json`, serialize it back to `json` response with `201` status code.

`weaver` does this underneath:
1. Uses `Json` extractor to parse Request's body as a json and extract `serde`-compatible type from it(in this case, `serde_json::Value`);
2. Constructs response part by part. It uses `Json` to serialize `serde`-compatible type and provide body, then sets response's status code from `StatusCode` enum.

## Architecture

Design aim of `weaver` is to minimize the maintanance effort.
It is achieved by leveraging well-known [hyper](https://github.com/hyperium/hyper) framework for handling HTTP proto.

`weaver` simply integrates `hyper` with `tarantool` I/O system and provides declarative single-threaded API.

## Contributing

If you want to ask a question, use Github discussion or contact [author](https://github.com/ftelnov).

If you want to contribute, feel free to submit PR or issue.

PR must be approved by maintainers and pass CI pipeline to be merged.
