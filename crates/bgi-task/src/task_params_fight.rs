use super::{combat_strategy_path, default_resin_priority};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FightFinishDetectParam {
    pub battle_end_progress_bar_color: String,
    pub battle_end_progress_bar_color_tolerance: String,
    pub fast_check_enabled: bool,
    pub fast_check_params: String,
    pub check_end_delay: String,
    pub before_detect_delay: String,
    pub rotate_find_enemy_enabled: bool,
    pub rotary_factor: i32,
    pub is_first_check: bool,
    pub check_before_burst: bool,
}

impl Default for FightFinishDetectParam {
    fn default() -> Self {
        Self {
            battle_end_progress_bar_color: String::new(),
            battle_end_progress_bar_color_tolerance: String::new(),
            fast_check_enabled: false,
            fast_check_params: String::new(),
            check_end_delay: String::new(),
            before_detect_delay: String::new(),
            rotate_find_enemy_enabled: false,
            rotary_factor: 10,
            is_first_check: true,
            check_before_burst: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFightParam {
    pub combat_strategy_path: String,
    pub team_names: String,
    pub finish_detect_config: FightFinishDetectParam,
    pub fight_finish_detect_enabled: bool,
    pub pick_drops_after_fight_enabled: bool,
    pub pick_drops_after_fight_seconds: i32,
    pub battle_threshold_for_loot: i32,
    pub timeout: i32,
    pub kazuha_pickup_enabled: bool,
    pub action_scheduler_by_cd: String,
    pub kazuha_party_name: String,
    pub only_pick_elite_drops_mode: String,
    pub guardian_avatar: String,
    pub guardian_combat_skip: bool,
    pub guardian_avatar_hold: bool,
    pub check_before_burst: bool,
    pub is_first_check: bool,
    pub rotary_factor: i32,
    pub burst_enabled: bool,
    pub qin_double_pick_up: bool,
    pub swimming_enabled: bool,
    pub exp_based_pickup_enabled: bool,
}

impl Default for AutoFightParam {
    fn default() -> Self {
        Self::new(None)
    }
}

impl AutoFightParam {
    pub fn new(strategy_name: Option<&str>) -> Self {
        Self {
            combat_strategy_path: combat_strategy_path(strategy_name),
            team_names: String::new(),
            finish_detect_config: FightFinishDetectParam::default(),
            fight_finish_detect_enabled: false,
            pick_drops_after_fight_enabled: false,
            pick_drops_after_fight_seconds: 15,
            battle_threshold_for_loot: -1,
            timeout: 120,
            kazuha_pickup_enabled: true,
            action_scheduler_by_cd: String::new(),
            kazuha_party_name: String::new(),
            only_pick_elite_drops_mode: String::new(),
            guardian_avatar: String::new(),
            guardian_combat_skip: false,
            guardian_avatar_hold: false,
            check_before_burst: false,
            is_first_check: true,
            rotary_factor: 10,
            burst_enabled: false,
            qin_double_pick_up: false,
            swimming_enabled: false,
            exp_based_pickup_enabled: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLineOutcropFightConfigParam {
    pub strategy_name: String,
    pub team_names: String,
    pub fight_finish_detect_enabled: bool,
    pub action_scheduler_by_cd: String,
    pub finish_detect_config: FightFinishDetectParam,
    pub guardian_avatar: String,
    pub guardian_combat_skip: bool,
    pub guardian_avatar_hold: bool,
    pub burst_enabled: bool,
    pub swimming_enabled: bool,
    pub kazuha_pickup_enabled: bool,
    pub qin_double_pick_up: bool,
    pub timeout: i32,
    pub seek_enemy_enabled: bool,
    pub seek_enemy_interval_seconds: i32,
    pub seek_enemy_rotary_factor: i32,
}

impl Default for AutoLeyLineOutcropFightConfigParam {
    fn default() -> Self {
        Self {
            strategy_name: String::new(),
            team_names: String::new(),
            fight_finish_detect_enabled: true,
            action_scheduler_by_cd: String::new(),
            finish_detect_config: FightFinishDetectParam {
                is_first_check: false,
                ..FightFinishDetectParam::default()
            },
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLineOutcropParam {
    pub count: i32,
    pub country: String,
    pub ley_line_outcrop_type: String,
    pub open_mode_count_min: bool,
    pub is_resin_exhaustion_mode: bool,
    pub use_adventurer_handbook: bool,
    pub friendship_team: String,
    pub team: String,
    pub timeout: i32,
    pub fight_config: AutoLeyLineOutcropFightConfigParam,
    pub is_go_to_synthesizer: bool,
    pub use_fragile_resin: bool,
    pub use_transient_resin: bool,
    pub is_notification: bool,
    pub scan_drops_after_reward_enabled: bool,
    pub scan_drops_after_reward_seconds: i32,
}

impl Default for AutoLeyLineOutcropParam {
    fn default() -> Self {
        Self {
            count: 0,
            country: String::new(),
            ley_line_outcrop_type: String::new(),
            open_mode_count_min: false,
            is_resin_exhaustion_mode: false,
            use_adventurer_handbook: false,
            friendship_team: String::new(),
            team: String::new(),
            timeout: 120,
            fight_config: AutoLeyLineOutcropFightConfigParam::default(),
            is_go_to_synthesizer: false,
            use_fragile_resin: false,
            use_transient_resin: false,
            is_notification: false,
            scan_drops_after_reward_enabled: false,
            scan_drops_after_reward_seconds: 0,
        }
    }
}

impl AutoLeyLineOutcropParam {
    pub fn new(
        count: i32,
        country: impl Into<String>,
        ley_line_outcrop_type: impl Into<String>,
    ) -> Self {
        Self {
            count,
            country: country.into(),
            ley_line_outcrop_type: ley_line_outcrop_type.into(),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoStygianOnslaughtParam {
    pub boss_num: i32,
    pub auto_artifact_salvage: bool,
    pub specify_resin_use: bool,
    pub resin_priority_list: Vec<String>,
    pub original_resin_use_count: i32,
    pub condensed_resin_use_count: i32,
    pub transient_resin_use_count: i32,
    pub fragile_resin_use_count: i32,
    pub fight_team_name: String,
    pub combat_script_bag_path: String,
}

impl Default for AutoStygianOnslaughtParam {
    fn default() -> Self {
        Self {
            boss_num: 0,
            auto_artifact_salvage: false,
            specify_resin_use: false,
            resin_priority_list: default_resin_priority(),
            original_resin_use_count: 0,
            condensed_resin_use_count: 0,
            transient_resin_use_count: 0,
            fragile_resin_use_count: 0,
            fight_team_name: String::new(),
            combat_script_bag_path: combat_strategy_path(None),
        }
    }
}

impl AutoStygianOnslaughtParam {
    pub fn new(combat_script_bag_path: Option<&str>) -> Self {
        Self {
            combat_script_bag_path: combat_script_bag_path
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| combat_strategy_path(None)),
            ..Self::default()
        }
    }

    pub fn set_combat_strategy_path(&mut self, strategy_name: Option<&str>) {
        self.combat_script_bag_path = combat_strategy_path(strategy_name);
    }

    pub fn set_resin_priority_list(
        &mut self,
        priorities: impl IntoIterator<Item = impl Into<String>>,
    ) {
        self.resin_priority_list = priorities.into_iter().map(Into::into).collect();
    }
}
