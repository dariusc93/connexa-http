mod config;
mod routes;

use std::net::{IpAddr, SocketAddr};
use axum::Router;
use axum::http::StatusCode;
use clap::Parser;
use connexa::prelude::{DefaultConnexaBuilder, Multiaddr};
use std::path::PathBuf;
use tokio::net::TcpListener;

#[derive(Debug, Parser)]
#[clap(name = "connexa-http")]
struct Opt {
    /// Path to a configuration file.
    /// Note that if a configuration file is used, other options will be ignored
    #[clap(long)]
    config: Option<PathBuf>,

    /// Path to a keypair file with the keypair encoded in base64
    #[clap(long)]
    keypair_file: Option<PathBuf>,

    /// Keypair encoded in base64.
    /// Note that this option is not ideal to use in production as it would
    /// expose your keypair
    #[clap(long)]
    keypair: Option<String>,

    /// Http port. Default is 8080
    #[clap(long)]
    http_port: Option<u16>,

    /// Bootstrap nodes
    #[clap(long)]
    bootstrap: Vec<Multiaddr>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // TODO: Construct connexa based on options provided from clap, prioritizing the config file over other options

    let connexa = DefaultConnexaBuilder::new_identity()
        .enable_quic()
        .enable_tcp()
        .with_request_response(vec![])
        .with_gossipsub()
        .with_kademlia()
        .with_ping()
        .with_peer_store()
        .with_blacklist()
        .with_autonat_v1()
        .with_rendezvous_client()
        .with_rendezvous_server()
        .with_upnp()
        .with_relay()
        .with_relay_server()
        .with_dcutr()
        .build()?;

    connexa
        .swarm()
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .await?;
    connexa
        .swarm()
        .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap())
        .await?;

    tokio::task::yield_now().await;

    let addrs = connexa.swarm().listening_addresses().await?;

    for addr in addrs {
        println!("Listening on: {}", addr);
    }

    let gossipsub_routes = Router::new()
        .route("/subscribe", axum::routing::post(null))
        .route("/topic/{name}", axum::routing::get(null))
        .route("/topic/{name}/unsubscribe", axum::routing::delete(null))
        .route("/topic/{name}/peers", axum::routing::get(null))
        .route("/topic/{name}/publish", axum::routing::put(null));

    let floodsub_routes = Router::new()
        .route("/subscribe", axum::routing::post(null))
        .route("/topic/{name}", axum::routing::get(null))
        .route("/topic/{name}/unsubscribe", axum::routing::delete(null))
        .route("/topic/{name}/publish", axum::routing::put(null));

    let kad_routes = Router::new()
        .route("/", axum::routing::get(routes::kademlia::listener))
        .route(
            "/find_peer",
            axum::routing::post(routes::kademlia::find_peer),
        )
        .route("/provide", axum::routing::get(routes::kademlia::provide))
        .route(
            "/stop_provide",
            axum::routing::delete(routes::kademlia::stop_provide),
        )
        .route(
            "/get_providers",
            axum::routing::get(routes::kademlia::get_providers),
        )
        .route(
            "/bootstrap",
            axum::routing::post(routes::kademlia::bootstrap),
        )
        .route("/get", axum::routing::get(routes::kademlia::get))
        .route("/put", axum::routing::post(routes::kademlia::put))
        .route(
            "/add_address",
            axum::routing::post(routes::kademlia::add_address),
        );

    let rz_routes = Router::new()
        .route(
            "/register",
            axum::routing::post(routes::rendezvous::register),
        )
        .route(
            "/unregister",
            axum::routing::delete(routes::rendezvous::unregister),
        )
        .route(
            "/discover",
            axum::routing::post(routes::rendezvous::discovery),
        );

    let blacklist_route = Router::new()
        .route("/add", axum::routing::post(routes::blacklist::add))
        .route("/remove", axum::routing::delete(routes::blacklist::remove))
        .route("/list", axum::routing::get(routes::blacklist::list));

    let whitelist_route = Router::new()
        .route("/add", axum::routing::post(routes::whitelist::add))
        .route("/remove", axum::routing::delete(routes::whitelist::remove))
        .route("/list", axum::routing::get(routes::whitelist::list));

    let peerstore_route = Router::new()
        .route("/add", axum::routing::post(routes::peerstore::add))
        .route("/remove", axum::routing::delete(routes::peerstore::remove))
        .route("/list", axum::routing::get(routes::peerstore::list));

    let swarm_route = Router::new()
        .route("/dial", axum::routing::post(routes::swarm::dial))
        .route(
            "/disconnect",
            axum::routing::delete(routes::swarm::disconnect),
        )
        .route(
            "/is_connected",
            axum::routing::get(routes::swarm::is_connected),
        )
        .route(
            "/connected",
            axum::routing::get(routes::swarm::connected_peers),
        )
        .route(
            "/addresses",
            axum::routing::get(routes::swarm::listening_addresses),
        )
        // .route("/listen_on", axum::routing::post(routes::swarm::listen_on))
        .route(
            "/external_addresses",
            axum::routing::get(routes::swarm::external_addresses),
        )
        .route(
            "/listening_addresses",
            axum::routing::get(routes::swarm::listening_addresses),
        )
        .route(
            "/add_peer_address",
            axum::routing::post(routes::swarm::add_peer_address),
        )
        .route(
            "/listener",
            axum::routing::get(routes::swarm::connection_listener),
        );

    let app = Router::new()
        .nest("/gossipsub", gossipsub_routes)
        .nest("/floodsub", floodsub_routes)
        .nest("/kademlia", kad_routes)
        .nest("/rendezvous", rz_routes)
        .nest("/blacklist", blacklist_route)
        .nest("/whitelist", whitelist_route)
        .nest("/peerstore", peerstore_route)
        .nest("/swarm", swarm_route)
        .with_state(connexa);

    let addr = SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 8080);

    let listener = TcpListener::bind(addr).await?;
    println!("Serving API at http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn null() -> StatusCode {
    StatusCode::OK
}
