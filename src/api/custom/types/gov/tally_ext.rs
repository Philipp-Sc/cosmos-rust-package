use std::fmt;
use std::hash::{Hash};
use serde::{Deserialize, Serialize};

use cosmos_sdk_proto::cosmos::gov::v1beta1::{QueryTallyResultResponse, TallyResult};
use crate::api::core::cosmos::channels::SupportedBlockchain;
use crate::api::custom::types::ProtoMessageWrapper;

#[derive(Serialize, Deserialize, Debug, Clone,PartialEq, Hash)]
pub struct TallyResultExt {
    pub blockchain: SupportedBlockchain,
    pub proposal_id: u64,
    pub tally_result: ProtoMessageWrapper<QueryTallyResultResponse>,
}

impl TallyResultExt {
    pub fn new(blockchain: SupportedBlockchain, proposal_id: u64, tally_result: QueryTallyResultResponse) -> Self {
        TallyResultExt{
            blockchain,
            proposal_id,
            tally_result: ProtoMessageWrapper(tally_result)
        }
    }
    pub fn spam_likelihood(&self) -> Option<f64> {
        if let Some(tally) = &self.tally_result.0.tally {
            TallyHelper(tally).spam_likelihood()
        }else {
            None
        }
    }
    pub fn total_votes(&self) -> Option<u64> {
        if let Some(tally) = &self.tally_result.0.tally {
            TallyHelper(tally).total_votes()
        }else {
            None
        }
    }
}

impl fmt::Display for TallyResultExt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        if let Some(tally) = &self.tally_result.0.tally {
            output.push_str(&TallyHelper(tally).current_tally_to_string());
        }
        write!(f, "{}", output)
    }
}



pub struct TallyHelper<'a>(pub &'a TallyResult);

impl <'a>TallyHelper<'a> {
    pub fn final_tally_to_string(&self) -> String {
        if let Some(tally) = self.tally_to_string() {
            format!("🗳 Final tally result:\n{}",tally)
        }else{
            "".to_string()
        }
    }
    pub fn current_tally_to_string(&self) -> String {
        if let Some(tally) = self.tally_to_string() {
            format!("🗳 Current voting results:\n{}",tally)
        }else{
            "".to_string()
        }
    }
    fn tally_to_string(&self) -> Option<String> {
        let mut output = None;
        let tally = self.0;
        if tally.yes != "0" || tally.abstain != "0" || tally.no != "0" || tally.no_with_veto != "0"
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
            output = Some(format!(
                r#"👍 {}%, 👎 {}%, 🕊️ {}%, ❌ {}% "#,
                yes_num, no_num, abstain_num, no_with_veto_num
            ));
        }
        output
    }

    pub fn spam_likelihood(&self) -> Option<f64> {
        let tally = self.0;
        if tally.yes != "0" || tally.abstain != "0" || tally.no != "0" || tally.no_with_veto != "0"
        {
        let abstain_num = tally.abstain.parse::<f64>().unwrap();
            let yes_num = tally.yes.parse::<f64>().unwrap();
            let no_num = tally.no.parse::<f64>().unwrap();
            let no_with_veto_num = tally.no_with_veto.parse::<f64>().unwrap();
            let total = abstain_num + yes_num + no_num + no_with_veto_num;
            let abstain_num = abstain_num / total;
            let yes_num = yes_num / total;
            let no_num = no_num / total;
            let no_with_veto_num = no_with_veto_num / total;
            Some(
                ((2.0 * no_with_veto_num) + no_num - yes_num - (2.0 * abstain_num))
                    / 2.0,
            )
        } else {
            None
        }
    }

    pub fn total_votes(&self) -> Option<u64> {
        let tally = self.0;
        if tally.yes != "0" || tally.abstain != "0" || tally.no != "0" || tally.no_with_veto != "0"
        {
            let abstain_num = tally.abstain.parse::<u64>().unwrap();
            let yes_num = tally.yes.parse::<u64>().unwrap();
            let no_num = tally.no.parse::<u64>().unwrap();
            let no_with_veto_num = tally.no_with_veto.parse::<u64>().unwrap();
            let total = abstain_num + yes_num + no_num + no_with_veto_num;
            Some(total)
        } else {
            None
        }
    }
}


