use std::marker::PhantomData;

/// Handler type that stores the user function/closure.
pub struct Handler<F, Fut, Resp, Args> {
    pub(crate) func: F,
    pub(crate) phantom: PhantomData<(Fut, Resp, Args)>,
}

impl<F, Fut, Resp, Args> Handler<F, Fut, Resp, Args> {
    pub fn new(func: F) -> Self {
        Handler {
            func,
            phantom: PhantomData,
        }
    }
}
