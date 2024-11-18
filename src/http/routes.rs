use super::{handler::Context, HttpMethod};

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
            .find(|r| r.method == method && r.matches(path))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub pattern: String,
    pub path_params: Vec<String>,
    pub method: HttpMethod,
    pub handler: fn(&Context) -> Vec<u8>,
}

impl Route {
    pub fn new(pattern: &str, method: HttpMethod, handler: fn(&Context) -> Vec<u8>) -> Self {
        let path_params = pattern
            .split('/')
            .filter(|s| s.starts_with(':'))
            .map(|s| s[1..].to_string())
            .collect();
        Self {
            pattern: pattern.to_string(),
            path_params,
            method,
            handler,
        }
    }

    fn matches(&self, path: &str) -> bool {
        let pattern_parts: Vec<_> = self.pattern.split('/').collect();
        let path_parts: Vec<_> = path.split('/').collect();

        if pattern_parts.len() != path_parts.len() {
            return false;
        }

        pattern_parts
            .iter()
            .zip(path_parts.iter())
            .all(|(p, u)| p.starts_with(':') || p == u)
    }
}
