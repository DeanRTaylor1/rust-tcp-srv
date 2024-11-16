use kqueue::{Event, EventFilter, EventFlag, FilterFlag, Ident, Watcher};
use std::{
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    os::fd::{AsRawFd, FromRawFd, RawFd},
};

pub const MAX_REQUEST_SIZE: usize = 1024;

fn main() {
    println!("hello")
}

pub struct ServerContext {
    pub watcher: Watcher,
    pub listener: TcpListener,
    pub address: SocketAddrV4,
    pub events: Vec<Event>,
}

pub const PORT: u16 = 8080;
pub const HOST: &str = "127.0.0.1";
pub const MAX_EVENTS: usize = 10;

impl ServerContext {
    pub fn init() -> std::io::Result<Self> {
        let watcher = Watcher::new()?;
        let listener = TcpListener::bind(format!("{HOST}:{PORT}"))?;
        let address = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, PORT);
        let events = Vec::with_capacity(MAX_EVENTS);

        let mut ctx: ServerContext = ServerContext {
            watcher,
            listener,
            address,
            events,
        };

        ctx.watcher.add_fd(
            ctx.listener.as_raw_fd(),
            EventFilter::EVFILT_READ,
            FilterFlag::NOTE_FFNOP,
        )?;

        Ok(ctx)
    }

    fn handle_new_connection(&mut self) -> io::Result<()> {
        let (stream, _) = self.listener.accept()?;
        stream.set_nonblocking(true)?;

        self.watcher.add_fd(
            stream.as_raw_fd(),
            EventFilter::EVFILT_READ,
            FilterFlag::NOTE_FFNOP,
        )?;

        Ok(())
    }

    fn handle_client_data(client_fd: RawFd, buffer: &mut Vec<u8>) -> io::Result<()> {
        let mut stream = unsafe { TcpStream::from_raw_fd(client_fd) };
        let mut temp_buf = [0u8; 1024];

        match stream.read(&mut temp_buf) {
            Ok(0) => {
                return Ok(());
            }
            Ok(n) => {
                buffer.extend_from_slice(&temp_buf[..n]);
                if let Some(pos) = find_header_end(buffer) {
                    let response = http_handler(&buffer[..pos]);
                    stream.write_all(&response)?;
                    buffer.clear();
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::mem::forget(stream);
                return Ok(());
            }
            Err(e) => return Err(e),
        }

        std::mem::forget(stream);
        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut buffer = Vec::with_capacity(MAX_REQUEST_SIZE);

        loop {
            let event = self
                .watcher
                .poll(None)
                .ok_or(io::Error::new(io::ErrorKind::Other, "no event"))?;

            match event.ident {
                Ident::Fd(fd) if fd == self.listener.as_raw_fd() => {
                    self.handle_new_connection()?;
                }
                Ident::Fd(client_fd) => {
                    Self::handle_client_data(client_fd, &mut buffer)?;
                }
                _ => continue,
            }
        }
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
}

const MAX_RESPONSE_SIZE: usize = 1024; // Adjust as needed

#[derive(Debug)]
enum HttpMethod {
    Get,
    Unsupported,
}

struct HttpRequest {
    method: HttpMethod,
    path: String,
}

fn parse_http_request(buffer: &[u8]) -> Option<HttpRequest> {
    let request = std::str::from_utf8(buffer).ok()?;
    let mut lines = request.lines();
    let first_line = lines.next()?;
    let mut parts = first_line.split_whitespace();

    let method = match parts.next()? {
        "GET" => HttpMethod::Get,
        _ => HttpMethod::Unsupported,
    };

    let path = parts.next()?.to_string();

    Some(HttpRequest { method, path })
}

fn http_handler(buffer: &[u8]) -> Vec<u8> {
    const BAD_REQUEST: &str =
        "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\ninvalid format";
    const OK_RESPONSE: &str =
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello from server";

    match parse_http_request(buffer) {
        Some(request) => match request.method {
            HttpMethod::Get if request.path == "/" => OK_RESPONSE.as_bytes().to_vec(),
            _ => BAD_REQUEST.as_bytes().to_vec(),
        },
        None => BAD_REQUEST.as_bytes().to_vec(),
    }
}
