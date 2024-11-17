#[derive(Default)]
pub struct ResponseBuilder {
    status_line: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl ResponseBuilder {
    // Status code constants
    pub const OK: (u16, &'static str) = (200, "OK");
    pub const NOT_FOUND: (u16, &'static str) = (404, "Not Found");
    pub const BAD_REQUEST: (u16, &'static str) = (400, "Bad Request");
    pub const INTERNAL_SERVER_ERROR: (u16, &'static str) = (500, "Internal Server Error");

    // Content type constants
    pub const PLAIN: &'static str = "text/plain";
    pub const HTML: &'static str = "text/html";
    pub const JSON: &'static str = "application/json";

    pub fn new() -> Self {
        Self {
            status_line: format!("HTTP/1.1 {} {}", Self::OK.0, Self::OK.1),
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn status(mut self, status: (u16, &str)) -> Self {
        self.status_line = format!("HTTP/1.1 {} {}", status.0, status.1);
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn content_type(self, content_type: &str) -> Self {
        self.header("Content-Type", content_type)
    }

    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        let length = self.body.len().to_string();
        self.header("Content-Length", &length)
    }

    pub fn text(self, body: impl AsRef<str>) -> Self {
        self.content_type(Self::PLAIN)
            .body(body.as_ref().as_bytes().to_vec())
    }

    pub fn json(self, body: impl AsRef<str>) -> Self {
        self.content_type(Self::JSON)
            .body(body.as_ref().as_bytes().to_vec())
    }

    pub fn html(self, body: impl AsRef<str>) -> Self {
        self.content_type(Self::HTML)
            .body(body.as_ref().as_bytes().to_vec())
    }

    pub fn build(self) -> Vec<u8> {
        let mut response = Vec::new();

        // Add status line
        response.extend_from_slice(self.status_line.as_bytes());
        response.extend_from_slice(b"\r\n");

        // Add headers
        for (key, value) in self.headers {
            response.extend_from_slice(key.as_bytes());
            response.extend_from_slice(b": ");
            response.extend_from_slice(value.as_bytes());
            response.extend_from_slice(b"\r\n");
        }

        // Add blank line and body
        response.extend_from_slice(b"\r\n");
        response.extend_from_slice(&self.body);

        response
    }

    pub fn ok_response(message: impl AsRef<str>) -> Vec<u8> {
        Self::new().status(Self::OK).text(message).build()
    }

    pub fn not_found_response(message: impl AsRef<str>) -> Vec<u8> {
        Self::new().status(Self::NOT_FOUND).text(message).build()
    }

    pub fn bad_request_response(message: impl AsRef<str>) -> Vec<u8> {
        Self::new().status(Self::BAD_REQUEST).text(message).build()
    }

    pub fn server_error_response(message: impl AsRef<str>) -> Vec<u8> {
        Self::new()
            .status(Self::INTERNAL_SERVER_ERROR)
            .text(message)
            .build()
    }

    // JSON variants
    pub fn ok_json(json: impl AsRef<str>) -> Vec<u8> {
        Self::new().status(Self::OK).json(json).build()
    }

    pub fn not_found_json(json: impl AsRef<str>) -> Vec<u8> {
        Self::new().status(Self::NOT_FOUND).json(json).build()
    }

    // Default responses with standard messages
    pub fn default_not_found() -> Vec<u8> {
        Self::not_found_response("Resource not found")
    }

    pub fn default_bad_request() -> Vec<u8> {
        Self::bad_request_response("Bad request")
    }

    pub fn default_server_error() -> Vec<u8> {
        Self::server_error_response("Internal server error")
    }
}

// Helper methods for common responses
impl ResponseBuilder {
    pub fn ok() -> Self {
        Self::new().status(Self::OK)
    }

    pub fn not_found() -> Self {
        Self::new().status(Self::NOT_FOUND)
    }

    pub fn bad_request() -> Self {
        Self::new().status(Self::BAD_REQUEST)
    }

    pub fn server_error() -> Self {
        Self::new().status(Self::INTERNAL_SERVER_ERROR)
    }
}
