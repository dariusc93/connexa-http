use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::response::Sse;
use axum::response::sse::Event;
use connexa::handle::Connexa;
use connexa::prelude::gossipsub::MessageId;
use connexa::prelude::{GossipsubEvent, GossipsubMessage, PeerId};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;

#[derive(Deserialize)]
pub struct SubscribeParam {
    topic: String,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PubsubEvent {
    Subscribed { peer_id: PeerId },
    Unsubscribed { peer_id: PeerId },
    Message { message: PubsubMessage },
}

impl From<GossipsubEvent> for PubsubEvent {
    fn from(ev: GossipsubEvent) -> Self {
        match ev {
            GossipsubEvent::Subscribed { peer_id } => Self::Subscribed { peer_id },
            GossipsubEvent::Unsubscribed { peer_id } => Self::Unsubscribed { peer_id },
            GossipsubEvent::Message { message } => Self::Message {
                message: message.into(),
            },
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct PubsubMessage {
    pub message_id: MessageId,
    pub propagated_source: PeerId,
    pub source: Option<PeerId>,
    pub data: Bytes,
    pub sequence_number: Option<u64>,
}

impl From<GossipsubMessage> for PubsubMessage {
    fn from(message: GossipsubMessage) -> Self {
        Self {
            message_id: message.message_id,
            propagated_source: message.propagated_source,
            source: message.source,
            data: message.data,
            sequence_number: message.sequence_number,
        }
    }
}

pub async fn subscribe(
    State(connexa): State<Connexa>,
    Json(param): Json<SubscribeParam>,
) -> Json<Value> {
    match connexa.gossipsub().subscribe(param.topic).await {
        Ok(_) => Json(serde_json::json!({ "status": 200 })),
        Err(e) => Json(serde_json::json!({ "status": 500, "error": e.to_string() })),
    }
}

pub async fn unsubscribe(Path(topic): Path<String>, State(connexa): State<Connexa>) -> Json<Value> {
    match connexa.gossipsub().unsubscribe(topic).await {
        Ok(_) => Json(serde_json::json!({ "status": 200 })),
        Err(e) => Json(serde_json::json!({ "status": 500, "error": e.to_string() })),
    }
}

pub async fn publish(
    Path(topic): Path<String>,
    State(connexa): State<Connexa>,
    Json(bytes): Json<Bytes>,
) -> Json<Value> {
    match connexa.gossipsub().publish(topic, bytes).await {
        Ok(_) => Json(serde_json::json!({ "status": 200 })),
        Err(e) => Json(serde_json::json!({ "status": 500, "error": e.to_string() })),
    }
}

pub async fn peers(Path(topic): Path<String>, State(connexa): State<Connexa>) -> Json<Value> {
    match connexa.gossipsub().peers(topic).await {
        Ok(peers) => Json(serde_json::json!({ "status": 200, "peers": peers })),
        Err(e) => Json(serde_json::json!({ "status": 500, "error": e.to_string() })),
    }
}

pub async fn topic_listener(
    Path(topic): Path<String>,
    State(connexa): State<Connexa>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut st = connexa
        .gossipsub()
        .listener(topic)
        .await
        .unwrap_or(futures::stream::empty().boxed());

    Sse::new(async_stream::try_stream! {
        while let Some(ev) = st.next().await {
            let event = PubsubEvent::from(ev);
            if let Ok(event) = Event::default().json_data(event) {
                yield event;
            }
        }
    })
}
