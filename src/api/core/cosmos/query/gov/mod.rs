
use tonic::transport::Channel;

use cosmos_sdk_proto::cosmos::gov::v1beta1::query_client::QueryClient;
use cosmos_sdk_proto::cosmos::gov::v1beta1::{QueryProposalsRequest, QueryProposalsResponse, QueryTallyResultRequest, QueryParamsResponse,QueryParamsRequest,  QueryTallyResultResponse};

pub async fn get_params(
    channel: Channel,
    query_tally_params_request: QueryParamsRequest,
) -> anyhow::Result<QueryParamsResponse> {
    let res = QueryClient::new(channel)
        .params(query_tally_params_request)
        .await?
        .into_inner();
    Ok(res)
}

pub async fn get_tally_result(
    channel: Channel,
    query_tally_result_request: QueryTallyResultRequest,
) -> anyhow::Result<QueryTallyResultResponse> {
    let res = QueryClient::new(channel)
        .tally_result(query_tally_result_request)
        .await?
        .into_inner();
    Ok(res)
}

pub async fn get_proposals(
    channel: Channel,
    query_proposal_request: QueryProposalsRequest,
) -> anyhow::Result<QueryProposalsResponse> {
    let res = QueryClient::new(channel)
        .proposals(query_proposal_request)
        .await?
        .into_inner();
    Ok(res)
}



/*
#[cfg(test)]
mod test {

    // cargo test -- --nocapture

    #[tokio::test]
    pub async fn get_proposals() -> anyhow::Result<()> {
        let channel = super::channels::get_supported_blockchains_from_chain_registry(
            "./packages/chain-registry".to_string(),
            true,
            None,
        )
        .await
        .get("osmosis")
        .unwrap()
        .channel()
        .await?;
        let res = super::get_proposals(
            channel,
            cosmos_sdk_proto::cosmos::gov::v1beta1::QueryProposalsRequest {
                proposal_status: 0x03,
                voter: "".to_string(),
                depositor: "".to_string(),
                pagination: None,
            },
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn cw20_balance_via_smart_contract_state() -> anyhow::Result<()> {
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
        let query_msg = cw20::Cw20QueryMsg::Balance {
            address: cosmwasm_std::HumanAddr(
                "terra1vcpt3p9p6rrqaw4zwt706p8vj7uhd0sf4p5snl".to_string(),
            ), //address: "terra1vcpt3p9p6rrqaw4zwt706p8vj7uhd0sf4p5snl".to_string()
        };
        let res = super::get_smart_contract_state(
            channel,
            "terra1ecgazyd0waaj3g7l9cmy5gulhxkps2gmxu9ghducvuypjq68mq2s5lvsct".to_string(),
            &query_msg,
        )
        .await?;

        /*println!("TEST: {}", "get_smart_contract_state(address, query_msg)");
        println!("{:?}", serde_json::from_slice::<cw20::BalanceResponse>(&res.data));
        println!("{:?}", std::str::from_utf8(&res.data));*/
        Ok(())
    }

    #[tokio::test]
    pub async fn query_account() -> anyhow::Result<()> {
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
            "terra1dp0taj85ruc299rkdvzp4z5pfg6z6swaed74e6".to_string(),
        )
        .await?;
        /*println!("TEST: {}", "query_account(address)");
        println!("{:?}", &account);*/
        Ok(())
    }

    #[tokio::test]
    pub async fn contract_info() -> anyhow::Result<()> {
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
        let res = super::get_contract_info(
            channel,
            "terra1ccxwgew8aup6fysd7eafjzjz6hw89n40h273sgu3pl4lxrajnk5st2hvfh".to_string(),
        )
        .await?;
        /*println!("TEST: {}", "get_contract_info(address)");
        println!("{:?}", &res);*/
        Ok(())
    }
}
*/