use std::collections::HashMap;
use tonic::transport::channel::Channel;

use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::GetNodeInfoRequest;


use std::fmt::Debug;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::pin::Pin;

use log::{debug, error, info};
use std::process::Command;
use std::process::Output;



use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use futures::stream::{FuturesUnordered, StreamExt, Stream};
use futures::task::*;

pub type SupportedBlockchainType = HashMap<String, SupportedBlockchain>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupportedBlockchain {
    pub rank: u32,
    pub display: String,
    pub name: String,
    pub prefix: String,
    pub grpc_service: GRPC_Service,
    pub governance_proposals_link: String,
}

impl PartialEq for SupportedBlockchain {
    fn eq(&self, other: &Self) -> bool {
            self.name == other.name &&
            self.prefix == other.prefix
    }
}

impl Hash for SupportedBlockchain {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash)]
pub struct GRPC_Service {
    pub grpc_urls: Vec<String>, // viable grpc_urls
    pub error: Option<String>,  // error msg if no url could be selected
}

impl SupportedBlockchain {
    pub fn get_name(&self) -> String {
        self.name.to_lowercase()
    }
    pub async fn channel(&self) -> Result<Channel, tonic::Status> {
        match &self.grpc_service.error {
            Some(err) => Err(tonic::Status::failed_precondition(format!(
                "Error: {} is not a supported cosmos blockchain: {}",
                self.name,
                err.to_string()
            ))),
            None => {
                let selected_grpc_service =
                    self.grpc_service.grpc_urls.choose(&mut rand::thread_rng());
                match selected_grpc_service {
                    Some(grpc_service) => get_channel(grpc_service.to_owned()).await,
                    None => Err(tonic::Status::cancelled(format!(
                        "Error: {} is not a supported cosmos blockchain: Error: Missing GRPC URL!",
                        self.name,
                    ))),
                }
            }
        }
    }
}

async fn get_channel(grpc_url: String) -> Result<Channel, tonic::Status> {
    let endpoint =
        tonic::transport::Endpoint::new(grpc_url.parse::<tonic::transport::Uri>().map_err(|err| tonic::Status::failed_precondition(err.to_string()))?)
            .map_err(|err| tonic::Status::cancelled(err.to_string()))?
            .timeout(std::time::Duration::from_secs(60))
            .connect_timeout(std::time::Duration::from_secs(60));
    match endpoint.connect().await {
        Ok(result) => Ok(result),
        Err(err) => Err(tonic::Status::aborted(err.to_string())),
    }
}

async fn test_grpc_url(grpc_url: String) -> anyhow::Result<String> {
    info!("Testing gRPC URL: {}", &grpc_url);
    match get_channel(grpc_url.to_owned()).await {
        Ok(c) => {
            info!("Got Channel for gRPC URL: {}", grpc_url);
            match cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::new(c).get_node_info(GetNodeInfoRequest{}).await {
                Ok(node_info_response) => {
                    info!("Successful GetNodeInfoResponse for {}",grpc_url);
                    debug!("GetNodeInfoResponse: {:?}",node_info_response);
                    Ok(grpc_url)
                },
                Err(e) => {
                    error!("GetNodeInfoRequest failed for {}: {:?}",grpc_url,e);
                    Err(anyhow::anyhow!(format!("GetNodeInfoRequest failed: {}",e.to_string())))
                }
            }
        }
        Err(e) => {
            error!("Unable to establish a connection to {}: {:?}", grpc_url, e);
            Err(anyhow::anyhow!(format!(
                "tonic::transport::Endpoint::connect() failed: {}",
                e.to_string()
            )))
        }
    }
}

async fn check_grpc_url(grpc_url: String) -> anyhow::Result<String> {
    let http = "http://";
    let https = "https://";

    let test_url = if grpc_url.contains(http) || grpc_url.contains(https) {
        grpc_url.to_owned()
    } else {
        format!("{}{}", https, &grpc_url)
    };

    match test_grpc_url(test_url).await {
        Ok(https_grpc_url) => Ok(https_grpc_url),
        Err(err) => match test_grpc_url(format!("{}{}", http, &grpc_url)).await {
            Ok(http_grpc_url) => Ok(http_grpc_url),
            Err(err2) => Err(anyhow::anyhow!("https: {}, http: {}", err, err2)),
        },
    }
}

/*
pub async fn select_channel_from_grpc_endpoints(key_grpc_url_list: Vec<(String,Vec<String>)>) -> Vec<(String,Result<String, anyhow::Error>)> {

    let mut channels: Vec<(String,Result<String, anyhow::Error>)> = Vec::new();

    for each in key_grpc_url_list.into_iter() {
        for grpc_url in each.1.into_iter() {
            channels.push((each.0.clone(), match  check_grpc_url(grpc_url).await {
                    Ok(passed) => {
                        Ok(passed)
                    }
                    Err(failed) => {
                        Err(failed)
                    }
                }));
        }
    }
    channels
}*/


