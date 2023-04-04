
use tonic::transport::Channel;

use cosmos_sdk_proto::cosmos::staking::v1beta1::QueryPoolRequest;
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