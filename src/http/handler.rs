use std::collections::HashMap;

use super::{routes::Route, HttpMethod, HttpRequest, ResponseBuilder, RouteManager};

pub struct RequestResponse {
    pub method: HttpMethod,
    pub path: String,
    pub ip: String,
    pub status: u16,
    pub duration: std::time::Duration,
}

#[derive(Debug)]
pub struct HttpHandler {
    routes: RouteManager,
}

pub struct Res {
    pub buffer: Vec<u8>,
    pub status: u16,
}

impl Res {
    pub fn new(buffer: Vec<u8>, status: u16) -> Self {
        Self { buffer, status }
    }
}

impl Default for HttpHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpHandler {
    pub fn new() -> Self {
        Self {
            routes: RouteManager::new(),
        }
    }

    pub fn get(&mut self, path: &str, handler: fn(&Context) -> Vec<u8>) {
        self.routes
            .add_route(Route::new(path, HttpMethod::Get, handler));
    }

    pub fn post(&mut self, path: &str, handler: fn(&Context) -> Vec<u8>) {
        self.routes
            .add_route(Route::new(path, HttpMethod::Post, handler));
    }

    pub fn put(&mut self, path: &str, handler: fn(&Context) -> Vec<u8>) {
        self.routes
            .add_route(Route::new(path, HttpMethod::Put, handler));
    }

    pub fn delete(&mut self, path: &str, handler: fn(&Context) -> Vec<u8>) {
        self.routes
            .add_route(Route::new(path, HttpMethod::Delete, handler));
    }

    pub fn handle(&self, buffer: &[u8]) -> Res {
        match HttpRequest::parse(buffer) {
            Some(request) => {
                if let Some(route) = self.routes.find_route(&request.path, request.method) {
                    let params = self.extract_params(&route.pattern, &request.path);
                    let context = Context { request, params };
                    Res::new((route.handler)(&context), 200)
                } else {
                    Res::new(ResponseBuilder::not_found().text("Not Found").build(), 404)
                }
            }
            None => Res::new(
                ResponseBuilder::bad_request().text("Bad Request").build(),
                400,
            ),
        }
    }

    fn extract_params(&self, pattern: &str, path: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let pattern_parts: Vec<_> = pattern.split('/').collect();
        let path_parts: Vec<_> = path.split('/').collect();

        for (p, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if p.starts_with(':') {
                params.insert(p[1..].to_string(), path_part.to_string());
            }
        }
        params
    }
}

pub struct Context {
    pub request: HttpRequest,
    params: HashMap<String, String>,
}

impl Context {
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }
}
