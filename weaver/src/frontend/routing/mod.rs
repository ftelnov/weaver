use super::middleware::Middleware;
use crate::{
    frontend::middleware::{Next, SharedMiddleware},
    server::{RequestHandler, Route, Server, SharedRequestHandler},
};

#[derive(Clone)]
pub struct Group {
    base_path: String,
    middlewares: Vec<SharedMiddleware>,
    routes: Vec<InnerRoute>,
}

impl Group {
    pub fn new() -> Self {
        Self {
            base_path: String::new(),
            middlewares: Default::default(),
            routes: Vec::new(),
        }
    }

    pub fn path(&mut self, path: impl Into<String>) -> &mut Self {
        self.base_path = path.into();
        self
    }

    pub fn middleware(&mut self, middleware: impl Into<SharedMiddleware>) -> &mut Self {
        self.middlewares.push(middleware.into());
        self
    }

    pub fn route(&mut self, route: Route, handler: impl Into<SharedRequestHandler>) -> &mut Self {
        self.routes.push(InnerRoute {
            route,
            handler: Next::from(handler.into()),
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
        let mut group = group.as_mut().take();
        // Wrap routes with the middlewares defined on the group.
        let wrapped_routes = std::mem::take(&mut group.routes)
            .into_iter()
            .map(|mut route| {
                route.route.path = concat_path(&group.base_path, &route.route.path);
                route.handler = group.apply_middlewares(route.handler);
                route
            });
        self.routes.extend(wrapped_routes);
        Ok(self)
    }

    fn apply_middlewares(&self, handler: Next) -> Next {
        self.middlewares
            .iter()
            .rev()
            .fold(handler, |stack, middleware| middleware.clone().wrap(stack))
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
        let mut group = group.as_mut().take();
        for mut route in std::mem::take(&mut group.routes).into_iter() {
            route.route.path = concat_path(&group.base_path, &route.route.path);
            let handler: SharedRequestHandler = group.apply_middlewares(route.handler).into();
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
