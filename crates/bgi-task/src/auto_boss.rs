use bgi_core::AutoBossConfig;
use bgi_vision::{Rect, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

use crate::task_params::{combat_strategy_path, AutoBossParam, AUTO_STRATEGY_NAME};
use crate::{Result, TaskError};

pub const AUTO_BOSS_TASK_KEY: &str = "AutoBoss";
pub const AUTO_BOSS_DISPLAY_NAME: &str = "自动首领讨伐";
pub const AUTO_BOSS_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_BOSS_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_BOSS_ORIGINAL_RESIN_COST: i32 = 40;
pub const AUTO_BOSS_ORIGINAL_RESIN_RECOVERY_INTERVAL_MINUTES: u64 = 8;
pub const AUTO_BOSS_MAX_QUICK_USE_QUANTITY: i32 = 20;
pub const AUTO_BOSS_PATHING_ASSET_DIR: &str = "GameTask/AutoBoss/Assets/Pathing";
pub const AUTO_BOSS_ORIGINAL_RESIN_TOP_ICON_ASSET: &str = "AutoBoss:original_resin_top_icon.png";
pub const AUTO_BOSS_REWARD_BOX_ASSET: &str = "AutoBoss:box.png";
pub const AUTO_BOSS_OPEN_RESIN_SUPPLEMENT_PANE_BUTTON_ASSET: &str =
    "AutoBoss:open_resin_supplement_pane_button.png";
pub const AUTO_BOSS_TRANSIENT_RESIN_ASSET: &str = "AutoBoss:transient_resin_in_supplement_pane.png";
pub const AUTO_BOSS_FRAGILE_RESIN_ASSET: &str = "AutoBoss:fragile_resin_in_supplement_pane.png";
pub const AUTO_BOSS_INCREASE_RESIN_QUANTITY_BUTTON_ASSET: &str =
    "AutoBoss:increase_resin_usage_quantity_button.png";

pub const AUTO_BOSS_TALK_TO_START_BOSSES: &[&str] =
    &["歌裴莉娅的葬送", "科培琉司的劫罚", "纯水精灵", "重拳出击鸭"];

pub const AUTO_BOSS_NO_PATHING_SUPPORT_BOSSES: &[&str] =
    &["蕴光月守宫", "超重型陆巡舰·机动战垒", "蕴光月幻蝶"];

const AUTO_BOSS_COUNTRY_TO_BOSSES: &[(&str, &[&str])] = &[
    ("蒙德", &["急冻树", "无相之雷", "守望者·堕天"]),
    (
        "璃月",
        &[
            "爆炎树",
            "纯水精灵",
            "古岩龙蜥",
            "无相之岩",
            "遗迹巨蛇",
            "隐山猊兽",
        ],
    ),
    (
        "稻妻",
        &[
            "无相之火",
            "恒常机关阵列",
            "雷音权现",
            "魔偶剑鬼",
            "无相之水",
        ],
    ),
    (
        "须弥",
        &[
            "掣电树",
            "半永恒统辖矩阵",
            "翠翎恐蕈",
            "风蚀沙虫",
            "无相之草",
            "深罪浸礼者",
            "兆载永劫龙兽",
        ],
    ),
    (
        "枫丹",
        &[
            "歌裴莉娅的葬送",
            "科培琉司的劫罚",
            "实验性场力发生装置",
            "魔像督军",
            "千年珍珠骏麟",
            "水形幻人",
            "铁甲熔火帝皇",
        ],
    ),
    (
        "纳塔",
        &[
            "金焰绒翼龙暴君",
            "灵觉隐修的迷者",
            "秘源机兵·构型械",
            "秘源机兵·统御械",
            "熔岩辉龙像",
            "深邃摹结株",
            "贪食匿叶龙山王",
        ],
    ),
    (
        "挪德卡莱",
        &[
            "蕴光月守宫",
            "深黯魇语之主",
            "超重型陆巡舰·机动战垒",
            "霜夜巡天灵主",
            "蕴光月幻蝶",
            "重拳出击鸭",
        ],
    ),
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub asset_scale: f64,
    pub param: AutoBossParam,
    pub boss_data: AutoBossDataPlan,
    pub validation_rule: AutoBossValidationRule,
    pub startup_rule: AutoBossStartupRule,
    pub loop_rule: AutoBossLoopRule,
    pub pathing_rule: AutoBossPathingRule,
    pub resin_rule: AutoBossResinRule,
    pub supplemental_resin_rule: AutoBossSupplementalResinRule,
    pub combat_rule: AutoBossCombatRule,
    pub reward_navigation_rule: AutoBossRewardNavigationRule,
    pub reward_rule: AutoBossRewardRule,
    pub reposition_rule: AutoBossRepositionRule,
    pub locators: AutoBossLocators,
    pub steps: Vec<AutoBossTaskStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossExecutionConfig {
    pub capture_size: Size,
    pub asset_scale: f64,
    pub param: AutoBossParam,
    pub auto_boss_config: AutoBossConfig,
}

impl Default for AutoBossExecutionConfig {
    fn default() -> Self {
        let auto_boss_config = AutoBossConfig::default();
        let mut param = AutoBossParam::default();
        apply_auto_boss_config(&mut param, &auto_boss_config);
        Self {
            capture_size: Size::new(
                AUTO_BOSS_DEFAULT_CAPTURE_WIDTH,
                AUTO_BOSS_DEFAULT_CAPTURE_HEIGHT,
            ),
            asset_scale: 1.0,
            param,
            auto_boss_config,
        }
    }
}

impl AutoBossExecutionConfig {
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

        let auto_boss_value = value
            .get("autoBossConfig")
            .or_else(|| value.get("AutoBossConfig"))
            .or_else(|| value.get("auto_boss_config"))
            .unwrap_or(value);
        config.auto_boss_config =
            serde_json::from_value(auto_boss_value.clone()).unwrap_or_default();

        let strategy_name = string_member_from(
            value
                .get("param")
                .or_else(|| value.get("Param"))
                .or_else(|| value.get("autoBossParam"))
                .or_else(|| value.get("AutoBossParam"))
                .or_else(|| value.get("auto_boss_param")),
            value,
            &["strategyName", "StrategyName", "strategy_name"],
        )
        .or_else(|| {
            Some(config.auto_boss_config.strategy_name.clone()).filter(|value| !value.is_empty())
        });
        let mut param = AutoBossParam::new(strategy_name.as_deref());
        apply_auto_boss_config(&mut param, &config.auto_boss_config);
        overlay_auto_boss_param_members(&mut param, value);
        if let Some(param_value) = value
            .get("param")
            .or_else(|| value.get("Param"))
            .or_else(|| value.get("autoBossParam"))
            .or_else(|| value.get("AutoBossParam"))
            .or_else(|| value.get("auto_boss_param"))
        {
            overlay_auto_boss_param_members(&mut param, param_value);
        }
        if param.combat_strategy_path.trim().is_empty() {
            param.combat_strategy_path = combat_strategy_path(Some(&param.strategy_name));
        }
        config.param = param;
        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossDataPlan {
    pub country_to_bosses: Vec<AutoBossCountryBosses>,
    pub supported_boss_count: usize,
    pub selected_boss_supported: bool,
    pub selected_boss_country: Option<String>,
    pub selected_boss_talk_to_start: bool,
    pub selected_boss_no_pathing_support: bool,
    pub talk_to_start_bosses: Vec<String>,
    pub no_pathing_support_bosses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossCountryBosses {
    pub country: String,
    pub bosses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossValidationRule {
    pub requires_boss_name: bool,
    pub requires_supported_boss: bool,
    pub requires_non_negative_revive_retry_count: bool,
    pub requires_positive_run_count_when_specified: bool,
    pub supplemental_resin_only_allowed_with_specified_run_count: bool,
    pub requires_existing_combat_strategy_file_or_directory: bool,
    pub requires_existing_route_files: Vec<String>,
    pub requires_16_to_9_resolution: bool,
    pub warns_below_1920x1080: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossStartupRule {
    pub parses_combat_script_bag_before_start: bool,
    pub logs_screen_resolution: bool,
    pub sends_start_notification: bool,
    pub sends_end_notification: bool,
    pub outer_retry_exception_respects_revive_retry_count: bool,
    pub retry_delay_ms: u64,
    pub releases_all_keys_on_finish: bool,
    pub releases_left_mouse_on_finish: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossLoopRule {
    pub prepares_main_ui_before_loop: bool,
    pub switches_party_when_team_name_configured: bool,
    pub specified_run_count_stops_after_rewards: bool,
    pub unspecified_run_count_runs_until_resin_exhausted: bool,
    pub reward_count_increments_only_after_successful_reward: bool,
    pub return_to_statue_after_each_round_option: bool,
    pub statue_delay_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossPathingRule {
    pub pathing_asset_directory: String,
    pub required_route_files: Vec<String>,
    pub first_navigation_files: Vec<String>,
    pub no_pathing_support_uses_force_teleport_and_key_mouse: bool,
    pub normal_boss_uses_go_to_route: bool,
    pub pathing_party_skip_party_switch: bool,
    pub pathing_party_auto_fight_enabled: bool,
    pub runs_return_main_ui_before_pathing_file: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossResinRule {
    pub original_resin_cost: i32,
    pub resin_recovery_interval_minutes: u64,
    pub precheck_opens_big_map: bool,
    pub precheck_returns_main_ui_finally: bool,
    pub resin_icon_search_rect: Rect,
    pub resin_count_ocr_rect_offset_from_icon_right: Rect,
    pub recovery_detail_rect_offset_from_icon: Rect,
    pub precheck_failure_falls_back_to_reward_prompt: bool,
    pub insufficient_resin_stops_when_not_specified_run_count: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossSupplementalResinRule {
    pub enabled_resin_options: Vec<AutoBossSupplementalResinOption>,
    pub target_quantity_formula: String,
    pub max_quick_use_quantity: i32,
    pub title_rect: Rect,
    pub open_button_roi: Rect,
    pub icon_roi: Rect,
    pub selected_name_rect: Rect,
    pub quick_use_title_rect: Rect,
    pub quick_use_available_count_rect: Rect,
    pub quick_use_quantity_rect: Rect,
    pub increase_button_roi: Rect,
    pub quick_use_retry_base_attempts: i32,
    pub quick_use_retry_multiplier: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossSupplementalResinOption {
    pub name: String,
    pub asset: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossCombatRule {
    pub initializes_team_with_retry: bool,
    pub team_initialization_retry_attempts: u64,
    pub team_initialization_retry_interval_ms: u64,
    pub switches_to_first_script_avatar_before_fight: bool,
    pub switch_avatar_sleep_ms: u64,
    pub auto_fight_finish_detection_enabled: bool,
    pub pick_drops_after_fight_enabled: bool,
    pub kazuha_pickup_enabled: bool,
    pub qin_double_pickup_enabled: bool,
    pub exp_based_pickup_enabled: bool,
    pub battle_threshold_for_loot: i32,
    pub only_pick_elite_drops_mode: String,
    pub normal_end_exception_is_logged: bool,
    pub calls_combat_scenes_after_task: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossRewardNavigationRule {
    pub navigation_timeout_seconds: u64,
    pub reward_prompt_ocr_rect: Rect,
    pub reward_box_kept_between_screen_x_ratio_min: f64,
    pub reward_box_kept_between_screen_x_ratio_max: f64,
    pub camera_missing_icon_move_x: i32,
    pub camera_retry_interval_ms: u64,
    pub climb_detection_rect: Rect,
    pub climb_escape_drop_delay_ms: u64,
    pub climb_escape_left_hold_ms: u64,
    pub move_forward_burst_ms: u64,
    pub jump_every_forward_bursts: u64,
    pub post_jump_delay_ms: u64,
    pub post_forward_release_delay_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossRewardRule {
    pub interact_reward_rect: Rect,
    pub interact_wait_ms: u64,
    pub use_original_resin_rect: Rect,
    pub use_original_resin_timeout_ms: u64,
    pub post_use_original_resin_delay_ms: u64,
    pub supplement_prompt_wait_ms: u64,
    pub reward_recognition_enabled: bool,
    pub reward_ready_close_rect: Rect,
    pub reward_ready_retry_attempts: u64,
    pub reward_ready_retry_interval_ms: u64,
    pub close_result_retry_attempts: u64,
    pub close_result_retry_interval_ms: u64,
    pub click_center_after_attempt: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossRepositionRule {
    pub talk_to_start_uses_after_fight_quick_route: bool,
    pub no_pathing_support_reruns_special_navigation: bool,
    pub normal_boss_replays_last_route_position: bool,
    pub normal_boss_post_reposition_delay_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossLocators {
    pub original_resin_top_icon: AutoBossTemplateLocator,
    pub reward_box: AutoBossTemplateLocator,
    pub open_resin_supplement_pane_button: AutoBossTemplateLocator,
    pub transient_resin: AutoBossTemplateLocator,
    pub fragile_resin: AutoBossTemplateLocator,
    pub increase_resin_usage_quantity_button: AutoBossTemplateLocator,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Option<Rect>,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossTaskStep {
    pub phase: AutoBossTaskPhase,
    pub action: AutoBossTaskAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoBossTaskPhase {
    Startup,
    Prepare,
    Resin,
    Navigation,
    Combat,
    Reward,
    Reposition,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoBossTaskAction {
    ValidateAndParseCombatStrategy,
    LogScreenResolution,
    ReturnMainUiAndSwitchParty,
    CheckOriginalResin,
    UseSupplementalResinWhenAllowed,
    NavigateToBoss,
    RunAutoFight,
    MoveToRewardFlower,
    TakeReward,
    RecognizeRewardWhenEnabled,
    RepositionForNextRound,
    ReleaseInputsAndNotifyEnd,
}

pub fn plan_auto_boss(
    working_directory: impl AsRef<Path>,
    config: AutoBossExecutionConfig,
) -> Result<AutoBossExecutionPlan> {
    let working_directory = working_directory.as_ref();
    let mut param = config.param;
    normalize_auto_boss_param_strategy(&mut param);
    validate_auto_boss_param(working_directory, &param)?;

    let required_route_files = auto_boss_required_route_files(&param.boss_name);
    let first_navigation_files = auto_boss_first_navigation_files(&param.boss_name);

    Ok(AutoBossExecutionPlan {
        task_key: AUTO_BOSS_TASK_KEY.to_string(),
        display_name: AUTO_BOSS_DISPLAY_NAME.to_string(),
        capture_size: config.capture_size,
        asset_scale: config.asset_scale,
        boss_data: auto_boss_data_plan(&param.boss_name),
        validation_rule: AutoBossValidationRule {
            requires_boss_name: true,
            requires_supported_boss: true,
            requires_non_negative_revive_retry_count: true,
            requires_positive_run_count_when_specified: true,
            supplemental_resin_only_allowed_with_specified_run_count: true,
            requires_existing_combat_strategy_file_or_directory: true,
            requires_existing_route_files: required_route_files.clone(),
            requires_16_to_9_resolution: true,
            warns_below_1920x1080: true,
        },
        startup_rule: AutoBossStartupRule {
            parses_combat_script_bag_before_start: true,
            logs_screen_resolution: true,
            sends_start_notification: true,
            sends_end_notification: true,
            outer_retry_exception_respects_revive_retry_count: true,
            retry_delay_ms: 2_000,
            releases_all_keys_on_finish: true,
            releases_left_mouse_on_finish: true,
        },
        loop_rule: AutoBossLoopRule {
            prepares_main_ui_before_loop: true,
            switches_party_when_team_name_configured: !param.team_name.trim().is_empty(),
            specified_run_count_stops_after_rewards: param.specify_run_count,
            unspecified_run_count_runs_until_resin_exhausted: !param.specify_run_count,
            reward_count_increments_only_after_successful_reward: true,
            return_to_statue_after_each_round_option: param.return_to_statue_after_each_round,
            statue_delay_ms: 3_000,
        },
        pathing_rule: AutoBossPathingRule {
            pathing_asset_directory: AUTO_BOSS_PATHING_ASSET_DIR.to_string(),
            required_route_files,
            first_navigation_files,
            no_pathing_support_uses_force_teleport_and_key_mouse: auto_boss_is_no_pathing_support(
                &param.boss_name,
            ),
            normal_boss_uses_go_to_route: !auto_boss_is_no_pathing_support(&param.boss_name),
            pathing_party_skip_party_switch: true,
            pathing_party_auto_fight_enabled: false,
            runs_return_main_ui_before_pathing_file: true,
        },
        resin_rule: AutoBossResinRule {
            original_resin_cost: AUTO_BOSS_ORIGINAL_RESIN_COST,
            resin_recovery_interval_minutes: AUTO_BOSS_ORIGINAL_RESIN_RECOVERY_INTERVAL_MINUTES,
            precheck_opens_big_map: true,
            precheck_returns_main_ui_finally: true,
            resin_icon_search_rect: Rect {
                x: 1200,
                y: 25,
                width: 250,
                height: 50,
            },
            resin_count_ocr_rect_offset_from_icon_right: Rect {
                x: 25,
                y: 37,
                width: 120,
                height: 24,
            },
            recovery_detail_rect_offset_from_icon: Rect {
                x: -13,
                y: 29,
                width: 220,
                height: 150,
            },
            precheck_failure_falls_back_to_reward_prompt: true,
            insufficient_resin_stops_when_not_specified_run_count: true,
        },
        supplemental_resin_rule: AutoBossSupplementalResinRule {
            enabled_resin_options: supplemental_resin_options(&param),
            target_quantity_formula: "max(0, (resin_limit - resin_count) / 60)".to_string(),
            max_quick_use_quantity: AUTO_BOSS_MAX_QUICK_USE_QUANTITY,
            title_rect: Rect {
                x: 834,
                y: 247,
                width: 256,
                height: 60,
            },
            open_button_roi: Rect {
                x: 1200,
                y: 25,
                width: 250,
                height: 50,
            },
            icon_roi: Rect {
                x: 644,
                y: 378,
                width: 620,
                height: 192,
            },
            selected_name_rect: Rect {
                x: 906,
                y: 587,
                width: 110,
                height: 31,
            },
            quick_use_title_rect: Rect {
                x: 875,
                y: 269,
                width: 184,
                height: 63,
            },
            quick_use_available_count_rect: Rect {
                x: 1191,
                y: 633,
                width: 72,
                height: 29,
            },
            quick_use_quantity_rect: Rect {
                x: 915,
                y: 540,
                width: 93,
                height: 81,
            },
            increase_button_roi: Rect {
                x: 1265,
                y: 620,
                width: 59,
                height: 55,
            },
            quick_use_retry_base_attempts: 6,
            quick_use_retry_multiplier: 3,
        },
        combat_rule: AutoBossCombatRule {
            initializes_team_with_retry: true,
            team_initialization_retry_attempts: 5,
            team_initialization_retry_interval_ms: 1_000,
            switches_to_first_script_avatar_before_fight: true,
            switch_avatar_sleep_ms: 200,
            auto_fight_finish_detection_enabled: true,
            pick_drops_after_fight_enabled: false,
            kazuha_pickup_enabled: false,
            qin_double_pickup_enabled: false,
            exp_based_pickup_enabled: false,
            battle_threshold_for_loot: -1,
            only_pick_elite_drops_mode: "DisableAutoPickupForNonElite".to_string(),
            normal_end_exception_is_logged: true,
            calls_combat_scenes_after_task: true,
        },
        reward_navigation_rule: AutoBossRewardNavigationRule {
            navigation_timeout_seconds: 15,
            reward_prompt_ocr_rect: Rect {
                x: 1210,
                y: 300,
                width: 200,
                height: 400,
            },
            reward_box_kept_between_screen_x_ratio_min: 0.45,
            reward_box_kept_between_screen_x_ratio_max: 0.55,
            camera_missing_icon_move_x: 200,
            camera_retry_interval_ms: 250,
            climb_detection_rect: Rect {
                x: 1686,
                y: 1030,
                width: 60,
                height: 23,
            },
            climb_escape_drop_delay_ms: 1_000,
            climb_escape_left_hold_ms: 800,
            move_forward_burst_ms: 1_000,
            jump_every_forward_bursts: 2,
            post_jump_delay_ms: 100,
            post_forward_release_delay_ms: 200,
        },
        reward_rule: AutoBossRewardRule {
            interact_reward_rect: Rect {
                x: 1210,
                y: 515,
                width: 200,
                height: 50,
            },
            interact_wait_ms: 800,
            use_original_resin_rect: Rect {
                x: 850,
                y: 740,
                width: 250,
                height: 35,
            },
            use_original_resin_timeout_ms: 3_000,
            post_use_original_resin_delay_ms: 1_000,
            supplement_prompt_wait_ms: 1_000,
            reward_recognition_enabled: param.reward_recognition_enabled,
            reward_ready_close_rect: Rect {
                x: 850,
                y: 960,
                width: 220,
                height: 35,
            },
            reward_ready_retry_attempts: 20,
            reward_ready_retry_interval_ms: 300,
            close_result_retry_attempts: 20,
            close_result_retry_interval_ms: 300,
            click_center_after_attempt: 5,
        },
        reposition_rule: AutoBossRepositionRule {
            talk_to_start_uses_after_fight_quick_route: auto_boss_is_talk_to_start(
                &param.boss_name,
            ),
            no_pathing_support_reruns_special_navigation: auto_boss_is_no_pathing_support(
                &param.boss_name,
            ),
            normal_boss_replays_last_route_position: !auto_boss_is_talk_to_start(&param.boss_name)
                && !auto_boss_is_no_pathing_support(&param.boss_name),
            normal_boss_post_reposition_delay_ms: 4_000,
        },
        locators: auto_boss_locators(),
        steps: auto_boss_steps(),
        executor_ready: false,
        pending_native: vec![
            "live capture, 16:9 game window probing, BvPage OCR/template matching, and Rust AutoBoss asset locator initialization".to_string(),
            "TpTask big-map resin probing, supplemental resin UI clicking/OCR, ReturnMainUi/SwitchParty common jobs, and statue teleport".to_string(),
            "PathExecutor, KeyMouseMacroPlayer, CombatScenes/AutoFightTask dispatch, reward flower camera/input navigation, RewardResultRecognizer, notifications, and cancellation/retry boundaries".to_string(),
        ],
        param,
    })
}

pub fn auto_boss_supported_boss_names() -> Vec<String> {
    AUTO_BOSS_COUNTRY_TO_BOSSES
        .iter()
        .flat_map(|(_, bosses)| bosses.iter().copied())
        .map(str::to_string)
        .collect()
}

pub fn auto_boss_is_supported(boss_name: &str) -> bool {
    AUTO_BOSS_COUNTRY_TO_BOSSES
        .iter()
        .any(|(_, bosses)| bosses.contains(&boss_name))
}

pub fn auto_boss_is_talk_to_start(boss_name: &str) -> bool {
    AUTO_BOSS_TALK_TO_START_BOSSES.contains(&boss_name)
}

pub fn auto_boss_is_no_pathing_support(boss_name: &str) -> bool {
    AUTO_BOSS_NO_PATHING_SUPPORT_BOSSES.contains(&boss_name)
}

pub fn auto_boss_required_route_files(boss_name: &str) -> Vec<String> {
    if auto_boss_is_no_pathing_support(boss_name) {
        return vec![
            format!("{boss_name}强制传送.json"),
            format!("{boss_name}键鼠前往.json"),
        ];
    }

    let mut routes = vec![format!("{boss_name}前往.json")];
    if auto_boss_is_talk_to_start(boss_name) {
        routes.push(format!("{boss_name}战斗后快速前往.json"));
    }
    routes
}

pub fn auto_boss_first_navigation_files(boss_name: &str) -> Vec<String> {
    if auto_boss_is_no_pathing_support(boss_name) {
        vec![
            format!("{boss_name}强制传送.json"),
            format!("{boss_name}键鼠前往.json"),
        ]
    } else {
        vec![format!("{boss_name}前往.json")]
    }
}

fn validate_auto_boss_param(working_directory: &Path, param: &AutoBossParam) -> Result<()> {
    if param.boss_name.trim().is_empty() {
        return invalid_auto_boss_config("请选择需要讨伐的首领");
    }
    if !auto_boss_is_supported(&param.boss_name) {
        return invalid_auto_boss_config(format!("暂不支持首领：{}", param.boss_name));
    }
    if param.revive_retry_count < 0 {
        return invalid_auto_boss_config("角色死亡后重试次数不能小于 0");
    }
    if param.specify_run_count && param.run_count < 1 {
        return invalid_auto_boss_config("指定讨伐次数必须大于 0");
    }
    if !param.specify_run_count && (param.use_transient_resin || param.use_fragile_resin) {
        return invalid_auto_boss_config("只有指定讨伐次数模式才能开启须臾树脂或脆弱树脂补充");
    }

    let strategy_path = working_directory.join(&param.combat_strategy_path);
    if !strategy_path.exists() {
        return invalid_auto_boss_config("当前选择的自动战斗策略文件不存在");
    }

    for route in auto_boss_required_route_files(&param.boss_name) {
        let path = auto_boss_pathing_asset_path(working_directory, &route);
        if !path.exists() {
            return Err(TaskError::InvalidTaskConfig {
                key: AUTO_BOSS_TASK_KEY.to_string(),
                message: format!("未找到首领路线文件：{route}"),
            });
        }
    }
    Ok(())
}

fn auto_boss_pathing_asset_path(working_directory: &Path, route: &str) -> PathBuf {
    working_directory
        .join(AUTO_BOSS_PATHING_ASSET_DIR)
        .join(route)
}

fn invalid_auto_boss_config<T>(message: impl Into<String>) -> Result<T> {
    Err(TaskError::InvalidTaskConfig {
        key: AUTO_BOSS_TASK_KEY.to_string(),
        message: message.into(),
    })
}

fn normalize_auto_boss_param_strategy(param: &mut AutoBossParam) {
    if param.strategy_name.trim().is_empty() {
        param.strategy_name = AUTO_STRATEGY_NAME.to_string();
    }
    if param.combat_strategy_path.trim().is_empty() {
        param.combat_strategy_path = combat_strategy_path(Some(&param.strategy_name));
    }
}

fn auto_boss_data_plan(selected_boss: &str) -> AutoBossDataPlan {
    let country_to_bosses = AUTO_BOSS_COUNTRY_TO_BOSSES
        .iter()
        .map(|(country, bosses)| AutoBossCountryBosses {
            country: (*country).to_string(),
            bosses: bosses.iter().map(|boss| (*boss).to_string()).collect(),
        })
        .collect::<Vec<_>>();
    let selected_boss_country = AUTO_BOSS_COUNTRY_TO_BOSSES
        .iter()
        .find(|(_, bosses)| bosses.contains(&selected_boss))
        .map(|(country, _)| (*country).to_string());
    let supported_boss_count = AUTO_BOSS_COUNTRY_TO_BOSSES
        .iter()
        .map(|(_, bosses)| bosses.len())
        .sum();

    AutoBossDataPlan {
        country_to_bosses,
        supported_boss_count,
        selected_boss_supported: auto_boss_is_supported(selected_boss),
        selected_boss_country,
        selected_boss_talk_to_start: auto_boss_is_talk_to_start(selected_boss),
        selected_boss_no_pathing_support: auto_boss_is_no_pathing_support(selected_boss),
        talk_to_start_bosses: AUTO_BOSS_TALK_TO_START_BOSSES
            .iter()
            .map(|boss| (*boss).to_string())
            .collect(),
        no_pathing_support_bosses: AUTO_BOSS_NO_PATHING_SUPPORT_BOSSES
            .iter()
            .map(|boss| (*boss).to_string())
            .collect(),
    }
}

fn supplemental_resin_options(param: &AutoBossParam) -> Vec<AutoBossSupplementalResinOption> {
    let mut options = Vec::new();
    if param.use_transient_resin {
        options.push(AutoBossSupplementalResinOption {
            name: "须臾树脂".to_string(),
            asset: AUTO_BOSS_TRANSIENT_RESIN_ASSET.to_string(),
        });
    }
    if param.use_fragile_resin {
        options.push(AutoBossSupplementalResinOption {
            name: "脆弱树脂".to_string(),
            asset: AUTO_BOSS_FRAGILE_RESIN_ASSET.to_string(),
        });
    }
    options
}

fn auto_boss_locators() -> AutoBossLocators {
    AutoBossLocators {
        original_resin_top_icon: template_locator(
            "AutoBossOriginalResinTopIcon",
            AUTO_BOSS_ORIGINAL_RESIN_TOP_ICON_ASSET,
        ),
        reward_box: template_locator("AutoBossRewardBox", AUTO_BOSS_REWARD_BOX_ASSET),
        open_resin_supplement_pane_button: template_locator(
            "AutoBossOpenResinSupplementPaneButton",
            AUTO_BOSS_OPEN_RESIN_SUPPLEMENT_PANE_BUTTON_ASSET,
        ),
        transient_resin: template_locator(
            "AutoBossTransientResinInSupplementPane",
            AUTO_BOSS_TRANSIENT_RESIN_ASSET,
        ),
        fragile_resin: template_locator(
            "AutoBossFragileResinInSupplementPane",
            AUTO_BOSS_FRAGILE_RESIN_ASSET,
        ),
        increase_resin_usage_quantity_button: template_locator(
            "AutoBossIncreaseResinUsageQuantityButton",
            AUTO_BOSS_INCREASE_RESIN_QUANTITY_BUTTON_ASSET,
        ),
    }
}

fn template_locator(name: &str, asset: &str) -> AutoBossTemplateLocator {
    AutoBossTemplateLocator {
        name: name.to_string(),
        asset: asset.to_string(),
        roi: None,
        threshold: 0.8,
        match_mode: TemplateMatchMode::CCoeffNormed,
        draw_on_window: false,
    }
}

fn auto_boss_steps() -> Vec<AutoBossTaskStep> {
    use AutoBossTaskAction::*;
    use AutoBossTaskPhase::*;
    vec![
        AutoBossTaskStep {
            phase: Startup,
            action: ValidateAndParseCombatStrategy,
        },
        AutoBossTaskStep {
            phase: Startup,
            action: LogScreenResolution,
        },
        AutoBossTaskStep {
            phase: Prepare,
            action: ReturnMainUiAndSwitchParty,
        },
        AutoBossTaskStep {
            phase: Resin,
            action: CheckOriginalResin,
        },
        AutoBossTaskStep {
            phase: Resin,
            action: UseSupplementalResinWhenAllowed,
        },
        AutoBossTaskStep {
            phase: Navigation,
            action: NavigateToBoss,
        },
        AutoBossTaskStep {
            phase: Combat,
            action: RunAutoFight,
        },
        AutoBossTaskStep {
            phase: Reward,
            action: MoveToRewardFlower,
        },
        AutoBossTaskStep {
            phase: Reward,
            action: TakeReward,
        },
        AutoBossTaskStep {
            phase: Reward,
            action: RecognizeRewardWhenEnabled,
        },
        AutoBossTaskStep {
            phase: Reposition,
            action: RepositionForNextRound,
        },
        AutoBossTaskStep {
            phase: Cleanup,
            action: ReleaseInputsAndNotifyEnd,
        },
    ]
}

fn apply_auto_boss_config(param: &mut AutoBossParam, config: &AutoBossConfig) {
    param.boss_name = config.boss_name.clone();
    param.set_strategy_name(Some(&config.strategy_name));
    param.team_name = config.team_name.clone();
    param.specify_run_count = config.specify_run_count;
    param.run_count = config.run_count as i32;
    param.use_transient_resin = config.use_transient_resin;
    param.use_fragile_resin = config.use_fragile_resin;
    param.revive_retry_count = config.revive_retry_count as i32;
    param.return_to_statue_after_each_round = config.return_to_statue_after_each_round;
    param.reward_recognition_enabled = config.reward_recognition_enabled;
}

fn overlay_auto_boss_param_members(param: &mut AutoBossParam, value: &Value) {
    if let Some(boss_name) = string_member(
        value,
        [
            "bossName",
            "BossName",
            "boss_name",
            "autoBossName",
            "AutoBossName",
        ],
    ) {
        param.boss_name = boss_name;
    }
    if let Some(strategy_name) =
        string_member(value, ["strategyName", "StrategyName", "strategy_name"])
    {
        param.set_strategy_name(Some(&strategy_name));
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
    if let Some(team_name) = string_member(value, ["teamName", "TeamName", "team_name"]) {
        param.team_name = team_name;
    }
    if let Some(value) = bool_member(
        value,
        ["specifyRunCount", "SpecifyRunCount", "specify_run_count"],
    ) {
        param.specify_run_count = value;
    }
    if let Some(value) = i32_member(value, ["runCount", "RunCount", "run_count"]) {
        param.run_count = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "useTransientResin",
            "UseTransientResin",
            "use_transient_resin",
        ],
    ) {
        param.use_transient_resin = value;
    }
    if let Some(value) = bool_member(
        value,
        ["useFragileResin", "UseFragileResin", "use_fragile_resin"],
    ) {
        param.use_fragile_resin = value;
    }
    if let Some(value) = i32_member(
        value,
        ["reviveRetryCount", "ReviveRetryCount", "revive_retry_count"],
    ) {
        param.revive_retry_count = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "returnToStatueAfterEachRound",
            "ReturnToStatueAfterEachRound",
            "return_to_statue_after_each_round",
        ],
    ) {
        param.return_to_statue_after_each_round = value;
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

fn capture_size_from_value(value: &Value) -> Option<Size> {
    value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn string_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<String> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(|value| value.as_str().map(str::to_string))
}

fn string_member_from<const N: usize>(
    primary: Option<&Value>,
    fallback: &Value,
    names: &[&str; N],
) -> Option<String> {
    primary
        .and_then(|value| {
            names
                .iter()
                .find_map(|name| value.get(*name))
                .and_then(|value| value.as_str().map(str::to_string))
        })
        .or_else(|| {
            names
                .iter()
                .find_map(|name| fallback.get(*name))
                .and_then(|value| value.as_str().map(str::to_string))
        })
}

fn bool_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<bool> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(Value::as_bool)
}

fn i32_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<i32> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(|value| value.as_i64())
        .and_then(|value| i32::try_from(value).ok())
}

fn f64_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<f64> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(Value::as_f64)
}
