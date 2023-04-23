# IPNS-ENTRY 📍🚪

Encode IPNS Record data into signed CBOR and Protobuf bytes.

Decode and verify protobuf IPNS records in Rust.

Built in accordance with the [IPNS spec](https://specs.ipfs.tech/ipns/ipns-record/).

# Usage

The API uses the [Rust Builder pattern](https://rust-lang.github.io/api-guidelines/type-safety.html?search=#builders-enable-construction-of-complex-values-c-builder) to create the data and IPNS entry.

```rust
let value = "QmWEekX7EZLUd9VXRNMRXW3LXe4F6x7mB8oPxY5XLptrBq";
let ttl = 60 * 60 * 48;
let validity: SystemTime = SystemTime::now() + Duration::from_secs(ttl);
let sequence = 0;

let data: Data = DataBuilder::new(value).validity(validity).sequence(sequence).ttl(ttl).build();
let Signables {v1, v2} = data.signables();

// Update the value and increment the sequence, keep the same ttl and validity
let signables: Signables = data.value(value).increment_sequence().signables();

// Provide a Signer (holding your private keys) that takes `Signables {v1, v2}` and returns `Signed {v1, v2, pub_key}`
let signed: Signed = signer.sign(signables);

let entry = Entry::new(data: Data, signed: Signed);
let routable_bytes = entry.as_protofbuf_bytes()

// Verify the bytes against the IPNS Name (public key)
let rxd_entry = IpnsEntry::from_bytes(&routable_bytes).unwrap();
let verified = rxd_entry.is_valid_for(pub_key);
```

See the [tests](tests/mod.rs) for example usage.

# Tests

`cargo test`

# Build from Source

To build, you will need your env var `PROTOC=` set to the bin location where protoc is saved,
as this uses [prost](https://github.com/tokio-rs/prost) to generate the Rust files from Proto files.

This is done in the [build step](https://doc.rust-lang.org/cargo/reference/build-scripts.html#build-scripts) using `./build.rs`.

The generated rust files are saved by prost to the `target/../build/ipns-entry-*/out/` directory.
