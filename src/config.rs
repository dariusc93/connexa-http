mod floodsub;
mod gossipsub;
mod identify;
mod kademlia;
mod relay;
mod rendezvous;
mod request_response;
mod webrtc;
mod websocket;

use connexa::prelude::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::str::FromStr;
use zeroize::Zeroizing;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub http: Vec<Http>,
    pub listen_on: Vec<Multiaddr>,
    pub announce: Vec<Multiaddr>,
    pub bootstrap: Vec<Multiaddr>,
    pub identity: Identity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            http: vec![Http::default()],
            listen_on: vec![
                Multiaddr::from_str("/ip4/0.0.0.0/tcp/4001").unwrap(),
                Multiaddr::from_str("/ip6/::/tcp/4001").unwrap(),
            ],
            announce: vec![],
            bootstrap: vec![],
            identity: Identity {
                peer_id: PeerId::random(),
                private_key: Zeroizing::new("".to_string()),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Http {
    pub port: u16,
    pub host: IpAddr,
}

impl Default for Http {
    fn default() -> Self {
        Self {
            port: 8080,
            host: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Identity {
    pub peer_id: PeerId,
    pub private_key: Zeroizing<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ProtocolFlags {
    pub identify: bool,
    pub autonat: bool,
    pub ping: bool,
    pub kademlia: bool,
    pub gossipsub: bool,
    pub floodsub: bool,
    pub relay: bool,
    pub dcutr: bool,
    pub mdns: bool,
    pub upnp: bool,
    pub request_response: bool,
    pub rendezvous: bool,
    pub stream: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TransportsFlags {
    pub tcp: bool,
    pub udp: bool,
    pub websocket: bool,
    pub webrtc_direct: bool,
}
