use crate::{
    config::Config,
    connection::Connection,
    http::{HttpHandler, RouteManager},
    logger::LogLevel,
    Logger,
};
use std::{io, sync::Arc};
use tokio::net::TcpListener;

pub struct Server {
    config: Config,
    logger: Logger,
    pub router: RouteManager,
    shared_router: Option<Arc<RouteManager>>,
    http_handler: Option<Arc<HttpHandler>>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            logger: Logger::new(),
            router: RouteManager::new(),
            shared_router: None,
            http_handler: None,
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        if self.router.routes().len() == 0 {
            self.logger.log(
                LogLevel::Application,
                "No routes have been registered, you can register routes by calling the `get`, `post`, `put` and `delete` methods on the route manager.",
            )
        }

        self.shared_router = Some(Arc::new(std::mem::replace(
            &mut self.router,
            RouteManager::new(),
        )));

        self.http_handler = Some(Arc::new(HttpHandler::new(
            self.shared_router.as_ref().unwrap().clone(),
        )));

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        self.logger.log(
            LogLevel::Info,
            &format!("Server is listening on Port: {}", self.config.port),
        );

        loop {
            let (socket, _addr) = listener.accept().await?;
            let handler = Arc::clone(self.http_handler.as_ref().unwrap());
            tokio::spawn(async move {
                if let Err(e) = Connection::new(socket, handler).unwrap().process().await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }
}
