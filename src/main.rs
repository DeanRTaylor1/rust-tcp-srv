use kqueue::{Event, EventFilter, FilterFlag, Ident, Watcher};
use std::{
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    os::fd::{AsRawFd, FromRawFd, RawFd},
    time::Duration,
};

pub const MAX_REQUEST_SIZE: usize = 1024;

fn main() -> io::Result<()> {
    let mut server = ServerContext::init()?;
    println!("Server initialized, starting run loop...");
    server.run()
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
    fn init() -> io::Result<ServerContext> {
        let watcher = Watcher::new()?;
        let listener = TcpListener::bind(format!("{HOST}:{PORT}"))?;
        listener.set_nonblocking(true)?;

        let address = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, PORT);
        let events = Vec::with_capacity(MAX_EVENTS);

        let mut ctx = ServerContext {
            watcher,
            listener,
            address,
            events,
        };

        // Using NOTE_FFNOP as per kqueue docs for basic monitoring
        ctx.watcher.add_fd(
            ctx.listener.as_raw_fd(),
            EventFilter::EVFILT_READ,
            FilterFlag::NOTE_FFNOP,
        )?;
        ctx.watcher.watch()?;

        println!("Registered listener with watcher");
        Ok(ctx)
    }

    fn handle_new_connection(&mut self) -> io::Result<()> {
        let (stream, addr) = self.listener.accept()?;
        println!("New connection from: {}", addr);
        stream.set_nonblocking(true)?;

        let fd = stream.as_raw_fd();
        self.watcher
            .add_fd(fd, EventFilter::EVFILT_READ, FilterFlag::NOTE_FFNOP)?;
        self.watcher.watch()?;

        std::mem::forget(stream);
        Ok(())
    }

    fn handle_client_data(client_fd: RawFd, buffer: &mut Vec<u8>) -> io::Result<()> {
        let mut stream = unsafe { TcpStream::from_raw_fd(client_fd) };
        let mut temp_buf = [0u8; 1024];

        match stream.read(&mut temp_buf) {
            Ok(0) => {
                println!("Connection closed by client");
                Ok(())
            }
            Ok(n) => {
                println!("Read {} bytes from client", n);
                buffer.extend_from_slice(&temp_buf[..n]);

                // Print received data for debugging
                if let Ok(str_data) = String::from_utf8(temp_buf[..n].to_vec()) {
                    println!("Received data: {}", str_data);
                }

                if let Some(pos) = find_header_end(buffer) {
                    println!("Found complete HTTP request");
                    let response = http_handler(&buffer[..pos]);
                    println!("Sending response");
                    stream.write_all(&response)?;
                    buffer.clear();
                } else {
                    println!("Incomplete HTTP request");
                }
                std::mem::forget(stream);
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                println!("Would block, trying again later");
                std::mem::forget(stream);
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::ConnectionReset => {
                println!("Connection reset by peer");
                Ok(())
            }
            Err(e) => {
                println!("Error reading from stream: {}", e);
                Err(e)
            }
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut buffer = Vec::with_capacity(MAX_REQUEST_SIZE);
        println!("Server listening on {}:{}", HOST, PORT);

        loop {
            let events = self.watcher.poll(Some(Duration::from_millis(100)));
            match events {
                Some(event) => {
                    println!("Received event: {:?}", event);
                    match event.ident {
                        Ident::Fd(fd) => {
                            if fd == self.listener.as_raw_fd() {
                                match self.handle_new_connection() {
                                    Ok(_) => println!("Successfully handled new connection"),
                                    Err(e) => println!("Error handling connection: {}", e),
                                }
                            } else {
                                match Self::handle_client_data(fd, &mut buffer) {
                                    Ok(_) => println!("Successfully handled client data"),
                                    Err(e) => println!("Error handling client data: {}", e),
                                }
                            }
                        }
                        _ => println!("Received non-fd event: {:?}", event),
                    }
                }
                None => {
                    // Timeout occurred, continue polling
                    continue;
                }
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
    println!("Method: {:?}, Path: {}", method, path);

    Some(HttpRequest { method, path })
}

// Update http_handler for more verbose response
fn http_handler(buffer: &[u8]) -> Vec<u8> {
    const BAD_REQUEST: &str = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nInvalid format";
    const OK_RESPONSE: &str =
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 11\r\n\r\nHello World!";

    println!("Parsing HTTP request");
    match parse_http_request(buffer) {
        Some(request) => match request.method {
            HttpMethod::Get if request.path == "/" => {
                println!("Sending OK response");
                OK_RESPONSE.as_bytes().to_vec()
            }
            _ => {
                println!("Sending BAD REQUEST response");
                BAD_REQUEST.as_bytes().to_vec()
            }
        },
        None => {
            println!("Failed to parse request");
            BAD_REQUEST.as_bytes().to_vec()
        }
    }
}
