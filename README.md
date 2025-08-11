# connexa-http

![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![Status: Alpha](https://img.shields.io/badge/status-alpha-orange)

HTTP REST API gateway for [Connexa](https://github.com/dariusc93/connexa). Provides web-friendly access to peer-to-peer
protocols including Kademlia DHT, GossipSub, FloodSub, and more through a simple REST interface built with Axum.

## Project Status

**Alpha** — This project is in early development. APIs, features, and design may change significantly.

## Features

- 🌐 **RESTful API** - Access connexa functions via standard HTTP endpoints
- 🔌 **Multiple Transports** - Support for TCP, QUIC, WebSocket, and WebRTC
- 📡 **Protocol Support**:
    - Kademlia DHT for content and peer discovery
    - GossipSub/FloodSub for pub/sub messaging (WIP)
    - Rendezvous protocol for peer discovery
    - Request/Response for custom protocols (WIP)
    - Identify protocol for peer information exchange
    - Relay protocol for NAT traversal
- 🔐 **Access Control** - Built-in whitelist/blacklist functionality (default to blacklist at this time)
- 📊 **Real-time Events** - Server-Sent Events (SSE) for streaming updates
- ⚙️ **Flexible Configuration** - config files or CLI arguments

## Installation

### From Source

```bash
git clone https://github.com/dariusc93/connexa-http.git
cd connexa-http
cargo build --release
```

## Usage

### Quick Start

Start the server with default settings:

```bash
connexa-http
```

This will start the HTTP server on port 8080 with a randomly generated peer identity.

## License

This project is dual-licensed under:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

You may choose either license for your use.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Built on top of [Connexa](https://github.com/dariusc93/connexa) - High-level libp2p abstraction
- Powered by [rust-libp2p](https://github.com/libp2p/rust-libp2p) - The Rust implementation of libp2p
- HTTP server built with [Axum](https://github.com/tokio-rs/axum) - Ergonomic web framework

## Support

For issues, questions, or suggestions, please open an issue
on [GitHub](https://github.com/dariusc93/connexa-http/issues).
