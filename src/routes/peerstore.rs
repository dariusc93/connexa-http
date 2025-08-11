use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use connexa::handle::Connexa;
use connexa::prelude::{Multiaddr, PeerId};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct Param {
    peer_id: PeerId,
    address: Option<Multiaddr>,
}

pub async fn add(State(connexa): State<Connexa>, Json(param): Json<Param>) -> Json<Value> {
    let Some(addr) = param.address else {
        return Json(serde_json::json!({
            "status": 500,
            "message": "address is required"
        }));
    };
    match connexa.peer_store().add_address(param.peer_id, addr).await {
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
    match param.address {
        Some(addr) => {
            match connexa
                .peer_store()
                .remove_address(param.peer_id, addr)
                .await
            {
                Ok(_) => Json(serde_json::json!({
                    "status": 200,
                })),
                Err(e) => Json(serde_json::json!({
                    "status": 500,
                    "message": e.to_string()
                })),
            }
        }
        None => match connexa.peer_store().remove_peer(param.peer_id).await {
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
    match connexa.peer_store().list(param.peer_id).await {
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
    }
}
