/// Macro to implement RequestHandler for async functions with up to X arguments
/// Each argument must implement FromRequest, and the return type must implement ResponsePart.
#[macro_export]
macro_rules! impl_request_handler {
    [
        $($arg:ident),*
    ] => {
        paste::paste! {
            mod [<impl_request_handler_ $($arg:snake)_*>] {
                use $crate::server::Body;
                use $crate::frontend::response::{ResponsePart};
                #[allow(unused_imports)]
                use $crate::frontend::request::FromRequest;

                #[async_trait::async_trait(?Send)]
                #[allow(unused_parens)]
                impl<FN, Fut, Resp, $($arg),*> $crate::server::RequestHandler for $crate::frontend::handler::HandlerFn<FN, Fut, Resp, ($($arg),*)>
                where
                    FN: Fn($($arg),*) -> Fut,
                    Fut: std::future::Future<Output = Resp>,
                    Resp: ResponsePart,
                    $( $arg: FromRequest, )*
                {
                    #[allow(unused)]
                    async fn handle_async(&self, mut request: $crate::server::Request) -> $crate::server::Response {
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
                        let parts = (self.func)($($arg),*).await;
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
            mod [<impl_request_handler_ $($arg:snake)_*with_request>] {
                use $crate::server::Body;
                use $crate::frontend::response::{ResponsePart};
                #[allow(unused_imports)]
                use $crate::frontend::request::FromRequest;
                use $crate::server::Request;

                #[async_trait::async_trait(?Send)]
                #[allow(unused_parens)]
                impl<FN, Fut, Resp, $($arg),*> $crate::server::RequestHandler for $crate::frontend::handler::HandlerFn<FN, Fut, Resp, ($($arg,)* Request)>
                where
                    FN: Fn($($arg,)* Request) -> Fut,
                    Fut: std::future::Future<Output = Resp>,
                    Resp: ResponsePart,
                    $( $arg: FromRequest, )*
                {
                    #[allow(unused)]
                    async fn handle_async(&self, mut request: Request) -> $crate::server::Response {
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
                        let parts = (self.func)($($arg,)* request).await;
                        parts.apply(&mut response).await;
                        response
                    }
                }
            }
        }
    };
}

// Generate impls for up to 12 arguments
impl_request_handler!();
impl_request_handler!(A);
impl_request_handler!(A, B);
impl_request_handler!(A, B, C);
impl_request_handler!(A, B, C, D);
impl_request_handler!(A, B, C, D, E);
impl_request_handler!(A, B, C, D, E, F);
impl_request_handler!(A, B, C, D, E, F, G);
impl_request_handler!(A, B, C, D, E, F, G, H);
impl_request_handler!(A, B, C, D, E, F, G, H, I);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J, K);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L);

impl_request_handler!(; Request);
impl_request_handler!(A; Request);
impl_request_handler!(A, B; Request);
impl_request_handler!(A, B, C; Request);
impl_request_handler!(A, B, C, D; Request);
impl_request_handler!(A, B, C, D, E; Request);
impl_request_handler!(A, B, C, D, E, F; Request);
impl_request_handler!(A, B, C, D, E, F, G; Request);
impl_request_handler!(A, B, C, D, E, F, G, H; Request);
impl_request_handler!(A, B, C, D, E, F, G, H, I; Request);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J; Request);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J, K; Request);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L; Request);
