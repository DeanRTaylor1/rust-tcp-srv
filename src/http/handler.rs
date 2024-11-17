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

    pub fn get(&mut self, path: &str, handler: fn(&HttpRequest) -> Vec<u8>) {
        self.routes
            .add_route(Route::new(path, HttpMethod::Get, handler));
    }

    pub fn post(&mut self, path: &str, handler: fn(&HttpRequest) -> Vec<u8>) {
        self.routes
            .add_route(Route::new(path, HttpMethod::Post, handler));
    }

    pub fn handle(&self, buffer: &[u8]) -> Res {
        match HttpRequest::parse(buffer) {
            Some(request) => {
                if let Some(route) = self.routes.find_route(&request.path, request.method) {
                    Res::new((route.handler)(&request), 200)
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
}
