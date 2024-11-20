use std::{collections::HashMap, sync::Arc};

use super::{HttpMethod, HttpRequest, MiddlewareHandler, ResponseBuilder, RouteManager};

pub struct RequestResponse {
    pub method: HttpMethod,
    pub path: String,
    pub ip: String,
    pub status: u16,
    pub duration: std::time::Duration,
}

#[derive(Debug)]
pub struct HttpHandler {
    routes: Arc<RouteManager>,
    middleware: Arc<MiddlewareHandler>,
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

impl HttpHandler {
    pub fn new(router: Arc<RouteManager>, middleware: Arc<MiddlewareHandler>) -> Self {
        Self {
            routes: router,
            middleware,
        }
    }

    pub fn handle(&self, buffer: &[u8]) -> Res {
        match HttpRequest::parse(buffer) {
            Some(request) => {
                if let Some(route) = self.routes.find_route(&request.path, request.method) {
                    let params = self.extract_params(&route.pattern, &request.path);
                    let context = Context { request, params };

                    match self.middleware.run(context, route) {
                        Ok(ctx) => Res::new((route.handler)(&ctx), 200),
                        Err(res) => res,
                    }
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
