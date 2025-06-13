use super::{handler::DynHandler, request::Request, response::ResponsePart};
use crate::server::{Body, Response};
use std::{future::Future, marker::PhantomData, pin::Pin, rc::Rc};

mod macro_impl;

pub trait Middleware {
    fn process(&self, request: &mut Request, next: Next) -> impl Future<Output = Response>;
}

/// Middleware takes [super::response::ResponsePart]
pub struct MiddlewareFn<F, Fut, Resp, Args> {
    pub(crate) func: F,
    pub(crate) phantom: PhantomData<(Fut, Resp, Args)>,
}

impl<F, Fut, Resp, Args> MiddlewareFn<F, Fut, Resp, Args> {
    pub fn new(func: F) -> Self {
        MiddlewareFn {
            func,
            phantom: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct Next {
    handler: DynHandler,
}

impl Next {
    pub async fn call(&self, request: Request) -> Response {
        self.handler.call(request).await
    }
}

impl From<DynHandler> for Next {
    fn from(handler: DynHandler) -> Self {
        Next { handler }
    }
}

#[derive(Clone)]
pub(crate) struct DynMiddleware(
    #[allow(clippy::type_complexity)]
    Rc<dyn Fn(Request, Next) -> Pin<Box<dyn Future<Output = Response>>>>,
);

impl DynMiddleware {
    pub fn chain<M: Middleware + 'static>(self, middleware: M) -> Self {
        let middleware = Rc::new(middleware);
        Self(Rc::new(move |mut request, next| {
            let middleware = middleware.clone();
            Box::pin(async move {
                let middleware = middleware.process(&mut request, next);
                let mut response = Response::new(Body::empty());
                let parts = middleware.await;
                parts.apply(&mut response).await;
                response
            })
        }))
    }

    pub fn wrap(&self, next: Next) -> Next {
        let middleware = self.clone();

        Next {
            handler: DynHandler::new(move |request: crate::server::Request| {
                let next = next.clone();
                let middleware = middleware.clone();

                Box::pin(async move {
                    let request = Request::from(request);
                    middleware.call(request, next).await
                })
            }),
        }
    }

    pub async fn call(&self, request: Request, next: Next) -> Response {
        self.0(request, next).await
    }
}

impl Default for DynMiddleware {
    fn default() -> Self {
        Self(Rc::new(move |request, next| {
            Box::pin(async move { next.call(request).await })
        }))
    }
}
