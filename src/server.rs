use crate::{
    config::Config,
    connection::Connection,
    http::{HttpHandler, MiddlewareHandler, RouteManager},
    logger::LogLevel,
    Logger,
};
use std::{collections::HashMap, io, sync::Arc};
use tokio::net::TcpListener;

pub struct Server {
    config: Config,
    logger: Logger,
    pub router: RouteManager,
    shared_router: Option<Arc<RouteManager>>,
    http_handler: Option<Arc<HttpHandler>>,
    pub middleware: MiddlewareHandler,
    shared_middleware: Option<Arc<MiddlewareHandler>>,
    static_files: HashMap<String, &'static str>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            logger: Logger::new(),
            router: RouteManager::new(),
            shared_router: None,
            http_handler: None,
            middleware: MiddlewareHandler::new(),
            shared_middleware: None,
            static_files: HashMap::new(),
        }
    }

    pub fn static_file(&mut self, route: &str, file_path: &'static str) {
        self.static_files.insert(route.to_string(), file_path);
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

        self.shared_middleware = Some(Arc::new(std::mem::replace(
            &mut self.middleware,
            MiddlewareHandler::new(),
        )));

        let static_files = Arc::new(std::mem::replace(&mut self.static_files, HashMap::new()));

        self.http_handler = Some(Arc::new(HttpHandler::new(
            self.shared_router.as_ref().unwrap().clone(),
            self.shared_middleware.as_ref().unwrap().clone(),
            static_files,
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
