//! CBOR serialization and deserialization for IPNS entries.
//!
//!
//!
//! # Example
//!
//! ```rust
//! use ipns_entry::cbor;
//!
//! let value = "QmWEekX7EZLUd9VXRNMRXW3LXe4F6x7mB8oPxY5XLptrBq";
//! let validity = "2033-05-18T03:33:20.000000000Z";
//! let sequence = 0;
//! let ttl = 0;
//!
//! let data = cbor::Data {
//!     value: value.as_bytes().to_vec(),
//!     validity: validity.as_bytes().to_vec(),
//!     validity_type: 0,
//!     sequence,
//!     ttl,
//! }
//! .to_bytes();
//!
//! // ...sign the data
//! ```
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use crate::entry::ValidityType;
use cbor4ii::serde::{from_slice, to_vec, DecodeError};
use serde_derive::{Deserialize, Serialize};
use std::convert::Infallible;

// Types annotated with `Serialize` can be stored as CBOR.
// To be able to load them again add `Deserialize`.
/// DAG-CBOR document with the same values for value, validity, validityType, sequence, and ttl
/// The Pascal case (e.g. ValidityType) is required for the CBOR serialization.
/// The types are set to match those created by the js-ipfs implementation so it interoperates.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct CborData<'a> {
    pub Sequence: u64,
    pub TTL: u64,
    #[serde(with = "serde_bytes")]
    pub Validity: &'a [u8],
    pub ValidityType: i32,
    #[serde(with = "serde_bytes")]
    pub Value: &'a [u8],
}

/// Struct to hold the data to create the CBOR bytes.
///
/// # Example
///
/// ```rust
/// use ipns_entry::cbor;
///
/// let value = "QmWEekX7EZLUd9VXRNMRXW3LXe4F6x7mB8oPxY5XLptrBq";
/// let validity = "2033-05-18T03:33:20.000000000Z";
/// let sequence = 0;
/// let ttl = 0;
///
/// let data = cbor::Data {
///     value: value.as_bytes().to_vec(),
///     validity: validity.as_bytes().to_vec(),
///     validity_type: 0,
///     sequence,
///     ttl,
/// }
/// .to_bytes();
///
/// // ...sign the data
/// ```
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Hash)]

pub struct Data {
    pub value: Vec<u8>,
    pub validity: Vec<u8>,
    pub sequence: u64,
    pub ttl: u64,
    pub validity_type: i32, // to match codegen by prost
}

// impl from CborData into Data
impl From<CborData<'_>> for Data {
    fn from(cbor_data: CborData) -> Self {
        Data {
            value: cbor_data.Value.to_vec(),
            validity: cbor_data.Validity.to_vec(),
            sequence: cbor_data.Sequence,
            ttl: cbor_data.TTL,
            validity_type: cbor_data.ValidityType,
        }
    }
}

