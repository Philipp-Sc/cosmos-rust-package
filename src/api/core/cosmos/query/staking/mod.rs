
use tonic::transport::Channel;

use cosmos_sdk_proto::cosmos::staking::v1beta1::{QueryPoolRequest, QueryValidatorsRequest, QueryValidatorsResponse};
use cosmos_sdk_proto::cosmos::staking::v1beta1::QueryPoolResponse;
use cosmos_sdk_proto::cosmos::staking::v1beta1::query_client::QueryClient as StakingQueryClient;

pub async fn get_pool(
    channel: Channel
) -> anyhow::Result<QueryPoolResponse> {
    let res = StakingQueryClient::new(channel)
        .pool(QueryPoolRequest {})
        .await?
        .into_inner();
    Ok(res)
}

pub async fn get_validators(
    channel: Channel,
    query_validators_request: QueryValidatorsRequest
) -> anyhow::Result<QueryValidatorsResponse> {
    let res = StakingQueryClient::new(channel)
        .validators(query_validators_request)
        .await?
        .into_inner();
    Ok(res)
}