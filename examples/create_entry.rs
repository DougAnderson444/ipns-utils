use ipns_utils::entry;
use std::time::{Duration, SystemTime};

fn main() {
    // To create an IPNS entry, you need: value, validity, validityType, sequence, and ttl
    // 1. A value: bytes, such as b"QmWEekX7EZLUd9VXRNMRXW3LXe4F6x7mB8oPxY5XLptrBq"
    // 2. Validity: bytes, either 0 or a timestamp in [rfc3339] (1970-01-01T00:00:00.000000001Z)
    // 3. ValidityType: uint64, The only supported value is 0
    // 4. Sequence: uint64, The sequence number for this entry. Starts at 0
    // 5. TTL: uint64, The time-to-live for this entry in seconds. Starts at 0
    // 6. Signing keypair (private key)

    // 1. value bytes
    let value = "QmWEekX7EZLUd9VXRNMRXW3LXe4F6x7mB8oPxY5XLptrBq";

    // set validity to 48 hours from now
    let hrs_48 = 60 * 60 * 48;
    let validity = SystemTime::now() + Duration::from_secs(hrs_48);

    // 2. validity in rfc3339 string format bytes
    let validity = humantime::format_rfc3339_nanos(validity).to_string();

    println!("validity: {}", validity);

    // 3. validityType uint64
    let validity_type = 0;

    // 4. sequence to 0
    let sequence = 0;

    // 5. set ttl to validity in uint64
    let ttl = hrs_48;

    // 6. create a keypair
    let keypair = ipns_entry::signer::generate();
}
