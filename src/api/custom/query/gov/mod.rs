
use std::hash::{Hash, Hasher};
use prost_types::Timestamp;
use crate::api::core::*;
use crate::api::core::cosmos::channels::{get_supported_blockchains, SupportedBlockchain};

use strum_macros::EnumIter;
use std::string::ToString;
use std::time::SystemTime;
use strum_macros;

use chrono::{NaiveDateTime, NaiveDate};
use chrono::{DateTime, TimeZone, Utc};
use cosmos_sdk_proto::cosmos::gov::v1beta1::TallyResult;

use regex::Regex;

#[derive(strum_macros::ToString,Debug,Clone,PartialEq,EnumIter)]
pub enum ProposalStatus { /*
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
    StatusFailed = 0x05
}


#[derive(strum_macros::ToString,Debug,Clone,PartialEq,EnumIter)]
pub enum ProposalTime {
    SubmitTime,
    DepositEndTime,
    VotingStartTime,
    VotingEndTime,
    LatestTime
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
            ProposalStatus::StatusNil => {
                "⚪".to_string()
            },
            ProposalStatus::StatusPassed => {
                "🟢".to_string()
            },
            ProposalStatus::StatusFailed => {
                "❌".to_string()
            },
            ProposalStatus::StatusRejected => {
                "🔴".to_string()
            },
            ProposalStatus::StatusVotingPeriod => {
                "🗳".to_string()
            },
            ProposalStatus::StatusDepositPeriod => {
                "💰".to_string()
            },
        }

    }
}

#[derive(strum_macros::ToString,Debug, Clone)]
pub enum ProposalContent {
    TextProposal(cosmos_sdk_proto::cosmos::gov::v1beta1::TextProposal),
    CommunityPoolSpendProposal(cosmos_sdk_proto::cosmos::distribution::v1beta1::CommunityPoolSpendProposal),
    ParameterChangeProposal(cosmos_sdk_proto::cosmos::params::v1beta1::ParameterChangeProposal),
    SoftwareUpgradeProposal(cosmos_sdk_proto::cosmos::upgrade::v1beta1::SoftwareUpgradeProposal),
    ClientUpdateProposal(cosmos_sdk_proto::ibc::core::client::v1::ClientUpdateProposal),
    UpdatePoolIncentivesProposal(osmosis_proto::osmosis::poolincentives::v1beta1::UpdatePoolIncentivesProposal),
    StoreCodeProposal(cosmos_sdk_proto::cosmwasm::wasm::v1::StoreCodeProposal),
    RemoveSuperfluidAssetsProposal(osmosis_proto::osmosis::superfluid::v1beta1::RemoveSuperfluidAssetsProposal),
    UnknownProposalType(String),
}

#[derive(Debug, Clone)]
pub struct ProposalExt {
    pub blockchain_name: String,
    pub status: ProposalStatus,
    pub proposal: cosmos_sdk_proto::cosmos::gov::v1beta1::Proposal,
    pub content: ProposalContent,
}

impl Hash for ProposalExt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.blockchain_name.hash(state);
        self.proposal.proposal_id.hash(state);
        self.status.to_string().hash(state);
    }
}

impl ProposalExt {
    pub fn new(blockchain: &SupportedBlockchain, proposal_status: &ProposalStatus, mut proposal: cosmos_sdk_proto::cosmos::gov::v1beta1::Proposal) -> ProposalExt {
        let content = ProposalExt::content(&proposal);
        proposal.content = None;
        ProposalExt {
            blockchain_name: blockchain.name.to_string(),
            status: proposal_status.clone(),
            proposal,
            content,
        }
    }
    fn content(proposal: &cosmos_sdk_proto::cosmos::gov::v1beta1::Proposal) -> ProposalContent {
        let p = proposal.content.as_ref().unwrap();
        if p.type_url == "/cosmos.gov.v1beta1.TextProposal" {
            let t: cosmos_sdk_proto::cosmos::gov::v1beta1::TextProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            ProposalContent::TextProposal(t)
        } else if p.type_url == "/cosmos.distribution.v1beta1.CommunityPoolSpendProposal" {
            let t: cosmos_sdk_proto::cosmos::distribution::v1beta1::CommunityPoolSpendProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            ProposalContent::CommunityPoolSpendProposal(t)
        } else if p.type_url == "/cosmos.params.v1beta1.ParameterChangeProposal" {
            let t: cosmos_sdk_proto::cosmos::params::v1beta1::ParameterChangeProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            ProposalContent::ParameterChangeProposal(t)
        } else if p.type_url == "/cosmos.upgrade.v1beta1.SoftwareUpgradeProposal" {
            let t: cosmos_sdk_proto::cosmos::upgrade::v1beta1::SoftwareUpgradeProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            ProposalContent::SoftwareUpgradeProposal(t)
        } else if p.type_url == "/ibc.core.client.v1.ClientUpdateProposal" {
            let t: cosmos_sdk_proto::ibc::core::client::v1::ClientUpdateProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            ProposalContent::ClientUpdateProposal(t)
        } else if p.type_url == "/osmosis.poolincentives.v1beta1.UpdatePoolIncentivesProposal" {
            let t: osmosis_proto::osmosis::poolincentives::v1beta1::UpdatePoolIncentivesProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            ProposalContent::UpdatePoolIncentivesProposal(t)
        } else if p.type_url == "/cosmwasm.wasm.v1.StoreCodeProposal" {
            let t: cosmos_sdk_proto::cosmwasm::wasm::v1::StoreCodeProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            ProposalContent::StoreCodeProposal(t)
        }else if p.type_url == "/osmosis.superfluid.v1beta1.RemoveSuperfluidAssetsProposal" {
            let t: osmosis_proto::osmosis::superfluid::v1beta1::RemoveSuperfluidAssetsProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            ProposalContent::RemoveSuperfluidAssetsProposal(t)
        }else{
            ProposalContent::UnknownProposalType(p.type_url.to_string())
        }
    }
    pub fn time(&self, time: ProposalTime) -> Option<Timestamp> {
        match time {
            ProposalTime::SubmitTime => {self.proposal.submit_time.clone()},
            ProposalTime::DepositEndTime => {self.proposal.deposit_end_time.clone()},
            ProposalTime::VotingEndTime => {self.proposal.voting_end_time.clone()},
            ProposalTime::VotingStartTime => {self.proposal.voting_start_time.clone()}
            ProposalTime::LatestTime => {self.latest_time()}
        }
    }
    pub fn latest_time(&self) -> Option<Timestamp> {
        match self.status {
            ProposalStatus::StatusNil | ProposalStatus::StatusDepositPeriod => {
                self.proposal.submit_time.clone()
            }
            ProposalStatus::StatusVotingPeriod => {
                self.proposal.voting_start_time.clone()
            }
            ProposalStatus::StatusPassed | ProposalStatus::StatusFailed | ProposalStatus::StatusRejected => {
                self.proposal.voting_end_time.clone()
            }
        }
    }
    pub fn custom_display(&self) -> String {
        let (title,description) = match &self.content {
            ProposalContent::TextProposal(p) => {(p.title.to_owned(), p.description.to_owned())}
            ProposalContent::CommunityPoolSpendProposal(p) => {(p.title.to_owned(), p.description.to_owned())}
            ProposalContent::ParameterChangeProposal(p) => {(p.title.to_owned(), p.description.to_owned())}
            ProposalContent::SoftwareUpgradeProposal(p) => {(p.title.to_owned(), p.description.to_owned())}
            ProposalContent::ClientUpdateProposal(p) => {(p.title.to_owned(), p.description.to_owned())}
            ProposalContent::UpdatePoolIncentivesProposal(p) => {(p.title.to_owned(), p.description.to_owned())}
            ProposalContent::StoreCodeProposal(p) => {(p.title.to_owned(),p.description.to_owned())}
            ProposalContent::RemoveSuperfluidAssetsProposal(p) => {(p.title.to_string(), p.description.to_owned())}
            ProposalContent::UnknownProposalType(_) => {("UnknownTitle".to_string(),"UnknownDescription".to_string())}
            _ => {panic!()}
        };
        let voting_start = match self.proposal.voting_start_time.as_ref() {
            Some(voting_start_time) => {
                if voting_start_time.seconds > 0 {
                    format!("Voting Start: {}",DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(voting_start_time.seconds, voting_start_time.nanos as u32), Utc).to_rfc2822())
                }else{
                    "".to_string()
                }
            }
            None => {
                "".to_string()
            }
        };
        let voting_end = match self.proposal.voting_end_time.as_ref() {
            Some(voting_end_time) => {
                if voting_end_time.seconds > 0 {
                    format!("Voting End: {}",DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(voting_end_time.seconds, voting_end_time.nanos as u32), Utc).to_rfc2822())
                }else{
                    "".to_string()
                }
            }
            None => {
                "".to_string()
            }
        };

        let tally_result = match self.proposal.final_tally_result.as_ref() {
            None => {
                "".to_string()
            }
            Some(tally) => {
                if !(tally.yes == "0" && tally.abstain == "0" && tally.no == "0" && tally.no_with_veto == "0") {
                    let abstain_num = tally.abstain.parse::<f64>().unwrap();
                    let yes_num = tally.yes.parse::<f64>().unwrap();
                    let no_num = tally.no.parse::<f64>().unwrap();
                    let no_with_veto_num = tally.no_with_veto.parse::<f64>().unwrap();
                    let total = (abstain_num + yes_num + no_num + no_with_veto_num) as f64;
                    let abstain_num = f64::trunc(abstain_num/total * 100.0  * 100.0) / 100.0;
                    let yes_num = f64::trunc(yes_num/total * 100.0  * 100.0) / 100.0;
                    let no_num = f64::trunc(no_num/total * 100.0  * 100.0) / 100.0;
                    let no_with_veto_num = f64::trunc(no_with_veto_num/total * 100.0  * 100.0) / 100.0;
                    format!("👍 {}%, 👎 {}%, 🕊️ {}%, ❌ {}%", yes_num, no_num, abstain_num,no_with_veto_num)
                }else{
                    "".to_string()
                }
            }

        };
        let gov_prop_link = format!("🌐{}{}",get_supported_blockchains().get(&self.blockchain_name.to_lowercase()).unwrap().governance_proposals_link,&self.proposal.proposal_id);
        let pointpoint = if description.len() >140 {"[...]"}else{""};
        let description = format!("{}{}",description.chars().into_iter().take(140).collect::<String>(),pointpoint);
        let info = format!("{}\n#{}  -  {}\n{}\n{}\n{}\n{}\n{}\n{}",&self.content.to_string(),&self.proposal.proposal_id,&self.status.to_icon(),title,voting_start,voting_end,tally_result,description,gov_prop_link);
        Regex::new("(\n+?)").unwrap().replace_all(&info,"\n").to_string()
    }
}

pub async fn get_proposals(blockchain: SupportedBlockchain, proposal_status: ProposalStatus) -> anyhow::Result<Vec<ProposalExt>> {

    let channel = blockchain.channel().await?;
    let res = cosmos::query::get_proposals(channel, cosmos_sdk_proto::cosmos::gov::v1beta1::QueryProposalsRequest {
        proposal_status: proposal_status as i32,
        voter: "".to_string(),
        depositor: "".to_string(),
        pagination: None
    }).await?;

    let mut list: Vec<ProposalExt> = Vec::new();
    for proposal in res.proposals {
        list.push(ProposalExt::new(&blockchain, &proposal_status,proposal));
    }
    Ok(list)
}


#[cfg(test)]
mod test {

    // cargo test -- --nocapture
    // cargo test api::custom::query::gov::get_proposals -- --exact --nocapture

    use crate::api::core::cosmos::channels;

    #[tokio::test]
    pub async fn get_proposals() -> anyhow::Result<()> {

        let res = super::get_proposals(channels::get_supported_blockchains().get("terra").unwrap() ,super::ProposalStatus::StatusPassed).await?;
        println!("{:?}",res.iter().map(|x| x.content).collect::<Vec<super::ProposalContent>>());
        Ok(())
    }
}