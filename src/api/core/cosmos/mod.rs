use cosmos_sdk_proto::cosmos::tx::v1beta1::service_client::ServiceClient;
use cosmos_sdk_proto::cosmos::tx::v1beta1::SimulateRequest;

use cosmrs::tx::Fee;
use cosmrs::tx::SignDoc;
use cosmrs::tx::SignerInfo;
use cosmrs::Coin;
use cosmos_sdk_proto::Any;

use cosmos_sdk_proto::cosmos::auth::v1beta1::BaseAccount;
use cosmos_sdk_proto::cosmos::crypto::secp256k1::PubKey;
use cosmos_sdk_proto::cosmwasm::wasm::v1::MsgExecuteContract;

use cosmrs::tendermint::chain::Id;

//use moneymarket::market::ExecuteMsg;

mod keys;

use secp256k1::Secp256k1;

use cosmrs::tx::AuthInfo;
use std::str::FromStr;

pub mod channels;
pub mod query;

/*
/// Chain ID to use for tests
//const CHAIN_ID: &str = "pisco-1";
const CHAIN_ID: &str = "phoenix-1";
/// Expected account number
const ACCOUNT_NUMBER: AccountNumber = 1;
/// Bech32 prefix for an account
const ACCOUNT_PREFIX: &str = "terra";
/// Denom name
const DENOM: &str = "uluna";
/// Example memo
const MEMO: &str = "test memo";
*/

pub fn raw_public_key_from_account(base_account: &BaseAccount) -> anyhow::Result<Vec<u8>> {
    let pub_key = base_account.pub_key.as_ref().unwrap();
    let pk: PubKey = cosmos_sdk_proto::traits::MessageExt::from_any(pub_key).unwrap();

    Ok(keys::public::PublicKey::from_public_key(&pk.key)
        .raw_pub_key
        .unwrap())
    /*
    let mut bech32_pubkey_data_prefix_secp256_k1: Vec<u8> = vec![0xeb, 0x5a, 0xe9, 0x87, 0x21]; // [235, 90, 233, 135, 33]
    bech32_pubkey_data_prefix_secp256_k1.append(&mut pk.key.clone());
    Ok(bech32_pubkey_data_prefix_secp256_k1)
    */
}

pub fn raw_public_key_from_seed_phrase(seed_phrase: String) -> anyhow::Result<Vec<u8>> {
    let coin_type: u32 = 330;
    let s = Secp256k1::new();
    let pk = keys::private::PrivateKey::from_words(&s, seed_phrase.as_str(), 0, 0, coin_type);
    Ok(pk?.public_key(&s).raw_pub_key.unwrap())
}

pub fn public_key_from_account(
    base_account: &BaseAccount,
) -> anyhow::Result<keys::public::PublicKey> {
    let pub_key = base_account
        .pub_key
        .as_ref()
        .ok_or(anyhow::anyhow!("no pub_key"))?;
    let pk: PubKey = cosmos_sdk_proto::traits::MessageExt::from_any(pub_key).unwrap();
    Ok(keys::public::PublicKey::from_public_key(&pk.key))
}

pub fn public_key_from_seed_phrase(seed_phrase: String) -> anyhow::Result<keys::public::PublicKey> {
    let coin_type: u32 = 330;
    let s = Secp256k1::new();
    let pk = keys::private::PrivateKey::from_words(&s, seed_phrase.as_str(), 0, 0, coin_type);
    Ok(pk?.public_key(&s))
}

pub fn private_key_from_seed_phrase(
    seed_phrase: String,
) -> anyhow::Result<cosmrs::crypto::secp256k1::SigningKey> {
    let coin_type: u32 = 330;
    let s = Secp256k1::new();
    let pk = keys::private::PrivateKey::from_words(&s, seed_phrase.as_str(), 0, 0, coin_type)?;
    let cosmos_private_key =
        cosmrs::crypto::secp256k1::SigningKey::from_bytes(&pk.raw_key()).unwrap();
    Ok(cosmos_private_key)
}

