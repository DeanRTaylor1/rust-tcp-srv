mod http;

use http::HttpHandler;
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
    /// Initializes a new `ServerContext` with a bound and non-blocking `TcpListener`, a
    /// `Watcher` for monitoring file descriptors, and an empty buffer for storing events.
    ///
    /// The `TcpListener` is registered with the `Watcher` using the `NOTE_FFNOP` flag for
    /// basic monitoring of new connections.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `TcpListener` cannot be bound to the
    /// specified address, or if the `Watcher` cannot be initialized.
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

    /// Handles data received from a connected client.
    ///
    /// This function will read all available data from the client, and store it in the
    /// provided buffer. If the client closes the connection, the function will return
    /// an error. If the client sends an incomplete HTTP request, the function will
    /// return an error.
    ///
    /// # Arguments
    ///
    /// * `client_fd` - The file descriptor of the client connection.
    /// * `buffer` - A mutable buffer to store the received data.
    ///
    /// # Returns
    ///
    /// Returns an error if the client connection is closed, or if the client sends
    /// an incomplete HTTP request.
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

                // Check if the received data contains a complete HTTP request
                if let Some(pos) = find_header_end(buffer) {
                    println!("Found complete HTTP request");
                    let response = HttpHandler::handle(&buffer[..pos]);
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

    /// Runs the server loop.
    ///
    /// This function will continuously poll the watcher for events, and handle
    /// them accordingly. If a new connection is detected, it will be accepted
    /// and registered with the watcher. If data is available on an existing
    /// connection, it will be read and processed. If the connection is closed
    /// by the client, it will be removed from the watcher.
    pub fn run(&mut self) -> io::Result<()> {
        let mut buffer = Vec::with_capacity(MAX_REQUEST_SIZE);
        println!("Server listening on {}:{}", HOST, PORT);

        loop {
            // Poll the watcher for events with a 100ms timeout
            let events = self.watcher.poll(Some(Duration::from_millis(100)));
            match events {
                Some(event) => {
                    println!("Received event: {:?}", event);
                    match event.ident {
                        // Handle new connection
                        Ident::Fd(fd) => {
                            if fd == self.listener.as_raw_fd() {
                                match self.handle_new_connection() {
                                    Ok(_) => println!("Successfully handled new connection"),
                                    Err(e) => println!("Error handling connection: {}", e),
                                }
                            } else {
                                // Handle data available on an existing connection
                                match Self::handle_client_data(fd, &mut buffer) {
                                    Ok(_) => println!("Successfully handled client data"),
                                    Err(e) => println!("Error handling client data: {}", e),
                                }
                            }
                        }
                        // Ignore non-fd events
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
