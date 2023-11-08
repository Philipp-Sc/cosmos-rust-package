
use std::string::ToString;

use strum_macros;
use strum_macros::EnumIter;
use serde::Deserialize;
use serde::Serialize;

pub trait ContentExt {
    fn from_any(any: &cosmos_sdk_proto::Any) -> Self;
    fn get_title(&self) -> Option<String>;
    fn get_description(&self) -> Option<String>;
}

type MsgExec = cosmos_sdk_proto::cosmos::authz::v1beta1::MsgExec;

impl ContentExt for Option<MsgExec> {
    fn from_any(any: &cosmos_sdk_proto::Any) -> Self {
        any.to_msg().ok()
    }
    fn get_title(&self) -> Option<String> {
        self.as_ref().map(|p| format!("Execute Messages for {}", p.grantee))
    }
    fn get_description(&self) -> Option<String> {
        self.as_ref().map(|p| {
                let mut description = String::new();
                description.push_str("This proposal requests the execution of the following messages:\n");
                for (index, msg) in p.msgs.iter().enumerate() {
                    description.push_str(&format!("{}. {}\n", index + 1, msg.type_url));
                }
                description
            })
    }

}

type MsgCommunityPoolSpend = injective_std::types::cosmos::distribution::v1beta1::MsgCommunityPoolSpend;

impl ContentExt for Option<MsgCommunityPoolSpend>{
    fn from_any(any: &cosmos_sdk_proto::Any) -> Self {
        osmosis_prost::Message::decode(&any.value[..]).ok()
    }
    fn get_title(&self) -> Option<String> {
        self.as_ref().map(|p|  format!("Community Pool Spend Proposal"))
    }
    fn get_description(&self) -> Option<String> {
        self.as_ref().map(|p| {
            format!(
                "This proposal requests a community pool spend of:\n- {}\n\nto the recipient: {}",
                p.amount.iter().map(|coin| format!("{} {}",coin.amount, coin.denom)).collect::<Vec<String>>().join("\n- "),
                p.recipient
            )
        })
    }
}

type MsgExecuteContract = cosmos_sdk_proto::cosmwasm::wasm::v1::MsgExecuteContract;

impl ContentExt for Option<MsgExecuteContract>{
    fn from_any(any: &cosmos_sdk_proto::Any) -> Self {
        any.to_msg().ok()
    }
    fn get_title(&self) -> Option<String> {
        self.as_ref().map(|p|  format!("Execute Smart Contract by {}", p.sender))
    }
    fn get_description(&self) -> Option<String> {
        self.as_ref().map(|p| {
            let formatted_funds = p
                .funds
                .iter()
                .map(|coin| format!("{} {}", coin.amount, coin.denom))
                .collect::<Vec<String>>()
                .join("\n- ");

            format!(
                "This proposal submits a message to a smart contract located at address: {}\n\
             Sender: {}\n\
             Msg:\n{}\n\
             Funds:\n- {}",
                p.contract, p.sender, String::from_utf8_lossy(&p.msg), formatted_funds
            )
        })
    }
}

type MsgUpdateInstantiateConfig = osmosis_std::types::cosmwasm::wasm::v1::MsgUpdateInstantiateConfig;

impl ContentExt for Option<MsgUpdateInstantiateConfig>{
    fn from_any(any: &cosmos_sdk_proto::Any) -> Self {
        osmosis_prost::Message::decode(&any.value[..]).ok()
    }
    fn get_title(&self) -> Option<String> {
        self.as_ref().map(|p|  format!("Update Instantiate Configuration for Code ID: {}", p.code_id))
    }
    fn get_description(&self) -> Option<String> {
        self.as_ref().map(|p| {
            let permission_description = match &p.new_instantiate_permission {
                Some(permission) => {
                    format!(
                        "New Permission:\n- Permission: {}\n- Addresses: \n- {}",
                        permission.permission,
                        permission.addresses.join("\n- ")
                    )
                },
                None => "No new permission specified".to_string(),
            };

            format!(
                "This proposal updates the instantiate configuration for Code ID: {}\n\
             Sender: {}\n\
             {}\n",
                p.code_id, p.sender, permission_description
            )
        })
    }
}

type MsgSoftwareUpgrade = cosmos_sdk_proto::cosmos::upgrade::v1beta1::MsgSoftwareUpgrade;

impl ContentExt for Option<MsgSoftwareUpgrade>{
    fn from_any(any: &cosmos_sdk_proto::Any) -> Self {
        cosmos_sdk_proto::traits::Message::decode(&any.value[..]).ok()
    }
    fn get_title(&self) -> Option<String> {
        self.as_ref().map(|p| format!("Software Upgrade Proposal"))
    }
    fn get_description(&self) -> Option<String> {
        self.as_ref().map(|p| {
            let plan_description = match &p.plan {
                Some(plan) => {
                    format!(
                        "Upgrade Plan:\n- Name: {}\n- Height: {}\n- Info: {}",
                        plan.name, plan.height, plan.info
                    )
                },
                None => "No upgrade plan specified".to_string(),
            };

            format!(
                "This proposal requests a software upgrade.\n\
             Authority: {}\n\
             {}\n",
                p.authority, plan_description
            )
        })
    }
}

