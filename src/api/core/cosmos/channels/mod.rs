use std::collections::HashMap;
use tonic::transport::channel::Channel;
use std::time::Duration;

use cosmos_sdk_proto::cosmwasm::wasm::v1::query_client::QueryClient;
use std::fmt::{self, Debug, Display};



#[derive(Debug, Clone, PartialEq)]
pub struct SupportedBlockchain{
    pub name: String,
    pub prefix: String,
    pub grpc_nodes: Vec<String>,
    pub governance_proposals_link: String,
}

impl SupportedBlockchain {
    pub async fn channel(&self) -> anyhow::Result<String> {
        if self.grpc_nodes.len() > 0 {
            for_blockchain(self.grpc_nodes[0].to_owned()).await
        }else{
            Err(anyhow::anyhow!(format!("Error: {:?} is not a supported cosmos blockchain!",self.name)))
        }
    }
}

pub async fn for_blockchain(grpc_url: String) -> anyhow::Result<String> {
    /*let channel = Channel::from_shared(grpc_url)?
        .timeout(Duration::from_secs(5))
        .rate_limit(5, Duration::from_secs(1))
        .concurrency_limit(256)
        .connect()
        .await?;
    Ok(channel)*/
    //Channel::builder(grpc_url.parse().unwrap()).connect().await?;
    //tonic::transport::Endpoint::from_shared(grpc_url);
    Ok(grpc_url)
}

// todo: load from file, so that users can add their own blockchains.
// todo: go trough grpc_nodes list if node offline, or transport error, etc.
pub fn get_supported_blockchains() -> HashMap<String,SupportedBlockchain> {
    let mut supported_blockchains: HashMap<String,SupportedBlockchain> = HashMap::new();
    supported_blockchains.insert("terra".to_string(),SupportedBlockchain{
        name: "Terra".to_string(),
        prefix: "terra".to_string(),
        grpc_nodes: vec!["http://n-fsn-7.zyons.com:29090".to_string()],
        governance_proposals_link: "https://station.terra.money/proposal/".to_string()
    });
    supported_blockchains.insert("osmosis".to_string(),SupportedBlockchain{
        name: "Osmosis".to_string(),
        prefix: "osmo".to_string(),
        grpc_nodes: vec!["http://osmosis.strange.love:9090".to_string()],
        governance_proposals_link: "https://www.mintscan.io/osmosis/proposals/".to_string()
    });
    supported_blockchains.insert("juno".to_string(),SupportedBlockchain{
        name: "Juno".to_string(),
        prefix: "juno".to_string(),
        grpc_nodes: vec!["http://juno-grpc.polkachu.com:9090".to_string()],
        governance_proposals_link: "https://www.mintscan.io/juno/proposals/".to_string()
    });
    supported_blockchains
}