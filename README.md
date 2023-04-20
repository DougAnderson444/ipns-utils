# IPNS-ENTRY 📍🚪

Encode IPNS Record data into signed CBOR and Protobuf bytes.

Decode and verify protobuf IPNS records in Rust.

Built in accordance with the [IPNS spec](https://specs.ipfs.tech/ipns/ipns-record/).

# Usage

See the [tests](tests/mod.rs) for example usage.

# Tests

`cargo test`

# Build from Source

To build, you will need your env var `PROTOC=` set to the bin location where protoc is saved,
as this uses [prost](https://github.com/tokio-rs/prost) to generate the Rust files from Proto files.

This is done in the [build step](https://doc.rust-lang.org/cargo/reference/build-scripts.html#build-scripts) using `./build.rs`.

The generated rust files are saved by prost to the `target/../build/ipns-entry-*/out/` directory.
