use crate::http::{Context, HttpHandler, HttpMethod, RequestResponse, ResponseBuilder};
use crate::logger::{LogLevel, Logger};

use bytes::BytesMut;
use std::io;
use std::str::FromStr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

#[derive(Debug)]
pub enum Protocol {
    Http1,
    WebSocket,
    Http2,
    Unknown,
}

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,

    buffer: BytesMut,

    logger: Logger,

    http_handler: HttpHandler,
}

#[derive(Debug, serde::Deserialize)]
pub struct JsonData {
    message: String,
    // Add other fields you expect in the JSON
}

impl Connection {
    pub fn new(stream: TcpStream) -> Result<Self, io::Error> {
        let stream = BufWriter::new(stream);
        let buffer = BytesMut::with_capacity(1024 * 1024);
        let logger = Logger::new();
        let mut http_handler = HttpHandler::new();

        // TODO: Move this somewhere else/public APi
        http_handler.get("/", root_handler);
        http_handler.get("/user/:id", user_handler);

        http_handler.post("/", |ctx| {
            match ctx.request.json_body::<JsonData>() {
                Some(body) => {
                    println!("JSON body: {}", body.message);
                }
                None => {
                    return ResponseBuilder::bad_request().text("Bad Request").build();
                }
            }
            return ResponseBuilder::ok_response("Hello from Dean's server!");
        });

        Ok(Self {
            stream,
            buffer,
            logger,
            http_handler,
        })
    }

    pub async fn process(mut self) -> io::Result<()> {
        if 0 == self.stream.read_buf(&mut self.buffer).await? {
            self.logger.log(LogLevel::Application, "Connection closed");
            return Ok(());
        }

        let first_bytes = self.peek(4);

        match self.detect_protocol(first_bytes) {
            Protocol::Http1 => {
                self.handle_http().await?;
            }
            Protocol::Unknown => self.logger.log(LogLevel::Application, "Unknown protocol"),
            _ => self
                .logger
                .log(LogLevel::Application, "Unsupported protocol"),
        }
        Ok(())
    }

    pub async fn handle_http(&mut self) -> io::Result<()> {
        let start_time = std::time::Instant::now();

        let ip = self.stream.get_ref().peer_addr()?.to_string();

        let request_line = std::str::from_utf8(&self.buffer)
            .ok()
            .and_then(|s| s.lines().next())
            .unwrap_or("");

        let mut parts = request_line.split_whitespace();
        let method = parts
            .next()
            .map(|s| HttpMethod::from_str(s).unwrap_or(HttpMethod::Unknown))
            .unwrap_or(HttpMethod::Unknown);

        let path = parts.next().unwrap_or("/").to_string();

        let response = self.http_handler.handle(&self.buffer);
        let duration = start_time.elapsed();

        Logger::log_http(&RequestResponse {
            method,
            path,
            ip,
            status: response.status,
            duration,
        });

        self.stream.write_all(&response.buffer).await?;
        self.stream.flush().await
    }

    fn peek(&self, n: usize) -> &[u8] {
        &self.buffer[..std::cmp::min(n, self.buffer.len())]
    }

    fn detect_protocol(&self, bytes: &[u8]) -> Protocol {
        if bytes.len() < 4 {
            return Protocol::Unknown;
        }

        match bytes {
            b"GET " => Protocol::Http1,
            b"POST" => Protocol::Http1,
            b"PUT " => Protocol::Http1,
            b"HEAD" => Protocol::Http1,
            b"PRI " => Protocol::Http2,
            _ => Protocol::Unknown,
        }
    }
}

fn root_handler(_ctx: &Context) -> Vec<u8> {
    ResponseBuilder::ok_response("Hello from Dean's server!")
}

fn user_handler(ctx: &Context) -> Vec<u8> {
    let user_id = ctx.param("id").unwrap_or("0");
    Logger::new().log(LogLevel::Debug, &format!("User ID: {}", user_id));
    ResponseBuilder::ok().text(format!("{}", user_id)).build()
}
