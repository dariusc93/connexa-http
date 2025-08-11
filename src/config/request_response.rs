use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    protocol: String,
    max_request_size: usize,
    max_response_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            protocol: "/connexa-http/request-response".to_string(),
            max_request_size: 128 * 1024,
            max_response_size: 1024 * 1024,
        }
    }
}
