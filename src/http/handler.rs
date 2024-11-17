use super::{HttpMethod, HttpRequest, ResponseBuilder};

pub struct HttpHandler {}

impl HttpHandler {
    pub fn handle(buffer: &[u8]) -> Vec<u8> {
        println!("Parsing HTTP request");
        match HttpRequest::parse(buffer) {
            Some(request) => match request.method {
                HttpMethod::Get if request.path == "/" => {
                    println!("Sending OK response");
                    return ResponseBuilder::ok().text("Hello, world!").build();
                }
                _ => {
                    println!("Sending BAD REQUEST response");
                    return ResponseBuilder::not_found().text("Not Found").build();
                }
            },
            None => {
                println!("Failed to parse request");
                return ResponseBuilder::bad_request().text("Bad Request").build();
            }
        }
    }
}
