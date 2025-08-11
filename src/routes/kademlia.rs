use axum::response::Sse;
use axum::response::sse::Event;
use axum::{Json, extract::State};
use connexa::prelude::DHTEvent;
use connexa::prelude::dht::{PeerRecord, ProviderRecord, Quorum, Record};
use connexa::{
    handle::Connexa,
    prelude::{Multiaddr, PeerId},
};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::Infallible;
use std::num::NonZeroUsize;
use std::time::Instant;

#[derive(Deserialize)]
pub struct FindPeerParam {
    peer_id: PeerId,
}

#[derive(Deserialize)]
pub struct RecordKeyParam {
    key: String,
}

pub async fn find_peer(
    State(connexa): State<Connexa>,
    Json(param): Json<FindPeerParam>,
) -> Json<Value> {
    match connexa.dht().find_peer(param.peer_id).await {
        Ok(info) => {
            let map: HashMap<PeerId, Vec<Multiaddr>> = info
                .into_iter()
                .map(|info| (info.peer_id, info.addrs))
                .collect();

            let info = serde_json::to_value(map).expect("correct serialization");

            Json(serde_json::json!({
                "status": 200,
                "list": info,
            }))
        }
        Err(e) => Json(serde_json::json!({
            "status": 500,
            "message": e.to_string()
        })),
    }
}

pub async fn provide(
    State(connexa): State<Connexa>,
    Json(param): Json<RecordKeyParam>,
) -> Json<Value> {
    match connexa.dht().provide(param.key).await {
        Ok(_) => Json(serde_json::json!({
            "status": 200,
        })),
        Err(e) => Json(serde_json::json!({
            "status": 500,
            "message": e.to_string()
        })),
    }
}

pub async fn stop_provide(
    State(connexa): State<Connexa>,
    Json(param): Json<RecordKeyParam>,
) -> Json<Value> {
    match connexa.dht().stop_provide(param.key).await {
        Ok(_) => Json(serde_json::json!({
            "status": 200,
        })),
        Err(e) => Json(serde_json::json!({
            "status": 500,
            "message": e.to_string()
        })),
    }
}

pub async fn bootstrap(State(connexa): State<Connexa>) -> Json<Value> {
    // we use lazy because we dont want have the http client having to wait until bootstrapping
    // completes
    match connexa.dht().bootstrap_lazy().await {
        Ok(_) => Json(serde_json::json!({
            "status": 200,
        })),
        Err(e) => Json(serde_json::json!({
            "status": 500,
            "message": e.to_string()
        })),
    }
}

#[derive(Deserialize)]
pub struct AddAddressParam {
    peer_id: PeerId,
    address: Multiaddr,
}

pub async fn add_address(
    State(connexa): State<Connexa>,
    Json(param): Json<AddAddressParam>,
) -> Json<Value> {
    // we use lazy because we dont want have the http client having to wait until bootstrapping
    // completes
    match connexa
        .dht()
        .add_address(param.peer_id, param.address)
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

#[derive(Deserialize)]
pub struct PutRecordParam {
    key: String,
    data: Vec<u8>,
    qourum: PutRecordQuorum,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PutRecordQuorum {
    One,
    Majority,
    All,
    N(usize),
}

impl From<PutRecordQuorum> for Quorum {
    fn from(quo: PutRecordQuorum) -> Self {
        match quo {
            PutRecordQuorum::One => Quorum::One,
            PutRecordQuorum::Majority => Quorum::Majority,
            PutRecordQuorum::All => Quorum::All,
            PutRecordQuorum::N(n) => {
                let n = n.max(1);
                let n = NonZeroUsize::new(n).expect("always non-zero");
                Quorum::N(n)
            }
        }
    }
}

pub async fn put(State(connexa): State<Connexa>, Json(param): Json<PutRecordParam>) -> Json<Value> {
    match connexa
        .dht()
        .put(param.key, param.data, param.qourum.into())
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

#[derive(Serialize)]
pub struct KadPeerRecord {
    pub peer_id: Option<PeerId>,
    pub record: KadRecord,
}

impl From<PeerRecord> for KadPeerRecord {
    fn from(record: PeerRecord) -> Self {
        KadPeerRecord {
            peer_id: record.peer,
            record: record.record.into(),
        }
    }
}

pub async fn get(
    State(connexa): State<Connexa>,
    Json(param): Json<RecordKeyParam>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut st = connexa
        .dht()
        .get(param.key)
        .await
        .unwrap_or(futures::stream::empty().boxed());

    Sse::new(async_stream::try_stream! {
        while let Some(Ok(event)) = st.next().await {
            let event = KadPeerRecord::from(event);
            if let Ok(event) = Event::default().json_data(event) {
                yield event;
            }
        }
    })
}

pub async fn get_providers(
    State(connexa): State<Connexa>,
    Json(param): Json<RecordKeyParam>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut st = connexa
        .dht()
        .get_providers(param.key)
        .await
        .unwrap_or(futures::stream::empty().boxed());

    Sse::new(async_stream::try_stream! {
        while let Some(Ok(event)) = st.next().await {
            if let Ok(event) = Event::default().json_data(event) {
                yield event;
            }
        }
    })
}

#[derive(Deserialize)]
pub struct OptionalRecordKeyParam {
    key: Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KadEvent {
    PutRecord {
        source: PeerId,
        record: Option<KadRecord>,
    },
    ProvideRecord {
        record: Option<KadProviderRecord>,
    },
}

#[derive(Serialize)]
pub struct KadRecord {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub publisher: Option<PeerId>,
    pub expires: Option<i64>,
}

impl From<Record> for KadRecord {
    fn from(record: Record) -> Self {
        KadRecord {
            key: record.key.to_vec(),
            value: record.value,
            publisher: record.publisher,
            expires: record
                .expires
                .map(|instant| instant.duration_since(Instant::now()).as_secs() as i64),
        }
    }
}

#[derive(Serialize)]
pub struct KadProviderRecord {
    pub key: Vec<u8>,
    pub provider: PeerId,
    pub expires: Option<i64>,
    pub addresses: Vec<Multiaddr>,
}

impl From<ProviderRecord> for KadProviderRecord {
    fn from(record: ProviderRecord) -> Self {
        KadProviderRecord {
            key: record.key.to_vec(),
            provider: record.provider,
            expires: record
                .expires
                .map(|instant| instant.duration_since(Instant::now()).as_secs() as i64),
            addresses: record.addresses,
        }
    }
}

impl From<DHTEvent> for KadEvent {
    fn from(ev: DHTEvent) -> Self {
        match ev {
            DHTEvent::PutRecord { source, record } => KadEvent::PutRecord {
                source,
                record: record.record.map(Into::into),
            },
            DHTEvent::ProvideRecord { record } => KadEvent::ProvideRecord {
                record: record.record.map(Into::into),
            },
        }
    }
}

pub async fn listener(
    State(connexa): State<Connexa>,
    Json(param): Json<OptionalRecordKeyParam>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut st = connexa
        .dht()
        .listener(param.key)
        .await
        .unwrap_or(futures::stream::empty().boxed());

    Sse::new(async_stream::try_stream! {
        while let Some(event) = st.next().await {
            let ev = KadEvent::from(event);
            if let Ok(event) = Event::default().json_data(ev) {
                yield event;
            }
        }
    })
}
