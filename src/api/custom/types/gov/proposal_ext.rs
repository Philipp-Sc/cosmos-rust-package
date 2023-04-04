use crate::api::core::cosmos::channels::{get_supported_blockchains, SupportedBlockchain};
use crate::api::core::*;
use prost_types::Timestamp;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};

use std::string::ToString;
use strum_macros;
use strum_macros::EnumIter;

use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use cosmos_sdk_proto::prost::EncodeError;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};

use cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;
use cosmos_sdk_proto::cosmos::gov::v1beta1::{QueryParamsResponse, QueryTallyResultResponse};
use linkify::LinkFinder;

use cosmos_sdk_proto::prost::Message;
use serde::ser::{Serializer,SerializeStruct};
use crate::api::custom::types::gov::tally_ext::TallyHelper;
use crate::api::custom::types::ProtoMessageWrapper;


#[derive(Deserialize, Serialize, strum_macros::Display, Debug, Clone, PartialEq, EnumIter, Hash)]
pub enum ProposalStatus {
    /*
    StatusNil           ProposalStatus = 0x00
    StatusDepositPeriod ProposalStatus = 0x01  // Proposal is submitted. Participants can deposit on it but not vote
    StatusVotingPeriod  ProposalStatus = 0x02  // MinDeposit is reached, participants can vote
    StatusPassed        ProposalStatus = 0x03  // Proposal passed and successfully executed
    StatusRejected      ProposalStatus = 0x04  // Proposal has been rejected
    StatusFailed        ProposalStatus = 0x05  // Proposal passed but failed execution
    */
    StatusNil = 0x00,
    StatusDepositPeriod = 0x01,
    StatusVotingPeriod = 0x02,
    StatusPassed = 0x03,
    StatusRejected = 0x04,
    StatusFailed = 0x05,
}

#[derive(Deserialize, Serialize, strum_macros::Display, Debug, Clone, PartialEq, EnumIter)]
pub enum ProposalTime {
    SubmitTime,
    DepositEndTime,
    VotingStartTime,
    VotingEndTime,
    LatestTime,
}

impl ProposalStatus {
    pub fn new(name: &str) -> ProposalStatus {
        match name {
            "nil" => ProposalStatus::StatusNil,
            "passed" => ProposalStatus::StatusPassed,
            "failed" => ProposalStatus::StatusFailed,
            "rejected" => ProposalStatus::StatusRejected,
            "deposit_period" => ProposalStatus::StatusDepositPeriod,
            "voting_period" => ProposalStatus::StatusVotingPeriod,
            _ => panic!(),
        }
    }
    pub fn to_icon(&self) -> String {
        match self {
            ProposalStatus::StatusNil => "⚪".to_string(),
            ProposalStatus::StatusPassed => "🟢".to_string(),
            ProposalStatus::StatusFailed => "❌".to_string(),
            ProposalStatus::StatusRejected => "🔴".to_string(),
            ProposalStatus::StatusVotingPeriod => "🗳".to_string(),
            ProposalStatus::StatusDepositPeriod => "💰".to_string(),
        }
    }
}

#[derive(strum_macros::Display, Debug, Clone, PartialEq)]
pub enum ProposalContent {
    TextProposal(Option<cosmos_sdk_proto::cosmos::gov::v1beta1::TextProposal>),
    CommunityPoolSpendProposal(
        Option<cosmos_sdk_proto::cosmos::distribution::v1beta1::CommunityPoolSpendProposal>,
    ),
    ParameterChangeProposal(
        Option<cosmos_sdk_proto::cosmos::params::v1beta1::ParameterChangeProposal>,
    ),
    SoftwareUpgradeProposal(
        Option<cosmos_sdk_proto::cosmos::upgrade::v1beta1::SoftwareUpgradeProposal>,
    ),
    ClientUpdateProposal(Option<cosmos_sdk_proto::ibc::core::client::v1::ClientUpdateProposal>),
    UpdatePoolIncentivesProposal(
        Option<osmosis_std::types::osmosis::poolincentives::v1beta1::UpdatePoolIncentivesProposal>,
    ),
    StoreCodeProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::StoreCodeProposal>),
    RemoveSuperfluidAssetsProposal(
        Option<osmosis_std::types::osmosis::superfluid::v1beta1::RemoveSuperfluidAssetsProposal>,
    ),
    InstantiateContractProposal(
        Option<cosmos_sdk_proto::cosmwasm::wasm::v1::InstantiateContractProposal>,
    ),
    SetSuperfluidAssetsProposal(
        Option<osmosis_std::types::osmosis::superfluid::v1beta1::SetSuperfluidAssetsProposal>,
    ),
    UpdateFeeTokenProposal(
        Option<osmosis_std::types::osmosis::txfees::v1beta1::UpdateFeeTokenProposal>,
    ),
    ReplacePoolIncentivesProposal(
        Option<osmosis_std::types::osmosis::poolincentives::v1beta1::ReplacePoolIncentivesProposal>,
    ),
    MigrateContractProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::MigrateContractProposal>),
    UpdateInstantiateConfigProposal(
        Option<cosmos_sdk_proto::cosmwasm::wasm::v1::UpdateInstantiateConfigProposal>,
    ),
    SudoContractProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::SudoContractProposal>),
    ExecuteContractProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::ExecuteContractProposal>),
    UpdateAdminProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::UpdateAdminProposal>),
    ClearAdminProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::ClearAdminProposal>),
    PinCodesProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::PinCodesProposal>),
    UnpinCodesProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::UnpinCodesProposal>),
    UnknownProposalType(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash)]
