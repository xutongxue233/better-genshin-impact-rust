use bgi_core::AutoDomainConfig;
use bgi_vision::{Rect, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::task_params::AutoDomainParam;
use crate::{Result, TaskError};

pub const AUTO_DOMAIN_TASK_KEY: &str = "AutoDomain";
pub const AUTO_DOMAIN_DISPLAY_NAME: &str = "自动秘境";
pub const AUTO_DOMAIN_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_DOMAIN_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_DOMAIN_UNLIMITED_ROUNDS: i32 = 9999;
pub const AUTO_DOMAIN_RESIN_SWITCH_ASSET: &str = "AutoDomain:resin_switch_btn.png";
pub const AUTO_DOMAIN_RESIN_SWITCH_DISABLED_ASSET: &str =
    "AutoDomain:resin_switch_btn_no_active.png";
pub const AUTO_DOMAIN_CONFIRM_ASSET: &str = "AutoFight:confirm.png";
pub const AUTO_DOMAIN_ARTIFACT_FLOWER_ASSET: &str = "AutoFight:artifact_flower_logo.png";
pub const AUTO_DOMAIN_CLICK_ANY_CLOSE_ASSET: &str = "AutoFight:click_any_close_tip.png";
pub const AUTO_DOMAIN_EXIT_ASSET: &str = "AutoFight:exit.png";
pub const AUTO_DOMAIN_ABNORMAL_ICON_ASSET: &str = "AutoFight:abnormal_icon.png";
pub const AUTO_DOMAIN_IN_DOMAIN_ASSET: &str = "Common/Element:in_domain.png";
pub const AUTO_DOMAIN_PARTY_CHOOSE_ASSET: &str = "Common/Element:party_btn_choose_view.png";
pub const AUTO_DOMAIN_PICK_KEY_ASSET: &str = "AutoPick:F.png";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoDomainExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub asset_scale: f64,
    pub param: AutoDomainParam,
    pub param_rule: AutoDomainParamRule,
    pub config_rule: AutoDomainConfigRule,
    pub startup_rule: AutoDomainStartupRule,
    pub retry_rule: AutoDomainRetryRule,
    pub locators: AutoDomainLocators,
    pub domain_entry_rule: AutoDomainEntryRule,
    pub sunday_reward_rule: AutoDomainSundayRewardRule,
    pub combat_rule: AutoDomainCombatRule,
    pub petrified_tree_rule: AutoDomainPetrifiedTreeRule,
    pub reward_rule: AutoDomainRewardRule,
    pub resin_rule: AutoDomainResinRule,
    pub artifact_salvage_rule: AutoDomainArtifactSalvageRule,
    pub steps: Vec<AutoDomainTaskStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainExecutionConfig {
    pub capture_size: Size,
    pub asset_scale: f64,
    pub param: AutoDomainParam,
    pub auto_domain_config: AutoDomainConfig,
}

impl Default for AutoDomainExecutionConfig {
    fn default() -> Self {
        let auto_domain_config = AutoDomainConfig::default();
        let mut param = AutoDomainParam::default();
        apply_auto_domain_config(&mut param, &auto_domain_config);
        Self {
            capture_size: Size::new(
                AUTO_DOMAIN_DEFAULT_CAPTURE_WIDTH,
                AUTO_DOMAIN_DEFAULT_CAPTURE_HEIGHT,
            ),
            asset_scale: 1.0,
            param,
            auto_domain_config,
        }
    }
}

impl AutoDomainExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };

        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }
        if let Some(asset_scale) = f64_member(value, ["assetScale", "AssetScale", "asset_scale"]) {
            config.asset_scale = asset_scale.max(0.0);
        }

        let auto_domain_value = value
            .get("autoDomainConfig")
            .or_else(|| value.get("AutoDomainConfig"))
            .or_else(|| value.get("auto_domain_config"))
            .unwrap_or(value);
        config.auto_domain_config =
            serde_json::from_value(auto_domain_value.clone()).unwrap_or_default();

        let param_value = value
            .get("param")
            .or_else(|| value.get("Param"))
            .or_else(|| value.get("autoDomainParam"))
            .or_else(|| value.get("AutoDomainParam"))
            .or_else(|| value.get("auto_domain_param"));
        let raw_round = i32_member_from(
            param_value,
            value,
            &["domainRoundNum", "DomainRoundNum", "domain_round_num"],
        )
        .unwrap_or(0);
        let strategy_name = string_member_from(
            param_value,
            value,
            &[
                "strategyName",
                "StrategyName",
                "combatStrategyName",
                "CombatStrategyName",
            ],
        );

        let mut param = AutoDomainParam::new(raw_round, strategy_name.as_deref());
        apply_auto_domain_config(&mut param, &config.auto_domain_config);
        overlay_auto_domain_param_members(&mut param, value);
        if let Some(param_value) = param_value {
            overlay_auto_domain_param_members(&mut param, param_value);
        }
        param.domain_round_num = normalize_domain_round_num(param.domain_round_num);
        config.param = param;
        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoDomainParamRule {
    pub raw_domain_round_num: i32,
    pub normalized_domain_round_num: i32,
    pub unlimited_round_sentinel: i32,
    pub combat_strategy_path: String,
    pub auto_team_strategy_directory: String,
    pub party_name: String,
    pub domain_name: String,
    pub sunday_selected_value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoDomainConfigRule {
    pub fight_end_delay_seconds: f64,
    pub short_movement: bool,
    pub walk_to_f: bool,
    pub left_right_move_times: u64,
    pub auto_eat_enabled: bool,
    pub auto_artifact_salvage_enabled: bool,
    pub specify_resin_use: bool,
    pub revive_retry_count: u64,
    pub reward_recognition_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoDomainStartupRule {
    pub logs_screen_resolution: bool,
    pub requires_16_to_9_resolution: bool,
    pub warns_below_1920x1080: bool,
    pub destroys_auto_fight_assets_before_start: bool,
    pub creates_bgi_tree_yolo_predictor: bool,
    pub parses_combat_script_bag_before_start: bool,
    pub adds_auto_eat_realtime_trigger_when_enabled: bool,
    pub sends_domain_start_notification: bool,
    pub sends_domain_end_notification: bool,
    pub waits_for_main_ui_after_domain_seconds: u64,
    pub pre_main_ui_delay_ms: u64,
    pub post_main_ui_delay_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoDomainRetryRule {
    pub revive_retry_count: u64,
    pub retry_only_when_domain_name_configured: bool,
    pub retry_exception_message_contains: String,
    pub retry_delay_ms: u64,
    pub notification_event_on_retry: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoDomainLocators {
    pub resin_switch_button: AutoDomainTemplateLocator,
    pub resin_switch_button_disabled: AutoDomainTemplateLocator,
    pub confirm_button: AutoDomainTemplateLocator,
    pub artifact_flower: AutoDomainTemplateLocator,
    pub click_any_close_tip: AutoDomainTemplateLocator,
    pub exit_button: AutoDomainTemplateLocator,
    pub abnormal_icon: AutoDomainTemplateLocator,
    pub in_domain_icon: AutoDomainTemplateLocator,
    pub party_choose_view: AutoDomainTemplateLocator,
    pub pick_key: AutoDomainTemplateLocator,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoDomainTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Option<Rect>,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoDomainEntryRule {
    pub teleports_when_domain_name_configured: bool,
    pub uses_domain_position_map: bool,
    pub post_teleport_delay_ms: u64,
    pub waits_for_main_ui_after_teleport: bool,
    pub door_pick_retry_attempts: u64,
    pub door_pick_retry_interval_ms: u64,
    pub challenge_menu_retry_attempts: u64,
    pub challenge_menu_retry_interval_ms: u64,
    pub enter_domain_retry_attempts: u64,
    pub enter_domain_retry_interval_ms: u64,
    pub team_ui_retry_attempts: u64,
    pub team_ui_retry_interval_ms: u64,
    pub start_fight_retry_attempts: u64,
    pub start_fight_retry_interval_ms: u64,
    pub post_start_fight_delay_ms: u64,
    pub special_domain_movements: Vec<AutoDomainSpecialMovementRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoDomainSpecialMovementRule {
    pub domain_name: String,
    pub movement: String,
    pub retry_attempts: u64,
    pub retry_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoDomainSundayRewardRule {
    pub enabled_when_sunday_after_4_or_monday_before_4: bool,
    pub enabled_when_limited_open_ocr_matches: bool,
    pub limited_open_patterns: Vec<String>,
    pub artifact_domain_template_skips_reward_selection: bool,
    pub scroll_left_side_first: bool,
    pub scroll_steps: u64,
    pub scroll_step_delay_ms: u64,
    pub after_scroll_delay_ms: u64,
    pub ocr_rect: AutoDomainRelativeRect,
    pub ley_line_disorder_pattern: String,
    pub selected_value_click_offsets: Vec<AutoDomainSundayClickOffset>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct AutoDomainRelativeRect {
    pub x_ratio: f64,
    pub y_ratio: f64,
    pub width_ratio: f64,
    pub height_ratio: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct AutoDomainSundayClickOffset {
    pub selected_value: u8,
    pub y_offset_capture_height_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoDomainCombatRule {
    pub initializes_team_on_first_round: bool,
    pub retries_team_init_before_fight: bool,
    pub switches_to_first_script_avatar: bool,
    pub switch_avatar_sleep_ms: u64,
    pub walk_to_f_timeout_ms: u64,
    pub walk_forward_start_delay_ms: u64,
    pub sprint_when_walk_to_f_disabled: bool,
    pub releases_forward_and_sprint_on_finish: bool,
    pub domain_end_detection_interval_ms: u64,
    pub finish_detection_ocr_engine: String,
    pub challenge_completed_pattern: String,
    pub auto_leaving_pattern: String,
    pub fight_end_delay_ms: u64,
    pub auto_eat_low_hp_check_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoDomainPetrifiedTreeRule {
    pub yolo_model: String,
    pub middle_click_before_search: bool,
    pub after_middle_click_delay_ms: u64,
    pub locks_camera_to_east: bool,
    pub east_angle_min_degrees: f64,
    pub east_angle_max_degrees: f64,
    pub continuous_east_count_before_move: u64,
    pub camera_adjust_interval_ms: u64,
    pub horizontal_move_interval_ms: u64,
    pub no_detect_switch_threshold: u64,
    pub left_right_move_times: u64,
    pub short_movement: bool,
    pub micro_step_ms: u64,
    pub after_forward_step_sleep_ms: u64,
    pub clears_draw_content_on_finish: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoDomainRewardRule {
    pub prompt_initial_delay_ms: u64,
    pub resin_prompt_ocr_rect: AutoDomainRelativeRect,
    pub resin_prompt_retry_attempts: u64,
    pub resin_prompt_retry_interval_ms: u64,
    pub after_prompt_detected_delay_ms: u64,
    pub no_original_resin_texts: Vec<String>,
    pub default_mode_uses_condensed_before_original: bool,
    pub default_mode_min_original_resin: u64,
    pub specified_mode_uses_record_order: bool,
    pub original_resin_type_switch_retry_attempts: u64,
    pub original_resin_type_switch_retry_interval_ms: u64,
    pub use_button_double_click_gap_ms: u64,
    pub sends_reward_notification: bool,
    pub reward_recognition_enabled: bool,
    pub continuation_poll_attempts: u64,
    pub continuation_poll_interval_ms: u64,
    pub continue_double_click_gap_ms: u64,
    pub no_resin_challenge_fallback_delay_ms: u64,
    pub exit_domain_sequence: AutoDomainExitDomainRule,
    pub normal_end_when_completion_prompt_missing: String,
    pub stop_reasons: Vec<AutoDomainStopReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoDomainExitDomainRule {
    pub first_escape_delay_ms: u64,
    pub second_escape_delay_ms: u64,
    pub clicks_black_confirm_button: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoDomainStopReason {
    OriginalResinExhaustedPrompt,
    DefaultResinInsufficient,
    SpecifiedResinUnavailable,
    LastConfiguredRound,
    CompletionPromptMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoDomainResinRule {
    pub specify_resin_use: bool,
    pub default_priority_list: Vec<String>,
    pub configured_priority_list: Vec<String>,
    pub specified_records: Vec<AutoDomainResinUseRecord>,
    pub original_resin_20_name: String,
    pub original_resin_40_name: String,
    pub original_resin_alias_for_button: String,
    pub transient_resin_name: String,
    pub fragile_resin_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoDomainResinUseRecord {
    pub name: String,
    pub remain_count: i32,
    pub max_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoDomainArtifactSalvageRule {
    pub enabled: bool,
    pub max_artifact_star: String,
    pub invalid_star_falls_back_to: u8,
    pub starts_auto_artifact_salvage_task: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoDomainTaskStep {
    pub phase: AutoDomainTaskPhase,
    pub action: AutoDomainTaskAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoDomainTaskPhase {
    Startup,
    Retry,
    Teleport,
    EnterDomain,
    RoundLoop,
    Fight,
    PetrifiedTree,
    Reward,
    Finish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoDomainTaskAction {
    InitializeAssetsAndConfig,
    NotifyDomainStart,
    RetryOnReviveWhenDomainConfigured,
    TeleportToDomainWhenConfigured,
    EnterSinglePlayerChallenge,
    CloseDomainTip,
    InitializeTeamOnce,
    SelectCombatScriptAndFirstAvatar,
    WalkToKeyAndPressF,
    RunAutoFightUntilDomainEnd,
    WaitAfterFight,
    DetectAndCenterPetrifiedTree,
    WalkToTreeAndPressF,
    ChooseAndUseResin,
    RecognizeRewardsWhenEnabled,
    ContinueOrExitDomain,
    WaitMainUiAndArtifactSalvage,
    NotifyDomainEnd,
}

pub fn plan_auto_domain(config: AutoDomainExecutionConfig) -> Result<AutoDomainExecutionPlan> {
    let mut param = config.param.clone();
    param.domain_round_num = normalize_domain_round_num(param.domain_round_num);
    let specified_records = build_domain_resin_records(&param)?;
    let fight_end_delay_ms = (config.auto_domain_config.fight_end_delay.max(0.0) * 1000.0) as u64;

    Ok(AutoDomainExecutionPlan {
        task_key: AUTO_DOMAIN_TASK_KEY.to_string(),
        display_name: AUTO_DOMAIN_DISPLAY_NAME.to_string(),
        capture_size: config.capture_size,
        asset_scale: config.asset_scale,
        param_rule: AutoDomainParamRule {
            raw_domain_round_num: config.param.domain_round_num,
            normalized_domain_round_num: param.domain_round_num,
            unlimited_round_sentinel: AUTO_DOMAIN_UNLIMITED_ROUNDS,
            combat_strategy_path: param.combat_strategy_path.clone(),
            auto_team_strategy_directory: "User/AutoFight/".to_string(),
            party_name: param.party_name.clone(),
            domain_name: param.domain_name.clone(),
            sunday_selected_value: param.sunday_selected_value.clone(),
        },
        config_rule: AutoDomainConfigRule {
            fight_end_delay_seconds: config.auto_domain_config.fight_end_delay,
            short_movement: config.auto_domain_config.short_movement,
            walk_to_f: config.auto_domain_config.walk_to_f,
            left_right_move_times: config.auto_domain_config.left_right_move_times,
            auto_eat_enabled: config.auto_domain_config.auto_eat,
            auto_artifact_salvage_enabled: param.auto_artifact_salvage,
            specify_resin_use: param.specify_resin_use,
            revive_retry_count: config.auto_domain_config.revive_retry_count,
            reward_recognition_enabled: param.reward_recognition_enabled,
        },
        startup_rule: AutoDomainStartupRule {
            logs_screen_resolution: true,
            requires_16_to_9_resolution: true,
            warns_below_1920x1080: true,
            destroys_auto_fight_assets_before_start: true,
            creates_bgi_tree_yolo_predictor: true,
            parses_combat_script_bag_before_start: true,
            adds_auto_eat_realtime_trigger_when_enabled: config.auto_domain_config.auto_eat,
            sends_domain_start_notification: true,
            sends_domain_end_notification: true,
            waits_for_main_ui_after_domain_seconds: 30,
            pre_main_ui_delay_ms: 2_000,
            post_main_ui_delay_ms: 2_000,
        },
        retry_rule: AutoDomainRetryRule {
            revive_retry_count: config.auto_domain_config.revive_retry_count,
            retry_only_when_domain_name_configured: true,
            retry_exception_message_contains: "复活".to_string(),
            retry_delay_ms: 2_000,
            notification_event_on_retry: "DomainRetry".to_string(),
        },
        locators: auto_domain_locators(config.capture_size, config.asset_scale),
        domain_entry_rule: AutoDomainEntryRule {
            teleports_when_domain_name_configured: !param.domain_name.is_empty(),
            uses_domain_position_map: true,
            post_teleport_delay_ms: 1_000,
            waits_for_main_ui_after_teleport: true,
            door_pick_retry_attempts: 20,
            door_pick_retry_interval_ms: 500,
            challenge_menu_retry_attempts: 20,
            challenge_menu_retry_interval_ms: 500,
            enter_domain_retry_attempts: 10,
            enter_domain_retry_interval_ms: 1_000,
            team_ui_retry_attempts: 10,
            team_ui_retry_interval_ms: 1_000,
            start_fight_retry_attempts: 10,
            start_fight_retry_interval_ms: 1_000,
            post_start_fight_delay_ms: 1_000,
            special_domain_movements: vec![
                AutoDomainSpecialMovementRule {
                    domain_name: "芬德尼尔之顶".to_string(),
                    movement: "hold MoveBackward while probing F".to_string(),
                    retry_attempts: 20,
                    retry_interval_ms: 500,
                },
                AutoDomainSpecialMovementRule {
                    domain_name: "无妄引咎密宫".to_string(),
                    movement: "tap MoveForward then hold MoveLeft while probing F".to_string(),
                    retry_attempts: 20,
                    retry_interval_ms: 500,
                },
                AutoDomainSpecialMovementRule {
                    domain_name: "太山府".to_string(),
                    movement: "probe F without movement".to_string(),
                    retry_attempts: 20,
                    retry_interval_ms: 500,
                },
            ],
        },
        sunday_reward_rule: AutoDomainSundayRewardRule {
            enabled_when_sunday_after_4_or_monday_before_4: true,
            enabled_when_limited_open_ocr_matches: true,
            limited_open_patterns: vec!["限时全部开放".to_string(), "限时开放".to_string()],
            artifact_domain_template_skips_reward_selection: true,
            scroll_left_side_first: true,
            scroll_steps: 100,
            scroll_step_delay_ms: 10,
            after_scroll_delay_ms: 400,
            ocr_rect: AutoDomainRelativeRect {
                x_ratio: 0.0,
                y_ratio: 0.0,
                width_ratio: 0.5,
                height_ratio: 1.0,
            },
            ley_line_disorder_pattern: "地脉异常".to_string(),
            selected_value_click_offsets: vec![
                AutoDomainSundayClickOffset {
                    selected_value: 1,
                    y_offset_capture_height_ratio: -0.2,
                },
                AutoDomainSundayClickOffset {
                    selected_value: 2,
                    y_offset_capture_height_ratio: -0.1,
                },
                AutoDomainSundayClickOffset {
                    selected_value: 3,
                    y_offset_capture_height_ratio: 0.0,
                },
            ],
        },
        combat_rule: AutoDomainCombatRule {
            initializes_team_on_first_round: true,
            retries_team_init_before_fight: true,
            switches_to_first_script_avatar: true,
            switch_avatar_sleep_ms: 200,
            walk_to_f_timeout_ms: 60_000,
            walk_forward_start_delay_ms: 30,
            sprint_when_walk_to_f_disabled: !config.auto_domain_config.walk_to_f,
            releases_forward_and_sprint_on_finish: true,
            domain_end_detection_interval_ms: 1_000,
            finish_detection_ocr_engine: "Paddle".to_string(),
            challenge_completed_pattern: "挑战达成".to_string(),
            auto_leaving_pattern: "自动退出".to_string(),
            fight_end_delay_ms,
            auto_eat_low_hp_check_interval_ms: 500,
        },
        petrified_tree_rule: AutoDomainPetrifiedTreeRule {
            yolo_model: "BgiTree".to_string(),
            middle_click_before_search: true,
            after_middle_click_delay_ms: 900,
            locks_camera_to_east: true,
            east_angle_min_degrees: 356.0,
            east_angle_max_degrees: 4.0,
            continuous_east_count_before_move: 5,
            camera_adjust_interval_ms: 100,
            horizontal_move_interval_ms: 60,
            no_detect_switch_threshold: 40,
            left_right_move_times: config.auto_domain_config.left_right_move_times,
            short_movement: config.auto_domain_config.short_movement,
            micro_step_ms: 60,
            after_forward_step_sleep_ms: 500,
            clears_draw_content_on_finish: true,
        },
        reward_rule: AutoDomainRewardRule {
            prompt_initial_delay_ms: 300,
            resin_prompt_ocr_rect: AutoDomainRelativeRect {
                x_ratio: 0.25,
                y_ratio: 0.2,
                width_ratio: 0.5,
                height_ratio: 0.6,
            },
            resin_prompt_retry_attempts: 10,
            resin_prompt_retry_interval_ms: 500,
            after_prompt_detected_delay_ms: 800,
            no_original_resin_texts: vec!["数量不足".to_string(), "补充原粹树脂".to_string()],
            default_mode_uses_condensed_before_original: true,
            default_mode_min_original_resin: 20,
            specified_mode_uses_record_order: true,
            original_resin_type_switch_retry_attempts: 10,
            original_resin_type_switch_retry_interval_ms: 500,
            use_button_double_click_gap_ms: 60,
            sends_reward_notification: true,
            reward_recognition_enabled: param.reward_recognition_enabled,
            continuation_poll_attempts: 30,
            continuation_poll_interval_ms: 300,
            continue_double_click_gap_ms: 60,
            no_resin_challenge_fallback_delay_ms: 900,
            exit_domain_sequence: AutoDomainExitDomainRule {
                first_escape_delay_ms: 500,
                second_escape_delay_ms: 800,
                clicks_black_confirm_button: true,
            },
            normal_end_when_completion_prompt_missing:
                "未检测到秘境结束，可能是背包物品已满。".to_string(),
            stop_reasons: vec![
                AutoDomainStopReason::OriginalResinExhaustedPrompt,
                AutoDomainStopReason::DefaultResinInsufficient,
                AutoDomainStopReason::SpecifiedResinUnavailable,
                AutoDomainStopReason::LastConfiguredRound,
                AutoDomainStopReason::CompletionPromptMissing,
            ],
        },
        resin_rule: AutoDomainResinRule {
            specify_resin_use: param.specify_resin_use,
            default_priority_list: vec!["浓缩树脂".to_string(), "原粹树脂".to_string()],
            configured_priority_list: param.resin_priority_list.clone(),
            specified_records,
            original_resin_20_name: "原粹树脂20".to_string(),
            original_resin_40_name: "原粹树脂40".to_string(),
            original_resin_alias_for_button: "原粹树脂".to_string(),
            transient_resin_name: "须臾树脂".to_string(),
            fragile_resin_name: "脆弱树脂".to_string(),
        },
        artifact_salvage_rule: AutoDomainArtifactSalvageRule {
            enabled: param.auto_artifact_salvage,
            max_artifact_star: param.max_artifact_star.clone(),
            invalid_star_falls_back_to: 4,
            starts_auto_artifact_salvage_task: param.auto_artifact_salvage,
        },
        steps: auto_domain_steps(),
        executor_ready: false,
        pending_native: vec![
            "TaskRunner solo-task lock, main-UI wait, cancellation-aware delay, and notifications"
                .to_string(),
            "domain-name to map-position lookup, teleport execution, and game-window input dispatch"
                .to_string(),
            "live capture, template matching, OCR, Bv page/click helpers, and localized resource lookup"
                .to_string(),
            "CombatScenes visual team recognition, combat script execution loop, and domain-end OCR thread"
                .to_string(),
            "BgiTree YOLO predictor, camera-orientation computation, mouse movement, and overlay cleanup"
                .to_string(),
            "resin status OCR, original-resin 20/40 switch clicks, reward recognition, and artifact salvage execution"
                .to_string(),
        ],
        param,
    })
}

pub fn normalize_domain_round_num(value: i32) -> i32 {
    if value == 0 {
        AUTO_DOMAIN_UNLIMITED_ROUNDS
    } else {
        value
    }
}

pub fn build_domain_resin_records(
    param: &AutoDomainParam,
) -> Result<Vec<AutoDomainResinUseRecord>> {
    if !param.specify_resin_use {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    push_resin_record(&mut records, "浓缩树脂", param.condensed_resin_use_count);
    push_resin_record(&mut records, "原粹树脂40", param.original_resin40_use_count);
    push_resin_record(&mut records, "原粹树脂20", param.original_resin20_use_count);
    push_resin_record(&mut records, "原粹树脂", param.original_resin_use_count);
    push_resin_record(&mut records, "须臾树脂", param.transient_resin_use_count);
    push_resin_record(&mut records, "脆弱树脂", param.fragile_resin_use_count);

    if records.is_empty() {
        return Err(TaskError::InvalidTaskConfig {
            key: AUTO_DOMAIN_TASK_KEY.to_string(),
            message: "指定树脂刷取次数时至少需要配置一种树脂的刷取次数".to_string(),
        });
    }

    Ok(records)
}

fn push_resin_record(records: &mut Vec<AutoDomainResinUseRecord>, name: &str, count: i32) {
    if count > 0 {
        records.push(AutoDomainResinUseRecord {
            name: name.to_string(),
            remain_count: count,
            max_count: count,
        });
    }
}

fn auto_domain_locators(capture_size: Size, asset_scale: f64) -> AutoDomainLocators {
    AutoDomainLocators {
        resin_switch_button: AutoDomainTemplateLocator {
            name: "ResinSwitchBtn".to_string(),
            asset: AUTO_DOMAIN_RESIN_SWITCH_ASSET.to_string(),
            roi: Some(scale_rect(
                Rect {
                    x: 960,
                    y: 430,
                    width: 400,
                    height: 130,
                },
                asset_scale,
            )),
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
        resin_switch_button_disabled: AutoDomainTemplateLocator {
            name: "ResinSwitchBtnNoActive".to_string(),
            asset: AUTO_DOMAIN_RESIN_SWITCH_DISABLED_ASSET.to_string(),
            roi: Some(scale_rect(
                Rect {
                    x: 960,
                    y: 430,
                    width: 400,
                    height: 130,
                },
                asset_scale,
            )),
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
        confirm_button: AutoDomainTemplateLocator {
            name: "Confirm".to_string(),
            asset: AUTO_DOMAIN_CONFIRM_ASSET.to_string(),
            roi: Some(Rect {
                x: (capture_size.width / 2) as i32,
                y: (capture_size.height / 2) as i32,
                width: (capture_size.width / 2) as i32,
                height: (capture_size.height / 2) as i32,
            }),
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
        artifact_flower: AutoDomainTemplateLocator {
            name: "ArtifactArea".to_string(),
            asset: AUTO_DOMAIN_ARTIFACT_FLOWER_ASSET.to_string(),
            roi: Some(Rect {
                x: (capture_size.width / 2) as i32,
                y: 0,
                width: (capture_size.width / 2) as i32,
                height: capture_size.height as i32,
            }),
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
        click_any_close_tip: AutoDomainTemplateLocator {
            name: "ClickAnyCloseTip".to_string(),
            asset: AUTO_DOMAIN_CLICK_ANY_CLOSE_ASSET.to_string(),
            roi: Some(Rect {
                x: 0,
                y: (capture_size.height / 2) as i32,
                width: capture_size.width as i32,
                height: (capture_size.height / 2) as i32,
            }),
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
        exit_button: AutoDomainTemplateLocator {
            name: "Exit".to_string(),
            asset: AUTO_DOMAIN_EXIT_ASSET.to_string(),
            roi: Some(Rect {
                x: 0,
                y: (capture_size.height / 2) as i32,
                width: (capture_size.width / 2) as i32,
                height: (capture_size.height / 2) as i32,
            }),
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
        abnormal_icon: AutoDomainTemplateLocator {
            name: "AbnormalIcon".to_string(),
            asset: AUTO_DOMAIN_ABNORMAL_ICON_ASSET.to_string(),
            roi: Some(Rect {
                x: 0,
                y: (capture_size.height as f64 * 0.08) as i32,
                width: (capture_size.width as f64 * 0.04) as i32,
                height: (capture_size.height as f64 * 0.07) as i32,
            }),
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
        in_domain_icon: AutoDomainTemplateLocator {
            name: "InDomain".to_string(),
            asset: AUTO_DOMAIN_IN_DOMAIN_ASSET.to_string(),
            roi: Some(Rect {
                x: 0,
                y: 0,
                width: (capture_size.width / 4) as i32,
                height: (capture_size.height / 4) as i32,
            }),
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
        party_choose_view: AutoDomainTemplateLocator {
            name: "PartyBtnChooseView".to_string(),
            asset: AUTO_DOMAIN_PARTY_CHOOSE_ASSET.to_string(),
            roi: Some(Rect {
                x: 0,
                y: capture_size.height as i32 - (120.0 * asset_scale) as i32,
                width: (capture_size.width / 7) as i32,
                height: (120.0 * asset_scale) as i32,
            }),
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
        pick_key: AutoDomainTemplateLocator {
            name: "F".to_string(),
            asset: AUTO_DOMAIN_PICK_KEY_ASSET.to_string(),
            roi: None,
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
    }
}

fn auto_domain_steps() -> Vec<AutoDomainTaskStep> {
    use AutoDomainTaskAction::*;
    use AutoDomainTaskPhase::*;

    vec![
        AutoDomainTaskStep {
            phase: Startup,
            action: InitializeAssetsAndConfig,
        },
        AutoDomainTaskStep {
            phase: Startup,
            action: NotifyDomainStart,
        },
        AutoDomainTaskStep {
            phase: Retry,
            action: RetryOnReviveWhenDomainConfigured,
        },
        AutoDomainTaskStep {
            phase: Teleport,
            action: TeleportToDomainWhenConfigured,
        },
        AutoDomainTaskStep {
            phase: EnterDomain,
            action: EnterSinglePlayerChallenge,
        },
        AutoDomainTaskStep {
            phase: RoundLoop,
            action: CloseDomainTip,
        },
        AutoDomainTaskStep {
            phase: RoundLoop,
            action: InitializeTeamOnce,
        },
        AutoDomainTaskStep {
            phase: RoundLoop,
            action: SelectCombatScriptAndFirstAvatar,
        },
        AutoDomainTaskStep {
            phase: Fight,
            action: WalkToKeyAndPressF,
        },
        AutoDomainTaskStep {
            phase: Fight,
            action: RunAutoFightUntilDomainEnd,
        },
        AutoDomainTaskStep {
            phase: Fight,
            action: WaitAfterFight,
        },
        AutoDomainTaskStep {
            phase: PetrifiedTree,
            action: DetectAndCenterPetrifiedTree,
        },
        AutoDomainTaskStep {
            phase: PetrifiedTree,
            action: WalkToTreeAndPressF,
        },
        AutoDomainTaskStep {
            phase: Reward,
            action: ChooseAndUseResin,
        },
        AutoDomainTaskStep {
            phase: Reward,
            action: RecognizeRewardsWhenEnabled,
        },
        AutoDomainTaskStep {
            phase: Reward,
            action: ContinueOrExitDomain,
        },
        AutoDomainTaskStep {
            phase: Finish,
            action: WaitMainUiAndArtifactSalvage,
        },
        AutoDomainTaskStep {
            phase: Finish,
            action: NotifyDomainEnd,
        },
    ]
}

fn apply_auto_domain_config(param: &mut AutoDomainParam, config: &AutoDomainConfig) {
    param.party_name = config.party_name.clone();
    param.domain_name = config.domain_name.clone();
    param.sunday_selected_value = config.sunday_selected_value.clone();
    param.auto_artifact_salvage = config.auto_artifact_salvage;
    param.specify_resin_use = config.specify_resin_use;
    param.resin_priority_list = config.resin_priority_list.clone();
    param.original_resin_use_count = u64_to_i32(config.original_resin_use_count);
    param.original_resin20_use_count = u64_to_i32(config.original_resin20_use_count);
    param.original_resin40_use_count = u64_to_i32(config.original_resin40_use_count);
    param.condensed_resin_use_count = u64_to_i32(config.condensed_resin_use_count);
    param.transient_resin_use_count = u64_to_i32(config.transient_resin_use_count);
    param.fragile_resin_use_count = u64_to_i32(config.fragile_resin_use_count);
    param.reward_recognition_enabled = config.reward_recognition_enabled;
}

fn overlay_auto_domain_param_members(param: &mut AutoDomainParam, value: &Value) {
    if let Some(round_num) = i32_member(
        value,
        ["domainRoundNum", "DomainRoundNum", "domain_round_num"],
    ) {
        param.domain_round_num = normalize_domain_round_num(round_num);
    }
    if let Some(path) = string_member(
        value,
        [
            "combatStrategyPath",
            "CombatStrategyPath",
            "combat_strategy_path",
        ],
    ) {
        param.combat_strategy_path = path;
    }
    if let Some(party_name) = string_member(value, ["partyName", "PartyName", "party_name"]) {
        param.party_name = party_name;
    }
    if let Some(domain_name) = string_member(value, ["domainName", "DomainName", "domain_name"]) {
        param.domain_name = domain_name;
    }
    if let Some(value) = string_member(
        value,
        [
            "sundaySelectedValue",
            "SundaySelectedValue",
            "sunday_selected_value",
        ],
    ) {
        param.sunday_selected_value = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "autoArtifactSalvage",
            "AutoArtifactSalvage",
            "auto_artifact_salvage",
        ],
    ) {
        param.auto_artifact_salvage = value;
    }
    if let Some(value) = string_member(
        value,
        ["maxArtifactStar", "MaxArtifactStar", "max_artifact_star"],
    ) {
        param.max_artifact_star = value;
    }
    if let Some(value) = bool_member(
        value,
        ["specifyResinUse", "SpecifyResinUse", "specify_resin_use"],
    ) {
        param.specify_resin_use = value;
    }
    if let Some(value) = string_vec_member(
        value,
        [
            "resinPriorityList",
            "ResinPriorityList",
            "resin_priority_list",
        ],
    ) {
        param.resin_priority_list = value;
    }
    if let Some(value) = i32_member(
        value,
        [
            "originalResinUseCount",
            "OriginalResinUseCount",
            "original_resin_use_count",
        ],
    ) {
        param.original_resin_use_count = value;
    }
    if let Some(value) = i32_member(
        value,
        [
            "originalResin20UseCount",
            "OriginalResin20UseCount",
            "original_resin20_use_count",
        ],
    ) {
        param.original_resin20_use_count = value;
    }
    if let Some(value) = i32_member(
        value,
        [
            "originalResin40UseCount",
            "OriginalResin40UseCount",
            "original_resin40_use_count",
        ],
    ) {
        param.original_resin40_use_count = value;
    }
    if let Some(value) = i32_member(
        value,
        [
            "condensedResinUseCount",
            "CondensedResinUseCount",
            "condensed_resin_use_count",
        ],
    ) {
        param.condensed_resin_use_count = value;
    }
    if let Some(value) = i32_member(
        value,
        [
            "transientResinUseCount",
            "TransientResinUseCount",
            "transient_resin_use_count",
        ],
    ) {
        param.transient_resin_use_count = value;
    }
    if let Some(value) = i32_member(
        value,
        [
            "fragileResinUseCount",
            "FragileResinUseCount",
            "fragile_resin_use_count",
        ],
    ) {
        param.fragile_resin_use_count = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "rewardRecognitionEnabled",
            "RewardRecognitionEnabled",
            "reward_recognition_enabled",
        ],
    ) {
        param.reward_recognition_enabled = value;
    }
}

fn u64_to_i32(value: u64) -> i32 {
    value.min(i32::MAX as u64) as i32
}

fn scale_rect(rect: Rect, scale: f64) -> Rect {
    Rect {
        x: (rect.x as f64 * scale) as i32,
        y: (rect.y as f64 * scale) as i32,
        width: (rect.width as f64 * scale) as i32,
        height: (rect.height as f64 * scale) as i32,
    }
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn i32_member_from(primary: Option<&Value>, fallback: &Value, names: &[&str]) -> Option<i32> {
    primary
        .and_then(|value| i32_member_slice(value, names))
        .or_else(|| i32_member_slice(fallback, names))
}

fn string_member_from(primary: Option<&Value>, fallback: &Value, names: &[&str]) -> Option<String> {
    primary
        .and_then(|value| string_member_slice(value, names))
        .or_else(|| string_member_slice(fallback, names))
}

fn i32_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<i32> {
    i32_member_slice(value, &names)
}

fn i32_member_slice(value: &Value, names: &[&str]) -> Option<i32> {
    member_value(value, names).and_then(|value| {
        value
            .as_i64()
            .and_then(|value| i32::try_from(value).ok())
            .or_else(|| value.as_u64().and_then(|value| i32::try_from(value).ok()))
            .or_else(|| value.as_str().and_then(|value| value.parse::<i32>().ok()))
    })
}

fn f64_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<f64> {
    member_value(value, &names).and_then(|value| {
        value
            .as_f64()
            .or_else(|| value.as_str().and_then(|value| value.parse::<f64>().ok()))
    })
}

fn string_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<String> {
    string_member_slice(value, &names)
}

fn string_member_slice(value: &Value, names: &[&str]) -> Option<String> {
    member_value(value, names).and_then(|value| value.as_str().map(str::to_string))
}

fn bool_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<bool> {
    member_value(value, &names).and_then(|value| {
        value.as_bool().or_else(|| {
            value
                .as_str()
                .and_then(|value| match value.to_ascii_lowercase().as_str() {
                    "true" => Some(true),
                    "false" => Some(false),
                    _ => None,
                })
        })
    })
}

fn string_vec_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<Vec<String>> {
    member_value(value, &names).and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn member_value<'a>(value: &'a Value, names: &[&str]) -> Option<&'a Value> {
    names.iter().find_map(|name| value.get(*name))
}
