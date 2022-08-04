use crate::api::core::*;
use crate::api::core::cosmos::channels::SupportedBlockchain;

pub enum ProposalStatus {
    StatusNil,
    StatusDepositPeriod,
    StatusVotingPeriod,
    StatusPassed,
    StatusRejected,
    StatusFailed
}

#[derive(Debug, Clone)]
pub enum Proposal{
    TextProposal(cosmos_sdk_proto::cosmos::gov::v1beta1::TextProposal),
    CommunityPoolSpendProposal(cosmos_sdk_proto::cosmos::distribution::v1beta1::CommunityPoolSpendProposal),
    ParameterChangeProposal(cosmos_sdk_proto::cosmos::params::v1beta1::ParameterChangeProposal),
    SoftwareUpgradeProposal(cosmos_sdk_proto::cosmos::upgrade::v1beta1::SoftwareUpgradeProposal),
    ClientUpdateProposal(cosmos_sdk_proto::ibc::core::client::v1::ClientUpdateProposal),
    UpdatePoolIncentivesProposal(osmosis_proto::osmosis::poolincentives::v1beta1::UpdatePoolIncentivesProposal),
    UnknownProposalType,
}

#[derive(Debug, Clone)]
pub struct ProposalExt(cosmos_sdk_proto::cosmos::gov::v1beta1::Proposal);

// Implementation block, all `ProposalExt` associated functions & methods go in here
impl ProposalExt {
    fn content(&self) -> Proposal {
        let p = self.0.content.as_ref().unwrap();
        if p.type_url == "/cosmos.gov.v1beta1.TextProposal" {
            let t: cosmos_sdk_proto::cosmos::gov::v1beta1::TextProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            Proposal::TextProposal(t)
        } else if p.type_url == "/cosmos.distribution.v1beta1.CommunityPoolSpendProposal" {
            let t: cosmos_sdk_proto::cosmos::distribution::v1beta1::CommunityPoolSpendProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            Proposal::CommunityPoolSpendProposal(t)
        } else if p.type_url == "/cosmos.params.v1beta1.ParameterChangeProposal" {
            let t: cosmos_sdk_proto::cosmos::params::v1beta1::ParameterChangeProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            Proposal::ParameterChangeProposal(t)
        } else if p.type_url == "/cosmos.upgrade.v1beta1.SoftwareUpgradeProposal" {
            let t: cosmos_sdk_proto::cosmos::upgrade::v1beta1::SoftwareUpgradeProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            Proposal::SoftwareUpgradeProposal(t)
        } else if p.type_url == "/ibc.core.client.v1.ClientUpdateProposal" {
            let t: cosmos_sdk_proto::ibc::core::client::v1::ClientUpdateProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            Proposal::ClientUpdateProposal(t)
        } else if p.type_url == "/osmosis.poolincentives.v1beta1.UpdatePoolIncentivesProposal" {
            let t: osmosis_proto::osmosis::poolincentives::v1beta1::UpdatePoolIncentivesProposal = cosmos_sdk_proto::traits::MessageExt::from_any(p).unwrap();
            Proposal::UpdatePoolIncentivesProposal(t)
        } else {
            Proposal::UnknownProposalType
        }
    }
    // todo! add display function
}

pub async fn get_proposals(blockchain: SupportedBlockchain, proposal_status: ProposalStatus) -> anyhow::Result<Vec<ProposalExt>> {
    /*
    StatusNil           ProposalStatus = 0x00
    StatusDepositPeriod ProposalStatus = 0x01  // Proposal is submitted. Participants can deposit on it but not vote
    StatusVotingPeriod  ProposalStatus = 0x02  // MinDeposit is reached, participants can vote
    StatusPassed        ProposalStatus = 0x03  // Proposal passed and successfully executed
    StatusRejected      ProposalStatus = 0x04  // Proposal has been rejected
    StatusFailed        ProposalStatus = 0x05  // Proposal passed but failed execution
    */
    let status_code = match proposal_status {
        ProposalStatus::StatusNil => {0x00},
        ProposalStatus::StatusDepositPeriod => {0x01},
        ProposalStatus::StatusVotingPeriod => {0x02},
        ProposalStatus::StatusPassed => {0x03},
        ProposalStatus::StatusRejected => {0x04},
        ProposalStatus::StatusFailed => {0x05},
    };
    let channel = cosmos::channels::channel(blockchain).await?;
    let res = cosmos::query::get_proposals(channel, cosmos_sdk_proto::cosmos::gov::v1beta1::QueryProposalsRequest {
        proposal_status: status_code,
        voter: "".to_string(),
        depositor: "".to_string(),
        pagination: None
    }).await?;

    let mut list: Vec<ProposalExt> = Vec::new();
    for proposal in res.proposals {
        list.push(ProposalExt(proposal));
    }
    Ok(list)
}


#[cfg(test)]
mod test {

    // cargo test -- --nocapture
    // cargo test api::custom::query::gov::get_proposals -- --exact --nocapture

    use crate::api::core::cosmos::channels::SupportedBlockchain;

    #[tokio::test]
    pub async fn get_proposals() -> anyhow::Result<()> {
        let res = super::get_proposals(SupportedBlockchain::Terra,super::ProposalStatus::StatusPassed).await?;
        println!("{:?}",res.iter().map(|x| x.content()).collect::<Vec<super::Proposal>>());
        Ok(())
    }
}