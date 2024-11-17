use crate::{config::Config, connection::Connection};
use std::io;
use tokio::net::TcpListener;

pub struct Server {
    config: Config,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(&self) -> io::Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        println!("Listening on {}", addr);

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
