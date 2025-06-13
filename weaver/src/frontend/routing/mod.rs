use super::{middleware::Middleware, request::Request};
use crate::{
    frontend::{
        handler::DynHandler,
        middleware::{DynMiddleware, Next},
    },
    server::{RequestHandler, Server},
};

pub struct Route {
    path: String,
    handler: Next,
}

pub struct Group {
    base_path: String,
    middleware: DynMiddleware,
    routes: Vec<Route>,
}

impl Group {
    pub fn new() -> Self {
        Self {
            base_path: String::new(),
            middleware: Default::default(),
            routes: Vec::new(),
        }
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.base_path = path.into();
        self
    }

    pub fn middleware(mut self, middleware: impl Middleware + 'static) -> Self {
        self.middleware = std::mem::take(&mut self.middleware).chain(middleware);
        self
    }

    pub fn route(
        mut self,
        path: impl Into<String>,
        handler: impl RequestHandler + 'static,
    ) -> Self {
        self.routes.push(Route {
            path: path.into(),
            handler: Next::from(DynHandler::new(handler)),
        });
        self
    }

    pub fn group(mut self, group: Group) -> Result<Self, crate::server::Error> {
        self.routes
            .extend(group.routes.into_iter().map(|route| Route {
                path: concat_path(&group.base_path, &route.path),
                handler: group.middleware.wrap(route.handler.clone()),
            }));
        Ok(self)
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn group(&mut self, group: Group) -> Result<&mut Self, crate::server::Error> {
        for route in group.routes.into_iter() {
            let path = concat_path(&group.base_path, route.path);
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
            self.route(path, handler)?;
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