pub struct ProposalExt {
    pub blockchain_name: String,
    pub status: ProposalStatus,
    pub proposal: ProtoMessageWrapper<cosmos_sdk_proto::cosmos::gov::v1beta1::Proposal>,
}

impl ProposalExt {
    pub fn new(
        blockchain: &SupportedBlockchain,
        proposal_status: &ProposalStatus,
        proposal: cosmos_sdk_proto::cosmos::gov::v1beta1::Proposal,
    ) -> Self {
        Self {
            blockchain_name: blockchain.name.to_string(),
            status: proposal_status.clone(),
            proposal: ProtoMessageWrapper(proposal),
        }
    }

    pub fn to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        &self.hash(&mut s);
        s.finish()
    }
    pub fn content_opt(&self) -> Option<ProposalContent> {
        let proposal_content = self.proposal.0.content.as_ref().map(|any| Self::content(&any));
        proposal_content
    }

    pub fn content(any: &cosmos_sdk_proto::Any) -> ProposalContent {
 
            let a = any.type_url.to_string();
            match a.as_ref() {
                "/cosmos.gov.v1beta1.TextProposal" => ProposalContent::TextProposal(
                    cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                ),
                "/cosmos.distribution.v1beta1.CommunityPoolSpendProposal" => {
                    ProposalContent::CommunityPoolSpendProposal(
                        cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                    )
                }
                "/cosmos.params.v1beta1.ParameterChangeProposal" => {
                    ProposalContent::ParameterChangeProposal(
                        cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                    )
                }
                "/cosmos.upgrade.v1beta1.SoftwareUpgradeProposal" => {
                    ProposalContent::SoftwareUpgradeProposal(
                        cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                    )
                }
                "/ibc.core.client.v1.ClientUpdateProposal" => {
                    ProposalContent::ClientUpdateProposal(
                        cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                    )
                }
                "/osmosis.poolincentives.v1beta1.UpdatePoolIncentivesProposal" => {
                    let decoded = osmosis_std::types::osmosis::poolincentives::v1beta1::UpdatePoolIncentivesProposal::decode(&*any.value).ok();
                    ProposalContent::UpdatePoolIncentivesProposal(decoded)
                }
                "/cosmwasm.wasm.v1.StoreCodeProposal" => {
                    ProposalContent::StoreCodeProposal(
                        cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                    )
                }
                "/cosmwasm.wasm.v1.InstantiateContractProposal" => {
                    ProposalContent::InstantiateContractProposal(
                        cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                    )
                }
                "/osmosis.superfluid.v1beta1.RemoveSuperfluidAssetsProposal" => {
                    let decoded = osmosis_std::types::osmosis::superfluid::v1beta1::RemoveSuperfluidAssetsProposal::decode(&*any.value).ok();
                    ProposalContent::RemoveSuperfluidAssetsProposal(decoded)
                }
                "/osmosis.superfluid.v1beta1.SetSuperfluidAssetsProposal" => {
                    let decoded = osmosis_std::types::osmosis::superfluid::v1beta1::SetSuperfluidAssetsProposal::decode(&*any.value).ok();
                    ProposalContent::SetSuperfluidAssetsProposal(decoded)
                }
                "/osmosis.txfees.v1beta1.UpdateFeeTokenProposal" => {
                    let decoded = osmosis_std::types::osmosis::txfees::v1beta1::UpdateFeeTokenProposal::decode(&*any.value).ok();
                    ProposalContent::UpdateFeeTokenProposal(decoded)
                }
                "/osmosis.poolincentives.v1beta1.ReplacePoolIncentivesProposal" => {
                    let decoded = osmosis_std::types::osmosis::poolincentives::v1beta1::ReplacePoolIncentivesProposal::decode(&*any.value).ok();
                    ProposalContent::ReplacePoolIncentivesProposal(decoded)
                }
                "/cosmwasm.wasm.v1.MigrateContractProposal" => {
                    ProposalContent::MigrateContractProposal(
                        cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                    )
                }
                "/cosmwasm.wasm.v1.UpdateInstantiateConfigProposal" => {
                    ProposalContent::UpdateInstantiateConfigProposal(
                        cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                    )
                }
                "/cosmwasm.wasm.v1.SudoContractProposal" => {
                    ProposalContent::SudoContractProposal(
                        cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                    )
                }
                "/cosmwasm.wasm.v1.ExecuteContractProposal" => {
                    ProposalContent::ExecuteContractProposal(
                        cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                    )
                }
                "/cosmwasm.wasm.v1.UpdateAdminProposal" => {
                    ProposalContent::UpdateAdminProposal(
                        cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                    )
                }
                "/cosmwasm.wasm.v1.ClearAdminProposal" => {
                    ProposalContent::ClearAdminProposal(
                        cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                    )
                }
                "/cosmwasm.wasm.v1.PinCodesProposal" => ProposalContent::PinCodesProposal(
                    cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                ),
                "/cosmwasm.wasm.v1.UnpinCodesProposal" => {
                    ProposalContent::UnpinCodesProposal(
                        cosmos_sdk_proto::traits::MessageExt::from_any(any).ok()
                    )
                }
                &_ => ProposalContent::UnknownProposalType(a),
            }
    }


    pub fn time(&self, time: &ProposalTime) -> Option<Timestamp> {
        let proposal = &self.proposal.0;
        match time {
            &ProposalTime::SubmitTime => proposal.submit_time.clone(),
            &ProposalTime::DepositEndTime => proposal.deposit_end_time.clone(),
            &ProposalTime::VotingEndTime => proposal.voting_end_time.clone(),
            &ProposalTime::VotingStartTime => proposal.voting_start_time.clone(),
            &ProposalTime::LatestTime => self.latest_time(proposal),
        }
    }
    pub fn latest_time(
        &self,
        proposal: &cosmos_sdk_proto::cosmos::gov::v1beta1::Proposal,
    ) -> Option<Timestamp> {
        match self.status {
            ProposalStatus::StatusNil | ProposalStatus::StatusDepositPeriod => {
                proposal.submit_time.clone()
            }
            ProposalStatus::StatusVotingPeriod => proposal.voting_start_time.clone(),
            ProposalStatus::StatusPassed
            | ProposalStatus::StatusFailed
            | ProposalStatus::StatusRejected => proposal.voting_end_time.clone(),
        }
    }

    pub fn get_description(&self) -> String {
        match &self.content_opt() {
            Some(ProposalContent::TextProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::CommunityPoolSpendProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::ParameterChangeProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::SoftwareUpgradeProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::ClientUpdateProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::UpdatePoolIncentivesProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::StoreCodeProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::RemoveSuperfluidAssetsProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::InstantiateContractProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::ReplacePoolIncentivesProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::SetSuperfluidAssetsProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::UpdateFeeTokenProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::MigrateContractProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::UpdateInstantiateConfigProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::SudoContractProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::ExecuteContractProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::UpdateAdminProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::ClearAdminProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::UnpinCodesProposal(Some(p))) => {
                p.description.to_owned()
            }
            Some(ProposalContent::UnknownProposalType(type_url)) =>
                format!("UnknownDescription\n\nType URL:\n{}", type_url)
            ,
            Some(_) =>
                "ContentDecodeErrorDescription".to_string()
            ,
            None =>
                "ProposalDecodeErrorDescription".to_string()
            ,
        }
    }
    pub fn get_title(&self) -> String {
        match &self.content_opt() {
            Some(ProposalContent::TextProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::CommunityPoolSpendProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::ParameterChangeProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::SoftwareUpgradeProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::ClientUpdateProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::UpdatePoolIncentivesProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::StoreCodeProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::RemoveSuperfluidAssetsProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::InstantiateContractProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::ReplacePoolIncentivesProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::SetSuperfluidAssetsProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::UpdateFeeTokenProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::MigrateContractProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::UpdateInstantiateConfigProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::SudoContractProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::ExecuteContractProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::UpdateAdminProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::ClearAdminProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::UnpinCodesProposal(Some(p))) => {
                p.title.to_owned()
            }
            Some(ProposalContent::UnknownProposalType(type_url)) =>
                "UnknownTitle".to_string()
            ,
            Some(_) =>
                "ContentDecodeErrorTitle".to_string()
            ,
            None =>
                "ProposalDecodeErrorTitle".to_string()
            ,
        }
    }

    fn timestamp_to_string(item: Option<Timestamp>) -> String {
        match item.as_ref() {
            Some(time) => {
                if time.seconds > 0 {
                    DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp_opt(time.seconds, time.nanos as u32).unwrap(),
                        Utc,
                    )
                        .to_rfc2822()
                        .replace("+0000", "UTC")
                } else {
                    "".to_string()
                }
            }
            None => "".to_string(),
        }
    }

    fn get_voting_start_and_end(&self) -> (String, String) {
        let proposal = &self.proposal.0;
        (
            format!("{}", Self::timestamp_to_string(proposal.voting_start_time.clone())),
            format!("{}", Self::timestamp_to_string(proposal.voting_end_time.clone())),
        )
    }

    pub fn spam_likelihood(&self) -> Option<f64> {
        let proposal = &self.proposal.0;
        if let Some(tally) = &proposal.final_tally_result {
            return TallyHelper(tally).spam_likelihood();
        }
        None
    }

    pub fn final_tally_with_no_with_veto_majority(&self) -> bool {
        let proposal = &self.proposal.0;
        proposal.final_tally_result.clone().map(|y| {
            let no_with_veto = y.no_with_veto.parse::<f64>().unwrap_or(0f64);
            let yes =  y.yes.parse::<f64>().unwrap_or(0f64);
            let no = y.no.parse::<f64>().unwrap_or(0f64);
            no_with_veto > yes && no_with_veto > no
        }).unwrap_or(false)
    }

    pub fn get_final_tally_result(&self) -> String {
        let proposal = &self.proposal.0;
        let mut output = String::new();
        if let Some(tally) = &proposal.final_tally_result {
            output.push_str(&TallyHelper(tally).final_tally_to_string());
        }
        output
    }

    pub fn get_proposal_id(&self) -> u64 {
        self.proposal.0.proposal_id
    }

    pub fn governance_proposal_link(&self) -> String {
        let mut proposal_id = self.get_proposal_id().to_string();

        let blockchain_name = self.blockchain_name.to_lowercase();

        match get_supported_blockchains()
            .get(&blockchain_name){
            Some(supported_blockchain) => {
                format!("{}{}",supported_blockchain.governance_proposals_link,proposal_id)
            },
            None => {
                format!("https://www.mintscan.io/{}/proposals/{}",blockchain_name,proposal_id)
            }
        }
    }

    pub fn proposal_state(&self) -> String {
        let proposal = &self.proposal.0;
        let (voting_start_text, voting_end_text) = self.get_voting_start_and_end();
        let mut tally_result = self.get_final_tally_result();

        let mut voting_state = "".to_string();
        if &self.status == &ProposalStatus::StatusVotingPeriod {
            let mut voting_start = false;
            if let Some(time) = &proposal.voting_start_time {
                match DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp_opt(time.seconds, time.nanos as u32).unwrap(),
                    Utc,
                )
                    .cmp(&Utc::now())
                {
                    Ordering::Less | Ordering::Equal => {
                        voting_start = true;
                    }
                    Ordering::Greater => {
                        voting_start = false;
                    }
                }
            }
            let mut voting_end = false;
            if let Some(time) = &proposal.voting_end_time {
                match DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp_opt(time.seconds, time.nanos as u32).unwrap(),
                    Utc,
                )
                    .cmp(&Utc::now())
                {
                    Ordering::Less | Ordering::Equal => {
                        voting_end = true;
                    }
                    Ordering::Greater => {
                        voting_end = false;
                    }
                }
            }
            voting_state = match (voting_start, voting_end) {
                (true, true) => format!("Voting finished"),
                (true, false) => format!("Voting ends at {}", voting_end_text),
                (false, false) => format!("Voting starts at {}", voting_start_text),
                (false, true) => format!("Voting ended before it started!"),
            };
        } else if &self.status == &ProposalStatus::StatusDepositPeriod {
            voting_state = format!("You can help the proposal move forward by depositing now. \nThe deposit period is open until {}",Self::timestamp_to_string(proposal.deposit_end_time.clone()))
        }

        format!(
            "{}\n\n{}\n\n{}",
            &self.status.to_icon(),
            voting_state,
            tally_result,
        )
    }
}