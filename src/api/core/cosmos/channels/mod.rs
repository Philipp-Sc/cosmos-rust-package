use tonic::transport::channel::Channel;
use std::time::Duration;

use cosmos_sdk_proto::cosmwasm::wasm::v1::query_client::QueryClient;


pub async fn channel(blockchain: &str) -> anyhow::Result<Channel> {
    match blockchain {
        "terra" => terra().await,
        "osmosis" => osmosis().await,
        _ => Err(anyhow::anyhow!(format!("Error: {} is not a supported cosmos blockchain!",blockchain))),
    }
}

pub async fn osmosis() -> anyhow::Result<Channel> {
    //let channel = Channel::from_static("http://46.38.251.100:9090") // Felix | Interbloc
    //let channel = Channel::from_static("http://v-terra-hel-1.zyons.com:29090")
    let channel = Channel::from_static("http://osmosis.strange.love:9090")
        //let channel = Channel::from_static("http://cosmoshub.validator.network:443")
        //let channel = Channel::from_static("http://cosmos.chorus.one:26657")
        //let channel = Channel::from_static("http://rpc.cosmos.network:26657")
        //let channel = Channel::from_static("http://a.client.sentry.neerajnet.bluzelle.com:9090")
        //let channel = Channel::from_static("http://grpc-osmosis-ia.notional.ventures:443")
        .timeout(Duration::from_secs(5))
        .rate_limit(5, Duration::from_secs(1))
        .concurrency_limit(256)
        .connect()
        .await?;
    Ok(channel)
}

pub async fn terra() -> anyhow::Result<Channel> {
    //let channel = Channel::from_static("http://v-terra-hel-1.zyons.com:29090")
    let channel = Channel::from_static("http://n-fsn-7.zyons.com:29090")
        .timeout(Duration::from_secs(5))
        .rate_limit(5, Duration::from_secs(1))
        .concurrency_limit(256)
        .connect()
        .await?;
    Ok(channel)
}