impl Data {
    pub fn to_bytes(&self) -> Vec<u8> {
        create_cbor_data(&self.value, &self.validity, &self.sequence, self.ttl)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Data, DecodeError<Infallible>> {
        let cbor_data = parse_cbor_data(bytes).expect("Valid cbor");

        Ok(Data {
            value: cbor_data.Value.to_vec(),
            validity: cbor_data.Validity.to_vec(),
            sequence: cbor_data.Sequence,
            ttl: cbor_data.TTL,
            validity_type: cbor_data.ValidityType,
        })
    }
}

fn create_cbor_data(value: &[u8], validity: &[u8], sequence: &u64, ttl: u64) -> Vec<u8> {
    let data = CborData {
        Value: value,
        Validity: validity,
        ValidityType: 0, // the only supported value is zero (0)
        Sequence: *sequence,
        TTL: ttl,
    };

    to_vec(Vec::new(), &data).expect("Cannot serialize data")
}

/// Convert CBOR bytes into a Data struct of IPNS Entry
/// Only really useful for roundtrip testing, as you'd never use this in production
fn parse_cbor_data(bytes: &[u8]) -> Result<CborData, DecodeError<Infallible>> {
    from_slice(bytes)
}

// impl trait `std::convert::From<pb::entry::mod_IpnsEntry::ValidityType>` for `isize`
impl From<ValidityType> for isize {
    fn from(v: ValidityType) -> Self {
        match v {
            ValidityType::Eol => 0,
        }
    }
}

// and isize too
impl From<isize> for ValidityType {
    fn from(v: isize) -> Self {
        match v {
            0 => ValidityType::Eol,
            _ => panic!("Invalid ValidityType"),
        }
    }
}

// and u64 too
impl From<u64> for ValidityType {
    fn from(v: u64) -> Self {
        match v {
            0 => ValidityType::Eol,
            _ => panic!("Invalid ValidityType"),
        }
    }
}

// and back from u64
impl From<ValidityType> for u64 {
    fn from(v: ValidityType) -> Self {
        match v {
            ValidityType::Eol => 0,
        }
    }
}
// do the trait `std::convert::From<pb::entry::mod_IpnsEntry::ValidityType>` for `usize`
impl From<ValidityType> for usize {
    fn from(v: ValidityType) -> Self {
        match v {
            ValidityType::Eol => 0,
        }
    }
}

// and back too
impl From<usize> for ValidityType {
    fn from(v: usize) -> Self {
        match v {
            0 => ValidityType::Eol,
            _ => panic!("Invalid ValidityType"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create IpnsEntry and set: value, validity, validityType, sequence, and ttl
    fn get_entry() -> CborData<'static> {
        CborData {
            TTL: 31838814734000000_u64,
            Value: b"QmWEekX7EZLUd9VXRNMRXW3LXe4F6x7mB8oPxY5XLptrBq",
            Sequence: 0,
            Validity: b"2033-05-18T03:33:20.000000000Z",
            ValidityType: ValidityType::Eol.into(),
        }
    }

    #[test]
    fn test_roundtrip() {
        use cbor4ii::serde::{from_slice, to_vec};

        // see how cbor4ii serializes bytes
        let entry = get_entry();
        let cbor = to_vec(Vec::new(), &entry).expect("Cannot serialize data");
        assert_eq!(parse_cbor_data(&cbor).expect("Ok"), entry);
        assert_eq!(from_slice::<CborData>(&cbor).expect("Ok"), entry);
    }

    #[test]
    fn test_read_js_bytes() {
        // Test to ensure we have good interop with non-Rust CBOR IPNS encodings (ie Javascript)
        let data = get_entry();

        let cbor = create_cbor_data(data.Value, data.Validity, &data.Sequence, data.TTL);

        assert_eq!(parse_cbor_data(&cbor).expect("Ok"), data);

        // CBOR bytes generated from Javascript:
        let bytes = vec![
            165, 99, 84, 84, 76, 27, 0, 113, 29, 59, 186, 74, 31, 128, 101, 86, 97, 108, 117, 101,
            88, 46, 81, 109, 87, 69, 101, 107, 88, 55, 69, 90, 76, 85, 100, 57, 86, 88, 82, 78, 77,
            82, 88, 87, 51, 76, 88, 101, 52, 70, 54, 120, 55, 109, 66, 56, 111, 80, 120, 89, 53,
            88, 76, 112, 116, 114, 66, 113, 104, 83, 101, 113, 117, 101, 110, 99, 101, 0, 104, 86,
            97, 108, 105, 100, 105, 116, 121, 88, 30, 50, 48, 51, 51, 45, 48, 53, 45, 49, 56, 84,
            48, 51, 58, 51, 51, 58, 50, 48, 46, 48, 48, 48, 48, 48, 48, 48, 48, 48, 90, 108, 86,
            97, 108, 105, 100, 105, 116, 121, 84, 121, 112, 101, 0,
        ];

        let data_from_js_bytes: CborData = from_slice(&bytes).unwrap();
        assert_eq!(data_from_js_bytes, data);
        assert_eq!(parse_cbor_data(&bytes).expect("Ok"), data);
    }
}
