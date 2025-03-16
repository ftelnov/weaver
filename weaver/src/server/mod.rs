use std::{
    fmt::Display,
    future::Future,
    net::{IpAddr, SocketAddr, TcpListener},
    pin::{pin, Pin},
    rc::Rc,
    task::{Context, Poll},
};

use anyhow::Context as _;
use derive_builder::Builder;
use http::response::Parts;
use hyper::{
    body::{Body as HttpBody, Bytes, Frame, Incoming},
    service::service_fn,
    Request, Response, StatusCode,
};
use matchit::Router;
use serde::Serialize;
use tarantool::{fiber, network::client::tcp::TcpStream};

use crate::runtime::{TarantoolAsyncIO, TarantoolHyperExecutor};

#[derive(Debug, Clone, Builder)]
pub struct ServerConfig {
    pub bind: BindParams,
    #[builder(default)]
    pub fiber_name: Option<String>,
}

#[derive(Debug, Clone, Builder)]
pub struct BindParams {
    pub host: String,
    pub port: u16,
}

pub struct Server {
    cfg: ServerConfig,
    router: Router<HandlerInternal>,
}

impl Server {
    pub fn new(cfg: ServerConfig) -> Self {
        Self {
            cfg,
            router: Router::new(),
        }
    }

    pub fn route(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> Result<&mut Self, Error> {
        let handler = Rc::new(handler);
        self.router
            .insert(
                path,
                HandlerInternal(Box::new(move |request| {
                    let handler = handler.clone();

                    Box::pin(async move { handler.handle_async(request).await })
                })),
            )
            .map_err(|err| Error::InitFailed(err.to_string()))?;
        Ok(self)
    }

    pub fn defer(self) -> Result<(), Error> {
        let fiber_name = self.cfg.fiber_name.unwrap_or_else(|| {
            format!(
                "weaver_http_server_{}_{}",
                self.cfg.bind.host, self.cfg.bind.port
            )
        });
        let bind = self.cfg.bind;

        let processor = Rc::new(ServerProcessor {
            router: self.router,
        });

        fiber::Builder::new()
            .name(&fiber_name)
            .func_async(async move {
                let stream = TcpStream::connect_async(&bind.host, bind.port).await?;
                let io = TarantoolAsyncIO::new(stream);

                let service = service_fn(move |request| {
                    let processor = processor.clone();
                    async move { processor.process_request(request).await }
                });

                hyper::server::conn::http2::Builder::new(TarantoolHyperExecutor::new(fiber_name))
                    .serve_connection(io, service)
                    .await
                    .context("error serving connection")?;

                Result::<(), anyhow::Error>::Err(anyhow::anyhow!(
                    "serve process is unexpectedly halted"
                ))
            })
            .defer_non_joinable()
            .map_err(|err| Error::InitFailed(format!("failed to create main fiber: {err}")))?;

        Ok(())
    }
}

struct ServerProcessor {
    router: Router<HandlerInternal>,
}

impl ServerProcessor {
    async fn process_request(&self, request: Request<Incoming>) -> Result<Response<Body>, Error> {
        let handler = self.router.at(request.uri().path())?;
        (handler.value.0)(request).await
    }
}

#[async_trait::async_trait]
pub trait RequestHandler {
    async fn handle_async(&self, request: Request<Incoming>) -> Result<Response<Body>, Error>;
}

struct HandlerInternal(
    Box<dyn Fn(Request<Incoming>) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Error>>>>>,
);

struct Body {
    data: Option<Bytes>,
}

impl From<String> for Body {
    fn from(a: String) -> Self {
        Body {
            data: Some(a.into()),
        }
    }
}

impl HttpBody for Body {
    type Data = Bytes;
    type Error = Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Poll::Ready(self.get_mut().data.take().map(|d| Ok(Frame::data(d))))
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("failed to init server: {}", .0)]
    InitFailed(String),
    #[error("path isn't registered")]
    InvalidPath(#[from] matchit::MatchError),
}

impl Error {
    pub fn code(&self) -> ErrorCode {
        match self {
            Error::InitFailed(_) => 1,
            Error::InvalidPath(_) => 2,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidPath(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<Error> for Response<Body> {
    fn from(err: Error) -> Self {
        let mut response = Self::new(Body::from(
            serde_json::to_string(&ErrorResponse {
                code: err.code(),
                details: err.to_string(),
            })
            .expect("error serialization should never fail"),
        ));
        *response.status_mut() = err.status_code();
        response
    }
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    code: ErrorCode,
    details: String,
}

pub type ErrorCode = u32;