pub fn auth_info_from(base_account: &BaseAccount) -> anyhow::Result<AuthInfo> {
    let gas_limit: u64 = 1_000_000;
    //const GAS_BUFFER: f64 = 1.2;

    let gas_denom = cosmrs::Denom::from_str("uluna").unwrap();
    let amount = Coin {
        amount: 0u8.into(),
        denom: gas_denom.clone(),
    };
    let fee = Fee::from_amount_and_gas(amount, gas_limit);

    Ok(SignerInfo::single_direct(
        Some(cosmrs::crypto::PublicKey::try_from(base_account.pub_key.as_ref().unwrap()).unwrap()),
        base_account.sequence,
    )
    .auth_info(fee))
}

pub fn msg_exec_contract(msg_json: String, contract: String, sender: String) -> Any {
    let msg_execute_contract_proto = MsgExecuteContract {
        sender: sender,
        contract: contract,
        msg: msg_json.as_bytes().to_vec(),
        funds: vec![],
    };
    cosmos_sdk_proto::traits::MessageExt::to_any(&msg_execute_contract_proto).unwrap()
    /*.to_any().unwrap()*/
}

pub async fn msg_execute(
    msg_json: String,
    contract: String,
    base_account: BaseAccount,
    memo: String,
    timeout_height: u64,
) -> anyhow::Result<()> {
    let _body = cosmrs::tx::Body::new(
        vec![msg_exec_contract(msg_json, contract, base_account.address)],
        memo,
        cosmrs::tendermint::block::Height::from(timeout_height as u32),
    );
    Ok(())
}

pub fn sign_doc(
    tx_body: cosmrs::tx::Body,
    auth_info: &AuthInfo,
    base_account: &BaseAccount,
    seed_phrase: String,
) -> anyhow::Result<Vec<u8>> {
    let sign_doc = SignDoc::new(
        &tx_body,
        &auth_info,
        &Id::try_from("phoenix-1")?,
        base_account.account_number,
    )
    .unwrap();
    let private_key = private_key_from_seed_phrase(seed_phrase).unwrap();
    let tx_raw = sign_doc.sign(&private_key).unwrap();

    Ok(tx_raw.to_bytes().unwrap())
}

pub async fn simulate_tx(tx_bytes: Vec<u8>) -> anyhow::Result<()> {
    let _channel = channels::get_supported_blockchains_from_chain_registry(
        "./packages/chain-registry".to_string(),
        true,
        None,
    )
    .await
    .get("terra2")
    .unwrap()
    .channel()
    .await?;
    let res = ServiceClient::connect("http://osmosis.strange.love:9090")
        .await?
        .simulate(SimulateRequest {
            tx: None, // deprecated
            tx_bytes, //prost::Message::encode_to_vec(&transaction),
        })
        .await?
        .into_inner();

    println!("{:?}", res);
    Ok(())
    //let gas_used = resp.gas_info.unwrap().gas_used;
    //Ok(gas_used)
}

/*
pub async fn pipes() -> anyhow::Result<()> {
    let account = query_account("terra18m6x653kj67jfsn9f9st97esp8l556swh3ty0d".to_string()).await?;
    println!("{:?}", &account);
    let pub_key = public_key_from_account(&account)?;
    println!("{:?}", &pub_key);
    let seed_phrase = "notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius".to_string();
    let auth_info = auth_info_from(&account)?;

    let contract = "terra1ccxwgew8aup6fysd7eafjzjz6hw89n40h273sgu3pl4lxrajnk5st2hvfh"; // ErisProtocol
    let msg_json;
    let msg = terraswap::pair::Cw20HookMsg::Swap {
        belief_price: None,
        max_spread: None,
        to: None,
    };

    let execute_msg = Cw20ExecuteMsg::Send {
        contract: contract,
        amount: Uint128::from(1u128),
        msg: to_binary(&msg).unwrap(),
    };
    let execute_msg_json = serde_json::to_string(&execute_msg)?;

    let tx_body = TxBody {
        messages: vec![msg_exec_contract(msg_json, contract, account.address)],
        memo: "memo",
        timeout_height: 900000,
        extension_options: vec![],
        non_critical_extension_options: vec![],
    };

    let tx_raw = sign_doc(tx_body, &auth_info, &account, seed_phrase)?;

    let res = simulate_tx(tx_raw).await?;
}
*/

