use crate::api::core::cosmos::channels::SupportedBlockchain;
use crate::api::custom::types::staking::pool_ext::PoolExt;

pub async fn get_pool(blockchain: SupportedBlockchain) -> anyhow::Result<PoolExt> {
    let channel = blockchain.channel().await?;
    let res = super::super::super::core::cosmos::query::staking::get_pool(channel).await?;
    Ok(PoolExt::new(blockchain, res))
}

#[cfg(test)]
mod test {

    // cargo test -- --nocapture
    // cargo test -- --list
    // cargo test api::custom::query::staking::test::test_get_pool_function -- --exact --nocapture

    use super::*;
    use crate::api::core::cosmos::channels::GRPC_Service;

    #[tokio::test]
    async fn test_get_pool_function() {
        let supported_blockchain = SupportedBlockchain {
            display: "Osmosis".to_string(),
            name: "osmosis".to_string(),
            prefix: "osmo".to_string(),
            grpc_service: GRPC_Service {
                grpc_urls: vec!["https://osmosis-grpc.lavenderfive.com:443".to_string()],
                error: None,
            },
            rank: 1,
            governance_proposals_link: "".to_string(),
        };
        let result = get_pool(supported_blockchain).await;
        println!("Result: {:?}", result);
        assert!(result.is_ok());

        let serialized = serde_json::to_string(&result.unwrap()).unwrap();
        println!("\n\nserialized = {}", &serialized);
        let deserialized: PoolExt = serde_json::from_str(&serialized).unwrap();
        println!("\ndeserialized = {:?}", deserialized);
    }
}
