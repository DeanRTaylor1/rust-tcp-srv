use crate::{config::Config, connection::Connection, http::RouteManager, logger::LogLevel, Logger};
use std::io;
use tokio::net::TcpListener;

pub struct Server {
    config: Config,
    logger: Logger,
    pub router: RouteManager,
}

impl Server {
    pub fn new(config: Config) -> Self {
        let logger = Logger::new();
        let router = RouteManager::new();
        Self {
            config,
            logger,
            router,
        }
    }

    pub async fn run(&self) -> io::Result<()> {
        if self.router.routes().len() == 0 {
            self.logger.log(
                LogLevel::Application,
                "No routes have been registered, you can register routes by calling the `get`, `post`, `put` and `delete` methods on the route manager.",
            )
        }

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        self.logger.log(
            LogLevel::Info,
            &format!("Server is listening on Port: {}", self.config.port),
        );

        loop {
            let (socket, addr) = listener.accept().await?;
            println!("New connection from {}", addr);

            let router = self.router.clone();

            tokio::spawn(async move {
                if let Err(e) = Connection::new(socket, router).unwrap().process().await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }
}