type MsgInstantiateContract = cosmos_sdk_proto::cosmwasm::wasm::v1::MsgInstantiateContract;

impl ContentExt for Option<MsgInstantiateContract>{
    fn from_any(any: &cosmos_sdk_proto::Any) -> Self {
        cosmos_sdk_proto::traits::Message::decode(&any.value[..]).ok()
    }
    fn get_title(&self) -> Option<String> {
        self.as_ref().map(|p| format!("Instantiate Smart Contract"))
    }
    fn get_description(&self) -> Option<String> {
        self.as_ref().map(|p| {
            let formatted_funds = p
                .funds
                .iter()
                .map(|coin| format!("{} {}", coin.amount, coin.denom))
                .collect::<Vec<String>>().join("\n- ");

            format!(
                "This proposal instantiates a new smart contract with the following details:\n\
             Sender: {}\n\
             Admin: {}\n\
             Code ID: {}\n\
             Label: {}\n\
             Msg: {}\n\
             Funds:\n- {}",
                p.sender,
                p.admin,
                p.code_id,
                p.label,
                String::from_utf8_lossy(&p.msg),
                formatted_funds
            )
        })
    }
}

type SetScalingFactorControllerProposal = osmosis_std::types::osmosis::gamm::v1beta1::SetScalingFactorControllerProposal;

impl ContentExt for Option<SetScalingFactorControllerProposal>{
    fn from_any(any: &cosmos_sdk_proto::Any) -> Self {
        osmosis_prost::Message::decode(&any.value[..]).ok()
    }
    fn get_title(&self) -> Option<String> {
        self.as_ref().map(|p|  p.title.clone())
    }
    fn get_description(&self) -> Option<String> {
        self.as_ref().map(|p| {
            format!("{}\n\
             Pool ID: {}\n\
             Controller Address: {}",
                p.description, p.pool_id, p.controller_address
            )
        })
    }
}

type PinCodesProposal = cosmos_sdk_proto::cosmwasm::wasm::v1::PinCodesProposal;

impl ContentExt for Option<PinCodesProposal>{
    fn from_any(any: &cosmos_sdk_proto::Any) -> Self {
        any.to_msg().ok()
    }
    fn get_title(&self) -> Option<String> {
        self.as_ref().map(|p|  p.title.clone())
    }
    fn get_description(&self) -> Option<String> {
        self.as_ref().map(|p| {
            let code_ids_description = p.code_ids.iter().map(|id| id.to_string()).collect::<Vec<String>>().join(", ");
            format!("{}\n\nCodeIDs: {}",
                p.description, code_ids_description
            )
        })
    }
}

#[derive(strum_macros::Display, Debug, Clone, PartialEq)]
pub enum ProposalContent {
    MsgExec(Option<MsgExec>),
    //MsgUpdateParams(Option<injective_std::types::cosmos::mint::v1beta1::MsgUpdateParams>),
    MsgCommunityPoolSpend(Option<MsgCommunityPoolSpend>),
    MsgExecuteContract(Option<MsgExecuteContract>),
    MsgUpdateInstantiateConfig(Option<MsgUpdateInstantiateConfig>),
    MsgSoftwareUpgrade(Option<MsgSoftwareUpgrade>),
    MsgInstantiateContract(Option<MsgInstantiateContract>),
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
        Option<SetScalingFactorControllerProposal>,
    ),
    MigrateContractProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::MigrateContractProposal>),
    UpdateInstantiateConfigProposal(
        Option<cosmos_sdk_proto::cosmwasm::wasm::v1::UpdateInstantiateConfigProposal>,
    ),
    SudoContractProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::SudoContractProposal>),
    ExecuteContractProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::ExecuteContractProposal>),
    UpdateAdminProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::UpdateAdminProposal>),
    ClearAdminProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::ClearAdminProposal>),
    PinCodesProposal(Option<PinCodesProposal>),
    UnpinCodesProposal(Option<cosmos_sdk_proto::cosmwasm::wasm::v1::UnpinCodesProposal>),
    UnknownProposalType(String),
}

impl ProposalContent{

