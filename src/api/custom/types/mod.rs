pub mod gov;
pub mod staking;

use cosmos_sdk_proto::prost::Message;
use serde::{Deserialize, Serialize};

use std::hash::{Hash, Hasher};

use crate::api::custom::types::gov::params_ext::ParamsExt;
use crate::api::custom::types::gov::proposal_ext::ProposalExt;
use crate::api::custom::types::gov::tally_ext::TallyResultExt;
use crate::api::custom::types::staking::pool_ext::PoolExt;
use crate::api::custom::types::staking::validators_ext::ValidatorsExt;


pub type GovernanceProposalsType = Vec<ProposalExt>;
pub type ParamsType = ParamsExt;
pub type NextKeyType = Option<Vec<u8>>;
pub type TallyResultType = TallyResultExt;
pub type PoolType = PoolExt;
pub type ValidatorsType = ValidatorsExt;

// This wrapper implements Serialize/Deserialize and Hash for the inner type ::prost::Message object.

#[derive(Clone, Debug, PartialEq)]
pub struct ProtoMessageWrapper<T>(pub T);

impl<T> ProtoMessageWrapper<T>
where
    T: Message + Default,
{
    fn into_inner(self) -> T {
        self.0
    }

    fn from_inner(proposal: T) -> Self {
        ProtoMessageWrapper(proposal)
    }
}

impl<T> Serialize for ProtoMessageWrapper<T>
where
    T: Message + Default,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = self.0.encode_to_vec();
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de, T> Deserialize<'de> for ProtoMessageWrapper<T>
where
    T: Message + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = serde::Deserialize::deserialize(deserializer)?;
        T::decode(&bytes[..])
            .map(ProtoMessageWrapper)
            .map_err(|e| serde::de::Error::custom(format!("Error decoding message: {}", e)))
    }
}

impl<T> Hash for ProtoMessageWrapper<T>
where
    T: Message,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bytes = self.0.encode_to_vec();
        bytes.hash(state);
    }
}

