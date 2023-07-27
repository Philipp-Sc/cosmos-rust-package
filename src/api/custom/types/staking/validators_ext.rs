use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};

use crate::api::core::cosmos::channels::SupportedBlockchain;
use crate::api::custom::types::ProtoMessageWrapper;
use cosmos_sdk_proto::cosmos::staking::v1beta1::Validator;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash)]
pub struct ValidatorsExt {
    pub blockchain: SupportedBlockchain,
    pub validators: ProtoMessageWrapper<Validator>,
}

impl ValidatorsExt {
    pub fn new(
        blockchain: &SupportedBlockchain,
        validator: Validator,
    ) -> Self {
        Self {
            blockchain: blockchain.clone(),
            validators: ProtoMessageWrapper(validator),
        }
    }
    pub fn object_to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        &self.hash(&mut s);
        s.finish()
    }
}
