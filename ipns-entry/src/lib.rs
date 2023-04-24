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

use crate::entry::ValidityType;
use humantime::Rfc3339Timestamp;
use signer::Signables;
use std::time::Duration;
use std::time::SystemTime;

/// # Example
///
/// ```rust
/// # use std::result::Result;
/// # use libp2p_identity::SigningError;
///
/// # fn main() -> Result<(), SigningError> {
/// use ipns_entry::DataBuilder;
/// use ipns_entry::entry::IpnsEntry;
/// use std::time::SystemTime;
/// use std::time::Duration;
/// use ipns_entry::signer::{Signables, Signed, Signer};
/// use libp2p_identity::PeerId;
/// use libp2p_identity::PublicKey;
/// use libp2p_identity::ed25519;
///
/// let value = "QmWEekX7EZLUd9VXRNMRXW3LXe4F6x7mB8oPxY5XLptrBq";
/// let ttl = 60 * 60 * 48; // 48 hours, the default
/// let validity: SystemTime = SystemTime::now() + Duration::from_secs(ttl);
/// let sequence = 0;
///
/// let (data, signables) = DataBuilder::new(value).validity(validity).sequence(sequence).ttl(ttl).build();
///
/// // Provide a Signer (holding your private keys) that takes `Signables {v1, v2}` and returns `Signed {v1, v2, pub_key}`
/// let signer = Signer::default();
/// let signed: Signed = signer.sign(signables)?;
///
/// let entry = IpnsEntry::new(data, signed);
/// let routable_bytes = entry.to_bytes();
///
/// // Decode received bytes into an Entry
/// let rxd_entry = IpnsEntry::from_bytes(&routable_bytes).expect("Valid routable bytes");
///
/// // Validate Entry against the IPNS Name (PeerId)
/// let peer_id = PeerId::from_public_key(&signer.public());
/// let verified = rxd_entry.is_valid_for(&peer_id).expect("valid against our peer id");
/// assert!(verified);
///
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct DataBuilder {
    value: String,
    validity: Rfc3339Timestamp,
    validity_type: ValidityType,
    sequence: u64,
    ttl: u64,
}

impl DataBuilder {
    /// Create a new DataBuilder with the required value.
    /// The default ttl is 48 hours.
    /// The default validity is 48 hours from now.
    /// The default sequence is 0.
    ///
    /// Customize the ttl, validity, and sequence with the builder methods.
    ///
    /// When the DataBuilder is ready, call `signables()` to get the Signables {v1, v2}
    /// which can be signed by the Signer.
    pub fn new(value: &str) -> Self {
        // default to 48 hours
        let ttl = 60 * 60 * 48;
        let validity: SystemTime = SystemTime::now() + Duration::from_secs(ttl);
        let validity = humantime::format_rfc3339_nanos(validity);

        DataBuilder {
            value: value.to_string(),
            validity,
            validity_type: ValidityType::Eol,
            sequence: 0,
            ttl,
        }
    }

    pub fn value(&mut self, value: &str) -> &mut DataBuilder {
        self.value = value.to_string();
        self
    }

    pub fn validity(&mut self, validity: SystemTime) -> &mut DataBuilder {
        // Convert validity to nanoseconds of rfc3339
        self.validity = humantime::format_rfc3339_nanos(validity);
        self
    }

    pub fn sequence(&mut self, sequence: u64) -> &mut DataBuilder {
        self.sequence = sequence;
        self
    }

    pub fn increment_sequence(&mut self) -> &mut DataBuilder {
        self.sequence += 1;
        self
    }

    pub fn ttl(&mut self, ttl: u64) -> &mut DataBuilder {
        self.ttl = ttl;
        self
    }

    /// Terminal method which generates the Signables from the Builder
    pub fn build(&self) -> (cbor::Data, Signables) {
        let v1 = vec![
            self.value.as_bytes(),
            self.validity.to_string().as_bytes(),
            &[ValidityType::Eol as u8],
        ]
        .concat();

        let data = cbor::Data {
            value: self.value.as_bytes().to_vec(),
            validity: self.validity.to_string().as_bytes().to_vec(),
            validity_type: self.validity_type.into(),
            sequence: self.sequence,
            ttl: self.ttl,
        };

        let v2 = vec!["ipns-signature:".as_bytes(), &data.to_bytes()].concat();

        (data, Signables { v1, v2 })
    }
}
