use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use connexa::handle::Connexa;
use connexa::prelude::PeerId;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct Param {
    peer_id: PeerId,
}

pub async fn add(State(connexa): State<Connexa>, Json(param): Json<Param>) -> Json<Value> {
    match connexa.whitelist().add(param.peer_id).await {
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
    match connexa.whitelist().remove(param.peer_id).await {
        Ok(_) => Json(serde_json::json!({
            "status": 200,
        })),
        Err(e) => Json(serde_json::json!({
            "status": 500,
            "message": e.to_string()
        })),
    }
}

pub async fn list(State(connexa): State<Connexa>) -> Json<Value> {
    match connexa.whitelist().list().await {
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