    pub fn new(any: &cosmos_sdk_proto::Any) -> ProposalContent {
        let a = any.type_url.to_string();
        match a.as_ref() {
            "/cosmos.authz.v1beta1.MsgExec" => ProposalContent::MsgExec(
                ContentExt::from_any(any),
            ),
            /*
            "/cosmos.mint.v1beta1.MsgUpdateParams" => ProposalContent::MsgUpdateParams(
                osmosis_prost::Message::decode(&any.value[..]).ok(),
            ),*/
            "/cosmos.distribution.v1beta1.MsgCommunityPoolSpend"=> ProposalContent::MsgCommunityPoolSpend(
                ContentExt::from_any(any),
            ),
            "/cosmwasm.wasm.v1.MsgExecuteContract"=> ProposalContent::MsgExecuteContract(
                ContentExt::from_any(any),
            ),
            "/cosmwasm.wasm.v1.MsgUpdateInstantiateConfig" => ProposalContent::MsgUpdateInstantiateConfig(
                ContentExt::from_any(any),
            ),
            "/cosmos.upgrade.v1beta1.MsgSoftwareUpgrade" => ProposalContent::MsgSoftwareUpgrade(
                ContentExt::from_any(any),
            ),
            "/cosmwasm.wasm.v1.MsgInstantiateContract" => ProposalContent::MsgInstantiateContract(
                ContentExt::from_any(any),
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
                ProposalContent::SetScalingFactorControllerProposal(
                    ContentExt::from_any(any),
                )
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
                ContentExt::from_any(any),
            ),
            "/cosmwasm.wasm.v1.UnpinCodesProposal" => ProposalContent::UnpinCodesProposal(
                any.to_msg().ok(),
            ),
            &_ => ProposalContent::UnknownProposalType(a),
        }
    }

    pub fn get_description(&self) -> String {
        match &self {
            ProposalContent::TextProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::CommunityPoolSpendProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::ParameterChangeProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::SoftwareUpgradeProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::ClientUpdateProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::UpdatePoolIncentivesProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::StoreCodeProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::RemoveSuperfluidAssetsProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::InstantiateContractProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::ReplacePoolIncentivesProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::SetSuperfluidAssetsProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::UpdateFeeTokenProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::MigrateContractProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::UpdateInstantiateConfigProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::SudoContractProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::ExecuteContractProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::UpdateAdminProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::ClearAdminProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::UnpinCodesProposal(p) => {
                p.clone().map(|x| x.description.to_owned())
            }
            ProposalContent::MsgExec(p) => {
                p.get_description()
            }
            ProposalContent::MsgCommunityPoolSpend(p) => {
                p.get_description()
            }
            ProposalContent::MsgExecuteContract(p) => {
                p.get_description()
            }
            ProposalContent::MsgUpdateInstantiateConfig(p) => {
                p.get_description()
            }
            ProposalContent::MsgSoftwareUpgrade(p) => {
                p.get_description()
            }
            ProposalContent::MsgInstantiateContract(p) => {
                p.get_description()
            }
            ProposalContent::SetScalingFactorControllerProposal(p) => {
                p.get_description()
            }
            ProposalContent::PinCodesProposal(p) => {
                p.get_description()
            }
            ProposalContent::UnknownProposalType(type_url) =>
                Some(format!("Error: UnknownProposalTypeError: ProposalContent can not be decoded for unknown ProposalType.\n\nType URL:\n{}", type_url))
            ,
        }.unwrap_or(
                format!("Error: DecodeError: ProposalContent could not be decoded for ProposalType.")
            ).replace("\\n", "\n")
    }
    pub fn get_title(&self) -> String {
        match &self {
            ProposalContent::TextProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::CommunityPoolSpendProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::ParameterChangeProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::SoftwareUpgradeProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::ClientUpdateProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::UpdatePoolIncentivesProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::StoreCodeProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::RemoveSuperfluidAssetsProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::InstantiateContractProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::ReplacePoolIncentivesProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::SetSuperfluidAssetsProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::UpdateFeeTokenProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::MigrateContractProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::UpdateInstantiateConfigProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::SudoContractProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::ExecuteContractProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::UpdateAdminProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::ClearAdminProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::UnpinCodesProposal(p) => p.clone().map(|x| x.title.to_owned()),
            ProposalContent::MsgExec(p) => {
                p.get_title()
            }
            ProposalContent::MsgCommunityPoolSpend(p) => {
                p.get_title()
            }
            ProposalContent::MsgExecuteContract(p) => {
                p.get_title()
            }
            ProposalContent::MsgUpdateInstantiateConfig(p) => {
                p.get_title()
            }
            ProposalContent::MsgSoftwareUpgrade(p) => {
                p.get_title()
            }
            ProposalContent::MsgInstantiateContract(p) => {
                p.get_title()
            }
            ProposalContent::SetScalingFactorControllerProposal(p) => {
                p.get_title()
            }
            ProposalContent::PinCodesProposal(p) => {
                p.get_title()
            }
            ProposalContent::UnknownProposalType(_type_url) => {
                Some("UnknownProposalTypeError".to_string())
            }
        }.unwrap_or("DecodeError".to_string())
    }
}


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

#[derive(Deserialize, Serialize, strum_macros::Display, Debug, Clone, PartialEq, EnumIter)]
pub enum ProposalTime {
    SubmitTime,
    DepositEndTime,
    VotingStartTime,
    VotingEndTime,
    LatestTime,
}