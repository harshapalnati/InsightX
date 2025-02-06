use dotenv::dotenv;
use std::env;

pub struct Config {
    pub storage_service_url: String,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();

        let storage_service_url = env::var("STORAGE_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:5000/logs".to_string());
        Self { storage_service_url }
    }
}
