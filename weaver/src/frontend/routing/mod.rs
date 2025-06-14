use super::{middleware::Middleware, request::Request};
use crate::{
    frontend::{
        handler::DynHandler,
        middleware::{DynMiddleware, Next},
    },
    server::{RequestHandler, Route, Server},
};

#[derive(Clone)]
pub struct Group {
    base_path: String,
    middleware: DynMiddleware,
    routes: Vec<InnerRoute>,
}

impl Group {
    pub fn new() -> Self {
        Self {
            base_path: String::new(),
            middleware: Default::default(),
            routes: Vec::new(),
        }
    }

    pub fn path(&mut self, path: impl Into<String>) -> &mut Self {
        self.base_path = path.into();
        self
    }

    pub fn middleware(&mut self, middleware: impl Middleware + 'static) -> &mut Self {
        self.middleware = std::mem::take(&mut self.middleware).chain(middleware);
        self
    }

    pub fn route(&mut self, route: Route, handler: impl RequestHandler + 'static) -> &mut Self {
        self.routes.push(InnerRoute {
            route,
            handler: Next::from(DynHandler::new(handler)),
        });
        self
    }

    /// Consume `group` and extend `self` with its routes.
    /// Group part is consumed, would be default after this call.
    ///
    /// Made as flexible as possible to allow easier nesting and chaining.
    pub fn group(
        &mut self,
        mut group: impl AsMut<Group>,
    ) -> Result<&mut Self, crate::server::Error> {
        let group = group.as_mut().take();
        self.routes
            .extend(group.routes.into_iter().map(|mut route| {
                route.route.path = concat_path(&group.base_path, &route.route.path);
                route.handler = group.middleware.wrap(route.handler.clone());
                route
            }));
        Ok(self)
    }

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

/// Convenient methods for quick route registration.
impl Group {
    pub fn get(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> &mut Self {
        self.route(Route::new(path, http::Method::GET), handler)
    }

    pub fn post(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> &mut Self {
        self.route(Route::new(path, http::Method::POST), handler)
    }

    pub fn put(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> &mut Self {
        self.route(Route::new(path, http::Method::PUT), handler)
    }

    pub fn patch(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> &mut Self {
        self.route(Route::new(path, http::Method::PATCH), handler)
    }

    pub fn delete(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> &mut Self {
        self.route(Route::new(path, http::Method::DELETE), handler)
    }

    pub fn connect(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> &mut Self {
        self.route(Route::new(path, http::Method::CONNECT), handler)
    }

    pub fn head(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> &mut Self {
        self.route(Route::new(path, http::Method::HEAD), handler)
    }

    pub fn options(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> &mut Self {
        self.route(Route::new(path, http::Method::OPTIONS), handler)
    }

    pub fn trace(
        &mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> &mut Self {
        self.route(Route::new(path, http::Method::TRACE), handler)
    }
}

impl AsMut<Group> for Group {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn group(
        &mut self,
        mut group: impl AsMut<Group>,
    ) -> Result<&mut Self, crate::server::Error> {
        let group = group.as_mut().take();
        for mut route in group.routes.into_iter() {
            route.route.path = concat_path(&group.base_path, &route.route.path);
            let next = route.handler.clone();
            let middleware = group.middleware.clone();
            let handler = move |request: crate::server::Request| {
                let next = next.clone();
                let middleware = middleware.clone();
                async move {
                    let request = Request::from(request);
                    middleware.call(request, next).await
                }
            };
            self.route(route.route, handler)?;
        }
        Ok(self)
    }
}

fn concat_path(a: impl Into<String>, b: impl Into<String>) -> String {
    let a: String = a.into();
    let b: String = b.into();
    format!(
        "{}/{}",
        a.strip_suffix("/").unwrap_or(&a),
        b.strip_prefix("/").unwrap_or(&b)
    )
}

#[derive(Clone)]
struct InnerRoute {
    route: Route,
    handler: Next,
}
