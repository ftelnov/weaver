/// Macro to implement RequestHandler for async functions with up to 12 arguments
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

            #[allow(unused_parens)]
            impl<F, Fut, Resp, $($arg),*> $crate::server::RequestHandler for $crate::frontend::handler::Handler<F, Fut, Resp, ($($arg),*)>
            where
                F: Fn($($arg),*) -> Fut,
                Fut: std::future::Future<Output = Resp>,
                Resp: ResponsePart,
                $( $arg: FromRequest, )*
            {
                async fn handle_async(&self, request: $crate::server::Request) -> $crate::server::Response {
                    #[allow(unused)]
                    let mut request = $crate::frontend::request::Request::new(request);

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
        }    };
}

// Generate impls for up to 12 arguments
impl_request_handler!();
impl_request_handler!(X);
impl_request_handler!(X, Y);
impl_request_handler!(X, Y, Z);
impl_request_handler!(X, Y, Z, U);
impl_request_handler!(X, Y, Z, U, V);
impl_request_handler!(X, Y, Z, U, V, W);
impl_request_handler!(X, Y, Z, U, V, W, Q);
impl_request_handler!(X, Y, Z, U, V, W, Q, R);
impl_request_handler!(X, Y, Z, U, V, W, Q, R, S);
impl_request_handler!(X, Y, Z, U, V, W, Q, R, S, T);
impl_request_handler!(X, Y, Z, U, V, W, Q, R, S, T, M);
impl_request_handler!(X, Y, Z, U, V, W, Q, R, S, T, M, N);
