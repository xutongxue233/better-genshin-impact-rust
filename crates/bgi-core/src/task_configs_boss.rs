use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoBossConfig {
    pub boss_name: String,
    pub strategy_name: String,
    pub team_name: String,
    pub specify_run_count: bool,
    pub run_count: u64,
    pub use_transient_resin: bool,
    pub use_fragile_resin: bool,
    pub revive_retry_count: u64,
    pub return_to_statue_after_each_round: bool,
    pub reward_recognition_enabled: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoBossConfig {
    fn default() -> Self {
        Self {
            boss_name: String::new(),
            strategy_name: "根据队伍自动选择".to_string(),
            team_name: String::new(),
            specify_run_count: false,
            run_count: 1,
            use_transient_resin: false,
            use_fragile_resin: false,
            revive_retry_count: 3,
            return_to_statue_after_each_round: false,
            reward_recognition_enabled: false,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoStygianOnslaughtConfig {
    pub strategy_name: String,
    pub boss_num: u64,
    pub auto_artifact_salvage: bool,
    pub specify_resin_use: bool,
    pub resin_priority_list: Vec<String>,
    pub original_resin_use_count: u64,
    pub condensed_resin_use_count: u64,
    pub transient_resin_use_count: u64,
    pub fragile_resin_use_count: u64,
    pub fight_team_name: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoStygianOnslaughtConfig {
    fn default() -> Self {
        Self {
            strategy_name: String::new(),
            boss_num: 1,
            auto_artifact_salvage: false,
            specify_resin_use: false,
            resin_priority_list: vec!["浓缩树脂".to_string(), "原粹树脂".to_string()],
            original_resin_use_count: 0,
            condensed_resin_use_count: 0,
            transient_resin_use_count: 0,
            fragile_resin_use_count: 0,
            fight_team_name: String::new(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoArtifactSalvageConfig {
    pub java_script: String,
    pub artifact_set_filter: String,
    pub regular_expression: String,
    pub max_artifact_star: String,
    pub max_num_to_check: u64,
    pub recognition_failure_policy: Value,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoArtifactSalvageConfig {
    fn default() -> Self {
        Self {
            java_script: "var hasATK = Array.from(ArtifactStat.MinorAffixes).some(affix => affix.Type == 'ATK');\nvar hasDEF = Array.from(ArtifactStat.MinorAffixes).some(affix => affix.Type == 'DEF');\nvar hasHP = Array.from(ArtifactStat.MinorAffixes).some(affix => affix.Type == 'HP');\nOutput = (hasATK && hasDEF) || (hasHP && hasDEF);".to_string(),
            artifact_set_filter: String::new(),
            regular_expression: r"(?=[\S\s]*攻击力\+[\d]*\n)(?=[\S\s]*防御力\+[\d]*\n)".to_string(),
            max_artifact_star: "4".to_string(),
            max_num_to_check: 100,
            recognition_failure_policy: Value::String("Skip".to_string()),
            extra: Map::new(),
        }
    }
}
