#![allow(rustdoc::invalid_html_tags)]
//! IPNS Entry built from Protobuf using prost in `./build.rs`
//! Actual rust file saved in target\debug\build\ipns-<hash>\out\ipns.rs
pub mod protobuf {
    include!(concat!(env!("OUT_DIR"), "/ipns.rs"));
}

use crate::cbor;
use anyhow::{anyhow, Error, Result};
use cbor::Data;
use libp2p_identity::{ed25519, PeerId};
use prost::Message; // so we can use trait Message
pub use protobuf::ipns_entry::ValidityType;
pub use protobuf::IpnsEntry;
use std::io::Cursor; // so we can use decode

impl IpnsEntry {
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

    pub fn get_public_key(&self, binary_id: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
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
            Some(pk) => Ok(pk.to_vec()),
            None => match &PeerId::from_bytes(binary_id) {
                Ok(peer_id) => Ok(peer_id.to_bytes().to_vec()),
                Err(_) => return Err(anyhow!("Invalid PeerId")),
            },
        }
    }

    /// Get Deserialized IpnsEntry.data as a DAG-CBOR document
    pub fn get_data(&self) -> Result<Data, Error> {
        // Confirm IpnsEntry.signatureV2 and IpnsEntry.data are present and are not empty
        if self.data.is_none() {
            return Err(anyhow!("Missing IpnsEntry.data"));
        }

        // Deserialize IpnsEntry.data as a DAG-CBOR document
        let data: &[u8] = &self.data.as_ref().unwrap().to_vec();
        match cbor::Data::from_bytes(data) {
            Ok(data) => Ok(data),
            Err(_) => return Err(anyhow!("Invalid DAG-CBOR")),
        }
    }

    /// Confirm values in IpnsEntry protobuf match deserialized ones from IpnsEntry.data:
    pub fn is_valid_for(&self, binary_id: Vec<u8>) -> Result<bool, Error> {
        let data = self.get_data()?;
        // IpnsEntry.value must match IpnsEntry.data[value]
        if self.value.as_ref().unwrap() != &data.value {
            return Err(anyhow!(
                "IpnsEntry.value does not match IpnsEntry.data[value]"
            ));
        }

        // IpnsEntry.validity must match IpnsEntry.data[validity]
        if self.validity.as_ref().unwrap() != &data.validity {
            return Err(anyhow!(
                "IpnsEntry.validity does not match IpnsEntry.data[validity]"
            ));
        }

        // IpnsEntry.validityType must match IpnsEntry.data[validityType]
        if self.validity_type.as_ref().unwrap() != &data.validity_type {
            return Err(anyhow!(
                "IpnsEntry.validityType does not match IpnsEntry.data[validityType]"
            ));
        }
        // IpnsEntry.sequence must match IpnsEntry.data[sequence]
        if self.sequence.as_ref().unwrap() != &data.sequence {
            return Err(anyhow!(
                "IpnsEntry.sequence does not match IpnsEntry.data[sequence]"
            ));
        }

        // IpnsEntry.ttl must match IpnsEntry.data[ttl]
        if self.ttl.as_ref().unwrap() != &data.ttl {
            return Err(anyhow!("IpnsEntry.ttl does not match IpnsEntry.data[ttl]"));
        }

        // Verify signature in IpnsEntry.signatureV2 against IpnsEntry pub_key and IpnsEntry.data
        // get_public_key
        let pub_key = self.get_public_key(binary_id.as_ref())?;
        let signature_v2 = self.signature_v2.as_ref().unwrap();
        let bytes_for_signing = data.to_bytes();

        // use V2Signer to create a signer with pubKey and verify signature
        let valid_sig =
            ed25519::PublicKey::try_from_bytes(&pub_key)?.verify(&bytes_for_signing, signature_v2);

        Ok(valid_sig)
    }
}

pub fn deserialize(buf: &[u8]) -> Result<IpnsEntry, prost::DecodeError> {
    IpnsEntry::decode(&mut Cursor::new(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipns_entry() {
        let entry = IpnsEntry::default();

        let bytes = entry.to_bytes();
        let entry2 = IpnsEntry::from_bytes(&bytes).unwrap();
        assert_eq!(entry, entry2);
    }
}
