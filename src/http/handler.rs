use super::{HttpMethod, HttpRequest, ResponseBuilder};

pub struct RequestResponse {
    pub method: HttpMethod,
    pub path: String,
    pub ip: String,
    pub status: u16,
    pub duration: std::time::Duration,
}

pub struct HttpHandler {}

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
    pub fn handle(buffer: &[u8]) -> Res {
        match HttpRequest::parse(buffer) {
            Some(request) => match request.method {
                HttpMethod::Get if request.path == "/" => {
                    return Res::new(
                        ResponseBuilder::ok_response("Hello from Dean's server!"),
                        200,
                    );
                }
                _ => {
                    return Res::new(
                        ResponseBuilder::bad_request().text("Bad Request").build(),
                        400,
                    );
                }
            },
            None => {
                return Res::new(
                    ResponseBuilder::bad_request().text("Bad Request").build(),
                    400,
                );
            }
        }
    }
}
