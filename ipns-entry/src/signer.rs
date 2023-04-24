//! Re-export of `libp2p_identity::Keypair`
use libp2p_identity::*;

/// Generate a new ed25519 keypair for signing cbor data.
pub fn generate() -> Keypair {
    Keypair::generate_ed25519()
}

/// Used to sign bytes created by `cbor::InputData{}.to_bytes()`
pub struct V2Signer {
    keypair: ed25519::Keypair,
}

impl V2Signer {
    pub fn new(keypair: &ed25519::Keypair) -> Self {
        Self {
            keypair: keypair.clone(),
        }
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        // Create bytes for signing by concatenating ipns-signature: prefix
        // (bytes in hex: 69706e732d7369676e61747572653a)
        // with raw CBOR bytes from data
        let bytes_for_signing = vec!["ipns-signature:".as_bytes(), data].concat();

        self.keypair.sign(&bytes_for_signing)
    }

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        // Create bytes for signing by concatenating ipns-signature: prefix
        // (bytes in hex: 69706e732d7369676e61747572653a)
        // with raw CBOR bytes from data
        let bytes_for_signing = vec!["ipns-signature:".as_bytes(), data].concat();

        self.keypair.public().verify(&bytes_for_signing, signature)
    }
}

/// *Do not use.* For backward compatibility only.
pub struct V1Signer {
    pub keypair: ed25519::Keypair,
    pub value: &'static [u8],
    pub validity: &'static [u8],
    pub validity_type: u8,
}

impl V1Signer {
    /// Creates a new V1Signer
    pub fn new(
        keypair: &ed25519::Keypair,
        value: &'static [u8],
        validity: &'static [u8],
        validity_type: u8,
    ) -> Self {
        Self {
            keypair: keypair.clone(),
            value,
            validity,
            validity_type,
        }
    }

    /// Creates a IpnsEntry.signatureV1 by concatenating IpnsEntry.value + IpnsEntry.validity + string(IpnsEntry.validityType)
    pub fn sign(&self) -> Vec<u8> {
        let bytes_for_signing = vec![
            self.value,
            self.validity,
            self.validity_type.to_string().as_bytes(),
        ]
        .concat();

        self.keypair.sign(&bytes_for_signing)
    }

    /// Verify the V1Signer data using the keypair and signature
    pub fn verify(&self, signature: &[u8]) -> bool {
        let bytes_for_signing = vec![
            self.value,
            self.validity,
            self.validity_type.to_string().as_bytes(),
        ]
        .concat();

        self.keypair.public().verify(&bytes_for_signing, signature)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::cbor;

    #[test]
    fn test_signers() {
        let keypair = Keypair::generate_ed25519()
            .try_into_ed25519()
            .expect("A ed25519 keypair");

        let value = "QmWEekX7EZLUd9VXRNMRXW3LXe4F6x7mB8oPxY5XLptrBq";
        let validity = "2033-05-18T03:33:20.000000000Z";
        let sequence = 0;
        let ttl = 0;

        let data = cbor::Data {
            value: value.as_bytes().to_vec(),
            validity: validity.as_bytes().to_vec(),
            validity_type: 0,
            sequence,
            ttl,
        }
        .to_bytes();

        let v2_signer = V2Signer::new(&keypair);
        let sig_v2 = v2_signer.sign(&data);

        let v1_signer = V1Signer {
            keypair,
            validity: validity.as_bytes(),
            value: value.as_bytes(),
            validity_type: 0,
        };
        let sig_v1 = v1_signer.sign();

        // verify signatures
        assert!(v2_signer.verify(&data, &sig_v2));
        assert!(v1_signer.verify(&sig_v1));
    }
}