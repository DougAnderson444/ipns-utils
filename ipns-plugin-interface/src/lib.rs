use ipns_entry::cbor;
use ipns_entry::signer;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
pub struct Output {
    pub count: i32,
    pub config: String,
    pub a: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
pub struct SignableData {
    pub data: cbor::Data,
    pub signables: signer::Signables,
}