pub async fn msg_send() -> anyhow::Result<()> {
    let _channel = channels::get_supported_blockchains_from_chain_registry(
        "./packages/chain-registry".to_string(),
        true,
        None,
    )
    .await
    .get("terra2")
    .unwrap()
    .channel()
    .await?;

    /*
    let auth_info =
        SignerInfo::single_direct(Some(sender_public_key), sequence_number).auth_info(fee);
    let sign_doc = SignDoc::new(&body, &auth_info, &chain_id, ACCOUNT_NUMBER).unwrap();
    let tx_raw = sign_doc.sign(&sender_private_key).unwrap();
    */

    /*
    let res = QueryClient::new(channel).contract_info(QueryContractInfoRequest { address: "terra".to_string() }).await?.into_inner();
    println!("{:?}", (res.address, res.contract_info.as_ref().unwrap().label.as_str()));

    let res = QueryClient::new(channel).contract_info(QueryContractInfoRequest { address: "terra".to_string() }).await?.into_inner();
    println!("{:?}", (res.address, res.contract_info.as_ref().unwrap().label.as_str()));
    */
    /*

        //let msgs: Vec<T: Msg> =;
        //let msgs: Result<Vec<Any>, _> = msgs.into_iter().map(Msg::into_any).collect();



        let timeout_height = 9001u16;
        let msgs: Result<Vec<Any>, _> = msgs.into_iter().map(Msg::into_any).collect();
        let msgs = msgs?;
        let gas_denom = self.network.gas_denom.clone();
        let amount = Coin {
            amount: 0u8.into(),
            denom: gas_denom.clone(),
        };
        let fee = Fee::from_amount_and_gas(amount, gas_limit);

        let tx_body = tx::Body::new(msgs, memo.unwrap_or_default(), timeout_height);
        let auth_info =
            SignerInfo::single_direct(Some(pk.public_key(&s)), base_account.sequence).auth_info(fee);
        let sign_doc = SignDoc::new(
            &tx_body,
            &auth_info,
            &Id::try_from("phoenix-1")?,
            account_number,
        )?;
        let tx_raw = sign_doc.sign(&pk)?;

        let sim_gas_used = self.simulate_tx(tx_raw.to_bytes()?).await?;
    */

    Ok(())
}
/*
pub async fn commit_tx<T: Msg>(
    &self,
    msgs: Vec<T>,
    memo: Option<&str>,
) -> Result<CosmTxResponse, TerraRustScriptError> {
    let timeout_height = 9001u16;
    let msgs: Result<Vec<Any>, _> = msgs.into_iter().map(Msg::into_any).collect();
    let msgs = msgs?;
    let gas_denom = self.network.gas_denom.clone();
    let amount = Coin {
        amount: 0u8.into(),
        denom: gas_denom.clone(),
    };
    let fee = Fee::from_amount_and_gas(amount, gas_limit);

    let BaseAccount {
        account_number,
        sequence,
        ..
    } = self.base_account().await?;

    let tx_body = tx::Body::new(msgs, memo.unwrap_or_default(), timeout_height);
    let auth_info =
        SignerInfo::single_direct(Some(self.private_key.public_key()), sequence).auth_info(fee);
    let sign_doc = SignDoc::new(
        &tx_body,
        &auth_info,
        &Id::try_from(self.network.network_id.clone())?,
        account_number,
    )?;
    let tx_raw = sign_doc.sign(&self.private_key)?;

    let sim_gas_used = self.simulate_tx(tx_raw.to_bytes()?).await?;

    log::debug!("{:?}", sim_gas_used);

    let gas_expected = (sim_gas_used as f64 * GAS_BUFFER);
    let amount_to_pay = gas_expected * self.network.gas_price;
    let amount = Coin {
        amount: (amount_to_pay as u64).into(),
        denom: gas_denom,
    };
    let fee = Fee::from_amount_and_gas(amount, gas_expected as u64);


    let auth_info =
        SignerInfo::single_direct(Some(self.private_key.public_key()), sequence).auth_info(fee);
    let sign_doc = SignDoc::new(
        &tx_body,
        &auth_info,
        &Id::try_from(self.network.network_id.clone())?,
        account_number,
    )?;
    let tx_raw = sign_doc.sign(&self.private_key)?;

    self.broadcast(tx_raw).await
}*/

