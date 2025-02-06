use std::env;

pub struct Config {
    pub processor_url: String,
}

impl Config {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let processor_url = env::var("LOG_PROCESSOR_URL").unwrap_or_else(|_| "http://localhost:4000/logs".to_string());
        Self { processor_url }
    }
}
