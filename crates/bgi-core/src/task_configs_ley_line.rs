use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoLeyLineOutcropConfig {
    pub ley_line_outcrop_type: String,
    pub country: String,
    pub is_resin_exhaustion_mode: bool,
    pub open_mode_count_min: bool,
    pub count: u64,
    pub use_transient_resin: bool,
    pub use_fragile_resin: bool,
    pub team: String,
    pub friendship_team: String,
    pub timeout: u64,
    pub use_adventurer_handbook: bool,
    pub is_notification: bool,
    pub is_go_to_synthesizer: bool,
    pub scan_drops_after_reward_enabled: bool,
    pub scan_drops_after_reward_seconds: u64,
    pub fight_config: AutoLeyLineOutcropFightConfig,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoLeyLineOutcropConfig {
    fn default() -> Self {
        Self {
            ley_line_outcrop_type: "启示之花".to_string(),
            country: "蒙德".to_string(),
            is_resin_exhaustion_mode: false,
            open_mode_count_min: false,
            count: 6,
            use_transient_resin: false,
            use_fragile_resin: false,
            team: String::new(),
            friendship_team: String::new(),
            timeout: 120,
            use_adventurer_handbook: false,
            is_notification: false,
            is_go_to_synthesizer: false,
            scan_drops_after_reward_enabled: false,
            scan_drops_after_reward_seconds: 12,
            fight_config: AutoLeyLineOutcropFightConfig::default(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoLeyLineOutcropFightConfig {
    pub strategy_name: String,
    pub team_names: String,
    pub fight_finish_detect_enabled: bool,
    pub action_scheduler_by_cd: String,
    pub finish_detect_config: LeyLineFightFinishDetectConfig,
    pub guardian_avatar: String,
    pub guardian_combat_skip: bool,
    pub guardian_avatar_hold: bool,
    pub burst_enabled: bool,
    pub swimming_enabled: bool,
    pub kazuha_pickup_enabled: bool,
    pub qin_double_pick_up: bool,
    pub timeout: u64,
    pub seek_enemy_enabled: bool,
    pub seek_enemy_interval_seconds: u64,
    pub seek_enemy_rotary_factor: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoLeyLineOutcropFightConfig {
    fn default() -> Self {
        Self {
            strategy_name: String::new(),
            team_names: String::new(),
            fight_finish_detect_enabled: true,
            action_scheduler_by_cd: String::new(),
            finish_detect_config: LeyLineFightFinishDetectConfig::default(),
            guardian_avatar: String::new(),
            guardian_combat_skip: false,
            guardian_avatar_hold: false,
            burst_enabled: false,
            swimming_enabled: false,
            kazuha_pickup_enabled: true,
            qin_double_pick_up: false,
            timeout: 120,
            seek_enemy_enabled: false,
            seek_enemy_interval_seconds: 3,
            seek_enemy_rotary_factor: 6,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LeyLineFightFinishDetectConfig {
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

impl Default for LeyLineFightFinishDetectConfig {
    fn default() -> Self {
        Self {
            battle_end_progress_bar_color: String::new(),
            battle_end_progress_bar_color_tolerance: String::new(),
            fast_check_enabled: false,
            rotate_find_enemy_enabled: false,
            fast_check_params: String::new(),
            check_end_delay: String::new(),
            before_detect_delay: String::new(),
            rotary_factor: 10,
            is_first_check: false,
            check_before_burst: false,
            extra: Map::new(),
        }
    }
}
