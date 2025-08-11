use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use connexa::handle::Connexa;
use connexa::prelude::{Multiaddr, PeerId};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct Param {
    peer_id: Option<PeerId>,
    address: Option<Multiaddr>,
}

pub async fn add(State(connexa): State<Connexa>, Json(param): Json<Param>) -> Json<Value> {
    let Some(peer_id) = param.peer_id else {
        return Json(serde_json::json!({
            "status": 500,
            "message": "peer id is required"
        }));
    };
    let Some(addr) = param.address else {
        return Json(serde_json::json!({
            "status": 500,
            "message": "address is required"
        }));
    };
    match connexa.peer_store().add_address(peer_id, addr).await {
        Ok(_) => Json(serde_json::json!({
            "status": 200,
        })),
        Err(e) => Json(serde_json::json!({
            "status": 500,
            "message": e.to_string()
        })),
    }
}

pub async fn remove(State(connexa): State<Connexa>, Json(param): Json<Param>) -> Json<Value> {
    let Some(peer_id) = param.peer_id else {
        return Json(serde_json::json!({
            "status": 500,
            "message": "peer id is required"
        }));
    };
    match param.address {
        Some(addr) => match connexa.peer_store().remove_address(peer_id, addr).await {
            Ok(_) => Json(serde_json::json!({
                "status": 200,
            })),
            Err(e) => Json(serde_json::json!({
                "status": 500,
                "message": e.to_string()
            })),
        },
        None => match connexa.peer_store().remove_peer(peer_id).await {
            Ok(_) => Json(serde_json::json!({
                "status": 200,
            })),
            Err(e) => Json(serde_json::json!({
                "status": 500,
                "message": e.to_string()
            })),
        },
    }
}

pub async fn list(State(connexa): State<Connexa>, Json(param): Json<Param>) -> Json<Value> {
    match param.peer_id {
        Some(peer_id) => match connexa.peer_store().list(peer_id).await {
            Ok(peers) => {
                let addrs: Vec<String> = peers.into_iter().map(|p| p.to_string()).collect();
                Json(serde_json::json!({
                    "status": 200,
                    "addresses": addrs,
                }))
            }
            Err(e) => {
                let status = StatusCode::INTERNAL_SERVER_ERROR;
                Json(serde_json::json!({
                    "status": status.as_u16(),
                    "message": e.to_string()
                }))
            }
        },
        None => match connexa.peer_store().list_all().await {
            Ok(peers) => {
                let addrs: Vec<_> = peers
                    .into_iter()
                    .map(|(peer_id, list)| {
                        let peer_id_str = peer_id.to_string();
                        let list = list
                            .into_iter()
                            .map(|addr| addr.to_string())
                            .collect::<Vec<String>>();
                        (peer_id_str, list)
                    })
                    .collect();
                Json(serde_json::json!({
                    "status": 200,
                    "addresses": addrs,
                }))
            }
            Err(e) => {
                let status = StatusCode::INTERNAL_SERVER_ERROR;
                Json(serde_json::json!({
                    "status": status.as_u16(),
                    "message": e.to_string()
                }))
            }
        },
    }
}
