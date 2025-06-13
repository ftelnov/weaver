use super::request::Request;
use crate::server::{RequestHandler, Response};
use std::{future::Future, marker::PhantomData, pin::Pin, rc::Rc};

mod macro_impl;

/// Handler type that stores the user function/closure.
pub struct HandlerFn<F, Fut, Resp, Args> {
    pub(crate) func: F,
    pub(crate) phantom: PhantomData<(Fut, Resp, Args)>,
}

impl<F, Fut, Resp, Args> HandlerFn<F, Fut, Resp, Args> {
    pub fn new(func: F) -> Self {
        HandlerFn {
            func,
            phantom: PhantomData,
        }
    }
}

#[derive(Clone)]
pub(crate) struct DynHandler(
    #[allow(clippy::type_complexity)]
    Rc<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response>>>>,
);

impl DynHandler {
    pub fn new(handler: impl RequestHandler + 'static) -> Self {
        let handler = Rc::new(handler);
        DynHandler(Rc::new(move |mut request| {
            let handler = handler.clone();
            Box::pin(async move { handler.handle_async(request.take()).await })
        }))
    }

    pub async fn call(&self, request: Request) -> Response {
        self.0(request).await
    }
}
