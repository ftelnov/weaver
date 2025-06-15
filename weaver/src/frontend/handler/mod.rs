use std::marker::PhantomData;

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
