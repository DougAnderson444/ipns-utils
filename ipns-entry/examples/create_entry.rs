use libp2p_identity::SigningError;
use std::result::Result;

fn main() -> Result<(), SigningError> {
    use ipns_entry::entry::IpnsEntry;
    use ipns_entry::signer::{Signed, Signer};
    use ipns_entry::DataBuilder;
    use libp2p_identity::PeerId;
    use std::time::Duration;
    use std::time::SystemTime;

    let value = "QmWEekX7EZLUd9VXRNMRXW3LXe4F6x7mB8oPxY5XLptrBq";
    let ttl = 60 * 60 * 48; // 48 hours, the default
    let validity: SystemTime = SystemTime::now() + Duration::from_secs(ttl);
    let sequence = 0;

    let (data, signables) = DataBuilder::new(value)
        .validity(validity)
        .sequence(sequence)
        .ttl(ttl)
        .build();

    // Provide a Signer (holding your private keys) that takes `Signables {v1, v2}` and returns `Signed {v1, v2, pub_key}`
    let signer = Signer::default();
    let signed: Signed = signer.sign(signables)?;

    let entry = IpnsEntry::new(data, signed);
    let routable_bytes = entry.to_bytes();

    // Decode received bytes into an Entry
    let rxd_entry = IpnsEntry::from_bytes(&routable_bytes).expect("Valid routable bytes");

    // Validate Entry against the IPNS Name (PeerId)
    let peer_id = PeerId::from_public_key(&signer.public());
    let verified = rxd_entry
        .is_valid_for(&peer_id)
        .expect("valid against our peer id");
    assert!(verified);

    Ok(())
}
