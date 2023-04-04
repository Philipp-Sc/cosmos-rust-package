
use serde::{Deserialize, Serialize};
use cosmos_sdk_proto::prost::Message;
use cosmos_sdk_proto::cosmos::staking::v1beta1::QueryPoolResponse;
use crate::api::custom::types::ProtoMessageWrapper;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash)]
pub struct PoolExt {
    pub blockchain_name: String,
    pub pool: ProtoMessageWrapper<QueryPoolResponse>,
}

impl PoolExt {
    pub fn new(blockchain_name: &str, query_pool_response: QueryPoolResponse) -> Self {
        Self {
            blockchain_name: blockchain_name.to_string(),
            pool: ProtoMessageWrapper(query_pool_response)
        }
    }
}