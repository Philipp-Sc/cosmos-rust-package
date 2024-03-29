use prost_types::Duration;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;

use crate::api::core::cosmos::channels::SupportedBlockchain;
use crate::api::custom::types::ProtoMessageWrapper;
use cosmos_sdk_proto::cosmos::gov::v1beta1::{QueryParamsResponse, TallyParams};
use num_format::{Locale, ToFormattedString};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash)]
pub struct ParamsExt {
    pub blockchain: SupportedBlockchain,
    pub params_type: String,
    pub params: ProtoMessageWrapper<QueryParamsResponse>,
}

impl ParamsExt {
    pub fn new(
        blockchain: SupportedBlockchain,
        params_type: &str,
        params: QueryParamsResponse,
    ) -> Self {
        Self {
            blockchain,
            params_type: params_type.to_string(),
            params: ProtoMessageWrapper(params),
        }
    }
}

impl fmt::Display for ParamsExt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if let Some(voting_params) = &self.params.0.voting_params {
            if let Some(voting_period) = &voting_params.voting_period {
                if voting_period.seconds != 0i64 || voting_period.nanos != 0i32 {
                    parts.push(format!(
                        "\nVoting period: {}",
                        DurationExt(voting_period).get_formatted_duration()
                    ));
                }
            }
        }
        if let Some(deposit_params) = &self.params.0.deposit_params {
            let min_deposit_str = deposit_params
                .min_deposit
                .iter()
                .map(|coin_ext| {
                    format!(
                        "{} {}",
                        coin_ext
                            .amount
                            .parse::<u128>()
                            .unwrap_or(0u128)
                            .to_formatted_string(&Locale::en),
                        coin_ext.denom
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            if !min_deposit_str.is_empty() {
                parts.push(format!("\nMin deposit: {}", min_deposit_str));
            }
            if let Some(max_deposit_period) = &deposit_params.max_deposit_period {
                if max_deposit_period.seconds != 0i64 || max_deposit_period.nanos != 0i32 {
                    parts.push(format!(
                        "\nMax deposit period: {}",
                        DurationExt(max_deposit_period).get_formatted_duration()
                    ));
                }
            }
        }
        if let Some(tally_params) = &self.params.0.tally_params {
            let tally_params_ext = TallyParamsExt(tally_params);
            let quorum = tally_params_ext.get_quorum();
            let threshold = tally_params_ext.get_threshold();
            let veto_threshold = tally_params_ext.get_veto_threshold();
            if quorum != 0f64 || threshold != 0f64 || veto_threshold != 0f64 {
                parts.push(format!(
                    "\nQuorum: {:.2}%,\nThreshold: {:.2}%,\nVeto threshold: {:.2}%",
                    quorum * 100.0,
                    threshold * 100.0,
                    veto_threshold * 100.0
                ));
            }
        }
        write!(f, "{}", parts.join(", "))
    }
}

pub struct DurationExt<'a>(pub &'a Duration);

impl<'a> DurationExt<'a> {
    pub fn to_duration(&self) -> chrono::Duration {
        chrono::Duration::from_std(std::time::Duration::new(
            self.0.seconds as u64,
            self.0.nanos as u32,
        ))
        .unwrap()
    }
    pub fn get_formatted_duration(&self) -> String {
        let seconds = self.to_duration().num_seconds();
        let minutes = seconds / 60;
        let hours = minutes / 60;
        let days = hours / 24;

        if days > 0 {
            format!(
                "{}d{}{}{}",
                days,
                if hours % 24 > 0 {
                    format!(" {}h", hours % 24)
                } else {
                    String::new()
                },
                if minutes % 60 > 0 {
                    format!(" {}m", minutes % 60)
                } else {
                    String::new()
                },
                if seconds % 60 > 0 {
                    format!(" {}s", seconds % 60)
                } else {
                    String::new()
                },
            )
        } else if hours > 0 {
            format!(
                "{}h{}{}",
                hours,
                if minutes % 60 > 0 {
                    format!(" {}m", minutes % 60)
                } else {
                    String::new()
                },
                if seconds % 60 > 0 {
                    format!(" {}s", seconds % 60)
                } else {
                    String::new()
                },
            )
        } else if minutes > 0 {
            format!(
                "{}m{}",
                minutes,
                if seconds % 60 > 0 {
                    format!(" {}s", seconds % 60)
                } else {
                    String::new()
                },
            )
        } else {
            format!("{}s", seconds)
        }
    }
}
pub struct TallyParamsExt<'a>(pub &'a TallyParams);

impl<'a> TallyParamsExt<'a> {
    fn decode_dec(&self, dec_encoded: &Vec<u8>) -> f64 {
        let decimal_string = String::from_utf8_lossy(dec_encoded).to_string();
        let decimal_int = decimal_string.parse::<u128>().unwrap();
        let decimal = decimal_int as f64 / 10_u128.pow(18) as f64;
        decimal
    }

    pub fn get_quorum(&self) -> f64 {
        let dec_encoded: &Vec<u8> = &self.0.quorum;
        self.decode_dec(dec_encoded)
    }

    pub fn get_threshold(&self) -> f64 {
        let dec_encoded: &Vec<u8> = &self.0.threshold;
        self.decode_dec(dec_encoded)
    }

    pub fn get_veto_threshold(&self) -> f64 {
        let dec_encoded: &Vec<u8> = &self.0.veto_threshold;
        self.decode_dec(dec_encoded)
    }
}
