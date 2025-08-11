use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub protocol: Option<String>,
    pub protocol_version: String,
    pub agent_version: String,
    pub push_address_update: bool,
    pub hide_listen_addrs: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            protocol: Some("/id/0.1.0".to_string()),
            protocol_version: "ipfs/0.1.0".to_string(),
            agent_version: "connexa-http/0.1.0".to_string(),
            push_address_update: true,
            hide_listen_addrs: false,
        }
    }
}
