mod http;
use http::HttpHandler;

use bytes::BytesMut;
use std::io;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::{TcpListener, TcpStream},
};

pub const PORT: u16 = 8080;
pub const HOST: &str = "127.0.0.1";
pub const MAX_EVENTS: usize = 10;

pub const MAX_REQUEST_SIZE: usize = 1024 * 1024;

#[derive(Debug)]
pub enum Protocol {
    Http1,
    WebSocket,
    Http2,
    Unknown,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind(format!("{HOST}:{PORT}")).await?;
    println!("Listening on {}:{}", HOST, PORT);
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);
        process(socket).await?;
    }
}

async fn process(socket: TcpStream) -> io::Result<()> {
    let mut conn = Connection::new(socket)?;
    conn.read_data().await
}

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,

    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Result<Self, io::Error> {
        let stream = BufWriter::new(stream);
        let buffer = BytesMut::with_capacity(MAX_REQUEST_SIZE);
        Ok(Self { stream, buffer })
    }

    pub async fn read_data(&mut self) -> io::Result<()> {
        // First read to get protocol
        if 0 == self.stream.read_buf(&mut self.buffer).await? {
            println!("Connection closed before data received");
            return Ok(());
        }

        let first_bytes = &self.buffer[..std::cmp::min(4, self.buffer.len())];
        println!("First bytes: {:?}", first_bytes);

        match self.detect_protocol().await? {
            Protocol::Http1 => {
                println!("HTTP/1.1 request detected");
                println!("Raw request: {}", String::from_utf8_lossy(&self.buffer));

                let response = HttpHandler::handle(&self.buffer);
                println!("Sending response: {:?}", String::from_utf8_lossy(&response));

                self.stream.write_all(&response).await?;
                self.stream.flush().await?;
                self.buffer.clear();
            }
            Protocol::Unknown => println!("Unknown protocol"),
            _ => println!("Unsupported protocol"),
        }
        Ok(())
    }

    async fn detect_protocol(&mut self) -> io::Result<Protocol> {
        let bytes = self.peek(4).await?;

        match bytes {
            b"GET " => Ok(Protocol::Http1),
            b"POST" => Ok(Protocol::Http1),
            b"PUT " => Ok(Protocol::Http1),
            b"HEAD" => Ok(Protocol::Http1),
            b"PRI " => Ok(Protocol::Http2),
            _ => Ok(Protocol::Unknown),
        }
    }

    async fn peek(&mut self, n: usize) -> io::Result<&[u8]> {
        // Ensure we have enough data
        if self.buffer.len() < n {
            self.stream.read_buf(&mut self.buffer).await?;
        }

        Ok(&self.buffer[..std::cmp::min(n, self.buffer.len())])
    }
}
