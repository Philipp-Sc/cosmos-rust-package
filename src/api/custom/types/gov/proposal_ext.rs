use crate::api::core::cosmos::channels::SupportedBlockchain;

use prost_types::Timestamp;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;

use std::hash::{Hash, Hasher};

use std::string::ToString;

use strum_macros;
use strum_macros::EnumIter;

use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use cosmos_sdk_proto::cosmos::gov::v1::TallyResult;


use serde::{Deserialize, Serialize};
use crate::api::custom::types::gov::common::{ProposalContent, ProposalStatus};
use crate::api::custom::types::gov::params_ext::ParamsExt;


use crate::api::custom::types::gov::tally_ext::TallyHelper;
use crate::api::custom::types::gov::tally_v1beta1_ext::TallyResultV1Beta1Ext;
use crate::api::custom::types::ProtoMessageWrapper;
use crate::api::custom::types::staking::pool_ext::PoolExt;


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProposalParams {
    pub fraud_classification: Option<f64>,
    pub tally_result: Option<TallyResultV1Beta1Ext>,
    pub tallying_param: Option<ParamsExt>,
    pub deposit_param: Option<ParamsExt>,
    pub voting_param: Option<ParamsExt>,
    pub blockchain_pool: Option<PoolExt>,
}

impl Hash for ProposalParams{
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

impl ProposalParams {
    pub fn new(
        fraud_classification: Option<f64>,
        tally_result: Option<TallyResultV1Beta1Ext>,
        tallying_param: Option<ParamsExt>,
        deposit_param: Option<ParamsExt>,
        voting_param: Option<ParamsExt>,
        blockchain_pool: Option<PoolExt>,
    ) -> Self {
        Self {
            fraud_classification,
            tally_result,
            tallying_param,
            deposit_param,
            voting_param,
            blockchain_pool,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash)]
pub struct ProposalExt {
    pub blockchain: SupportedBlockchain,
    pub proposal: ProtoMessageWrapper<cosmos_sdk_proto::cosmos::gov::v1::Proposal>,
    pub params: Option<ProposalParams>
}

impl ProposalExt {
    pub fn new(
        blockchain: &SupportedBlockchain,
        proposal: cosmos_sdk_proto::cosmos::gov::v1::Proposal,
    ) -> Self {
        Self {
            blockchain: blockchain.clone(),
            proposal: ProtoMessageWrapper(proposal),
            params: None
        }
    }

    pub fn from_v1beta1(
        blockchain: &SupportedBlockchain,
        proposal: cosmos_sdk_proto::cosmos::gov::v1beta1::Proposal,
    ) -> Self {
        Self {
            blockchain: blockchain.clone(),
            proposal: ProtoMessageWrapper(cosmos_sdk_proto::cosmos::gov::v1::Proposal {
                id: proposal.proposal_id,
                messages: proposal.content.map(|msg| {vec![msg]}).unwrap_or(vec![]),
                status: proposal.status,
                final_tally_result: proposal.final_tally_result.map(|tally| TallyResult {
                    yes_count: tally.yes,
                    abstain_count: tally.abstain,
                    no_count: tally.no,
                    no_with_veto_count: tally.no_with_veto,
                }),
                submit_time: proposal.submit_time,
                deposit_end_time: proposal.deposit_end_time,
                total_deposit: proposal.total_deposit,
                voting_start_time: proposal.voting_start_time,
                voting_end_time: proposal.voting_end_time,
                metadata: "".to_string(),
            }),
            params: None
        }
    }

    pub fn add_params(&mut self, params: ProposalParams){
        self.params = Some(params);
    }

    pub fn object_to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        &self.hash(&mut s);
        s.finish()
    }
    pub fn messages_as_proposal_content(&self) -> Vec<ProposalContent> {
        let proposal_content = self
            .proposal
            .0
            .messages.iter()
            .map(|any| Self::content(&any)).collect();
        proposal_content
    }
    
    pub fn get_proposal_status(&self) -> ProposalStatus {
        ProposalStatus::from_i32(self.proposal.0.status).unwrap()
    }

    pub fn get_proposal_types(&self) -> Vec<String>{
        self.messages_as_proposal_content().iter().map(|x| x.to_string()).collect::<Vec<String>>()
    }

    pub fn is_final_state(&self) -> bool {
        self.proposal.0.status > 0x02
    }

    pub fn content(any: &cosmos_sdk_proto::Any) -> ProposalContent {
        ProposalContent::new(any)
    }

    pub fn is_in_deposit_period(&self) -> bool {
        self.get_proposal_status() == ProposalStatus::StatusDepositPeriod
    }

    pub fn is_in_voting_period(&self) -> bool {
        self.get_proposal_status() == ProposalStatus::StatusVotingPeriod
    }

    pub fn get_timestamp_based_on_proposal_status(&self) -> &Option<Timestamp> {
        match self.get_proposal_status() {
            ProposalStatus::StatusNil | ProposalStatus::StatusDepositPeriod => {
                &self.proposal.0.submit_time
            }
            ProposalStatus::StatusVotingPeriod => &self.proposal.0.voting_start_time,
            ProposalStatus::StatusPassed
            | ProposalStatus::StatusFailed
            | ProposalStatus::StatusRejected => &self.proposal.0.voting_end_time,
        }
    }

