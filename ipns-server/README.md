# IPNS Server

Basic Libp2p Rust Server that connects to the IPFS DHT and pubsub.

Uses the ipns-entry to parse the IPNS entries and store them in a database.

## Run Binary

From this package root run:

```bash
cargo run --bin ipns-server
```

From the `ipns-utils` workspace root run:

```bash
cargo run --package ipns-server --bin ipns-server
```

## Hacks & Notes

The server sends a hearbeat message every 15 seconds to the browsers, because Libp2p-WebRTC is still in alpha and gets disconnected after inactivity.

WebRTC is used over WebTransport as it is [not yet supported](https://github.com/libp2p/rust-libp2p/issues/2993).

AutoNat doesn't seem to play well when connected to WebRTC transported browsers, so it is not enabled.

### Known Bugs

If multiple browsers on the same machine using the same host connect to the same server, when one browser disconnects, the other browser will also disconnect. This is due to a flaw somewhere in the WebRTC stack, when the WebRTC disconnects from one host it disconnects from them all. If the browsers are on different hosts, even on the same machine, this does not happen. To avoid this when testing, use different hosts for each browser (localhost, localhost with a differnt port or subdomain, ngrok, etc).
