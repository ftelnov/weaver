use std::{
    collections::HashMap,
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use derive_builder::Builder;
use hyper::{
    body::{Body as HttpBody, Bytes, Frame, Incoming},
    service::service_fn,
    Request as HyperRequest, Response as HyperResponse,
};
use log::{debug, error, info, trace};
use matchit::Router;
use tarantool::{
    fiber::{self},
    network::tcp::{listener::TcpListener, stream::TcpStream},
};

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
        self.into_fiber()
            .defer_non_joinable()
            .map_err(|err| Error::InitFailed(format!("failed to create main fiber: {err}")))?;

        Ok(())
    }

    pub fn into_fiber(self) -> fiber::Builder<impl FnOnce() -> std::result::Result<(), Error>> {
        let fiber_name = self.cfg.fiber_name.unwrap_or_else(|| {
            format!(
                "weaver_http_server_{}_{}",
                self.cfg.bind.host, self.cfg.bind.port
            )
        });
        let bind = self.cfg.bind;

        let processor = ServerProcessor {
            state: Rc::new(ServerState {
                router: self.router,
                server_name: fiber_name.clone(),
            }),
        };

        fiber::Builder::new()
            .name(&fiber_name)
            .func_async(async move {
                let listener = TcpListener::bind(&bind.host, bind.port).map_err(|err| {
                    Error::InitFailed(format!("failed to bind to needed address: {err}"))
                })?;
                info!(
                    "Server bind to address {}:{} successfully",
                    bind.host, bind.port
                );

                loop {
                    let stream = listener
                        .accept()
                        .await
                        .map_err(|err| Error::ConnectionError(err.to_string()))?;

                    debug!("Server accepted new connection");
                    let processor = processor.clone();
                    fiber::Builder::new()
                        .name(&fiber_name)
                        .func_async(async move {
                            if let Err(err) = processor.process_single_stream(stream).await {
                                error!("Failure during single connection stream processing: {err}")
                            }
                        })
                        .defer_non_joinable()
                        .map_err(|err| {
                            Error::ConnectionError(format!(
                                "unable to spawn fiber for connection handle: {err}"
                            ))
                        })?;
                }
            })
    }
}

#[derive(Clone)]
struct ServerProcessor {
    state: Rc<ServerState>,
}

impl ServerProcessor {
    async fn process_single_stream(&self, stream: TcpStream) -> Result<(), Error> {
        let io = TarantoolAsyncIO::new(stream);
        let processor = self.clone();

        let service = service_fn(move |request| {
            trace!(ctx = processor.log_ctx(); "accepted request: {request:?}");
            let processor = processor.clone();
            async move { processor.process_request(request).await }
        });

        hyper_util::server::conn::auto::Builder::new(TarantoolHyperExecutor::new(
            &self.state.server_name,
        ))
        .serve_connection(io, service)
        .await
        .map_err(|err| Error::ServeExited(format!("serve process resulted in error: {err}")))?;
        debug!(ctx = self.log_ctx(); "connection is finished");
        Ok(())
    }

    async fn process_request(&self, request: HyperRequest<Incoming>) -> Result<Response, Error> {
        let path = request.uri().path().to_owned();
        let handler = self.state.router.at(&path)?;
        let params: HashMap<String, String> = handler
            .params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Ok((handler.value.0)(Request {
            content: request,
            params,
        })
        .await)
    }

    fn log_ctx(&self) -> &str {
        &self.state.server_name
    }
}

struct ServerState {
    router: Router<HandlerInternal>,
    server_name: String,
}

pub struct Request {
    pub content: HyperRequest<Incoming>,
    pub params: HashMap<String, String>,
}

impl Deref for Request {
    type Target = HyperRequest<Incoming>;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl DerefMut for Request {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

pub type Response = HyperResponse<Body>;

pub trait RequestHandler {
    fn handle_async(&self, request: Request) -> impl Future<Output = Response>;
}

struct HandlerInternal(
    #[allow(clippy::type_complexity)]
    Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response>>>>,
);

pub struct Body {
    data: Option<Bytes>,
}

impl Body {
    pub fn empty() -> Self {
        Self {
            data: Some(vec![].into()),
        }
    }
}

impl From<String> for Body {
    fn from(a: String) -> Self {
        Body {
            data: Some(a.into()),
        }
    }
}

impl From<Bytes> for Body {
    fn from(a: Bytes) -> Self {
        Body { data: Some(a) }
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
    #[error("user handler resulted in error: {}", .0)]
    UserHandler(String),
    #[error("failed to init server: {}", .0)]
    InitFailed(String),
    #[error("server unexpectedly exited from serving: {}", .0)]
    ServeExited(String),
    #[error("unexpected error occurred with connection: {}", .0)]
    ConnectionError(String),
    #[error("path isn't registered")]
    InvalidPath(#[from] matchit::MatchError),
}
