use super::{HttpMethod, HttpRequest};

#[derive(Debug)]
pub struct RouteManager {
    routes: Vec<Route>,
}

impl RouteManager {
    pub fn new() -> Self {
        Self { routes: vec![] }
    }

    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    pub fn find_route(&self, path: &str, method: HttpMethod) -> Option<&Route> {
        self.routes
            .iter()
            .find(|r| r.path == path && r.method == method)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub path: String,
    pub method: HttpMethod,
    pub handler: fn(&HttpRequest) -> Vec<u8>,
}

impl Route {
    pub fn new(
        path: impl Into<String>,
        method: HttpMethod,
        handler: fn(&HttpRequest) -> Vec<u8>,
    ) -> Self {
        Self {
            path: path.into(),
            method,
            handler,
        }
    }
}
