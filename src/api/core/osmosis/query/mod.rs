/*
use cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;

//use cosmos_sdk_proto::cosmwasm::wasm::v1::query_client::QueryClient;
//use cosmos_sdk_proto::cosmwasm::wasm::v1::*;
//use prost_types::Any;

use osmosis_proto::osmosis::gamm::v1beta1::query_client::QueryClient as OsmosisQueryClient;
use osmosis_proto::osmosis::gamm::v1beta1::{
    Pool, QueryNumPoolsRequest, QueryNumPoolsResponse, QueryPoolRequest, /*QueryPoolResponse,*/
    QueryPoolsRequest, /*QueryPoolsResponse,*/ QuerySwapExactAmountInRequest,
    QuerySwapExactAmountInResponse, QuerySwapExactAmountOutRequest,
    QuerySwapExactAmountOutResponse, SwapAmountInRoute, SwapAmountOutRoute,
};

//use cosmos_sdk_proto::cosmos::auth::v1beta1::query_client::QueryClient as AuthQueryClient;
/*use cosmos_sdk_proto::cosmos::auth::v1beta1::{
    BaseAccount, QueryAccountRequest, QueryAccountResponse,
};*/
//use cosmos_sdk_proto::cosmos::vesting::v1beta1::PeriodicVestingAccount;

//use serde_json;
use std::str;

//use crate::api::core::cosmos::channels;
use tonic::transport::Channel;

// Swap a maximum amount of tokens for an exact amount of another token, similar to swapping a token on the trade screen GUI.
pub async fn get_estimate_swap_exact_amount_out(
    channel: Channel,
    sender: &str,
    pool_id: u64,
    token_out: &str,
    routes: Vec<SwapAmountOutRoute>,
) -> anyhow::Result<QuerySwapExactAmountOutResponse> {
    let res = OsmosisQueryClient::new(channel)
        .estimate_swap_exact_amount_out(QuerySwapExactAmountOutRequest {
            sender: sender.to_string(),
            pool_id,
            routes,
            token_out: token_out.to_string(),
        })
        .await?
        .into_inner();
    //println!("{:?}", &res.num_pools);
    Ok(res)
}

// Swap an exact amount of tokens for a minimum of another token, similar to swapping a token on the trade screen GUI.
pub async fn get_estimate_swap_exact_amount_in(
    channel: Channel,
    sender: &str,
    pool_id: u64,
    token_in: &str,
    routes: Vec<SwapAmountInRoute>,
) -> anyhow::Result<QuerySwapExactAmountInResponse> {
    let res = OsmosisQueryClient::new(channel)
        .estimate_swap_exact_amount_in(QuerySwapExactAmountInRequest {
            sender: sender.to_string(),
            pool_id,
            token_in: token_in.to_string(),
            routes,
        })
        .await?
        .into_inner();
    //println!("{:?}", &res.num_pools);
    Ok(res)
}

pub async fn get_pool_count(channel: Channel) -> anyhow::Result<QueryNumPoolsResponse> {
    let res = OsmosisQueryClient::new(channel)
        .num_pools(QueryNumPoolsRequest {})
        .await?
        .into_inner();
    //println!("{:?}", &res.num_pools);
    Ok(res)
}

pub async fn get_pools_info(
    channel: Channel,
    pagination: Option<PageRequest>,
) -> anyhow::Result<Vec<Pool>> {
    let res = OsmosisQueryClient::new(channel)
        .pools(QueryPoolsRequest { pagination })
        .await?
        .into_inner();

    let pools: Vec<Pool> = res
        .pools
        .into_iter()
        .map(|x| cosmos_sdk_proto::traits::MessageExt::from_any(&x).unwrap())
        .collect();
    //println!("{:?}", pools);
    Ok(pools)
}

pub async fn get_pool_info(channel: Channel, pool_id: u64) -> anyhow::Result<Pool> {
    let res = OsmosisQueryClient::new(channel)
        .pool(QueryPoolRequest { pool_id: pool_id })
        .await?
        .into_inner();

    let pool: Pool = cosmos_sdk_proto::traits::MessageExt::from_any(&res.pool.unwrap()).unwrap();
    //println!("{:?}", pool);
    Ok(pool)
}

#[cfg(test)]
mod test {

    // cargo test -- --nocapture

    #[tokio::test]
    pub async fn get_estimate_swap_exact_amount_out() -> anyhow::Result<()> {
        let channel = super::channels::get_supported_blockchains_from_chain_registry("./packages/chain-registry".to_string(),true,None)
            .await.get("osmosis")
            .unwrap()
            .channel()
            .await?;
        let res = super::get_estimate_swap_exact_amount_out(
            channel,
            "osmo10885ryvnfvu7hjt8lqvge77uderycqcu50nmmh",
            497,
            "1000000uosmo",
            vec![osmosis_proto::osmosis::gamm::v1beta1::SwapAmountOutRoute {
                pool_id: 497,
                token_in_denom:
                    "ibc/46B44899322F3CD854D2D46DEEF881958467CDD4B3B10086DA49296BBED94BED"
                        .to_string(),
            }],
        )
        .await?;
        //println!("{:?}", res);
        Ok(())
    }
    #[tokio::test]
    pub async fn get_estimate_swap_exact_amount_in() -> anyhow::Result<()> {
        let channel = super::channels::get_supported_blockchains_from_chain_registry("./packages/chain-registry".to_string(),true,None)
            .await.get("osmosis")
            .unwrap()
            .channel()
            .await?;
        let res = super::get_estimate_swap_exact_amount_in(
            channel,
            "osmo10885ryvnfvu7hjt8lqvge77uderycqcu50nmmh",
            497,
            "2704031ibc/46B44899322F3CD854D2D46DEEF881958467CDD4B3B10086DA49296BBED94BED",
            vec![osmosis_proto::osmosis::gamm::v1beta1::SwapAmountInRoute {
                pool_id: 497,
                token_out_denom: "uosmo".to_string(),
            }],
        )
        .await?;
        //println!("{:?}", res);
        Ok(())
    }

    #[tokio::test]
    pub async fn get_pool_count() -> anyhow::Result<()> {
        let channel = super::channels::get_supported_blockchains_from_chain_registry("./packages/chain-registry".to_string(),true,None)
            .await.get("osmosis")
            .unwrap()
            .channel()
            .await?;
        let pool_count = super::get_pool_count(channel).await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn get_pools_info() -> anyhow::Result<()> {
        let channel = super::channels::get_supported_blockchains_from_chain_registry("./packages/chain-registry".to_string(),true,None)
            .await.get("osmosis")
            .unwrap()
            .channel()
            .await?;
        let pools = super::get_pools_info(
            channel,
            Some(
                cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest {
                    key: vec![],
                    offset: 0,
                    limit: 100,
                    count_total: false,
                    reverse: false,
                },
            ),
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn get_pool_info() -> anyhow::Result<()> {
        let channel = super::channels::get_supported_blockchains_from_chain_registry("./packages/chain-registry".to_string(),true,None)
            .await.get("osmosis")
            .unwrap()
            .channel()
            .await?;
        let pool_count = super::get_pool_info(channel, 1).await?;
        Ok(())
    }
}
*/