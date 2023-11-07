use crate::api::core::cosmos::channels::SupportedBlockchain;
use crate::api::core::*;

use std::string::ToString;

use cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;
use tonic::{Code};

use crate::api::custom::types::gov::params_ext::ParamsExt;
use crate::api::custom::types::gov::proposal_ext::{ProposalExt, ProposalStatus};

use crate::api::custom::types::gov::tally_ext::TallyResultExt;
use crate::api::custom::types::gov::tally_v1beta1_ext::TallyResultV1Beta1Ext;
use crate::api::custom::types::staking::validators_ext::ValidatorsExt;


use async_recursion::async_recursion;

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



/// Alternative retrieval method for fetching proposals (v1beta1 version).
///
/// This function is used to retrieve proposals using a fallback method when
/// the primary retrieval method (`get_proposals`) encounters certain error codes.
///
/// # Arguments
/// ... (same as `get_proposals`)
///
/// # Returns
/// ... (same as `get_proposals`)
///
/// # Errors
/// ... (same as `get_proposals`)
#[async_recursion]
pub async fn get_proposals_v1beta1(
    blockchain: SupportedBlockchain,
    proposal_status: ProposalStatus,
    next_key: Option<Vec<u8>>,
    offset: Option<u64>,
    limit: Option<u64>,
    skip_server_error: bool
) -> Result<(Option<Vec<u8>>, Vec<ProposalExt>), tonic::Status> {
    let channel = blockchain.channel().await?;
    let res = cosmos::query::gov::get_proposals_v1beta1(
        channel,
        cosmos_sdk_proto::cosmos::gov::v1beta1::QueryProposalsRequest {
            proposal_status: proposal_status.clone() as i32,
            voter: "".to_string(),
            depositor: "".to_string(),
            pagination: Some(PageRequest {
                key: next_key.clone().unwrap_or(vec![]),
                offset: offset.clone().unwrap_or(0),
                limit: limit.clone().unwrap_or(100),
                count_total: false,
                reverse: true,
            }),
        },
    )
    .await;

    if let Err(tonic_status) = &res {
        match tonic_status.code() {
            Code::OutOfRange => {
                if limit.unwrap_or(0) == 1 && skip_server_error {
                    let res = get_proposals_v1beta1(blockchain.clone(), proposal_status.clone(), next_key.clone(), Some(offset.map(|x| x + 1).unwrap_or(0)), Some(1),skip_server_error).await;
                    return res;
                } else {
                    let res = get_proposals_v1beta1(blockchain.clone(), proposal_status.clone(), next_key.clone(), offset.clone(), Some(1),skip_server_error).await;
                    return res;
                }
            }
            _ => {}
        }
    }

    let res = res?;

    let mut list: Vec<ProposalExt> = Vec::new();
    for proposal in res.proposals {
        list.push(ProposalExt::from_v1beta1(&blockchain, proposal));
    }
    Ok((res.pagination.map(|x| x.next_key), list))
}


/// Retrieve proposals using the v1 version of the API.
///
/// This function retrieves proposals using the v1 version of the API, with pagination
/// and fallback methods for handling certain error codes.
///
/// # Arguments
/// ... (same as `get_proposals`)
///
/// # Returns
/// ... (same as `get_proposals`)
///
/// # Errors
/// ... (same as `get_proposals`)
#[async_recursion]
pub async fn get_proposals_v1(
    blockchain: SupportedBlockchain,
    proposal_status: ProposalStatus,
    next_key: Option<Vec<u8>>,
    offset: Option<u64>,
    limit: Option<u64>,
    skip_server_error: bool
) -> Result<(Option<Vec<u8>>, Vec<ProposalExt>), tonic::Status> {
    let channel = blockchain.channel().await?;
    let res = cosmos::query::gov::get_proposals_v1(
        channel,
        cosmos_sdk_proto::cosmos::gov::v1::QueryProposalsRequest {
            proposal_status: proposal_status.clone() as i32,
            voter: "".to_string(),
            depositor: "".to_string(),
            pagination: Some(PageRequest {
                key: next_key.clone().unwrap_or(vec![]),
                offset: offset.clone().unwrap_or(0),
                limit: limit.clone().unwrap_or(100),
                count_total: false,
                reverse: true,
            }),
        },
    )
        .await;

    if let Err(tonic_status) = &res {
        match tonic_status.code() {
            Code::OutOfRange => {
                if limit.unwrap_or(0) == 1 && skip_server_error {
                    // error is for this proposal, needs to be skipped.
                    let res = get_proposals_v1(blockchain.clone(), proposal_status.clone(), next_key.clone(), Some(offset.map(|x| x + 1).unwrap_or(0)), Some(1),skip_server_error).await;
                    return res;
                } else {
                    let res = get_proposals_v1(blockchain.clone(), proposal_status.clone(), next_key.clone(), offset.clone(), Some(1),skip_server_error).await;
                    return res;
                }
            }
            _ => {}
        }
    }

    let res = res?;

    let mut list: Vec<ProposalExt> = Vec::new();
    for proposal in res.proposals {
        list.push(ProposalExt::new(&blockchain, proposal));
    }
    Ok((res.pagination.map(|x| x.next_key), list))
}

