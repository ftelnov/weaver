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

use crate::{
    runtime::{TarantoolAsyncIO, TarantoolHyperExecutor},
    utils::SmallMap,
};
use http::StatusCode;

#[derive(Debug, Clone, Builder, Default)]
pub struct ServerConfig {
    #[builder(default)]
    pub bind: BindParams,
    /// Server name, used for logging and fiber name.
    /// If not provided, default name with host and port will be used.
    #[builder(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Builder)]
pub struct BindParams {
    pub host: String,
    pub port: u16,
}

impl Default for BindParams {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8000,
        }
    }
}
pub struct Server {
    cfg: ServerConfig,
    name: String,
    router: InnerRouter,
}

type InnerRouter = Router<SmallMap<http::Method, HandlerInternal>>;

impl Server {
    pub fn new(cfg: ServerConfig) -> Self {
        let name = cfg
            .name
            .clone()
            .unwrap_or_else(|| format!("weaver_http_server_{}_{}", cfg.bind.host, cfg.bind.port));
        Self {
            cfg,
            name,
            router: Router::new(),
        }
    }

    /// Register new handler for given route.
    /// If route is already registered(both method and path are occupied), error would be returned.
    pub fn route(
        &mut self,
        route: Route,
        handler: impl RequestHandler + 'static,
    ) -> Result<&mut Self, Error> {
        let handler = Rc::new(handler);
        let mut bucket = self.router.remove(&route.path).unwrap_or_default();

        let existing = bucket.insert(
            route.method.clone(),
            HandlerInternal(Box::new(move |request| {
                let handler = handler.clone();
                Box::pin(async move { handler.handle_async(request).await })
            })),
        );

        if existing.is_some() {
            return Err(Error::RouteOccupied {
                path: route.path,
                method: route.method,
            });
        }

        self.router
            .insert(&route.path, bucket)
            .map_err(|err| Error::InvalidPath {
                path: route.path.clone(),
                error: err,
            })?;

        debug!(ctx = self.log_ctx(); "registering handler for path: {route:?}");

        Ok(self)
    }

    pub fn defer(self) -> Result<(), Error> {
        self.into_fiber()
            .defer_non_joinable()
            .map_err(|err| Error::InitFailed(format!("failed to create main fiber: {err}")))?;

        Ok(())
    }

    pub fn into_fiber(self) -> fiber::Builder<impl FnOnce() -> std::result::Result<(), Error>> {
        let bind = self.cfg.bind;
        let fiber_name = self.name;

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

    fn log_ctx(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Default, Builder, Debug)]
pub struct Route {
    pub path: String,
    #[builder(default)]
    pub method: http::Method,
}

impl Route {
    pub fn new(path: impl Into<String>, method: http::Method) -> Self {
        Self {
            path: path.into(),
            method,
        }
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    pub fn method(mut self, method: http::Method) -> Self {
        self.method = method;
        self
    }
}

/// Convenient methods for quick route registration.
impl Server {
    pub fn get(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> Result<&mut Self, crate::server::Error> {
        self.route(Route::new(path, http::Method::GET), handler)
    }

    pub fn post(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> Result<&mut Self, crate::server::Error> {
        self.route(Route::new(path, http::Method::POST), handler)
    }

    pub fn put(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> Result<&mut Self, crate::server::Error> {
        self.route(Route::new(path, http::Method::PUT), handler)
    }

    pub fn patch(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> Result<&mut Self, crate::server::Error> {
        self.route(Route::new(path, http::Method::PATCH), handler)
    }

    pub fn delete(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> Result<&mut Self, crate::server::Error> {
        self.route(Route::new(path, http::Method::DELETE), handler)
    }

    pub fn connect(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> Result<&mut Self, crate::server::Error> {
        self.route(Route::new(path, http::Method::CONNECT), handler)
    }

    pub fn head(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> Result<&mut Self, crate::server::Error> {
        self.route(Route::new(path, http::Method::HEAD), handler)
    }

    pub fn options(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> Result<&mut Self, crate::server::Error> {
        self.route(Route::new(path, http::Method::OPTIONS), handler)
    }

    pub fn trace(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> Result<&mut Self, crate::server::Error> {
        self.route(Route::new(path, http::Method::TRACE), handler)
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
            async move {
                processor
                    .process_request(request)
                    .await
                    .or_else(|err| processor.handle_error(err))
            }
        });

        hyper_util::server::conn::auto::Builder::new(TarantoolHyperExecutor::new(
            &self.state.server_name,
        ))
        .serve_connection(io, service)
        .await
        .map_err(|err| {
            Error::ServeExited(format!(
                "serve process resulted in error: {}",
                error_with_causes(err)
            ))
        })?;
        debug!(ctx = self.log_ctx(); "connection is finished");
        Ok(())
    }

    async fn process_request(&self, request: HyperRequest<Incoming>) -> Result<Response, Error> {
        let bucket = self
            .state
            .router
            .at(request.uri().path())
            .map_err(|_| Error::NotFound)?;

        let handler = bucket
            .value
            .get(request.method())
            .ok_or(Error::MethodNotAllowed)?;

        let params: HashMap<String, String> = bucket
            .params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Ok((handler.0)(Request {
            content: request,
            params,
        })
        .await)
    }

    fn handle_error(&self, error: Error) -> Result<Response, Error> {
        match error {
            err @ Error::NotFound => {
                let mut response = Response::new(err.to_string().into());
                *response.status_mut() = StatusCode::NOT_FOUND;
                Ok(response)
            }
            err @ Error::MethodNotAllowed => {
                let mut response = Response::new(err.to_string().into());
                *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
                Ok(response)
            }
            e => Err(e),
        }
    }

    fn log_ctx(&self) -> &str {
        &self.state.server_name
    }
}

struct ServerState {
    router: InnerRouter,
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

impl<F: Future<Output = Response> + 'static, FN: Fn(Request) -> F> RequestHandler for FN {
    fn handle_async(&self, request: Request) -> impl Future<Output = Response> {
        (self)(request)
    }
}

struct HandlerInternal(
    #[allow(clippy::type_complexity)]
    Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response>>>>,
);

#[derive(Debug, Clone)]
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
    #[error("failed to init server: {}", .0)]
    InitFailed(String),
    #[error("server unexpectedly exited from serving: {}", .0)]
    ServeExited(String),
    #[error("unexpected error occurred with connection: {}", .0)]
    ConnectionError(String),
    #[error("404 Not Found")]
    NotFound,
    #[error("405 Method Not Allowed")]
    MethodNotAllowed,
    #[error("invalid path, unable to register: {path}: {error}")]
    InvalidPath {
        path: String,
        #[source]
        error: matchit::InsertError,
    },
    #[error("route {path} is already occupied for method {method}")]
    RouteOccupied { path: String, method: http::Method },
}

/// Format error as a string with sources traversal.
/// Mainly used for extreme cases - when failure is unexpected.
fn error_with_causes(error: Box<dyn std::error::Error>) -> String {
    let mut error: &dyn std::error::Error = &*error;
    let mut result = error.to_string();
    while let Some(source) = error.source() {
        result.push_str(" -> ");
        result.push_str(&source.to_string());
        error = source;
    }
    result
}
