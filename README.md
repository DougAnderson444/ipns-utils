# IPNS Utilities 🛠️

Work in Public. This is a work in progress, not ready for production use. Pull Requests welcome.

-   Fork/Clone
-   Create new `feat`/`fix` branch
-   Make changes
-   Add tests
-   Submit PR

## Crates

-   `ipns-entry`: [~Complete] The crate for encoding and decoding IPNS records. Encode IPNS Record data into signed CBOR and Protobuf bytes. Decode and verify protobuf IPNS records in Rust. Built in accordance with the [IPNS spec](https://specs.ipfs.tech/ipns/ipns-record/).

-   `ipns-server`: [WIP] Libp2p server that spins up Kad-DHT and IPNS record publishing.

-   `ipns-interop-test`: [TODO] A crate for testing IPNS interop with Go and JS. This crate is used in the [interop test](todo!).

-   `ipns-publish-persist`: [TODO] A crate for ongoing publishing and persisting IPNS records to a datastore.

# Usage

See the [tests](tests/mod.rs) for example usage.

# Examples

```cli
cargo run --example ipns-entry
```

# Tests

`cargo test --workspace`

# Build from Source

To build, you will need your env var `PROTOC=` set to the bin location where protoc is saved,
as this uses [prost](https://github.com/tokio-rs/prost) to generate the Rust files from Proto files.

This is done in the [build step](https://doc.rust-lang.org/cargo/reference/build-scripts.html#build-scripts) using `./build.rs`.

The generated rust files are saved by prost to the `target/../build/ipns-entry-*/out/` directory.