pub struct CheckUrls {
    join_set: FuturesUnordered<tokio::task::JoinHandle<Result<(String, String), (String, anyhow::Error)>>>,
}

impl CheckUrls {
    pub fn stream(key_grpc_url_list: Vec<(String, Vec<String>)>) -> Self {
        let join_set = FuturesUnordered::new();

        for (key, urls) in key_grpc_url_list.into_iter() {
            for url in urls.into_iter() {
                let key_clone = key.clone();
                join_set.push(tokio::spawn(async move {
                    match check_grpc_url(url).await {
                        Ok(passed) => Ok((key_clone, passed)),
                        Err(failed) => Err((key_clone, failed)),
                    }
                }));
            }
        }
        Self { join_set }
    }
}

impl Stream for CheckUrls {
    type Item = (String, Result<String, anyhow::Error>);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.join_set.poll_next_unpin(cx) {
            Poll::Ready(Some(result)) => match result {
                Ok(Ok((key, url))) => Poll::Ready(Some((key,Ok(url)))),
                Ok(Err((key, err))) => Poll::Ready(Some((key.clone(), Err(anyhow::anyhow!("Failed to check URL of {}: {}", key, err))))),
                Err(_err) => {
                    if self.join_set.is_empty() {
                        Poll::Ready(None)
                    }else{
                        Poll::Pending
                    }
                },
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/*
pub async fn select_channel_from_grpc_endpoints(
    key_grpc_url_list: Vec<(String, Vec<String>)>,
) -> Vec<(String, Result<String, anyhow::Error>)> {
    let mut join_set: JoinSet<_> = JoinSet::new();

    //let mut key_abort_handles: HashMap<String,Vec<AbortHandle>> = HashMap::new();

    for each in key_grpc_url_list.into_iter() {
        //let mut abort_handles: Vec<AbortHandle> = Vec::new();
        let key = each.0.clone();
        for grpc_url in each.1.into_iter() {
            let key_clone = key.clone();

            /*abort_handles.push(*/
            join_set.spawn(async move {
                (key_clone, match  check_grpc_url(grpc_url).await {
                        Ok(passed) => {
                            Ok(passed)
                        }
                        Err(failed) => {
                            Err(failed)
                        }
                })
            })/*)*/;
        }
        //key_abort_handles.insert(each.0.to_owned(),abort_handles);
    }
    let mut channels: Vec<(String, Result<String, anyhow::Error>)> = Vec::new();

    while let Some(res) = join_set.join_next().await {
        info!("join_set.len(): {}", join_set.len());
        match res {
            Ok((key, result)) => {
                if result.is_ok() {
                    /*
                    if let Some(irrelevant) = key_abort_handles.remove(&key) {
                        for each in irrelevant {
                            each.abort();
                        }
                    }
                    */
                }
                channels.push((key, result));
            }
            Err(_err) => {
                // task did not complete because of a unexpected error or abort
            }
        }
    }
    channels
}*/

fn run_cmd(cmd: &str, args: Option<Vec<&str>>) -> anyhow::Result<Output> {
    let command = format!("{}, {:?}", &cmd, &args);
    let mut exit_output = Command::new(cmd);
    if let Some(args) = args {
        exit_output.args(args);
    }
    let exit_output = exit_output.output();
    if let Ok(false) = exit_output.as_ref().map(|x| x.status.success()) {
        error!("run_cmd: cmd: {}, output: {:?}", cmd, exit_output);
    } else {
        debug!("run_cmd: cmd: {}, output: {:?}", cmd, exit_output);
    }
    Ok(exit_output.map_err(|err| anyhow::anyhow!("{}, command: {}", err.to_string(), command))?)
}

pub fn update_git(
    git_path: &str,
    git_pull: bool,
    sync_interval_in_secs: Option<u64>,
) -> anyhow::Result<()> {
    if git_pull {
        let mut update: bool = false;
        if let Some(pull_interval) = sync_interval_in_secs {
            let path = format!("{}/.git/FETCH_HEAD", git_path);
            //println!("{}",&path);
            let date_git_fetch_head = run_cmd("date", Some(vec!["+%s", "-r", path.as_str()]))?;
            let date_system = run_cmd("date", Some(vec!["+%s"]))?;
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
            if d2 - d1 >= pull_interval {
                update = true;
            }
        } else {
            update = true;
        }
        if update {
            run_cmd("ls", Some(vec!["-a"]))?;
            run_cmd("git", Some(vec!["-C", git_path, "pull"]))?;
        }
    }
    Ok(())
}

