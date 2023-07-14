
use serde::{Deserialize, Serialize};
use cosmos_sdk_proto::prost::Message;
use cosmos_sdk_proto::cosmos::staking::v1beta1::{QueryValidatorsResponse};
use crate::api::core::cosmos::channels::SupportedBlockchain;
use crate::api::custom::types::ProtoMessageWrapper;


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash)]
pub struct ValidatorsExt {
    pub blockchain: SupportedBlockchain,
    pub validators: ProtoMessageWrapper<QueryValidatorsResponse>,
}

impl ValidatorsExt {
    pub fn new(blockchain: SupportedBlockchain, query_validators_response: QueryValidatorsResponse) -> Self {
        Self {
            blockchain,
            validators: ProtoMessageWrapper(query_validators_response)
        }
    }
}