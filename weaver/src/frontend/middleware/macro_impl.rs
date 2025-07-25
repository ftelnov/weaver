/// Macro to implement middleware for async functions with up to X arguments.
/// Each argument must implement FromRequest(and take last as any request handler),
/// and the return type must implement ResponsePart.
#[macro_export]
macro_rules! impl_middleware {
    [
        $($arg:ident),*
    ] => {
        paste::paste! {
            mod [<impl_middleware_ $($arg:snake)_*>] {
                use $crate::server::Body;
                use $crate::frontend::response::{ResponsePart};
                #[allow(unused_imports)]
                use $crate::frontend::request::FromRequest;
                use $crate::frontend::middleware::Next;
                use $crate::server::Request;

                #[async_trait::async_trait(?Send)]
                #[allow(unused_parens)]
                impl<FN, Fut, Resp, $($arg),*> $crate::frontend::middleware::Middleware for $crate::frontend::middleware::MiddlewareFn<FN, Fut, Resp, ($($arg),*)>
                where
                    FN: for<'a> Fn($($arg,)* Next) -> Fut,
                    Fut: std::future::Future<Output = Resp>,
                    Resp: ResponsePart,
                    $( $arg: FromRequest, )*
                {
                    #[allow(unused)]
                    async fn process(&self, mut request: Request, next: Next) -> $crate::server::Response {
                        // Apply request extractors.
                        $(
                            #[allow(non_snake_case)]
                            let $arg = match <$arg as FromRequest>::from_request(&mut request).await {
                                Ok(val) => val,
                                Err(rej) => {
                                    let mut response = $crate::server::Response::new(Body::empty());
                                    rej.apply(&mut response).await;
                                    return response;
                                },
                            };
                        )*

                        // Apply response parts.
                        let mut response = $crate::server::Response::new(Body::empty());
                        let parts = (self.func)($($arg,)* next).await;
                        parts.apply(&mut response).await;
                        response
                    }
                }
            }
        }
    };
    [
        $($arg:ident),*; Request
    ] => {
        paste::paste! {
            mod [<impl_middleware_ $($arg:snake)_*with_request>] {
                use $crate::server::Body;
                use $crate::frontend::response::{ResponsePart};
                #[allow(unused_imports)]
                use $crate::frontend::request::FromRequest;
                use $crate::frontend::middleware::Next;
                use $crate::server::Request;

                #[async_trait::async_trait(?Send)]
                #[allow(unused_parens)]
                impl<FN, Fut, Resp, $($arg),*> $crate::frontend::middleware::Middleware for $crate::frontend::middleware::MiddlewareFn<FN, Fut, Resp, ($($arg,)* Request)>
                where
                    FN: for<'a> Fn($($arg,)* Request, Next) -> Fut,
                    Fut: std::future::Future<Output = Resp>,
                    Resp: ResponsePart,
                    $( $arg: FromRequest, )*
                {
                    #[allow(unused)]
                    async fn process(&self, mut request: Request, next: Next) -> $crate::server::Response {
                        // Apply request extractors.
                        $(
                            #[allow(non_snake_case)]
                            let $arg = match <$arg as FromRequest>::from_request(&mut request).await {
                                Ok(val) => val,
                                Err(rej) => {
                                    let mut response = $crate::server::Response::new(Body::empty());
                                    rej.apply(&mut response).await;
                                    return response;
                                },
                            };
                        )*

                        // Apply response parts.
                        let mut response = $crate::server::Response::new(Body::empty());
                        let parts = (self.func)($($arg,)* request, next).await;
                        parts.apply(&mut response).await;
                        response
                    }
                }
            }
        }
    };
}

// Generate impls for up to 12 arguments
impl_middleware!();
impl_middleware!(A);
impl_middleware!(A, B);
impl_middleware!(A, B, C);
impl_middleware!(A, B, C, D);
impl_middleware!(A, B, C, D, E);
impl_middleware!(A, B, C, D, E, F);
impl_middleware!(A, B, C, D, E, F, G);
impl_middleware!(A, B, C, D, E, F, G, H);
impl_middleware!(A, B, C, D, E, F, G, H, I);
impl_middleware!(A, B, C, D, E, F, G, H, I, J);
impl_middleware!(A, B, C, D, E, F, G, H, I, J, K);
impl_middleware!(A, B, C, D, E, F, G, H, I, J, K, L);

impl_middleware!(; Request);
impl_middleware!(A; Request);
impl_middleware!(A, B; Request);
impl_middleware!(A, B, C; Request);
impl_middleware!(A, B, C, D; Request);
impl_middleware!(A, B, C, D, E; Request);
impl_middleware!(A, B, C, D, E, F; Request);
impl_middleware!(A, B, C, D, E, F, G; Request);
impl_middleware!(A, B, C, D, E, F, G, H; Request);
impl_middleware!(A, B, C, D, E, F, G, H, I; Request);
impl_middleware!(A, B, C, D, E, F, G, H, I, J; Request);
impl_middleware!(A, B, C, D, E, F, G, H, I, J, K; Request);
impl_middleware!(A, B, C, D, E, F, G, H, I, J, K, L; Request);