pub fn prepare_blockchain_list(
    json_path: &str,
) -> anyhow::Result<HashMap<String, SupportedBlockchain>> {
    let data = std::fs::read_to_string(json_path)
        .map_err(|err| anyhow::anyhow!("{}, File: {}", err.to_string(), json_path))?;
    let supported_blockchains: HashMap<String, SupportedBlockchain> =
        serde_json::from_str(&data)
            .map_err(|err| anyhow::anyhow!("{}, File: {}", err.to_string(), json_path))?;

    Ok(supported_blockchains)
}


pub fn get_channel_list(
    git_path: &str,
    supported_blockchains: &HashMap<String, SupportedBlockchain>
) -> anyhow::Result<Vec<(String, Vec<String>)>> {
    let mut list: Vec<(String, Vec<String>)> = Vec::new();

    for (k, v) in supported_blockchains.iter() {
        let chain_json = format!("{}/{}/chain.json", &git_path, k);
        let file = File::open(&chain_json)
            .map_err(|err| anyhow::anyhow!("{}, File: {}", err.to_string(), chain_json))?;
        let reader = BufReader::new(file);
        let chain_info: chain_registry::chain::ChainInfo = serde_json::from_reader(reader)?;

        let mut try_these_grpc_urls: Vec<String> = chain_info
            .apis
            .grpc
            .iter()
            .map(|x| x.address.clone())
            .collect();
        for hard_coded_grpc_url in v.grpc_service.grpc_urls.iter() {
            try_these_grpc_urls.push(hard_coded_grpc_url.to_owned());
        }
        list.push((k.clone(), try_these_grpc_urls));
    }
    Ok(list)
}

pub struct SupportedBlockchainIter {
    check_urls: Pin<Box<dyn Stream<Item=(String, Result<String, anyhow::Error>)> + Send>>,
    supported_blockchains: HashMap<String, SupportedBlockchain>,
}

impl SupportedBlockchainIter {
    pub fn new(
        git_path: &str,
        git_pull: bool,
        json_path: &str,
        sync_interval_in_secs: Option<u64>,
    ) -> anyhow::Result<Self> {
        update_git(git_path, git_pull, sync_interval_in_secs)?;
        let supported_blockchains = prepare_blockchain_list(json_path)?;
        let list = get_channel_list(git_path,&supported_blockchains)?;
        let check_urls = CheckUrls::stream(list);

        Ok(Self {
            check_urls: Box::pin(check_urls),
            supported_blockchains,
        })
    }

    pub async fn next(&mut self) -> Option<&HashMap<String, SupportedBlockchain>> {
        match self.check_urls.next().await {
            Some(channel) => {
                self.supported_blockchains = merge_channels_with_supported_blockchains(
                    self.supported_blockchains.clone(),
                    vec![channel],
                );
                Some(&self.supported_blockchains)
            }
            None => None,
        }
    }
}

fn merge_channels_with_supported_blockchains(mut supported_blockchains: HashMap<String, SupportedBlockchain>, channels: Vec<(String, Result<String, anyhow::Error>)> ) -> HashMap<String, SupportedBlockchain> {
    for (k, v) in supported_blockchains.iter_mut() {
        let mut selected: Vec<String> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        for each in channels.iter().filter(|x| &x.0 == k) {
            match &each.1 {
                Ok(val) => {
                    selected.push(val.to_owned());
                }
                Err(err) => {
                    errors.push(format!("{:?}", err));
                }
            }
        }
        if selected.is_empty() {
            v.grpc_service.error =
                Some(format!("No viable endpoint for {} found: {:?}", &k, errors));
        } else {
            v.grpc_service.error = None;
        }
        v.grpc_service.grpc_urls = selected;
    }
    supported_blockchains
}


#[cfg(test)]
mod test {

    // cargo test -- --nocapture
    // cargo test -- --list
    // cargo test api::custom::query::gov::teset::get_proposals -- --exact --nocapture
    // cargo test api::custom::query::gov::teset::get_proposals -- --exact --nocapture

    use super::*;

    #[tokio::test]
    async fn test_grpc_url_function() {
        let grpc_url = "https://secret-mainnet-grpc.autostake.net:443".to_owned();
        let result = test_grpc_url(grpc_url).await;
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    /*
    #[tokio::test]
    async fn test_select_channel_from_grpc_endpoints_function() {
        let name = "test";
        let grpc_url = "https://secret-mainnet-grpc.autostake.net:443".to_owned();

        let result =
            select_channel_from_grpc_endpoints(vec![(name.to_string(), vec![grpc_url])]).await;
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_get_supported_blockchains_from_chain_registrys_function() {
        let path = "../chain-registry";

        let result = get_supported_blockchains_from_chain_registry(
            path,
            true,
            "./tmp/supported_blockchains.json",
            None,
        )
        .await;
        println!("{:?}", result);
    }*/
}
