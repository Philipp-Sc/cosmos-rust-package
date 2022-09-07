use std::collections::HashMap;
use std::time::Duration;
use tonic::transport::channel::Channel;

use cosmos_sdk_proto::cosmwasm::wasm::v1::query_client::QueryClient;
use std::fmt::{self, Debug, Display};

#[derive(Debug, Clone, PartialEq)]
pub struct SupportedBlockchain {
    pub name: String,
    pub prefix: String,
    pub grpc_nodes: Vec<String>,
    pub governance_proposals_link: String,
}

impl SupportedBlockchain {
    pub async fn channel(&self) -> anyhow::Result<String> {
        if self.grpc_nodes.len() > 0 {
            for_blockchain(&self.grpc_nodes).await
        } else {
            Err(anyhow::anyhow!(format!(
                "Error: {:?} is not a supported cosmos blockchain!",
                self.name
            )))
        }
    }
}

pub struct MyEndpoint {
    pub endpoint: tonic::transport::Endpoint,
}

impl MyEndpoint {
    fn try_into(self) -> Result<tonic::transport::Endpoint, anyhow::Error> {
        Ok(self.endpoint)
    }
}

impl From<MyEndpoint> for tonic::transport::channel::endpoint::Endpoint {
    fn from(item: MyEndpoint) -> Self {
        item.try_into().unwrap()
    }
}

pub async fn for_blockchain(grpc_urls: &Vec<String>) -> anyhow::Result<String> {
    // iterate over all grpc_urls and test connectivity.

    /*let channel = Channel::from_shared(grpc_url)?
        .timeout(Duration::from_secs(5))
        .rate_limit(5, Duration::from_secs(1))
        .concurrency_limit(256)
        .connect()
        .await?;
    Ok(channel)*/
    let mut channel: Result<Channel, anyhow::Error> =
        Err(anyhow::anyhow!("Error: Empty Set of gRPC Nodes!"));
    for grpc_url in grpc_urls {
        let my_endpoint = MyEndpoint {
            endpoint: tonic::transport::Endpoint::new(grpc_url.parse().unwrap()).unwrap(), //Channel::builder(grpc_url.parse().unwrap()),
        };
        QueryClient::new(my_endpoint.endpoint.connect().await?);
        channel = match Channel::builder(grpc_url.parse().unwrap()).connect().await {
            Ok(c) => Ok(c),
            Err(e) => Err(anyhow::anyhow!(e)),
        };
        if let Ok(_) = channel {
            return Ok(grpc_url.to_string());
        }
    }
    //tonic::transport::Endpoint::from_shared(grpc_url);
    match channel {
        Ok(_) => {
            panic!()
        }
        Err(e) => Err(e),
    }
}

// todo: load from file, so that users can add their own blockchains.
// todo: go trough grpc_nodes list if node offline, or transport error, etc.
pub fn get_supported_blockchains() -> HashMap<String, SupportedBlockchain> {
    let mut supported_blockchains: HashMap<String, SupportedBlockchain> = HashMap::new();
    supported_blockchains.insert(
        "terra".to_string(),
        SupportedBlockchain {
            name: "Terra".to_string(),
            prefix: "terra".to_string(),
            grpc_nodes: vec!["http://n-fsn-7.zyons.com:29090".to_string()],
            governance_proposals_link: "https://station.terra.money/proposal/".to_string(),
        },
    );
    supported_blockchains.insert(
        "osmosis".to_string(),
        SupportedBlockchain {
            name: "Osmosis".to_string(),
            prefix: "osmo".to_string(),
            grpc_nodes: vec![
                "http://osmosis.strange.love:9090".to_string(),
                "htto://grpc-osmosis-ia.cosmosia.notional.ventures:443".to_string(),
                "https://osmosis-grpc.polkachu.com:9090".to_string(),
            ],
            governance_proposals_link: "https://wallet.keplr.app/chains/osmosis/proposals/"
                .to_string(),
        },
    );
    supported_blockchains.insert(
        "juno".to_string(),
        SupportedBlockchain {
            name: "Juno".to_string(),
            prefix: "juno".to_string(),
            grpc_nodes: vec!["http://juno-grpc.polkachu.com:9090".to_string()],
            governance_proposals_link: "https://wallet.keplr.app/chains/juno/proposals/"
                .to_string(),
        },
    );
    // todo: add atom cosmos-hub
    supported_blockchains
}