pub async fn get_proposal_v1(
    blockchain: SupportedBlockchain,
    proposal_id: u64,
) -> Result<Option<ProposalExt>, tonic::Status> {
    let channel = blockchain.channel().await?;
    let res = cosmos::query::gov::get_proposal_v1(
        channel,
        cosmos_sdk_proto::cosmos::gov::v1::QueryProposalRequest { proposal_id }
    ).await?;
    Ok(if let Some(proposal) = res.proposal {
        Some(ProposalExt::new(&blockchain, proposal))
    }else{
        None
    })
}

pub async fn get_proposal_v1beta1(
    blockchain: SupportedBlockchain,
    proposal_id: u64,
) -> Result<Option<ProposalExt>, tonic::Status> {
    let channel = blockchain.channel().await?;
    let res = cosmos::query::gov::get_proposal_v1beta1(
        channel,
        cosmos_sdk_proto::cosmos::gov::v1beta1::QueryProposalRequest { proposal_id }
    ).await?;
    Ok(if let Some(proposal) = res.proposal {
        Some(ProposalExt::from_v1beta1(&blockchain, proposal))
    }else{
        None
    })
}

pub async fn get_proposal(
    blockchain: SupportedBlockchain,
    proposal_id: u64,
) -> Result<Option<ProposalExt>, tonic::Status> {
    let res = get_proposal_v1(blockchain.clone(), proposal_id).await;

    if let Err(tonic_status) = &res {
        match tonic_status.code() {
            Code::Unimplemented | Code::Unknown | Code::Aborted | Code::Cancelled => {
                let res = get_proposal_v1beta1(blockchain, proposal_id).await;
                return res;
            }
            _ => {}
        }
    }
    return res;
}

/// Retrieve a list of proposals based on the specified criteria.
///
/// This function retrieves a list of proposals from a supported blockchain
/// with the given proposal status. The proposals are fetched using pagination,
/// where either the `next_key` parameter or the `offset` indicates the starting point
/// and `limit` parameters control the number of proposals to be fetched.
/// If an error occurs during the retrieval, the function falls back to alternative
/// retrieval methods depending on the error code.
///
/// # Arguments
///
/// * `blockchain` - The supported blockchain from which to fetch proposals.
/// * `proposal_status` - The status of the proposals to fetch.
/// * `next_key` - The pagination key indicating the starting point for fetching proposals.
/// * `offset` - The offset for pagination, indicating the number of proposals to skip.
/// * `limit` - The maximum number of proposals to fetch per request.
/// * `skip_server_error` - skipping requests for proposals that return `OutOfRange` (only works using the `offset` pagination method).
///
/// # Returns
///
/// A tuple containing an optional pagination key for the next batch of proposals and
/// a vector of `ProposalExt` instances representing the fetched proposals.
///
/// # Errors
///
/// Returns a `tonic::Status` error if there's an issue with the retrieval process.
pub async fn get_proposals(
    blockchain: SupportedBlockchain,
    proposal_status: ProposalStatus,
    next_key: Option<Vec<u8>>,
    offset: Option<u64>,
    limit: Option<u64>,
    skip_server_error: bool
) -> Result<(Option<Vec<u8>>, Vec<ProposalExt>),tonic::Status> {
    let res = get_proposals_v1(blockchain.clone(), proposal_status.clone(), next_key.clone(), offset.clone(),limit.clone(),skip_server_error).await;

    if let Err(tonic_status) = &res {
        match tonic_status.code() {
            Code::Unimplemented | Code::Unknown | Code::Aborted | Code::Cancelled => {
                let res = get_proposals_v1beta1(blockchain, proposal_status, next_key, offset.clone(),limit.clone(),skip_server_error).await;
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
        let result = get_proposals(supported_blockchain, ProposalStatus::StatusNil, None, None,None, false).await;
        assert!(result.is_ok());
        for each in result.unwrap().1 {
            println!("Decoded Proposal:\n{:?}", &each.proposal);
            println!("Proposal Status:\n{:?}", each.get_proposal_status());

            //println!("Decoded Proposal Content:\n{:?}", each.content_opt());

            let serialized = serde_json::to_string(&each).unwrap();
            println!("\n\nserialized = {}", &serialized);
            let deserialized: ProposalExt = serde_json::from_str(&serialized).unwrap();
            println!("\ndeserialized = {:?}", deserialized);
            break;
        }
    }
}
