use rust_tcp_srv::{Config, Server};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let server = Server::new(Config::default());
    server.run().await
}