    pub fn get_description(&self) -> String {
        self.messages_as_proposal_content().iter().map(|msg| msg.get_description().replace("\\n","\n")).collect::<Vec<String>>().join("\n")
    }
    pub fn get_title(&self) -> String {
        self.messages_as_proposal_content().iter().map(|msg| msg.get_title()).collect::<Vec<String>>().join("\n")
    }


    pub fn preview_msg(&self) -> String {
        let title = self.get_title();

        let proposal_id = self.get_proposal_id();

        let mut display = format!(
            "{}\n\n{}\n#{}  -  {}\n{}",
            &self.blockchain.display,
            self.messages_as_proposal_content().iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>().join("\n"),
            proposal_id,
            &self.get_proposal_status().to_icon(),
            title,
        );

        if let Some(prediction) =  self.params.as_ref().map(|x| x.fraud_classification).flatten() {
            let label = if prediction >= 0.7 {
                format!("🚨 {}", Self::map_prediction_to_string(prediction))
            } else if prediction >= 0.35 {
                format!("⚠ {}", Self::map_prediction_to_string(prediction))
            } else if prediction >= 0.30 {
                format!("❗ {}", Self::map_prediction_to_string(prediction))
            } else if prediction > 0.25 {
                format!("⁉️️ {}", Self::map_prediction_to_string(prediction))
            } else {
                format!("🛡️ {}", Self::map_prediction_to_string(prediction))
            };
            display = format!("{}\n\n{}", display, label);
        }
        display
    }

    fn map_prediction_to_string(number: f64) -> String {
        let mut result = String::new();

        let filled_blocks = (number * 10.0).round() as usize;
        for i in 0..filled_blocks {
            result.push('●');
            if i == 3 {
                result.push('|');
            }
        }

        let empty_blocks = 10 - filled_blocks;
        for i in 0..empty_blocks {
            result.push('○');
            if i + filled_blocks == 3 {
                result.push('|');
            }
        }

        result
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
            format!(
                "{}",
                Self::timestamp_to_string(proposal.voting_start_time.clone())
            ),
            format!(
                "{}",
                Self::timestamp_to_string(proposal.voting_end_time.clone())
            ),
        )
    }

    pub fn spam_likelihood(&self) -> Option<f64> {
        let is_bad_proposal = match self.get_proposal_status() {
            ProposalStatus::StatusRejected | ProposalStatus::StatusFailed => {
                true
            },
            ProposalStatus::StatusPassed => {
                return  Some(0.0);
            }
            _ => {false}
        };
        let mut result: Option<f64> = None;
        if let Some(tally) = &self.proposal.0.final_tally_result {
            result = TallyHelper(tally).spam_likelihood();
        }
        if is_bad_proposal {
            Some(result.map(|spam_likelihood| (spam_likelihood +1.0)/2.0).unwrap_or(1.0))
        }else{
            result
        }
    }
    pub fn total_votes(&self) -> Option<f64> {
        let proposal = &self.proposal.0;
        if let Some(tally) = &proposal.final_tally_result {
            return TallyHelper(tally).total_votes();
        }
        None
    }

    pub fn final_tally_with_no_with_veto_majority(&self) -> bool {
        let proposal = &self.proposal.0;
        proposal
            .final_tally_result
            .clone()
            .map(|y| {
                let no_with_veto = y.no_with_veto_count.parse::<f64>().unwrap_or(0f64);
                let yes = y.yes_count.parse::<f64>().unwrap_or(0f64);
                let no = y.no_count.parse::<f64>().unwrap_or(0f64);
                no_with_veto > yes && no_with_veto > no
            })
            .unwrap_or(false)
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
        self.proposal.0.id
    }

    pub fn governance_proposal_link(&self) -> String {
        // format!("https://www.mintscan.io/{}/proposals/{}",blockchain_name,proposal_id)
        format!(
            "{}{}",
            self.blockchain.governance_proposals_link,
            self.get_proposal_id()
        )
    }

    pub fn tally_details(&self) -> Option<String> {
        let proposal = &self.proposal.0;
        if let Some(tally) = &proposal.final_tally_result {
            Some(TallyHelper(tally).tally_details())
        } else {
            None
        }
    }

    pub fn proposal_submitted(&self) -> String {
        let proposal = &self.proposal.0;
        Self::timestamp_to_string(proposal.submit_time.clone())
    }

    pub fn proposal_state(&self) -> String {
        let proposal = &self.proposal.0;
        let (voting_start_text, voting_end_text) = self.get_voting_start_and_end();
        let tally_result = self.get_final_tally_result();

        let mut voting_state = "".to_string();
        if &self.get_proposal_status() == &ProposalStatus::StatusVotingPeriod {
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
                (true, true) => format!("Voting finished\n\n"),
                (true, false) => format!("Voting ends at {}\n\n", voting_end_text),
                (false, false) => format!("Voting starts at {}\n\n", voting_start_text),
                (false, true) => format!("Voting ended before it started!\n\n"),
            };
        } else if &self.get_proposal_status() == &ProposalStatus::StatusDepositPeriod {
            voting_state = format!("You can help the proposal move forward by depositing now. \nThe deposit period is open until {}\n\n",Self::timestamp_to_string(proposal.deposit_end_time.clone()))
        }

        format!(
            "{} {}{}",
            &self.get_proposal_status().to_icon(),
            voting_state,
            tally_result,
        )
        .trim()
        .to_string()
    }
}
