use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub certificates: Option<Vec<String>>,
    pub keypair: Option<String>,
}
