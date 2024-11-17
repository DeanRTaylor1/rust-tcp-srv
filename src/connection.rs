use crate::http::HttpHandler;

use bytes::BytesMut;
use std::io;
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
}

impl Connection {
    pub fn new(stream: TcpStream) -> Result<Self, io::Error> {
        let stream = BufWriter::new(stream);
        let buffer = BytesMut::with_capacity(1024 * 1024);
        Ok(Self { stream, buffer })
    }

    pub async fn process(mut self) -> io::Result<()> {
        if 0 == self.stream.read_buf(&mut self.buffer).await? {
            println!("Connection closed before data received");
            return Ok(());
        }

        let first_bytes = self.peek(4);
        println!("First bytes: {:?}", first_bytes);

        match self.detect_protocol(first_bytes) {
            Protocol::Http1 => {
                println!("HTTP/1.1 request detected");
                println!("Raw request: {}", String::from_utf8_lossy(&self.buffer));
                self.handle_http().await?;
            }
            Protocol::Unknown => println!("Unknown protocol"),
            _ => println!("Unsupported protocol"),
        }
        Ok(())
    }

    async fn handle_http(&mut self) -> io::Result<()> {
        let response = HttpHandler::handle(&self.buffer);
        println!("Sending response: {:?}", String::from_utf8_lossy(&response));
        self.stream.write_all(&response).await?;
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
