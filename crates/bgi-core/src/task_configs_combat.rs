use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoWoodConfig {
    pub after_z_sleep_delay: u64,
    pub wood_count_ocr_enabled: bool,
    pub use_wonderland_refresh: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoWoodConfig {
    fn default() -> Self {
        Self {
            after_z_sleep_delay: 0,
            wood_count_ocr_enabled: false,
            use_wonderland_refresh: true,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoFightConfig {
    pub strategy_name: String,
    pub team_names: String,
    pub fight_finish_detect_enabled: bool,
    pub action_scheduler_by_cd: String,
    pub only_pick_elite_drops_mode: String,
    pub finish_detect_config: FightFinishDetectConfig,
    pub pick_drops_after_fight_enabled: bool,
    pub pick_drops_after_fight_seconds: u64,
    pub battle_threshold_for_loot: Option<u64>,
    pub kazuha_pickup_enabled: bool,
    pub qin_double_pick_up: bool,
    pub guardian_avatar: String,
    pub guardian_combat_skip: bool,
    pub skip_model: bool,
    pub guardian_avatar_hold: bool,
    pub burst_enabled: bool,
    pub kazuha_party_name: String,
    pub swimming_enabled: bool,
    pub exp_based_pickup_enabled: bool,
    pub timeout: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoFightConfig {
    fn default() -> Self {
        Self {
            strategy_name: "根据队伍自动选择".to_string(),
            team_names: String::new(),
            fight_finish_detect_enabled: true,
            action_scheduler_by_cd: String::new(),
            only_pick_elite_drops_mode: "Closed".to_string(),
            finish_detect_config: FightFinishDetectConfig::default(),
            pick_drops_after_fight_enabled: false,
            pick_drops_after_fight_seconds: 15,
            battle_threshold_for_loot: None,
            kazuha_pickup_enabled: true,
            qin_double_pick_up: false,
            guardian_avatar: String::new(),
            guardian_combat_skip: false,
            skip_model: false,
            guardian_avatar_hold: false,
            burst_enabled: false,
            kazuha_party_name: String::new(),
            swimming_enabled: true,
            exp_based_pickup_enabled: false,
            timeout: 120,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FightFinishDetectConfig {
    pub battle_end_progress_bar_color: String,
    pub battle_end_progress_bar_color_tolerance: String,
    pub fast_check_enabled: bool,
    pub rotate_find_enemy_enabled: bool,
    pub fast_check_params: String,
    pub check_end_delay: String,
    pub before_detect_delay: String,
    pub rotary_factor: u64,
    pub is_first_check: bool,
    pub check_before_burst: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for FightFinishDetectConfig {
    fn default() -> Self {
        Self {
            battle_end_progress_bar_color: String::new(),
            battle_end_progress_bar_color_tolerance: String::new(),
            fast_check_enabled: false,
            rotate_find_enemy_enabled: false,
            fast_check_params: String::new(),
            check_end_delay: "0.4;钟离,1.4;".to_string(),
            before_detect_delay: "0.4".to_string(),
            rotary_factor: 12,
            is_first_check: false,
            check_before_burst: false,
            extra: Map::new(),
        }
    }
}
