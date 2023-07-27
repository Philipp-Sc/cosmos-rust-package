use crate::api::core::cosmos::channels::SupportedBlockchain;
use crate::api::core::*;

use std::string::ToString;

use cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;
use tonic::{Code, Status};

use crate::api::custom::types::gov::params_ext::ParamsExt;
use crate::api::custom::types::gov::proposal_ext::{ProposalExt, ProposalStatus};
use crate::api::custom::types::gov::proposal_v1beta1_ext::{ProposalV1Beta1Ext, ProposalStatusV1Beta1};
use crate::api::custom::types::gov::tally_ext::TallyResultExt;
use crate::api::custom::types::gov::tally_v1beta1_ext::TallyResultV1Beta1Ext;
use crate::api::custom::types::staking::validators_ext::ValidatorsExt;

pub async fn get_validators_v1beta1(
    blockchain: SupportedBlockchain,
    next_key: Option<Vec<u8>>,
) -> anyhow::Result<(Option<Vec<u8>>, Vec<ValidatorsExt>)> {
    let channel = blockchain.channel().await?;
    let res = cosmos::query::staking::get_validators(
        channel,
        cosmos_sdk_proto::cosmos::staking::v1beta1::QueryValidatorsRequest {
            status: "".to_string(),
            pagination: Some(PageRequest {
                key: next_key.unwrap_or(vec![]),
                offset: 0,
                limit: 0,
                count_total: false,
                reverse: false,
            }),
        },
    )
    .await?;

    let mut list: Vec<ValidatorsExt> = Vec::new();
    for validator in res.validators {
        list.push(ValidatorsExt::new(&blockchain, validator));
    }
    Ok((res.pagination.map(|x| x.next_key), list))
}

pub async fn get_params_v1beta1(
    blockchain: SupportedBlockchain,
    params_type: String,
) -> anyhow::Result<ParamsExt> {
    let channel = blockchain.channel().await?;
    let res = cosmos::query::gov::get_params_v1beta1(
        channel,
        cosmos_sdk_proto::cosmos::gov::v1beta1::QueryParamsRequest {
            params_type: params_type.clone(),
        },
    )
    .await?;
    Ok(ParamsExt::new(blockchain, &params_type, res))
}

pub async fn get_tally_v1beta1(
    blockchain: SupportedBlockchain,
    proposal_id: u64,
) -> anyhow::Result<TallyResultV1Beta1Ext> {
    let channel = blockchain.channel().await?;
    let res = cosmos::query::gov::get_tally_result_v1beta1(
        channel,
        cosmos_sdk_proto::cosmos::gov::v1beta1::QueryTallyResultRequest { proposal_id },
    )
    .await?;
    Ok(TallyResultV1Beta1Ext::new(blockchain, proposal_id, res))
}

pub async fn get_tally_v1(
    blockchain: SupportedBlockchain,
    proposal_id: u64,
) -> anyhow::Result<TallyResultExt> {
    let channel = blockchain.channel().await?;
    let res = cosmos::query::gov::get_tally_result_v1(
        channel,
        cosmos_sdk_proto::cosmos::gov::v1::QueryTallyResultRequest { proposal_id },
    )
        .await?;
    Ok(TallyResultExt::new(blockchain, proposal_id, res))
}

pub async fn get_proposals_v1beta1(
    blockchain: SupportedBlockchain,
    proposal_status: ProposalStatus,
    next_key: Option<Vec<u8>>,
) -> Result<(Option<Vec<u8>>, Vec<ProposalExt>), tonic::Status> {
    let channel = blockchain.channel().await?;
    let res = cosmos::query::gov::get_proposals_v1beta1(
        channel,
        cosmos_sdk_proto::cosmos::gov::v1beta1::QueryProposalsRequest {
            proposal_status: proposal_status.clone() as i32,
            voter: "".to_string(),
            depositor: "".to_string(),
            pagination: Some(PageRequest {
                key: next_key.unwrap_or(vec![]),
                offset: 0,
                limit: 0,
                count_total: false,
                reverse: false,
            }),
        },
    )
    .await?;

    let mut list: Vec<ProposalExt> = Vec::new();
    for proposal in res.proposals {
        list.push(ProposalExt::from_v1beta1(&blockchain, &proposal_status, proposal));
    }
    Ok((res.pagination.map(|x| x.next_key), list))
}


pub async fn get_proposals_v1(
    blockchain: SupportedBlockchain,
    proposal_status: ProposalStatus,
    next_key: Option<Vec<u8>>,
) -> Result<(Option<Vec<u8>>, Vec<ProposalExt>), tonic::Status> {
    let channel = blockchain.channel().await?;
    let res = cosmos::query::gov::get_proposals_v1(
        channel,
        cosmos_sdk_proto::cosmos::gov::v1::QueryProposalsRequest {
            proposal_status: proposal_status.clone() as i32,
            voter: "".to_string(),
            depositor: "".to_string(),
            pagination: Some(PageRequest {
                key: next_key.unwrap_or(vec![]),
                offset: 0,
                limit: 0,
                count_total: false,
                reverse: false,
            }),
        },
    )
        .await?;

    let mut list: Vec<ProposalExt> = Vec::new();
    for proposal in res.proposals {
        list.push(ProposalExt::new(&blockchain, &proposal_status, proposal));
    }
    Ok((res.pagination.map(|x| x.next_key), list))
}


pub async fn get_proposals(
    blockchain: SupportedBlockchain,
    proposal_status: ProposalStatus,
    next_key: Option<Vec<u8>>,
) -> Result<(Option<Vec<u8>>, Vec<ProposalExt>),tonic::Status> {
    let res = get_proposals_v1(blockchain.clone(), proposal_status.clone(), next_key.clone()).await;

    if let Err(tonic_status) = &res {
        match tonic_status.code() {
            Code::Unimplemented | Code::Unknown | Code::Aborted | Code::Cancelled => {
                let res = get_proposals_v1beta1(blockchain, proposal_status, next_key).await;
                return res;
            }
            _ => {}
        }
    }
    return res;

}

#[cfg(test)]
mod test {

    // cargo test -- --nocapture
    // cargo test -- --list
    // cargo test api::custom::query::gov::test::test_get_proposals_function -- --exact --nocapture

    use super::*;
    use crate::api::core::cosmos::channels::GRPC_Service;

    #[tokio::test]
    async fn test_get_proposals_function() {
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
        let result = get_proposals(supported_blockchain, ProposalStatus::StatusPassed, None).await;
        assert!(result.is_ok());
        for each in result.unwrap().1 {
            println!("Decoded Proposal:\n{:?}", each.proposal);
            println!("Decoded Proposal Content:\n{:?}", each.content_opt());

            let serialized = serde_json::to_string(&each).unwrap();
            println!("\n\nserialized = {}", &serialized);
            let deserialized: ProposalExt = serde_json::from_str(&serialized).unwrap();
            println!("\ndeserialized = {:?}", deserialized);
            break;
        }
    }
}
