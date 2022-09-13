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
    pub async fn channel(&self) -> anyhow::Result<Channel> {
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

pub async fn for_blockchain(grpc_urls: &Vec<String>) -> anyhow::Result<Channel> {
    let mut channel: Result<Channel, anyhow::Error> =
        Err(anyhow::anyhow!("Error: Empty Set of gRPC Nodes!"));
    for grpc_url in grpc_urls {
        let mut endpoint = tonic::transport::Endpoint::new(grpc_url.parse::<tonic::transport::Uri>().unwrap()).unwrap();
        endpoint = endpoint.http2_adaptive_window(true).keep_alive_while_idle(true);
        channel = match endpoint.connect().await {
            Ok(c) => Ok(c),
            Err(e) => Err(anyhow::anyhow!(e)),
        };
        if let Ok(c) = channel {
            //println!("{:?}",&grpc_url);
            return Ok(c);
        }
    }
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
    /* 
    supported_blockchains.insert(
        "terra1".to_string(),
        SupportedBlockchain {
            name: "Terra1".to_string(),
            prefix: "terra".to_string(),
            grpc_nodes: vec!["http://grpc-terra-ia.cosmosia.notional.ventures:443".to_string()],
            governance_proposals_link: "https://station.terra.money/proposal/".to_string(),
        },
    );*/
    supported_blockchains.insert(
        "terra2".to_string(),
        SupportedBlockchain {
            name: "Terra2".to_string(),
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
             //   "http://grpc.osmosis.interbloc.org:443".to_string(),
             //   "http://osmosis.strange.love:9090".to_string(),
             //   "http://grpc-osmosis-ia.cosmosia.notional.ventures:443".to_string(),
                "http://osmosis-grpc.polkachu.com:9090".to_string(),
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
