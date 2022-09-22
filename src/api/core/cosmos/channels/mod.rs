use std::collections::HashMap;
use tonic::transport::channel::Channel;

use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::GetNodeInfoRequest;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::process::Output;
use tokio::task::JoinSet;

#[derive(Debug, Clone, PartialEq)]
pub struct SupportedBlockchain {
    pub name: String,
    pub prefix: String,
    pub grpc_url: Option<String>,
    pub governance_proposals_link: String,
}

impl SupportedBlockchain {
    pub async fn channel(&self) -> anyhow::Result<Channel> {
        match &self.grpc_url {
            None => Err(anyhow::anyhow!(format!(
                "Error: {:?} is not a supported cosmos blockchain!",
                self.name
            ))),
            Some(grpc_url) => get_channel(grpc_url.to_owned()).await,
        }
    }
}

async fn get_channel(grpc_url: String) -> anyhow::Result<Channel> {
    let endpoint =
        tonic::transport::Endpoint::new(grpc_url.parse::<tonic::transport::Uri>().unwrap())
            .unwrap();
    match endpoint.connect().await {
        Ok(result) => Ok(result),
        Err(err) => Err(anyhow::anyhow!(err)),
    }
}

async fn check_grpc_url(grpc_url: String) -> anyhow::Result<String> {
    match get_channel(grpc_url.to_owned()).await {
        Ok(c) => {
            match cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::new(c).get_node_info(GetNodeInfoRequest{}).await {
                Ok(_node_info_response) => {
                    // TODO potentially check versions, check if the node is up to par.
                    Ok(grpc_url)
                },
                Err(e) => {
                    //println!("{:?}",e);
                    Err(anyhow::anyhow!(e))
                }
            }
        }
        Err(e) => Err(anyhow::anyhow!(e)),
    }
}

pub async fn select_channel_from_grpc_endpoints(grpc_urls: &Vec<String>) -> anyhow::Result<String> {
    let mut join_set: JoinSet<anyhow::Result<String>> = JoinSet::new();
    for grpc_url in grpc_urls.iter().map(|x| x.to_owned()) {
        join_set.spawn(async move { check_grpc_url(grpc_url).await });
    }
    let mut channel: Result<String, anyhow::Error> =
        Err(anyhow::anyhow!("Error: No gRPC url passed the check!"));
    while !join_set.is_empty() && channel.is_err() {
        match join_set.join_next().await {
            Some(Ok(check)) => {
                if let Ok(passed) = check {
                    channel = Ok(passed);
                }
            }
            _ => {}
        }
    }
    join_set.shutdown().await;
    channel
}

fn run_cmd(cmd: impl AsRef<OsStr>, args: Option<Vec<&str>>) -> anyhow::Result<Output> {
    let mut exit_output = Command::new(&cmd);
    if let Some(args) = args {
        exit_output.args(args);
    }
    let exit_output = exit_output.output();
    //println!("{:?}",exit_output);
    Ok(exit_output?)
}

pub fn get_supported_blockchains() -> HashMap<String, SupportedBlockchain> {
    let mut supported_blockchains: HashMap<String, SupportedBlockchain> = HashMap::new();

    /*
    // outdated cosmos-sdk
    supported_blockchains.insert(
        "terra".to_string(),
        SupportedBlockchain {
            name: "Terra".to_string(),
            prefix: "terra".to_string(),
            grpc_list: Vec::new(),
            governance_proposals_link: "https://station.terra.money/proposal/".to_string(),
        },
    );*/
    supported_blockchains.insert(
        "terra2".to_string(),
        SupportedBlockchain {
            name: "Terra2".to_string(),
            prefix: "terra".to_string(),
            grpc_url: Some("http://n-fsn-7.zyons.com:29090".to_string()),
            governance_proposals_link: "https://station.terra.money/proposal/".to_string(),
        },
    );
    supported_blockchains.insert(
        "osmosis".to_string(),
        SupportedBlockchain {
            name: "Osmosis".to_string(),
            prefix: "osmo".to_string(),
            grpc_url: None,
            governance_proposals_link: "https://wallet.keplr.app/chains/osmosis/proposals/"
                .to_string(),
        },
    );
    supported_blockchains.insert(
        "juno".to_string(),
        SupportedBlockchain {
            name: "Juno".to_string(),
            prefix: "juno".to_string(),
            grpc_url: None,
            governance_proposals_link: "https://wallet.keplr.app/chains/juno/proposals/"
                .to_string(),
        },
    );
    supported_blockchains.insert(
        "cosmoshub".to_string(),
        SupportedBlockchain {
            name: "CosmosHub".to_string(),
            prefix: "atom".to_string(),
            grpc_url: None,
            governance_proposals_link: "https://wallet.keplr.app/chains/cosmos-hub/proposals/"
                .to_string(),
        },
    );
    supported_blockchains
}

// refresh_rate in seconds
pub async fn get_supported_blockchains_from_chain_registry(
    path: String,
    git_pull: bool,
    chain_registry_refresh_rate: Option<u64>,
) -> HashMap<String, SupportedBlockchain> {
    if git_pull {
        let mut update: bool = false;
        if let Some(refresh_rate) = chain_registry_refresh_rate {
            let path = format!("{}/.git/FETCH_HEAD", &path);
            //println!("{}",&path);
            let date_git_fetch_head = run_cmd("date", Some(vec!["+%s", "-r", path.as_str()]));
            let date_system = run_cmd("date", Some(vec!["+%s"]));
            match (date_git_fetch_head, date_system) {
                (Ok(date_git_fetch_head), Ok(date_system)) => {
                    let d1: u64 = String::from_utf8_lossy(&date_git_fetch_head.stdout)
                        .to_string()
                        .replace("\n", "")
                        .parse()
                        .unwrap_or(0);
                    let d2: u64 = String::from_utf8_lossy(&date_system.stdout)
                        .to_string()
                        .replace("\n", "")
                        .parse()
                        .unwrap_or(0);
                    if d2 - d1 >= refresh_rate {
                        update = true;
                    }
                }
                _ => {}
            }
        } else {
            update = true;
        }
        if update {
            run_cmd("git", Some(vec!["-C", path.as_str(), "pull"])).ok();
        }
    }

    let mut supported_blockchains: HashMap<String, SupportedBlockchain> =
        get_supported_blockchains();

    for (k, v) in supported_blockchains.iter_mut() {
        let file = File::open(format!("{}/{}/chain.json", &path, k).as_str()).unwrap();
        let reader = BufReader::new(file);
        let chain_info: chain_registry::chain::ChainInfo = serde_json::from_reader(reader).unwrap();

        let mut try_these_grpc_urls: Vec<String> = chain_info
            .apis
            .grpc
            .iter()
            .map(|x| format!("http://{}", x.address))
            .collect();
        if let Some(ref hard_coded_grpc_url) = v.grpc_url {
            try_these_grpc_urls.push(hard_coded_grpc_url.to_owned());
        }
        v.grpc_url = match select_channel_from_grpc_endpoints(&try_these_grpc_urls).await {
            Ok(passed_url) => Some(passed_url),
            Err(_) => None,
        };
    }
    supported_blockchains
}
