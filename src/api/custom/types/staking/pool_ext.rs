
use serde::{Deserialize, Serialize};
use cosmos_sdk_proto::prost::Message;
use cosmos_sdk_proto::cosmos::staking::v1beta1::QueryPoolResponse;
use crate::api::core::cosmos::channels::SupportedBlockchain;
use crate::api::custom::types::ProtoMessageWrapper;

use num_format::{Locale, ToFormattedString};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash)]
pub struct PoolExt {
    pub blockchain: SupportedBlockchain,
    pub pool: ProtoMessageWrapper<QueryPoolResponse>,
}

impl PoolExt {
    pub fn new(blockchain: SupportedBlockchain, query_pool_response: QueryPoolResponse) -> Self {
        Self {
            blockchain,
            pool: ProtoMessageWrapper(query_pool_response)
        }
    }
    pub fn get_voter_turnout(&self, proposal_total_votes: Option<f64>) -> Option<String> {
        let bonded_tokens = self.pool.0.pool.as_ref().map(|x| x.bonded_tokens.parse::<f64>().ok()).flatten();
        if let Some(bonded) = bonded_tokens {
            if let Some(total_votes) = proposal_total_votes{
                if total_votes == 0f64 || bonded == 0f64 {
                    None
                }else{
                    Some(format!("Voter turnout: {:.2}%",(total_votes/ bonded) * 100.0 ))
                }
            }else{
                None
            }
        }else{
            None
        }
    }
    pub fn get_pool_details(&self) -> Option<String> {
        let mut output = String::new();
        let bonded_tokens = self.pool.0.pool.as_ref().map(|x| x.bonded_tokens.parse::<u128>().ok()).flatten();
        let not_bonded_tokens = self.pool.0.pool.as_ref().map(|x| x.not_bonded_tokens.parse::<u128>().ok()).flatten();

        if let Some(bonded) = bonded_tokens {
            output.push_str(&format!("\nBonded tokens: {}",bonded.to_formatted_string(&Locale::en)));
        }
        if let Some(not_bonded) = not_bonded_tokens {
            output.push_str(&format!("\nNot bonded tokens: {}",not_bonded.to_formatted_string(&Locale::en)));
        }
        if output.is_empty() {
            None
        }else {
            Some(output)
        }
    }

}