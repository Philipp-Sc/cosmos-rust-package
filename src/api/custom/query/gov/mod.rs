use crate::api::core::cosmos::channels::{get_supported_blockchains, SupportedBlockchain};
use crate::api::core::*;
use prost_types::Timestamp;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use std::string::ToString;
use strum_macros;
use strum_macros::EnumIter;

use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use prost::EncodeError;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;
use linkify::LinkFinder;

use prost::Message;

lazy_static! {
    pub static ref LINK_FINDER: LinkFinder = get_link_finder();
    pub static ref LINK_MARKDOWN_REGEX: regex::Regex =
        Regex::new(r#"\[([^\]]+)\]\(([^\)"]+)\)"#).unwrap();
}

pub fn get_link_finder() -> LinkFinder {
    let mut finder = LinkFinder::new();
    finder.url_must_have_scheme(false);
    finder
}

#[derive(Deserialize, Serialize, strum_macros::Display, Debug, Clone, PartialEq, EnumIter)]
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

#[derive(strum_macros::Display, Debug, Clone)]
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalExt {
    pub blockchain_name: String,
    pub status: ProposalStatus,
    pub proposal_bytes: Vec<u8>, //Result<Vec<u8>,EncodeError>

    #[serde(skip)]
    proposal: Option<cosmos_sdk_proto::cosmos::gov::v1beta1::Proposal>,
    #[serde(skip)]
    content: Option<ProposalContent>,
    // if performance is an issue for proposal(), content(),.. (e.g to many conversions add private field with serde skip)
}

impl Hash for ProposalExt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.blockchain_name.hash(state);
        self.proposal_bytes.hash(state);
        self.status.to_string().hash(state);
    }
}
impl ProposalExt {
    pub fn new(
        blockchain: &SupportedBlockchain,
        proposal_status: &ProposalStatus,
        proposal: cosmos_sdk_proto::cosmos::gov::v1beta1::Proposal,
    ) -> ProposalExt {
        ProposalExt {
            blockchain_name: blockchain.name.to_string(),
            status: proposal_status.clone(),
            proposal_bytes: cosmos_sdk_proto::traits::MessageExt::to_bytes(&proposal)
                .unwrap_or(Vec::new()),
            proposal: Some(proposal),
            content: None,
        }
    }
    pub fn to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        &self.hash(&mut s);
        s.finish()
    }
    pub fn proposal(&mut self) -> Option<cosmos_sdk_proto::cosmos::gov::v1beta1::Proposal> {
        if self.proposal.is_none() {
            self.proposal = prost::Message::decode(&self.proposal_bytes[..]).ok();
        }
        self.proposal.clone()
    }

    pub fn content(&mut self) -> Option<ProposalContent> {
        if self.content.is_none() {
            if let Some(p) = self.proposal() {
                if let Some(p) = p.content {
                    let a = p.type_url.to_string();
                    self.content = Some(match a.as_ref() {
                        "/cosmos.gov.v1beta1.TextProposal" => ProposalContent::TextProposal(
                            cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                        ),
                        "/cosmos.distribution.v1beta1.CommunityPoolSpendProposal" => {
                            ProposalContent::CommunityPoolSpendProposal(
                                cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                            )
                        }
                        "/cosmos.params.v1beta1.ParameterChangeProposal" => {
                            ProposalContent::ParameterChangeProposal(
                                cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                            )
                        }
                        "/cosmos.upgrade.v1beta1.SoftwareUpgradeProposal" => {
                            ProposalContent::SoftwareUpgradeProposal(
                                cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                            )
                        }
                        "/ibc.core.client.v1.ClientUpdateProposal" => {
                            ProposalContent::ClientUpdateProposal(
                                cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                            )
                        }
                        "/osmosis.poolincentives.v1beta1.UpdatePoolIncentivesProposal" => {
                            let encoded_any = &p.encode_to_vec();
                            let decoded = osmosis_std::types::osmosis::poolincentives::v1beta1::UpdatePoolIncentivesProposal::decode(&encoded_any[..]).ok();
                            ProposalContent::UpdatePoolIncentivesProposal(decoded)
                        }
                        "/cosmwasm.wasm.v1.StoreCodeProposal" => {
                            ProposalContent::StoreCodeProposal(
                                cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                            )
                        }
                        "/cosmwasm.wasm.v1.InstantiateContractProposal" => {
                            ProposalContent::InstantiateContractProposal(
                                cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                            )
                        }
                        "/osmosis.superfluid.v1beta1.RemoveSuperfluidAssetsProposal" => {
                            let encoded_any = &p.encode_to_vec();
                            let decoded = osmosis_std::types::osmosis::superfluid::v1beta1::RemoveSuperfluidAssetsProposal::decode(&encoded_any[..]).ok();
                            ProposalContent::RemoveSuperfluidAssetsProposal(decoded)
                        }
                        "/osmosis.superfluid.v1beta1.SetSuperfluidAssetsProposal" => {
                            let encoded_any = &p.encode_to_vec();
                            let decoded = osmosis_std::types::osmosis::superfluid::v1beta1::SetSuperfluidAssetsProposal::decode(&encoded_any[..]).ok();
                            ProposalContent::SetSuperfluidAssetsProposal(decoded)
                        }
                        "/osmosis.txfees.v1beta1.UpdateFeeTokenProposal" => {
                            let encoded_any = &p.encode_to_vec();
                            let decoded = osmosis_std::types::osmosis::txfees::v1beta1::UpdateFeeTokenProposal::decode(&encoded_any[..]).ok();
                            ProposalContent::UpdateFeeTokenProposal(decoded)
                        }
                        "/osmosis.poolincentives.v1beta1.ReplacePoolIncentivesProposal" => {
                            let encoded_any = &p.encode_to_vec();
                            let decoded = osmosis_std::types::osmosis::poolincentives::v1beta1::ReplacePoolIncentivesProposal::decode(&encoded_any[..]).ok();
                            ProposalContent::ReplacePoolIncentivesProposal(decoded)
                        }
                        "/cosmwasm.wasm.v1.MigrateContractProposal" => {
                            ProposalContent::MigrateContractProposal(
                                cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                            )
                        }
                        "/cosmwasm.wasm.v1.UpdateInstantiateConfigProposal" => {
                            ProposalContent::UpdateInstantiateConfigProposal(
                                cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                            )
                        }
                        "/cosmwasm.wasm.v1.SudoContractProposal" => {
                            ProposalContent::SudoContractProposal(
                                cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                            )
                        }
                        "/cosmwasm.wasm.v1.ExecuteContractProposal" => {
                            ProposalContent::ExecuteContractProposal(
                                cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                            )
                        }
                        "/cosmwasm.wasm.v1.UpdateAdminProposal" => {
                            ProposalContent::UpdateAdminProposal(
                                cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                            )
                        }
                        "/cosmwasm.wasm.v1.ClearAdminProposal" => {
                            ProposalContent::ClearAdminProposal(
                                cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                            )
                        }
                        "/cosmwasm.wasm.v1.PinCodesProposal" => ProposalContent::PinCodesProposal(
                            cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                        ),
                        "/cosmwasm.wasm.v1.UnpinCodesProposal" => {
                            ProposalContent::UnpinCodesProposal(
                                cosmos_sdk_proto::traits::MessageExt::from_any(&p).ok(),
                            )
                        }

                        &_ => ProposalContent::UnknownProposalType(a),
                    });
                }
            }
        }
        self.content.clone()
    }
    pub fn time(&mut self, time: &ProposalTime) -> Option<Timestamp> {
        self.proposal()
            .map(|x| match time {
                &ProposalTime::SubmitTime => x.submit_time.clone(),
                &ProposalTime::DepositEndTime => x.deposit_end_time.clone(),
                &ProposalTime::VotingEndTime => x.voting_end_time.clone(),
                &ProposalTime::VotingStartTime => x.voting_start_time.clone(),
                &ProposalTime::LatestTime => self.latest_time(&x),
            })
            .flatten()
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

    pub fn get_title_and_description(&mut self) -> (String, String) {
        match &self.content() {
            Some(ProposalContent::TextProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::CommunityPoolSpendProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::ParameterChangeProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::SoftwareUpgradeProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::ClientUpdateProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::UpdatePoolIncentivesProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::StoreCodeProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::RemoveSuperfluidAssetsProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::InstantiateContractProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::ReplacePoolIncentivesProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::SetSuperfluidAssetsProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::UpdateFeeTokenProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::MigrateContractProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::UpdateInstantiateConfigProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::SudoContractProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::ExecuteContractProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::UpdateAdminProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::ClearAdminProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::UnpinCodesProposal(Some(p))) => {
                (p.title.to_owned(), p.description.to_owned())
            }
            Some(ProposalContent::UnknownProposalType(type_url)) => (
                "UnknownTitle".to_string(),
                format!("UnknownDescription\n\nType URL:\n{}", type_url),
            ),
            Some(_) => (
                "ContentDecodeErrorTitle".to_string(),
                "ContentDecodeErrorDescription".to_string(),
            ),
            None => (
                "ProposalDecodeErrorDescription".to_string(),
                "ProposalDecodeErrorDescription".to_string(),
            ),
        }
    }

    pub fn timestamp_to_string(item: Option<Timestamp>) -> String {
        match item.as_ref() {
            Some(time) => {
                if time.seconds > 0 {
                    DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp(time.seconds, time.nanos as u32),
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

    pub fn get_voting_start_and_end(&mut self) -> (String, String) {
        if let Some(proposal) = self.proposal() {
            (
                format!("{}", Self::timestamp_to_string(proposal.voting_start_time)),
                format!("{}", Self::timestamp_to_string(proposal.voting_end_time)),
            )
        } else {
            ("".to_string(), "".to_string())
        }
    }

    pub fn generate_html(&mut self, fraud_risk: Option<f64>) -> String {
        let blockchain_name = self.blockchain_name.clone();
        let mut proposal_id = self
            .proposal()
            .map(|x| x.proposal_id.to_string())
            .unwrap_or("".to_string());
        let (title, mut description) = self.get_title_and_description();

        let mut gov_prop_link = get_supported_blockchains()
            .get(&self.blockchain_name.to_lowercase())
            .unwrap()
            .governance_proposals_link
            .as_str()
            .to_string();
        gov_prop_link.push_str(&proposal_id);

        let proposal_type = self
            .content()
            .map(|x| x.to_string())
            .unwrap_or("Proposal".to_string());

        let proposal_status = &self.status.to_icon();

        let proposal_state = self.proposal_state();

        let css_style = r#"body {
                  font-family: Arial, sans-serif;
                  margin: 0;
                  padding: 0;
                  background-color: #1d2021;
                  color: #d8dee9;
                }
                .container {
                  width: 80%;
                  margin: 50px auto;
                  padding: 30px;
                  background-color: #2e3440;
                  border-radius: 5px;
                }
                .title {
                  text-align: center;
                  font-size: 36px;
                  margin-top: 0;
                  background-color: #3b4252;
                  padding: 20px;
                  border-radius: 5px 5px 0 0;
                  color: #88C0D0;

                }
                .description {
                  margin-top: 30px;
                  background-color: #3b4252;
                  padding: 20px;
                  border-radius: 0 0 5px 5px;
                }
                p {
                  font-size: 18px;
                  line-height: 1.5;
                  margin-top: 20px;
                }
                span {
                  font-size: 18px;
                  line-height: 1.5;
                  margin-top: 20px;
                }
                .button {
                  display: inline-block;
                  padding: 10px 20px;
                  font-size: 18px;
                  margin-top: 30px;
                  border-radius: 5px;
                  transition: background-color 0.3s ease;
                  border: none;
                  background-color: #5c616c;
                  color: #d8dee9;
                }
                .button:hover {
                  background-color: #373b41;
                  cursor: pointer;
                }
                .alert {
                  background-color: #dc3545;
                  padding: 10px;
                  text-align: center;
                  margin-top: 20px;
                  color: #fff;
                  border-radius: 5px;
                  font-size: 16px;
                  font-weight: bold;
                }
                .warning {
                  background-color: #ffc107;
                  padding: 10px;
                  text-align: center;
                  margin-top: 20px;
                  color: #212529;
                  border-radius: 5px;
                  font-size: 16px;
                  font-weight: bold;
                }
                .description {
                  margin-top: 30px;
                }
                .description span {
                  max-height: 300px;
                  overflow: hidden;
                }
                .show-more {
                  margin-top: 10px;
                  text-align: center;
                }
                .show-more button {
                  background-color: transparent;
                  border: none;
                  color: #5c616c;
                  cursor: pointer;
                  text-decoration: underline;
                  font-size: 16px;
                }
                .show-more button:hover {
                  color: #373b41;
                }
                footer {
                  text-align: center;
                  margin-top: 50px;
                  font-size: 16px;
                  color: #6c8d9b;
                  background-color: #1c2331;
                  padding: 10px;
                }
                footer a {
                  color: #8ec07c;
                  text-decoration: none;
                }
                footer a:hover {
                  color: #ebdbb2;
                }
                .button-container {
                  text-align: center;
                  margin-top: 50px;
            }
                .button-container button {
                  display: inline-block;
                  padding: 10px 20px;
                  font-size: 18px;
                  margin: 0 10px;
                  border-radius: 5px;
                  transition: background-color 0.3s ease;
                  border: none;
                  background-color: #5c616c;
                  color: #d8dee9;
                }
                .button-container button:hover {
                  background-color: #373b41;
                  cursor: pointer;
                }

                #status-btn {
                  padding: 10px;
                  background-color: #2E3440;
                  color: white;
                  border: none;
                  cursor: pointer;
                }

                #status-btn:hover {
                  background-color: #3B4252;
                }

                #status-btn:active {
                  background-color: #4C566A;
                }

                #status-text {
                  display: none;
                  padding: 10px;
                  background-color: #4C566A;
                  color: white;
                  border: 1px solid #D8DEE9;
                }
        "#;

        format!(
            "<!DOCTYPE html>
        <html>
        <head>
          <meta charset=\"UTF-8\">
          <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
          <title>#{}</title>
          <style>
               {}
          </style>
        </head>
       <div class=\"container\">
    <h3 class=\"title\" >{}</h3>

    <h2>{}</h2>
    <h2>#{} - {}</h2>
    <h3>{}</h3>
    <div class=\"description\">
      <span style=\"white-space: pre-wrap\">{}</span>
      <div class=\"show-more\">
        <button id=\"show-more-btn\">Show More</button>
      </div>
    </div>
    <div id=\"fraud-alert\"></div>
    <div class=\"button-container\">
  <button id=\"status-btn\" onclick=\"toggleStatus()\">📊 Status</button>
  <button id=\"status-btn\" onclick=\"toggleStatus()\">📝 Start Briefing</button>
  <button id=\"status-btn\" onclick=\"window.open('{}', '_blank')\">Open in 🛰️/🅺</button>

  <div id=\"status-text\" style=\"display: none;\">{}</div>
</div>

  </div>
  <script>
    const fraudRisk = {}; // replace with actual fraud risk value
    {}
</script>
<footer>
  This website was created by <a href=\"\"> CosmosRustBot</a>. All rights reserved.
</footer>

  </body>
        </html>",
            proposal_id,
            css_style,
            blockchain_name,
            proposal_type,
            proposal_id,
            proposal_status,
            title,
            description,
            gov_prop_link,
            proposal_state,
            fraud_risk.unwrap_or(1.0).to_string(),
            r#"
            function toggleStatus() {
      var statusText = document.getElementById("status-text");
      if (statusText.style.display === "none") {
        statusText.style.display = "block";
      } else {
        statusText.style.display = "none";
      }
    }

            if (fraudRisk > 0.7) {
  const alertDiv = document.createElement('div');
  alertDiv.classList.add('alert');
alertDiv.innerText = '🚨  Be cautious. Be careful, avoid suspicious links/URLs, and remember, if it seems too good to be true, it probably is. 🚨';
document.getElementById('fraud-alert').appendChild(alertDiv);
} else if (fraudRisk > 0.4) {
const warningDiv = document.createElement('div');
warningDiv.classList.add('warning');
warningDiv.innerText = '⚠  Stay safe. Be cautious of links/URLs. ⚠';
document.getElementById('fraud-alert').appendChild(warningDiv);
}
  const showMoreBtn = document.getElementById('show-more-btn');
  const description = document.querySelector('.description span');
  showMoreBtn.addEventListener('click', () => {
    description.style.maxHeight = 'none';
    showMoreBtn.style.display = 'none';
  });"#
        )
    }

    pub fn spam_likelihood(&mut self) -> Option<f64> {
        if let Some(proposal) = self.proposal() {
            match proposal.final_tally_result.as_ref() {
                None => {
                    return None;
                }
                Some(tally) => {
                    if !(tally.yes == "0"
                        && tally.abstain == "0"
                        && tally.no == "0"
                        && tally.no_with_veto == "0")
                    {
                        let abstain_num = tally.abstain.parse::<f64>().unwrap();
                        let yes_num = tally.yes.parse::<f64>().unwrap();
                        let no_num = tally.no.parse::<f64>().unwrap();
                        let no_with_veto_num = tally.no_with_veto.parse::<f64>().unwrap();
                        let total = (abstain_num + yes_num + no_num + no_with_veto_num) as f64;
                        let abstain_num = abstain_num / total;
                        let yes_num = yes_num / total;
                        let no_num = no_num / total;
                        let no_with_veto_num = no_with_veto_num / total;
                        return Some(
                            ((2.0 * no_with_veto_num) + no_num - yes_num - (2.0 * abstain_num))
                                / 2.0,
                        );
                    } else {
                        return None;
                    }
                }
            };
        }
        return None;
    }

    pub fn get_tally_result(&mut self) -> String {
        if let Some(proposal) = self.proposal() {
            match proposal.final_tally_result.as_ref() {
                None => {}
                Some(tally) => {
                    if !(tally.yes == "0"
                        && tally.abstain == "0"
                        && tally.no == "0"
                        && tally.no_with_veto == "0")
                    {
                        let abstain_num = tally.abstain.parse::<f64>().unwrap();
                        let yes_num = tally.yes.parse::<f64>().unwrap();
                        let no_num = tally.no.parse::<f64>().unwrap();
                        let no_with_veto_num = tally.no_with_veto.parse::<f64>().unwrap();
                        let total = (abstain_num + yes_num + no_num + no_with_veto_num) as f64;
                        let abstain_num = f64::trunc(abstain_num / total * 100.0 * 100.0) / 100.0;
                        let yes_num = f64::trunc(yes_num / total * 100.0 * 100.0) / 100.0;
                        let no_num = f64::trunc(no_num / total * 100.0 * 100.0) / 100.0;
                        let no_with_veto_num =
                            f64::trunc(no_with_veto_num / total * 100.0 * 100.0) / 100.0;
                        return format!(
                            r#"👍 {}%, 👎 {}%, 🕊️ {}%, ❌ {}%"#,
                            yes_num, no_num, abstain_num, no_with_veto_num
                        );
                    }
                }
            };
        }
        return "".to_string();
    }

    pub fn id_title_and_description_to_hash(&mut self) -> u64 {
        let mut s = DefaultHasher::new();
        let proposal_id = self
            .proposal()
            .map(|x| x.proposal_id.to_string())
            .unwrap_or("??".to_string());
        proposal_id.hash(&mut s);
        &self.get_title_and_description().hash(&mut s);
        s.finish()
    }

    pub fn governance_proposal_link(&mut self) -> String {
        let mut gov_prop_link = get_supported_blockchains()
            .get(&self.blockchain_name.to_lowercase())
            .unwrap()
            .governance_proposals_link
            .as_str()
            .to_string();
        let mut proposal_id = self
            .proposal()
            .map(|x| x.proposal_id.to_string())
            .unwrap_or("??".to_string());
        gov_prop_link.push_str(&proposal_id);
        gov_prop_link
    }

    pub fn proposal_clickbait(&mut self, fraud_classification: Option<f64>) -> String {
        let (title, _) = self.get_title_and_description();

        let mut proposal_id = self
            .proposal()
            .map(|x| x.proposal_id.to_string())
            .unwrap_or("??".to_string());

        let mut display = format!(
            "{}\n\n{}\n#{}  -  {}\n{}",
            self.blockchain_name.clone(),
            self.content()
                .map(|x| x.to_string())
                .unwrap_or("Proposal".to_string()),
            proposal_id,
            &self.status.to_icon(),
            title,
        );

        if let Some(prediction) = fraud_classification {
            let label = if prediction >= 0.70 {
                format!("🚨 ({})\nBe cautious. Check URLs, avoid suspicious links, and remember, if it seems too good to be true, it probably is.",((100.0*prediction).trunc()/100.0))
            } else if prediction > 0.50 {
                format!(
                    "⚠ ({})\nStay safe. Be cautious of links and URLs.",
                    ((100.0 * prediction).trunc() / 100.0)
                )
            } else {
                format!("🛡️ ({})", ((100.0 * prediction).trunc() / 100.0))
            };
            display = format!("{}\n\n{}", display, label);
        }
        display
    }

    pub fn proposal_content(&mut self) -> String {
        let (_, mut description) = self.get_title_and_description();
        let proposal_id = self
            .proposal()
            .map(|x| x.proposal_id.to_string())
            .unwrap_or("".to_string());

        description = LINK_MARKDOWN_REGEX
            .replace_all(description.as_str(), ";;; $1;;; $2;;;")
            .to_string();
        let mut tmp_description = description.split(";;;").collect::<Vec<&str>>();
        tmp_description.dedup();
        description = tmp_description.join("");

        for link in LINK_FINDER.links(&description.to_owned()) {
            let l = link.as_str();
            description = description.replace(l, &format!("{} ⚠️ ", l));
        }

        format!(
            "#{}  -  Content\n{}",
            proposal_id,
            description.replace("\\n", "\n"),
        )
    }

    pub fn proposal_state(&mut self) -> String {
        let (voting_start_text, voting_end_text) = self.get_voting_start_and_end();
        let mut tally_result = self.get_tally_result();
        let mut proposal_id = self
            .proposal()
            .map(|x| x.proposal_id.to_string())
            .unwrap_or("".to_string());

        let mut voting_state = "".to_string();
        if let Some(proposal) = self.proposal() {
            if &self.status == &ProposalStatus::StatusVotingPeriod {
                let mut voting_start = false;
                if let Some(time) = proposal.voting_start_time {
                    match DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp(time.seconds, time.nanos as u32),
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
                if let Some(time) = proposal.voting_end_time {
                    match DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp(time.seconds, time.nanos as u32),
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
                voting_state = format!("You can help the proposal move forward by depositing now. \nThe deposit period is open until {}",Self::timestamp_to_string(proposal.deposit_end_time))
            }
        }

        format!(
            "{}\n\n{}\n\n{}",
            &self.status.to_string(),
            voting_state,
            tally_result,
        )
    }

    pub fn proposal_details(&mut self, fraud_classification: Option<f64>) -> String {
        let (title, mut description) = self.get_title_and_description();
        let (voting_start, voting_end) = self.get_voting_start_and_end();
        let mut tally_result = self.get_tally_result();
        let mut gov_prop_link = get_supported_blockchains()
            .get(&self.blockchain_name.to_lowercase())
            .unwrap()
            .governance_proposals_link
            .as_str()
            .to_string();
        let mut proposal_id = self
            .proposal()
            .map(|x| x.proposal_id.to_string())
            .unwrap_or("".to_string());
        gov_prop_link.push_str(&proposal_id);

        description = LINK_MARKDOWN_REGEX
            .replace_all(description.as_str(), ";;; $1;;; $2;;;")
            .to_string();
        let mut tmp_description = description.split(";;;").collect::<Vec<&str>>();
        tmp_description.dedup();
        description = tmp_description.join("");

        for link in LINK_FINDER.links(&description.to_owned()) {
            let l = link.as_str();
            description = description.replace(l, &format!("{} ⚠️ ", l));
        }

        if let Some(prediction) = fraud_classification {
            if prediction >= 0.70 {
                return format!(
                    "{}\n{}\n#{}  -  {}\n{}\n\n{}\n\n🚨 ({})\nBe cautious. Check URLs, avoid suspicious links, and remember, if it seems too good to be true, it probably is.",
                    self.blockchain_name.clone(),
                    self.content().map(|x| x.to_string()).unwrap_or("Proposal".to_string()),
                    proposal_id,
                    &self.status.to_icon(),
                    title,
                    gov_prop_link,
                    ((100.0*prediction).trunc()/100.0)
                );
            } else if prediction > 0.50 {
                return format!(
                    "{}\n{}\n#{}  -  {}\n{}\n{}\n{}\n{}\n{}\n{}\n\n⚠ ({})\nStay safe. Be cautious of links and URLs.",
                    self.blockchain_name.clone(),
                    self.content().map(|x| x.to_string()).unwrap_or("Proposal".to_string()),
                    proposal_id,
                    &self.status.to_icon(),
                    title,
                    voting_start,
                    voting_end,
                    tally_result,
                    description,
                    gov_prop_link,
                    ((100.0*prediction).trunc()/100.0)
                );
            } else {
                return format!(
                    "{}\n{}\n#{}  -  {}\n{}\n{}\n{}\n{}\n{}\n{}\n\n🛡️ ({})",
                    self.blockchain_name.clone(),
                    self.content()
                        .map(|x| x.to_string())
                        .unwrap_or("Proposal".to_string()),
                    proposal_id,
                    &self.status.to_icon(),
                    title,
                    voting_start,
                    voting_end,
                    tally_result,
                    description,
                    gov_prop_link,
                    ((100.0 * prediction).trunc() / 100.0),
                );
            }
        }
        format!(
            "{}\n{}\n#{}  -  {}\n{}\n{}\n{}\n{}\n{}\n{}",
            self.blockchain_name.clone(),
            self.content()
                .map(|x| x.to_string())
                .unwrap_or("Proposal".to_string()),
            proposal_id,
            &self.status.to_icon(),
            title,
            voting_start,
            voting_end,
            tally_result,
            description,
            gov_prop_link
        )
    }
}

pub async fn get_proposals(
    blockchain: SupportedBlockchain,
    proposal_status: ProposalStatus,
    next_key: Option<Vec<u8>>,
) -> anyhow::Result<(Option<Vec<u8>>, Vec<ProposalExt>)> {
    let channel = blockchain.channel().await?;
    let res = cosmos::query::get_proposals(
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
        list.push(ProposalExt::new(&blockchain, &proposal_status, proposal));
    }
    //log::error!("you dropped this: {:?}",res.pagination);
    Ok((res.pagination.map(|x| x.next_key), list))
}

#[cfg(test)]
mod test {

    // cargo test -- --nocapture
    // cargo test -- --list
    // cargo test api::custom::query::gov::teset::get_proposals -- --exact --nocapture

    use crate::api::core::cosmos::channels;

    #[tokio::test]
    pub async fn get_channel() -> anyhow::Result<()> {
        println!(
            "{:?}",
            channels::get_supported_blockchains_from_chain_registry(
                "./packages/chain-registry".to_string(),
                true,
                None
            )
            .await
            .get("osmosis")
            .unwrap()
            .channel()
            .await?
        );
        Ok(())
    }

    #[tokio::test]
    pub async fn get_proposals() -> anyhow::Result<()> {
        /*
        let channel = channels::get_supported_blockchains_from_chain_registry("/home/user/Documents/cosmos-rust-bot/packages/chain-registry".to_string(),true,Some(60))
                .await.get("terra2")
                .unwrap()
                .to_owned();
        println!("{:?}",&channel);
        let res = super::get_proposals(
            channel,
            super::ProposalStatus::StatusPassed
        )
        .await?;
        println!(
            "{:?}",
            res.iter()
                .map(|x| x.content.clone())
                .collect::<Vec<super::ProposalContent>>()
        );*/
        let channel = channels::get_supported_blockchains_from_chain_registry("/home/user/Documents/cosmos-rust-bot-workspace/cosmos-rust-bot/packages/chain-registry".to_string(),true,None)
                .get("cosmoshub")
                .unwrap()
                .to_owned();
        println!("{:?}", &channel);
        let res = super::get_proposals(channel, super::ProposalStatus::StatusPassed).await?;
        println!(
            "{:?}",
            res.iter()
                .map(|x| x.content())
                .collect::<Vec<super::ProposalContent>>()
        );
        Ok(())
    }
}