/* // Example for MsgExecuteContract
pub async fn msg_send() -> anyhow::Result<()> {

    let channel = terra().await?;

    let contract_addr_mm_market = "terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal".to_string();

    let execute_msg = ExecuteMsg::ClaimRewards { to: None };
    let execute_msg_json = serde_json::to_string(&execute_msg)?;

    let msg_execute_contract_proto = MsgExecuteContract {
        sender: "terra".to_string(),
        contract: contract_addr_mm_market,
        msg: execute_msg_json.as_bytes().to_vec(),
        funds: vec![],
    };
    let msg_execute = cosmos_sdk_proto::traits::MessageExt::to_any(&msg_execute_contract_proto).unwrap()/*.to_any().unwrap()*/;

    let body = TxBody {
        messages: vec![msg_execute.into()],
        memo: MEMO.to_string(),
        timeout_height: 100000u64,
        extension_options: vec![],
        non_critical_extension_options: vec![],
    };

    let transaction = Tx {
        body: Some(body/*.into_proto()*/),
        auth_info: None,
        signatures: vec![],
    };
    let res = ServiceClient::new(channel).simulate(SimulateRequest {
        tx: None, // deprecated
        tx_bytes: prost::Message::encode_to_vec(&transaction),
    }).await?.into_inner();

    println!("{:?}", res);

    Ok(())
}*/

/* // Example to use tendermint_rpc
use cosmrs::{
    query,
    bank::MsgSend,
    crypto::secp256k1,
    dev, rpc,
    tx::{self, Fee, Msg, SignDoc, SignerInfo},
    Coin,
};
use std::{panic, str};
use cosmrs::rpc::query::Query;
use cosmrs::rpc::{Client, query_client};

use terra_cosmwasm::{TerraQuerier, SwapResponse, TaxRateResponse, TaxCapResponse, ExchangeRatesResponse};

/// RPC port
const RPC_PORT: u16 = 26657;

pub async fn msg_send() {
    let rpc_address = format!("http://v-terra-hel-1.zyons.com:{}", RPC_PORT);
    //let rpc_address = format!("http://n-fsn-7.zyons.com:{}", RPC_PORT);

    let rpc_client = rpc::HttpClient::new(rpc_address.as_str()).unwrap();
    println!("rpc_client loaded");
    println!("{:?}", rpc_client.latest_block().await);
    // https://docs.cosmos.network/master/core/grpc_rest.html
    rpc_client.abci_query()

    ::default();
}
*/

/*
#[cfg(test)]
mod test {

    // cargo test -- --nocapture

    use crate::api::core::cosmos::channels::SupportedBlockchain;

    #[tokio::test]
    pub async fn key_from_account() -> anyhow::Result<()> {
        let channel = super::channels::get_supported_blockchains_from_chain_registry(
            "./packages/chain-registry".to_string(),
            true,
            None,
        )
        .await
        .get("terra2")
        .unwrap()
        .channel()
        .await?;
        let account = super::query_account(
            channel,
            "terra16f874e52x5704ecrxyg5m9ljfv20cn0hajpng7".to_string(),
        )
        .await?;
        /*println!("TEST: {}", "query_account(address)");
        println!("{:?}", &account);*/

        let pub_key = super::public_key_from_account(&account)?;

        /*println!("TEST: {}", "public_key_from_account(account)");
        println!("{:?}", &pub_key);*/
        Ok(())
    }
    #[test]
    pub fn public_key_from_seed_phrase() -> anyhow::Result<()> {
        let pub_key = super::public_key_from_seed_phrase("notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius".to_string())?;
        /*println!("TEST: {}", "public_key_from_seed_phrase(seed_phrase)");*/
        println!("{:?}", &pub_key);
        let account = pub_key.account("terra")?;
        println!("{:?}", &account);
        Ok(())
    }
}
*/