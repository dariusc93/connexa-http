use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Sse;
use axum::response::sse::Event;
use connexa::handle::Connexa;
use connexa::prelude::swarm::ConnectionId;
use connexa::prelude::swarm::dial_opts::DialOpts;
use connexa::prelude::{ConnectionEvent, ConnectionTarget, Multiaddr, PeerId};
use futures::Stream;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;

#[derive(Debug, Deserialize)]
pub struct DialParam {
    pub peer_id: Option<PeerId>,
    pub addresses: Vec<Multiaddr>,
}

#[derive(Debug, Deserialize)]
pub struct DisconnectParam {
    pub peer_id: Option<PeerId>,
    pub connection_id: Option<usize>,
}

impl DisconnectParam {
    fn to_connection_target(self) -> Option<ConnectionTarget> {
        let DisconnectParam {
            peer_id,
            connection_id,
        } = self;
        match (peer_id, connection_id) {
            (None, None) => None,
            (Some(peer_id), None) => Some(ConnectionTarget::PeerId(peer_id)),
            (None, Some(id)) | (Some(_), Some(id)) => Some(ConnectionTarget::ConnectionId(
                ConnectionId::new_unchecked(id),
            )),
        }
    }
}

pub async fn dial(State(connexa): State<Connexa>, Json(param): Json<DialParam>) -> Json<Value> {
    let opt = match param.peer_id {
        Some(peer_id) => DialOpts::peer_id(peer_id)
            .addresses(param.addresses)
            .build(),
        None if param.addresses.len() == 1 => DialOpts::unknown_peer_id()
            .address(param.addresses.get(0).cloned().expect("should exist"))
            .build(),
        None => {
            let status = StatusCode::BAD_REQUEST;
            return Json(serde_json::json!({
                "status": status.as_u16(),
                "message": "invalid dial param"
            }));
        }
    };

    let id = match connexa.swarm().dial(opt).await {
        Ok(id) => id,
        Err(e) => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            return Json(serde_json::json!({
                "status": status.as_u16(),
                "message": e.to_string()
            }));
        }
    };

    Json(serde_json::json!({
        "status": 200,
        "message": "success",
        "connection_id": id.to_string()
    }))
}

pub async fn disconnect(
    State(connexa): State<Connexa>,
    Json(param): Json<DisconnectParam>,
) -> Json<Value> {
    let connection_target = match param.to_connection_target() {
        Some(connection_target) => connection_target,
        None => {
            let status = StatusCode::BAD_REQUEST;
            return Json(serde_json::json!({
                "status": status.as_u16(),
                "message": "invalid disconnect param"
            }));
        }
    };

    if let Err(e) = connexa.swarm().disconnect(connection_target).await {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        return Json(serde_json::json!({
            "status": status.as_u16(),
            "message": e.to_string()
        }));
    }

    Json(serde_json::json!({
        "status": 200,
        "message": "success",
    }))
}

#[derive(Debug, Deserialize)]
pub struct IsConnectedParam {
    pub peer_id: PeerId,
}

pub async fn is_connected(
    State(connexa): State<Connexa>,
    Json(param): Json<IsConnectedParam>,
) -> Json<Value> {
    match connexa.swarm().is_connected(param.peer_id).await {
        Ok(connected) => Json(serde_json::json!({
            "status": 200,
            "connected": connected,
        })),
        Err(e) => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            Json(serde_json::json!({
                "status": status.as_u16(),
                "message": e.to_string()
            }))
        }
    }
}

pub async fn connected_peers(State(connexa): State<Connexa>) -> Json<Value> {
    match connexa.swarm().connected_peers().await {
        Ok(peers) => {
            Json(serde_json::json!({
                "status": 200,
                "peers": peers,
            }))
        }
        Err(e) => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            Json(serde_json::json!({
                "status": status.as_u16(),
                "message": e.to_string()
            }))
        }
    }
}

pub async fn listening_addresses(State(connexa): State<Connexa>) -> Json<Value> {
    match connexa.swarm().listening_addresses().await {
        Ok(addresses) => {
            Json(serde_json::json!({
                "status": 200,
                "addresses": addresses,
            }))
        }
        Err(e) => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            Json(serde_json::json!({
                "status": status.as_u16(),
                "message": e.to_string()
            }))
        }
    }
}

// #[derive(Debug, Deserialize)]
// pub struct ListenParam {
//     pub address: Multiaddr,
// }
//
// pub async fn listen_on(
//     State(connexa): State<Connexa>,
//     Json(param): Json<ListenParam>,
// ) -> Json<Value> {
//     match connexa.swarm().listen_on(param.address.clone()).await {
//         Ok(listener_id) => Json(serde_json::json!({
//             "status": 200,
//             "listener_id": listener_id.to_string(),
//             "address": param.address.to_string()
//         })),
//         Err(e) => {
//             let status = StatusCode::INTERNAL_SERVER_ERROR;
//             Json(serde_json::json!({
//                 "status": status.as_u16(),
//                 "message": e.to_string()
//             }))
//         }
//     }
// }

pub async fn external_addresses(State(connexa): State<Connexa>) -> Json<Value> {
    match connexa.swarm().external_addresses().await {
        Ok(addresses) => {
            Json(serde_json::json!({
                "status": 200,
                "external_addresses": addresses,
            }))
        }
        Err(e) => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            Json(serde_json::json!({
                "status": status.as_u16(),
                "message": e.to_string()
            }))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddPeerAddressParam {
    pub peer_id: PeerId,
    pub address: Multiaddr,
}

pub async fn add_peer_address(
    State(connexa): State<Connexa>,
    Json(param): Json<AddPeerAddressParam>,
) -> Json<Value> {
    match connexa
        .swarm()
        .add_peer_address(param.peer_id, param.address.clone())
        .await
    {
        Ok(()) => Json(serde_json::json!({
            "status": 200,
            "message": "success",
            "peer_id": param.peer_id,
            "address": param.address
        })),
        Err(e) => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            Json(serde_json::json!({
                "status": status.as_u16(),
                "message": e.to_string()
            }))
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionListenerEvent {
    ConnectionEstablished { peer_id: PeerId, address: Multiaddr },
    ConnectionClosed { peer_id: PeerId, address: Multiaddr },
}

pub async fn connection_listener(
    State(connexa): State<Connexa>,
) -> Sse<impl Stream<Item=Result<Event, Infallible>>> {
    let mut st = connexa
        .swarm()
        .listener()
        .await
        .expect("valid listener")
        .map(|event| match event {
            ConnectionEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => ConnectionListenerEvent::ConnectionEstablished {
                peer_id,
                address: endpoint.get_remote_address().clone(),
            },
            ConnectionEvent::ConnectionClosed {
                peer_id, endpoint, ..
            } => ConnectionListenerEvent::ConnectionClosed {
                peer_id,
                address: endpoint.get_remote_address().clone(),
            },
        })
        .map(|e| Event::default().json_data(e));

    Sse::new(async_stream::try_stream! {
        while let Some(Ok(event)) = st.next().await {
            yield event;
        }
    })
}
