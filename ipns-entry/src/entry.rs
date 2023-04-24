#![allow(rustdoc::invalid_html_tags)]
//! IPNS Entry built from Protobuf using prost in `./build.rs`
//! Actual rust file saved in target\debug\build\ipns-<hash>\out\ipns.rs
pub mod protobuf {
    include!(concat!(env!("OUT_DIR"), "/ipns.rs"));
}

use crate::cbor;
use crate::signer::{Signables, Signed};
use anyhow::{anyhow, Error, Result};
use cbor::Data;
use libp2p_identity::{ed25519, PeerId, PublicKey};
use multihash::Multihash;
use prost::Message; // so we can use trait Message
pub use protobuf::ipns_entry::ValidityType;
pub use protobuf::IpnsEntry;
use std::io::Cursor; // so we can use decode

impl IpnsEntry {
    pub fn new(data: Data, signed: Signed) -> Self {
        Self {
            data: Some(data.to_bytes()),
            value: Some(data.value),
            validity: Some(data.validity),
            validity_type: Some(ValidityType::Eol.into()),
            signature_v1: Some(signed.v1),
            signature_v2: Some(signed.v2),
            sequence: Some(data.sequence),
            ttl: Some(data.ttl),
            pub_key: None,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.reserve(self.encoded_len());
        // Unwrap is safe, since we have reserved sufficient capacity in the vector.
        self.encode(&mut buf).unwrap();

        // assert that serialized IpnsEntry less than or equal to 10 KiB in size.
        // This is the maximum size of an IPNS record.
        assert!(buf.len() <= 10 * 1024);

        buf
    }

    /// Decode protobuf bytes into an IpnsEntry
    /// Ensures the bytes are less than or equal to 10 KiB in size.
    pub fn from_bytes(buf: &[u8]) -> Result<Self, prost::DecodeError> {
        // assert that serialized IpnsEntry less than or equal to 10 KiB in size.
        // This is the maximum size of an IPNS record.
        assert!(buf.len() <= 10 * 1024);

        Self::decode(&mut Cursor::new(buf))
    }

    pub fn get_public_key(&self, peer_id: &PeerId) -> Result<PublicKey, anyhow::Error> {
        // Confirm IpnsEntry.signatureV2 and IpnsEntry.data are present and are not empty
        if self.signature_v2.is_none() || self.data.is_none() {
            return Err(anyhow!(
                "Missing both IpnsEntry.signatureV2 and IpnsEntry.data"
            ));
        }

        // Extract public key
        // Public key is either:
        // A. IpnsEntry.pubKey
        // B. inlined in the IPNS Name itself (e.g., Ed25519 inlined using identity multihash)
        //
        // If IpnsEntry.pubKey is present, use that.
        // If not, use the public key from the IPNS Name.
        match self.pub_key.as_ref() {
            Some(pk) => Ok(PublicKey::try_decode_protobuf(pk)?),
            None => match Multihash::from_bytes(&peer_id.to_bytes()) {
                Ok(mh) => {
                    let d = mh.digest();
                    let ed_key =
                        ed25519::PublicKey::try_from_bytes(&d[4..]).expect("a Valid ed25519 key");
                    let pub_key: PublicKey = PublicKey::from(ed_key);
                    Ok(pub_key)
                }
                Err(_) => return Err(anyhow!("Invalid PeerId")),
            },
        }
    }

    /// Get Deserialized IpnsEntry.data as a DAG-CBOR document
    pub fn decode_data(&self) -> Result<Data, Error> {
        // Confirm IpnsEntry.signatureV2 and IpnsEntry.data are present and are not empty
        if self.data.is_none() {
            return Err(anyhow!("Missing IpnsEntry.data"));
        }

        // Deserialize IpnsEntry.data as a DAG-CBOR document
        let data: &[u8] = self.data.as_ref().expect("Valid data");
        match cbor::Data::from_bytes(data) {
            Ok(data) => Ok(data),
            Err(_) => return Err(anyhow!("Invalid DAG-CBOR")),
        }
    }

    /// Confirm values in IpnsEntry protobuf match deserialized ones from IpnsEntry.data:
    pub fn is_valid_for(&self, peer_id: &PeerId) -> Result<bool, Error> {
        let data = self.decode_data()?;
        // IpnsEntry.value must match IpnsEntry.data[value]
        if self.value.as_ref().expect("Valid value") != &data.value {
            return Err(anyhow!(
                "IpnsEntry.value does not match IpnsEntry.data[value]"
            ));
        }

        // IpnsEntry.validity must match IpnsEntry.data[validity]
        if self.validity.as_ref().expect("Valid validity") != &data.validity {
            return Err(anyhow!(
                "IpnsEntry.validity does not match IpnsEntry.data[validity]"
            ));
        }

        // IpnsEntry.validityType must match IpnsEntry.data[validityType]
        if self.validity_type.as_ref().expect("Valid validity_type") != &data.validity_type {
            return Err(anyhow!(
                "IpnsEntry.validityType does not match IpnsEntry.data[validityType]"
            ));
        }
        // IpnsEntry.sequence must match IpnsEntry.data[sequence]
        if self.sequence.as_ref().expect("Valid sequence") != &data.sequence {
            return Err(anyhow!(
                "IpnsEntry.sequence does not match IpnsEntry.data[sequence]"
            ));
        }

        // IpnsEntry.ttl must match IpnsEntry.data[ttl]
        if self.ttl.as_ref().expect("Valid TTL") != &data.ttl {
            return Err(anyhow!("IpnsEntry.ttl does not match IpnsEntry.data[ttl]"));
        }

        // Verify signature in IpnsEntry.signatureV2 against IpnsEntry pub_key and IpnsEntry.data
        // get_public_key
        let pub_key = self.get_public_key(peer_id)?;
        let signature_v2 = self.signature_v2.as_ref().expect("Valid signature_v2");
        let v2_signable = generate_v2_signable(self.data.as_ref().expect("Valid data"));

        // use V2Signer to create a signer with pubKey and verify signature
        let valid_sig = pub_key.verify(&v2_signable, signature_v2);

        Ok(valid_sig)
    }

    pub fn signables(&self) -> Result<Signables> {
        Ok(Signables {
            v1: generate_v1_signable(
                self.value.as_ref().expect("Valid value"),
                self.validity.as_ref().expect("Valid validity"),
            ),
            v2: generate_v2_signable(self.data.as_ref().expect("Valid data")),
        })
    }
}

fn generate_v1_signable(value: &[u8], validity: &[u8]) -> Vec<u8> {
    vec![value, validity, &[ValidityType::Eol as u8]].concat()
}

fn generate_v2_signable(data: &[u8]) -> Vec<u8> {
    vec!["ipns-signature:".as_bytes(), data].concat()
}

pub fn deserialize(buf: &[u8]) -> Result<IpnsEntry, prost::DecodeError> {
    IpnsEntry::decode(&mut Cursor::new(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p_identity::Keypair;

    #[test]
    fn test_ipns_entry() {
        let entry = IpnsEntry::default();

        let bytes = entry.to_bytes();
        let entry2 = IpnsEntry::from_bytes(&bytes).unwrap();
        assert_eq!(entry, entry2);
    }

    #[test]
    fn test_is_valid_for() {
        let keypair = Keypair::generate_ed25519()
            .try_into_ed25519()
            .expect("Valid keypair");

        // create entry with valid signature first
        let mut entry = IpnsEntry {
            value: Some(b"value".to_vec()),
            validity: Some(b"0".to_vec()),
            validity_type: Some(ValidityType::Eol as i32),
            sequence: Some(0),
            ttl: Some(0),
            ..Default::default()
        };

        // create data from entry fields
        let data = Data {
            value: entry.value.clone().unwrap(),
            validity: entry.validity.clone().unwrap(),
            validity_type: entry.validity_type.unwrap(),
            sequence: entry.sequence.unwrap(),
            ttl: entry.ttl.unwrap(),
        };

        entry.data = Some(data.to_bytes());

        // create signature from data
        let signables = entry.signables().expect("valid signables");

        // print signables.v2 bytes
        println!("signables.v2: {:?}", signables.v2);
        let signature_v2 = keypair.sign(&signables.v2);
        let signature_v1 = keypair.sign(&signables.v1);

        // set entry fields
        entry.signature_v2 = Some(signature_v2);
        entry.signature_v1 = Some(signature_v1);

        // confirm entry is valid
        let peer_id = PeerId::from_public_key(&PublicKey::from(keypair.public()));
        assert!(entry.is_valid_for(&peer_id).unwrap());
    }
}
