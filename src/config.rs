pub struct Config {
    pub host: String,
    pub port: u16,
    pub max_request_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_request_size: 1024 * 1024,
        }
    }
}
