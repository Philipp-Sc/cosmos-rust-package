
use serde::{Deserialize, Serialize};
use cosmos_sdk_proto::prost::Message;
use cosmos_sdk_proto::cosmos::staking::v1beta1::QueryPoolResponse;
use crate::api::core::cosmos::channels::SupportedBlockchain;
use crate::api::custom::types::ProtoMessageWrapper;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash)]
pub struct PoolExt {
    pub blockchain: SupportedBlockchain,
    pub pool: ProtoMessageWrapper<QueryPoolResponse>,
}

impl PoolExt {
    pub fn new(blockchain: SupportedBlockchain, query_pool_response: QueryPoolResponse) -> Self {
        Self {
            blockchain,
            pool: ProtoMessageWrapper(query_pool_response)
        }
    }
}