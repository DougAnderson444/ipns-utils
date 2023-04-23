//! # Create IPNS Entries in Rust
//!
//! Create protobuf IPNS Entries in Rust. Follows the [IPNS Spec](https://specs.ipfs.tech/ipns/ipns-record/).
//!
//! Library includes CBOR and Protobuf modules that convert IPNS entry
//! to/from bytes that can be saved to the routing channels.
//!
//! The output from this library can be published to the IPFS DHT, Pubsub, or anywhere else.
//!
pub mod cbor;
pub mod entry;
pub mod signer;
