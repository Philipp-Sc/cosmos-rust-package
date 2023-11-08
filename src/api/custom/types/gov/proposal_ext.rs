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


//use cosmos_sdk_proto::prost::Message;
//use cosmos_sdk_proto::traits::MessageExt;

use crate::api::custom::types::gov::tally_ext::TallyHelper;
use crate::api::custom::types::ProtoMessageWrapper;


#[derive(
    Deserialize, Serialize, strum_macros::Display, Debug, Clone, Eq, PartialEq, EnumIter, Hash,
)]
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

impl ProposalStatus {
    // Function to convert i32 to the corresponding ProposalStatus
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0x00 => Some(ProposalStatus::StatusNil),
            0x01 => Some(ProposalStatus::StatusDepositPeriod),
            0x02 => Some(ProposalStatus::StatusVotingPeriod),
            0x03 => Some(ProposalStatus::StatusPassed),
            0x04 => Some(ProposalStatus::StatusRejected),
            0x05 => Some(ProposalStatus::StatusFailed),
            _ => None, // Handle invalid values or return a default variant
        }
    }
}

impl From<ProposalStatus> for i32 {
    fn from(status: ProposalStatus) -> i32 {
        status as i32
    }
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
    pub fn to_name(&self) -> &str {
        match self {
            ProposalStatus::StatusNil => {"nil"}
            ProposalStatus::StatusDepositPeriod => {"deposit_period"}
            ProposalStatus::StatusVotingPeriod => {"voting_period"}
            ProposalStatus::StatusPassed => {"passed"}
            ProposalStatus::StatusRejected => {"rejected"}
            ProposalStatus::StatusFailed => {"failed"}
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
    MsgExec(Option<cosmos_sdk_proto::cosmos::authz::v1beta1::MsgExec>),
    //MsgUpdateParams(Option<injective_std::types::cosmos::mint::v1beta1::MsgUpdateParams>),
    MsgCommunityPoolSpend(Option<injective_std::types::cosmos::distribution::v1beta1::MsgCommunityPoolSpend>),
    MsgExecuteContract(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::MsgExecuteContract>),
    MsgUpdateInstantiateConfig(Option<osmosis_std::types::cosmwasm::wasm::v1::MsgUpdateInstantiateConfig>),
    MsgSoftwareUpgrade(Option<cosmos_sdk_proto::cosmos::upgrade::v1beta1::MsgSoftwareUpgrade>),
    MsgInstantiateContract(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::MsgInstantiateContract>),
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
    SetScalingFactorControllerProposal(
        Option<osmosis_std::types::osmosis::gamm::v1beta1::SetScalingFactorControllerProposal>,
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
    pub blockchain: SupportedBlockchain,
    pub proposal: ProtoMessageWrapper<cosmos_sdk_proto::cosmos::gov::v1::Proposal>,
}

impl ProposalExt {
    pub fn new(
        blockchain: &SupportedBlockchain,
        proposal: cosmos_sdk_proto::cosmos::gov::v1::Proposal,
    ) -> Self {
        Self {
            blockchain: blockchain.clone(),
            proposal: ProtoMessageWrapper(proposal),
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
        }
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
        let a = any.type_url.to_string();
        match a.as_ref() {
            "/cosmos.authz.v1beta1.MsgExec" => ProposalContent::MsgExec(
                any.to_msg().ok(),
            ),
            /*
            "/cosmos.mint.v1beta1.MsgUpdateParams" => ProposalContent::MsgUpdateParams(
                osmosis_prost::Message::decode(&any.value[..]).ok(),
            ),*/
            "/cosmos.distribution.v1beta1.MsgCommunityPoolSpend"=> ProposalContent::MsgCommunityPoolSpend(
                osmosis_prost::Message::decode(&any.value[..]).ok(),
            ),
            "/cosmwasm.wasm.v1.MsgExecuteContract"=> ProposalContent::MsgExecuteContract(
                any.to_msg().ok(),
            ),
            "/cosmwasm.wasm.v1.MsgUpdateInstantiateConfig" => ProposalContent::MsgUpdateInstantiateConfig(
                osmosis_prost::Message::decode(&any.value[..]).ok(),
            ),
            "/cosmos.upgrade.v1beta1.MsgSoftwareUpgrade" => ProposalContent::MsgSoftwareUpgrade(
                cosmos_sdk_proto::traits::Message::decode(&any.value[..]).ok(),
            ),
            "/cosmwasm.wasm.v1.MsgInstantiateContract" => ProposalContent::MsgInstantiateContract(
                any.to_msg().ok(),
            ),
            "/cosmos.gov.v1beta1.TextProposal" => ProposalContent::TextProposal(
                any.to_msg().ok(),
            ),
            "/cosmos.distribution.v1beta1.CommunityPoolSpendProposal" => {
                ProposalContent::CommunityPoolSpendProposal(
                    any.to_msg().ok(),
                )
            }
            "/cosmos.params.v1beta1.ParameterChangeProposal" => {
                ProposalContent::ParameterChangeProposal(
                    any.to_msg().ok(),
                )
            }
            "/cosmos.upgrade.v1beta1.SoftwareUpgradeProposal" => {
                ProposalContent::SoftwareUpgradeProposal(
                    any.to_msg().ok(),
                )
            }
            "/ibc.core.client.v1.ClientUpdateProposal" => ProposalContent::ClientUpdateProposal(
                any.to_msg().ok(),
            ),
            "/osmosis.poolincentives.v1beta1.UpdatePoolIncentivesProposal" => {
                ProposalContent::UpdatePoolIncentivesProposal(osmosis_prost::Message::decode(&any.value[..]).ok())
            }
            "/cosmwasm.wasm.v1.StoreCodeProposal" => ProposalContent::StoreCodeProposal(
                any.to_msg().ok(),
            ),
            "/cosmwasm.wasm.v1.InstantiateContractProposal" => {
                ProposalContent::InstantiateContractProposal(
                    any.to_msg().ok(),
                )
            }
            "/osmosis.superfluid.v1beta1.RemoveSuperfluidAssetsProposal" => {
                ProposalContent::RemoveSuperfluidAssetsProposal(osmosis_prost::Message::decode(&any.value[..]).ok())
            }
            "/osmosis.superfluid.v1beta1.SetSuperfluidAssetsProposal" => {
                ProposalContent::SetSuperfluidAssetsProposal(osmosis_prost::Message::decode(&any.value[..]).ok())
            }
            "/osmosis.txfees.v1beta1.UpdateFeeTokenProposal" => {
                ProposalContent::UpdateFeeTokenProposal(osmosis_prost::Message::decode(&any.value[..]).ok())
            }
            "/osmosis.poolincentives.v1beta1.ReplacePoolIncentivesProposal" => {
                ProposalContent::ReplacePoolIncentivesProposal(osmosis_prost::Message::decode(&any.value[..]).ok())
            }
            "/osmosis.gamm.v1beta1.SetScalingFactorControllerProposal" => {
                ProposalContent::SetScalingFactorControllerProposal(osmosis_prost::Message::decode(&any.value[..]).ok())
            }
            "/cosmwasm.wasm.v1.MigrateContractProposal" => {
                ProposalContent::MigrateContractProposal(
                    any.to_msg().ok(),
                )
            }
            "/cosmwasm.wasm.v1.UpdateInstantiateConfigProposal" => {
                ProposalContent::UpdateInstantiateConfigProposal(
                    any.to_msg().ok(),
                )
            }
            "/cosmwasm.wasm.v1.SudoContractProposal" => ProposalContent::SudoContractProposal(
                any.to_msg().ok(),
            ),
            "/cosmwasm.wasm.v1.ExecuteContractProposal" => {
                ProposalContent::ExecuteContractProposal(
                    any.to_msg().ok(),
                )
            }
            "/cosmwasm.wasm.v1.UpdateAdminProposal" => ProposalContent::UpdateAdminProposal(
                any.to_msg().ok(),
            ),
            "/cosmwasm.wasm.v1.ClearAdminProposal" => ProposalContent::ClearAdminProposal(
                any.to_msg().ok(),
            ),
            "/cosmwasm.wasm.v1.PinCodesProposal" => ProposalContent::PinCodesProposal(
                any.to_msg().ok(),
            ),
            "/cosmwasm.wasm.v1.UnpinCodesProposal" => ProposalContent::UnpinCodesProposal(
                any.to_msg().ok(),
            ),
            &_ => ProposalContent::UnknownProposalType(a),
        }
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
        self.messages_as_proposal_content().iter().map(|msg| {
            match msg {
                ProposalContent::MsgExec(Some(p)) => {
                    p.grantee.to_owned()
                }
                ProposalContent::MsgExecuteContract(Some(p)) => {
                    p.contract.to_owned()
                }
                ProposalContent::MsgUpdateInstantiateConfig(Some(p)) => {
                    p.sender.to_owned()
                }
                ProposalContent::MsgSoftwareUpgrade(Some(p)) => {
                    p.authority.to_owned()
                }
                ProposalContent::MsgInstantiateContract(Some(p)) => {
                    p.label.to_owned()
                }
                ProposalContent::TextProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::CommunityPoolSpendProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::ParameterChangeProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::SoftwareUpgradeProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::ClientUpdateProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::UpdatePoolIncentivesProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::StoreCodeProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::RemoveSuperfluidAssetsProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::InstantiateContractProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::ReplacePoolIncentivesProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::SetSuperfluidAssetsProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::UpdateFeeTokenProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::MigrateContractProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::UpdateInstantiateConfigProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::SudoContractProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::ExecuteContractProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::UpdateAdminProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::ClearAdminProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::UnpinCodesProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::SetScalingFactorControllerProposal(Some(p)) => {
                    p.description.to_owned()
                }
                ProposalContent::UnknownProposalType(type_url) =>
                    format!("Error: UnknownProposalTypeError: ProposalContent can not be decoded for unknown ProposalType.\n\nType URL:\n{}", type_url)
                ,
                proposal_content =>
                    format!("Error: ProposalContentDecodeError: Failed to decode ProposalContent: {}",proposal_content)
                ,
            }.replace("\\n","\n")
        }).collect::<Vec<String>>().join("\n")
    }
    pub fn get_title(&self) -> String {
        self.messages_as_proposal_content().iter().map(|msg| {
            match msg {
                ProposalContent::MsgExec(Some(p)) => {
                    p.grantee.to_owned()
                }
                ProposalContent::MsgExecuteContract(Some(p)) => {
                    p.contract.to_owned()
                }
                ProposalContent::MsgUpdateInstantiateConfig(Some(p)) => {
                    p.sender.to_owned()
                }
                ProposalContent::MsgSoftwareUpgrade(Some(p)) => {
                    p.authority.to_owned()
                }
                ProposalContent::MsgInstantiateContract(Some(p)) => {
                    p.label.to_owned()
                }
                ProposalContent::TextProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::CommunityPoolSpendProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::ParameterChangeProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::SoftwareUpgradeProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::ClientUpdateProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::UpdatePoolIncentivesProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::StoreCodeProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::RemoveSuperfluidAssetsProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::InstantiateContractProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::ReplacePoolIncentivesProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::SetSuperfluidAssetsProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::UpdateFeeTokenProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::MigrateContractProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::UpdateInstantiateConfigProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::SudoContractProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::ExecuteContractProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::UpdateAdminProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::ClearAdminProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::UnpinCodesProposal(Some(p)) => p.title.to_owned(),
                ProposalContent::SetScalingFactorControllerProposal(Some(p)) => {
                    p.title.to_owned()
                }
                ProposalContent::UnknownProposalType(_type_url) => {
                    "UnknownProposalTypeError".to_string()
                }
                _ => "ProposalContentDecodeError".to_string(),
            }
        }).collect::<Vec<String>>().join("\n")
    }

    pub fn preview_msg(&self, fraud_classification: Option<f64>) -> String {
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

        if let Some(prediction) = fraud_classification {
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
