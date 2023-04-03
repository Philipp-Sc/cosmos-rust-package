use std::collections::HashMap;
use tonic::transport::channel::Channel;

use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::GetNodeInfoRequest;
use lazy_static::lazy_static;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::process::Output;
use log::{debug, error, info};
use tokio::task::JoinSet;

use serde::{Deserialize, Serialize};

lazy_static! {
    static ref SUPPORTED_BLOCKCHAINS: HashMap<String, SupportedBlockchain> = {
        let data = std::fs::read_to_string("./tmp/supported_blockchains.json")
            .expect("Unable to read file");
        let supported_blockchains: HashMap<String, SupportedBlockchain> =
            serde_json::from_str(&data).expect("Unable to parse JSON");
        supported_blockchains
    };
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SupportedBlockchain {
    pub display: String,
    pub name: String,
    pub prefix: String,
    pub grpc_service: GRPC_Service,
    pub governance_proposals_link: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GRPC_Service {
    pub grpc_url: Option<String>, // selected grpc_url
    pub error: Option<String>,    // error msg if no url could be selected
}

impl SupportedBlockchain {
    pub async fn channel(&self) -> anyhow::Result<Channel> {
        match &self.grpc_service.error {
            Some(err) => Err(anyhow::anyhow!(format!(
                "Error: {} is not a supported cosmos blockchain: {}",
                self.name,
                err.to_string()
            ))),
            None => match &self.grpc_service.grpc_url {
                Some(grpc_url) => get_channel(grpc_url.to_owned()).await,
                None => Err(anyhow::anyhow!(format!(
                    "Error: {} is not a supported cosmos blockchain: Error: Missing GRPC URL!",
                    self.name,
                ))),
            },
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
                Ok(node_info_response) => {
                    info!("Successful GetNodeInfoResponse for {}",grpc_url);
                    debug!("GetNodeInfoResponse: {:?}",node_info_response);
                    // TODO potentially check versions, check if the node is up to par.
                    Ok(grpc_url)
                },
                Err(e) => {
                    error!("GetNodeInfoRequest failed for {}: {:?}",grpc_url,e);
                    //println!("{:?}",e);
                    Err(anyhow::anyhow!(format!("GetNodeInfoRequest failed: {}",e.to_string())))
                }
            }
        }
        Err(e) => {
            error!("Unable to establish a connection to {}: {:?}",grpc_url,e);
            Err(anyhow::anyhow!(format!(
            "tonic::transport::Endpoint::connect() failed: {}",
            e.to_string()
        )))
        },
    }
}

pub async fn select_channel_from_grpc_endpoints(grpc_urls: &Vec<String>) -> anyhow::Result<String> {
    let mut join_set: JoinSet<anyhow::Result<String>> = JoinSet::new();
    for grpc_url in grpc_urls.iter().map(|x| x.to_owned()) {
        let https_grpc_url = format!("https://{}", grpc_url);
        let http_grpc_url = format!("http://{}", grpc_url);
        join_set.spawn(async move { check_grpc_url(http_grpc_url).await });
        join_set.spawn(async move { check_grpc_url(https_grpc_url).await });
    }
    let mut channel: Result<String, anyhow::Error> =
        Err(anyhow::anyhow!("Error: No gRPC url passed the check!"));
    let mut errors: Vec<anyhow::Error> = Vec::new();
    while !join_set.is_empty() && channel.is_err() {
        match join_set.join_next().await {
            Some(Ok(check)) => match check {
                Ok(passed) => {
                    channel = Ok(passed);
                }
                Err(failed) => {
                    errors.push(failed);
                }
            },
            _ => {}
        }
    }
    join_set.shutdown().await;

    if channel.is_err() {
        let error_msg: String = errors
            .into_iter()
            .map(|err| err.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        channel = Err(anyhow::anyhow!(format!(
            "Error: No gRPC url passed the check! {}",
            error_msg
        )));
    }
    channel
}

fn run_cmd(cmd: &str, args: Option<Vec<&str>>) -> anyhow::Result<Output> {
    let mut exit_output = Command::new(cmd);
    if let Some(args) = args {
        exit_output.args(args);
    }
    let exit_output = exit_output.output();
    debug!("run_cmd: cmd: {}, output: {:?}",cmd, exit_output);
    Ok(exit_output?)
}

pub fn get_supported_blockchains() -> HashMap<String, SupportedBlockchain> {
    (*SUPPORTED_BLOCKCHAINS).clone()
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
            run_cmd("ls", Some(vec!["-a"])).ok();
            match run_cmd("git", Some(vec!["-C", path.as_str(), "pull"])) {
                Ok(_) => {
                    info!("git pull successful for {}",&path);
                },
                Err(e) => {
                    error!("git pull failed for {}: {:?}",&path,e);
                },
            };
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
            .map(|x| x.address.clone())
            .collect();
        if let Some(ref hard_coded_grpc_url) = v.grpc_service.grpc_url {
            try_these_grpc_urls.push(hard_coded_grpc_url.to_owned());
        }
        match select_channel_from_grpc_endpoints(&try_these_grpc_urls).await {
            Ok(grpc_url) => {
                v.grpc_service.grpc_url = Some(grpc_url);
                v.grpc_service.error = None;
            }
            Err(err) => {
                v.grpc_service.grpc_url = None;
                v.grpc_service.error = Some(format!("{}", err.to_string()));
            }
        }
    }
    supported_blockchains
}
