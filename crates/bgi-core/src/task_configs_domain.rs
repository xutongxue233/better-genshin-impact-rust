use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoDomainConfig {
    pub fight_end_delay: f64,
    pub short_movement: bool,
    pub walk_to_f: bool,
    pub left_right_move_times: u64,
    pub auto_eat: bool,
    pub party_name: String,
    pub domain_name: String,
    pub auto_artifact_salvage: bool,
    pub sunday_selected_value: String,
    pub specify_resin_use: bool,
    pub resin_priority_list: Vec<String>,
    pub original_resin_use_count: u64,
    pub original_resin20_use_count: u64,
    pub original_resin40_use_count: u64,
    pub condensed_resin_use_count: u64,
    pub transient_resin_use_count: u64,
    pub fragile_resin_use_count: u64,
    pub revive_retry_count: u64,
    pub reward_recognition_enabled: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoDomainConfig {
    fn default() -> Self {
        Self {
            fight_end_delay: 5.0,
            short_movement: false,
            walk_to_f: true,
            left_right_move_times: 3,
            auto_eat: false,
            party_name: String::new(),
            domain_name: String::new(),
            auto_artifact_salvage: false,
            sunday_selected_value: String::new(),
            specify_resin_use: false,
            resin_priority_list: vec!["浓缩树脂".to_string(), "原粹树脂".to_string()],
            original_resin_use_count: 0,
            original_resin20_use_count: 0,
            original_resin40_use_count: 0,
            condensed_resin_use_count: 0,
            transient_resin_use_count: 0,
            fragile_resin_use_count: 0,
            revive_retry_count: 3,
            reward_recognition_enabled: false,
            extra: Map::new(),
        }
    }
}
