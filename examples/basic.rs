use std::time::{Duration, SystemTime};

fn main() {
    // To create an IPNS entry, you need: value, validity, validityType, sequence, and ttl
    // 1. A value: bytes, such as b"QmWEekX7EZLUd9VXRNMRXW3LXe4F6x7mB8oPxY5XLptrBq"
    // 2. Validity: bytes, either 0 or a timestamp in [rfc3339] (1970-01-01T00:00:00.000000001Z)
    // 3. ValidityType: uint64, The only supported value is 0
    // 4. Sequence: uint64, The sequence number for this entry. Starts at 0
    // 5. TTL: uint64, The time-to-live for this entry in seconds. Starts at 0
    // 6. Signing keypair (private key)

    let duration_since_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let timestamp_nanos = duration_since_epoch.as_nanos(); // u128

    println!("Hello from an example!");
}
