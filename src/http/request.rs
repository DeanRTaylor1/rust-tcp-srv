#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Unsupported,
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
}

impl HttpRequest {
    /// Constructs a new `HttpRequest` with the given method and path.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method of the request.
    /// * `path` - The path of the request.
    pub fn new(method: HttpMethod, path: String) -> Self {
        Self { method, path }
    }

    /// Parses a byte buffer into an `HttpRequest` object.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A byte slice containing the HTTP request data.
    ///
    /// # Returns
    ///
    /// Returns an `Option<HttpRequest>` which is `Some` if parsing was successful,
    /// and `None` otherwise.
    pub fn parse(buffer: &[u8]) -> Option<HttpRequest> {
        parse_http_request(buffer)
    }
}

/// Internal function to parse HTTP request from a byte buffer.
///
/// # Arguments
///
/// * `buffer` - A byte slice containing the HTTP request data.
///
/// # Returns
///
/// Returns an `Option<HttpRequest>` which is `Some` if parsing was successful,
/// and `None` otherwise.
fn parse_http_request(buffer: &[u8]) -> Option<HttpRequest> {
    // Convert the buffer to a UTF-8 string.
    let request = std::str::from_utf8(buffer).ok()?;
    let mut lines = request.lines();

    // Extract the first line of the request.
    let first_line = lines.next()?;
    let mut parts = first_line.split_whitespace();

    let method = match parts.next()? {
        "GET" => HttpMethod::Get,
        _ => HttpMethod::Unsupported,
    };

    let path = parts.next()?.to_string();
    println!("Method: {:?}, Path: {}", method, path);

    Some(HttpRequest { method, path })
}
