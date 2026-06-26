use serde::{Deserialize, Serialize};

#[path = "task_params_fight.rs"]
mod fight_params;

pub use fight_params::*;

pub const AUTO_STRATEGY_NAME: &str = "根据队伍自动选择";
const DEFAULT_RESIN_PRIORITY: &[&str] = &["浓缩树脂", "原粹树脂"];

fn default_resin_priority() -> Vec<String> {
    DEFAULT_RESIN_PRIORITY
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub fn combat_strategy_path(strategy_name: Option<&str>) -> String {
    let strategy_name = strategy_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(AUTO_STRATEGY_NAME);
    if strategy_name == AUTO_STRATEGY_NAME {
        "User/AutoFight/".to_string()
    } else {
        format!("User/AutoFight/{strategy_name}.txt")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoSkipConfigParam {
    pub enabled: bool,
    pub quickly_skip_conversations_enabled: bool,
    pub after_choose_option_sleep_delay: i32,
    pub auto_wait_dialogue_option_voice_enabled: bool,
    pub dialogue_option_voice_max_wait_seconds: i32,
    pub before_click_confirm_delay: i32,
    pub auto_get_daily_rewards_enabled: bool,
    pub auto_re_explore_enabled: bool,
    pub click_chat_option: String,
    pub custom_priority_options_enabled: bool,
    pub custom_priority_options: String,
    pub auto_hangout_event_enabled: bool,
    pub auto_hangout_end_choose: String,
    pub auto_hangout_choose_option_sleep_delay: i32,
    pub auto_hangout_press_skip_enabled: bool,
    pub run_background_enabled: bool,
    pub bring_game_to_front_after_background_dialog_enabled: bool,
    pub submit_goods_enabled: bool,
    pub picture_in_picture_enabled: bool,
    pub picture_in_picture_source_type: String,
    pub close_popup_paged_enabled: bool,
    pub skip_built_in_click_options: bool,
}

impl Default for AutoSkipConfigParam {
    fn default() -> Self {
        Self {
            enabled: true,
            quickly_skip_conversations_enabled: true,
            after_choose_option_sleep_delay: 0,
            auto_wait_dialogue_option_voice_enabled: false,
            dialogue_option_voice_max_wait_seconds: 30,
            before_click_confirm_delay: 0,
            auto_get_daily_rewards_enabled: true,
            auto_re_explore_enabled: true,
            click_chat_option: "优先选择第一个选项".to_string(),
            custom_priority_options_enabled: false,
            custom_priority_options: String::new(),
            auto_hangout_event_enabled: false,
            auto_hangout_end_choose: String::new(),
            auto_hangout_choose_option_sleep_delay: 0,
            auto_hangout_press_skip_enabled: true,
            run_background_enabled: false,
            bring_game_to_front_after_background_dialog_enabled: false,
            submit_goods_enabled: true,
            picture_in_picture_enabled: false,
            picture_in_picture_source_type: "CaptureLoop".to_string(),
            close_popup_paged_enabled: true,
            skip_built_in_click_options: false,
        }
    }
}

impl AutoSkipConfigParam {
    pub fn is_click_first_chat_option(&self) -> bool {
        self.click_chat_option == "优先选择第一个选项"
    }

    pub fn is_click_random_chat_option(&self) -> bool {
        self.click_chat_option == "随机选择选项"
    }

    pub fn is_click_none_chat_option(&self) -> bool {
        self.click_chat_option == "不选择选项"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoDomainParam {
    pub domain_round_num: i32,
    pub combat_strategy_path: String,
    pub party_name: String,
    pub domain_name: String,
    pub sunday_selected_value: String,
    pub auto_artifact_salvage: bool,
    pub max_artifact_star: String,
    pub specify_resin_use: bool,
    pub resin_priority_list: Vec<String>,
    pub original_resin_use_count: i32,
    pub original_resin20_use_count: i32,
    pub original_resin40_use_count: i32,
    pub condensed_resin_use_count: i32,
    pub transient_resin_use_count: i32,
    pub fragile_resin_use_count: i32,
    pub reward_recognition_enabled: bool,
}

impl Default for AutoDomainParam {
    fn default() -> Self {
        Self::new(0, None)
    }
}

impl AutoDomainParam {
    pub fn new(domain_round_num: i32, strategy_name: Option<&str>) -> Self {
        Self {
            domain_round_num: if domain_round_num == 0 {
                9999
            } else {
                domain_round_num
            },
            combat_strategy_path: combat_strategy_path(strategy_name),
            party_name: String::new(),
            domain_name: String::new(),
            sunday_selected_value: String::new(),
            auto_artifact_salvage: false,
            max_artifact_star: "4".to_string(),
            specify_resin_use: false,
            resin_priority_list: default_resin_priority(),
            original_resin_use_count: 0,
            original_resin20_use_count: 0,
            original_resin40_use_count: 0,
            condensed_resin_use_count: 0,
            transient_resin_use_count: 0,
            fragile_resin_use_count: 0,
            reward_recognition_enabled: false,
        }
    }

    pub fn set_resin_priority_list(
        &mut self,
        priorities: impl IntoIterator<Item = impl Into<String>>,
    ) {
        self.resin_priority_list = priorities.into_iter().map(Into::into).collect();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoBossParam {
    pub boss_name: String,
    pub strategy_name: String,
    pub combat_strategy_path: String,
    pub team_name: String,
    pub specify_run_count: bool,
    pub run_count: i32,
    pub use_transient_resin: bool,
    pub use_fragile_resin: bool,
    pub revive_retry_count: i32,
    pub return_to_statue_after_each_round: bool,
    pub reward_recognition_enabled: bool,
}

impl Default for AutoBossParam {
    fn default() -> Self {
        Self::new(None)
    }
}

impl AutoBossParam {
    pub fn new(strategy_name: Option<&str>) -> Self {
        let strategy_name = strategy_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(AUTO_STRATEGY_NAME)
            .to_string();
        Self {
            combat_strategy_path: combat_strategy_path(Some(&strategy_name)),
            strategy_name,
            boss_name: String::new(),
            team_name: String::new(),
            specify_run_count: false,
            run_count: 1,
            use_transient_resin: false,
            use_fragile_resin: false,
            revive_retry_count: 3,
            return_to_statue_after_each_round: false,
            reward_recognition_enabled: false,
        }
    }

    pub fn set_strategy_name(&mut self, strategy_name: Option<&str>) {
        let strategy_name = strategy_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(AUTO_STRATEGY_NAME)
            .to_string();
        self.combat_strategy_path = combat_strategy_path(Some(&strategy_name));
        self.strategy_name = strategy_name;
    }

    pub fn set_run_count(&mut self, value: i32) {
        self.run_count = value.max(1);
    }

    pub fn set_revive_retry_count(&mut self, value: i32) {
        self.revive_retry_count = value.max(0);
    }

    pub fn set_specify_run_count(&mut self, enabled: bool) {
        self.specify_run_count = enabled;
        if !enabled {
            self.use_transient_resin = false;
            self.use_fragile_resin = false;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskParameterModels {
    pub auto_skip: AutoSkipConfigParam,
    pub auto_domain: AutoDomainParam,
    pub auto_boss: AutoBossParam,
    pub auto_fight: AutoFightParam,
    pub auto_ley_line_outcrop: AutoLeyLineOutcropParam,
    pub auto_stygian_onslaught: AutoStygianOnslaughtParam,
}

impl Default for TaskParameterModels {
    fn default() -> Self {
        Self {
            auto_skip: AutoSkipConfigParam::default(),
            auto_domain: AutoDomainParam::default(),
            auto_boss: AutoBossParam::default(),
            auto_fight: AutoFightParam::default(),
            auto_ley_line_outcrop: AutoLeyLineOutcropParam::default(),
            auto_stygian_onslaught: AutoStygianOnslaughtParam::default(),
        }
    }
}

pub fn task_parameter_models() -> TaskParameterModels {
    TaskParameterModels::default()
}
