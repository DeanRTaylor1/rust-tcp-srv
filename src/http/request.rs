use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Unknown,
    Unsupported,
}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "PATCH" => Ok(HttpMethod::Patch),
            "DELETE" => Ok(HttpMethod::Delete),
            _ => Ok(HttpMethod::Unknown),
        }
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn new(
        method: HttpMethod,
        path: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    ) -> Self {
        Self {
            method,
            path,
            headers,
            body,
        }
    }

    pub fn parse(buffer: &[u8]) -> Option<HttpRequest> {
        let request = std::str::from_utf8(buffer).ok()?;
        let mut parts = request.split("\r\n\r\n");
        let headers_part = parts.next()?;
        let body_part = parts.next().unwrap_or("");

        let mut lines = headers_part.lines();
        let request_line = lines.next()?;
        let mut parts = request_line.split_whitespace();

        let method = HttpMethod::from_str(parts.next()?).ok()?;
        let path = parts.next()?.to_string();

        let mut headers = HashMap::new();
        for line in lines {
            if let Some((key, value)) = line.split_once(": ") {
                headers.insert(key.to_lowercase(), value.to_string());
            }
        }

        let body = body_part.as_bytes().to_vec();
        Some(HttpRequest {
            method,
            path,
            headers,
            body,
        })
    }

    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type").map(|s| s.as_str())
    }

    pub fn content_length(&self) -> Option<usize> {
        self.headers.get("content-length")?.parse().ok()
    }

    pub fn json_body<T: serde::de::DeserializeOwned>(&self) -> Option<T> {
        if self.content_type()? != "application/json" {
            return None;
        }
        serde_json::from_slice(&self.body).ok()
    }
}
