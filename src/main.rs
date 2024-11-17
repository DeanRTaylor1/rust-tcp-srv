use rust_tcp_srv::{Config, EnvValidator, Logger, Server};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let logger = Logger::new(true);
    let validator = EnvValidator::new(logger);
    let config = Config::from_env(&validator);
    let server = Server::new(config);
    server.run().await
}
