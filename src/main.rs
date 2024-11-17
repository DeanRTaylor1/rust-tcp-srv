use rust_tcp_srv::{Config, Server};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = Config::default();
    let server = Server::new(config);
    server.run().await
}
