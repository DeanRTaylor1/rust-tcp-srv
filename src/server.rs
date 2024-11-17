use crate::{config::Config, connection::Connection, logger::LogLevel, Logger};
use std::io;
use tokio::net::TcpListener;

pub struct Server {
    config: Config,
    logger: Logger,
}

impl Server {
    pub fn new(config: Config) -> Self {
        let logger = Logger::new();
        Self { config, logger }
    }

    pub async fn run(&self) -> io::Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        self.logger.log(
            LogLevel::Info,
            &format!("Server is listening on Port: {}", self.config.port),
        );

        loop {
            let (socket, addr) = listener.accept().await?;
            println!("New connection from {}", addr);

            tokio::spawn(async move {
                if let Err(e) = Connection::new(socket).unwrap().process().await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }
}
