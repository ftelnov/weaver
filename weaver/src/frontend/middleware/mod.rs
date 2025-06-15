use crate::server::{Request, RequestHandler, Response, SharedRequestHandler};
use std::{marker::PhantomData, rc::Rc};

mod macro_impl;

#[async_trait::async_trait(?Send)]
pub trait Middleware {
    async fn process(&self, request: Request, next: Next) -> Response;
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
    handler: SharedRequestHandler,
}

impl Next {
    pub async fn call(&self, request: Request) -> Response {
        self.handler.as_handler().handle_async(request).await
    }
}

impl From<SharedRequestHandler> for Next {
    fn from(handler: SharedRequestHandler) -> Self {
        Next { handler }
    }
}

impl From<Next> for SharedRequestHandler {
    fn from(next: Next) -> Self {
        next.handler
    }
}

#[derive(Clone)]
pub struct SharedMiddleware(Rc<dyn Middleware>);

impl SharedMiddleware {
    pub fn new(middleware: impl Middleware + 'static) -> Self {
        Self(Rc::new(middleware))
    }

    pub fn as_middleware(&self) -> &dyn Middleware {
        &*self.0
    }
}

impl<T: Middleware + 'static> From<T> for SharedMiddleware {
    fn from(middleware: T) -> Self {
        Self::new(middleware)
    }
}

impl SharedMiddleware {
    pub fn wrap(self, next: Next) -> Next {
        struct Wrapper {
            middleware: SharedMiddleware,
            next: Next,
        }

        #[async_trait::async_trait(?Send)]
        impl RequestHandler for Wrapper {
            async fn handle_async(&self, request: Request) -> crate::server::Response {
                self.middleware
                    .as_middleware()
                    .process(request, self.next.clone())
                    .await
            }
        }

        Next {
            handler: SharedRequestHandler::new(Wrapper {
                middleware: self,
                next,
            }),
        }
    }
}

impl Default for SharedMiddleware {
    fn default() -> Self {
        struct DefaultMiddleware;

        #[async_trait::async_trait(?Send)]
        impl Middleware for DefaultMiddleware {
            async fn process(&self, request: Request, next: Next) -> Response {
                next.call(request).await
            }
        }

        Self::new(DefaultMiddleware)
    }
}
