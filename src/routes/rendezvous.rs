use axum::Json;
use axum::extract::State;
use connexa::handle::Connexa;
use connexa::prelude::PeerId;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct Param {
    peer_id: PeerId,
    namespace: Option<String>,
    ttl: Option<u64>, // TODO: impl cookie?
}

pub async fn register(State(connexa): State<Connexa>, Json(param): Json<Param>) -> Json<Value> {
    let Some(namespace) = param.namespace else {
        return Json(serde_json::json!({
            "status": 500,
            "message": "namespace is required"
        }));
    };

    match connexa
        .rendezvous()
        .register(param.peer_id, namespace, param.ttl)
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

pub async fn unregister(State(connexa): State<Connexa>, Json(param): Json<Param>) -> Json<Value> {
    let Some(namespace) = param.namespace else {
        return Json(serde_json::json!({
            "status": 500,
            "message": "namespace is required"
        }));
    };

    match connexa
        .rendezvous()
        .unregister(param.peer_id, namespace)
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

pub async fn discovery(State(connexa): State<Connexa>, Json(param): Json<Param>) -> Json<Value> {
    match connexa
        .rendezvous()
        .discovery(param.peer_id, param.namespace, param.ttl, None)
        .await
    {
        Ok((_, peers)) => Json(serde_json::json!({
            "status": 200,
            "peers": peers,
        })),
        Err(e) => Json(serde_json::json!({
            "status": 500,
            "message": e.to_string()
        })),
    }
}
