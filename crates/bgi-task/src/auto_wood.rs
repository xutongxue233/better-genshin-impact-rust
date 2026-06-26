use bgi_core::AutoWoodConfig;
use bgi_vision::{Rect, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const AUTO_WOOD_TASK_KEY: &str = "AutoWood";
pub const AUTO_WOOD_DISPLAY_NAME: &str = "自动伐木";
pub const AUTO_WOOD_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_WOOD_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_WOOD_DEFAULT_ROUND_NUM_RAW: u64 = 0;
pub const AUTO_WOOD_DEFAULT_DAILY_MAX_COUNT_RAW: u64 = 2000;
pub const AUTO_WOOD_UNLIMITED_COUNT: u64 = 9999;
pub const AUTO_WOOD_THE_BOON_ASSET: &str = "AutoWood:TheBoonOfTheElderTree.png";
pub const AUTO_WOOD_MENU_BAG_ASSET: &str = "AutoWood:menu_bag.png";
pub const AUTO_WOOD_CONFIRM_ASSET: &str = "AutoWood:confirm.png";
pub const AUTO_WOOD_ENTER_GAME_ASSET: &str = "AutoWood:exit_welcome.png";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoWoodExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub asset_scale: f64,
    pub param_rule: AutoWoodParamRule,
    pub config_rule: AutoWoodConfigRule,
    pub startup_rule: AutoWoodStartupRule,
    pub locators: AutoWoodLocators,
    pub press_gadget_rule: AutoWoodPressGadgetRule,
    pub ocr_rule: AutoWoodOcrRule,
    pub refresh_rule: AutoWoodRefreshRule,
    pub legacy_exit_enter_rule: AutoWoodLegacyExitEnterRule,
    pub third_party_login_rule: AutoWoodThirdPartyLoginRule,
    pub loop_rule: AutoWoodLoopRule,
    pub steps: Vec<AutoWoodTaskStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodExecutionConfig {
    pub capture_size: Size,
    pub asset_scale: f64,
    pub auto_wood_config: AutoWoodConfig,
    pub wood_round_num: u64,
    pub wood_daily_max_count: u64,
}

impl Default for AutoWoodExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_WOOD_DEFAULT_CAPTURE_WIDTH,
                AUTO_WOOD_DEFAULT_CAPTURE_HEIGHT,
            ),
            asset_scale: 1.0,
            auto_wood_config: AutoWoodConfig::default(),
            wood_round_num: AUTO_WOOD_DEFAULT_ROUND_NUM_RAW,
            wood_daily_max_count: AUTO_WOOD_DEFAULT_DAILY_MAX_COUNT_RAW,
        }
    }
}

impl AutoWoodExecutionConfig {
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
        if let Some(round_num) = u64_member(
            value,
            [
                "woodRoundNum",
                "WoodRoundNum",
                "autoWoodRoundNum",
                "AutoWoodRoundNum",
                "roundNum",
                "round_num",
            ],
        ) {
            config.wood_round_num = round_num;
        }
        if let Some(daily_max_count) = u64_member(
            value,
            [
                "woodDailyMaxCount",
                "WoodDailyMaxCount",
                "autoWoodDailyMaxCount",
                "AutoWoodDailyMaxCount",
                "dailyMaxCount",
                "daily_max_count",
            ],
        ) {
            config.wood_daily_max_count = daily_max_count;
        }

        let auto_wood_value = value
            .get("autoWoodConfig")
            .or_else(|| value.get("AutoWoodConfig"))
            .or_else(|| value.get("auto_wood_config"))
            .unwrap_or(value);
        config.auto_wood_config =
            serde_json::from_value(auto_wood_value.clone()).unwrap_or_default();
        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoWoodParamRule {
    pub raw_round_num: u64,
    pub normalized_round_num: u64,
    pub raw_daily_max_count: u64,
    pub normalized_daily_max_count: u64,
    pub unlimited_sentinel: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoWoodConfigRule {
    pub after_z_sleep_delay_ms: u64,
    pub wood_count_ocr_enabled: bool,
    pub use_wonderland_refresh: bool,
    pub press_two_esc_is_legacy_commented_out: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoWoodStartupRule {
    pub destroys_asset_singleton_before_start: bool,
    pub initializes_auto_wood_assets: bool,
    pub creates_wonderland_cycle_job: bool,
    pub prevents_system_sleep: bool,
    pub restores_execution_state_on_finish: bool,
    pub refreshes_third_party_login_mode: bool,
    pub activates_game_window_before_loop: bool,
    pub clears_draw_content_after_each_round: bool,
    pub post_round_sleep_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoWoodLocators {
    pub wood_count_upper_rect: Rect,
    pub the_boon_of_the_elder_tree: AutoWoodTemplateLocator,
    pub menu_bag: AutoWoodTemplateLocator,
    pub confirm: AutoWoodTemplateLocator,
    pub enter_game: AutoWoodTemplateLocator,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoWoodTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Option<Rect>,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoWoodPressGadgetRule {
    pub focuses_game_window_before_press: bool,
    pub first_round_requires_boon_template: bool,
    pub missing_first_boon_ends_normally: bool,
    pub later_round_retry_interval_ms: u64,
    pub later_round_retry_attempts: u64,
    pub retry_pre_capture_sleep_ms: u64,
    pub action: AutoWoodInputAction,
    pub later_round_post_press_sleep_ms: u64,
    pub post_press_base_sleep_ms: u64,
    pub post_press_extra_sleep_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoWoodInputAction {
    QuickUseGadget,
    Escape,
    ClickExitButton,
    ClickConfirm,
    ClickEnterGame,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoWoodOcrRule {
    pub enabled: bool,
    pub engine: String,
    pub wood_count_rect: Rect,
    pub first_ocr_timeout_ms: u64,
    pub first_ocr_interval_ms: u64,
    pub later_ocr_interval_ms: u64,
    pub empty_statistics_stop_count: u64,
    pub first_empty_disables_ocr_when_no_metrics: bool,
    pub first_detection_requires_obtained_text: bool,
    pub first_detection_requires_multiply_mark: bool,
    pub later_detection_requires_obtained_text: bool,
    pub parse_regex: String,
    pub unknown_wood_discarded: bool,
    pub best_first_ocr_prefers_longest_valid_result: bool,
    pub later_ocr_reuses_first_metrics_when_match_count_not_greater: bool,
    pub reached_max_uses_min_total_count: bool,
    pub known_woods: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoWoodRefreshRule {
    pub skips_refresh_on_last_round: bool,
    pub default_strategy: AutoWoodRefreshStrategy,
    pub wonderland_common_job_key: String,
    pub fallback_strategy: AutoWoodRefreshStrategy,
    pub manual_gc_after_refresh: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoWoodRefreshStrategy {
    WonderlandCycle,
    LegacyExitEnter,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoWoodLegacyExitEnterRule {
    pub opens_menu_with_escape: bool,
    pub menu_open_sleep_ms: u64,
    pub menu_bag_retry_interval_ms: u64,
    pub menu_bag_retry_attempts: u64,
    pub retries_escape_when_menu_bag_missing: bool,
    pub exit_button_click: AutoWoodScaledClick,
    pub after_exit_click_sleep_ms: u64,
    pub confirm_with_template: bool,
    pub enter_game_loop_attempts: u64,
    pub enter_game_pre_check_sleep_ms: u64,
    pub enter_game_click_1080p: AutoWoodScreenPoint,
    pub enter_game_loop_sleep_ms: u64,
    pub enter_game_after_seen_missing_min_clicks: u64,
    pub enter_game_after_seen_missing_sleep_ms: u64,
    pub throws_when_enter_game_never_seen: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoWoodScaledClick {
    pub x_scale_offset_1080p: f64,
    pub y_from_bottom_scale_offset_1080p: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoWoodScreenPoint {
    pub x_1080p: f64,
    pub y_1080p: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoWoodThirdPartyLoginRule {
    pub detects_bilibili_by_yuanshen_config_channel_14: bool,
    pub login_retry_attempts_before_give_up: u64,
    pub login_retry_interval_ms: u64,
    pub agreement_window_title_contains: String,
    pub login_window_title_contains: String,
    pub agreement_click_relative_to_center: (i32, i32),
    pub login_click_relative_to_center: (i32, i32),
    pub login_click_pre_sleep_ms: u64,
    pub login_click_post_sleep_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoWoodLoopRule {
    pub checks_ocr_before_each_round: bool,
    pub breaks_when_ocr_empty_count_reached: bool,
    pub breaks_when_daily_max_reached: bool,
    pub cancellation_checked_before_round: bool,
    pub felling_sequence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoWoodTaskStep {
    pub phase: AutoWoodTaskPhase,
    pub action: AutoWoodTaskAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoWoodTaskPhase {
    Startup,
    LoopGuard,
    PressGadget,
    Ocr,
    Refresh,
    LegacyExitEnter,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoWoodTaskAction {
    PreventSystemSleep,
    DetectThirdPartyLoginMode,
    ActivateGameWindow,
    CheckWoodStatisticsEmpty,
    CheckDailyMaxCount,
    ProbeBoonTemplate,
    QuickUseGadget,
    SleepAfterGadget,
    OcrWoodCount,
    ParseWoodStatistics,
    RunWonderlandCycle,
    PressEscape,
    ClickExitButton,
    ClickConfirm,
    ClickEnterGame,
    ClearOverlayAndSleep,
}

pub fn plan_auto_wood(config: AutoWoodExecutionConfig) -> AutoWoodExecutionPlan {
    let normalized_round_num = normalize_round_num(config.wood_round_num);
    let normalized_daily_max_count = normalize_daily_max_count(config.wood_daily_max_count);
    let use_wonderland_refresh = config.auto_wood_config.use_wonderland_refresh;
    let refresh_strategy = if use_wonderland_refresh {
        AutoWoodRefreshStrategy::WonderlandCycle
    } else {
        AutoWoodRefreshStrategy::LegacyExitEnter
    };

    AutoWoodExecutionPlan {
        task_key: AUTO_WOOD_TASK_KEY.to_string(),
        display_name: AUTO_WOOD_DISPLAY_NAME.to_string(),
        capture_size: config.capture_size,
        asset_scale: config.asset_scale,
        param_rule: AutoWoodParamRule {
            raw_round_num: config.wood_round_num,
            normalized_round_num,
            raw_daily_max_count: config.wood_daily_max_count,
            normalized_daily_max_count,
            unlimited_sentinel: AUTO_WOOD_UNLIMITED_COUNT,
        },
        config_rule: AutoWoodConfigRule {
            after_z_sleep_delay_ms: config.auto_wood_config.after_z_sleep_delay,
            wood_count_ocr_enabled: config.auto_wood_config.wood_count_ocr_enabled,
            use_wonderland_refresh,
            press_two_esc_is_legacy_commented_out: true,
        },
        startup_rule: AutoWoodStartupRule {
            destroys_asset_singleton_before_start: true,
            initializes_auto_wood_assets: true,
            creates_wonderland_cycle_job: true,
            prevents_system_sleep: true,
            restores_execution_state_on_finish: true,
            refreshes_third_party_login_mode: true,
            activates_game_window_before_loop: true,
            clears_draw_content_after_each_round: true,
            post_round_sleep_ms: 500,
        },
        locators: auto_wood_locators(config.capture_size, config.asset_scale),
        press_gadget_rule: AutoWoodPressGadgetRule {
            focuses_game_window_before_press: true,
            first_round_requires_boon_template: true,
            missing_first_boon_ends_normally: true,
            later_round_retry_interval_ms: 1_000,
            later_round_retry_attempts: 120,
            retry_pre_capture_sleep_ms: 1,
            action: AutoWoodInputAction::QuickUseGadget,
            later_round_post_press_sleep_ms: 500,
            post_press_base_sleep_ms: 300,
            post_press_extra_sleep_ms: config.auto_wood_config.after_z_sleep_delay,
        },
        ocr_rule: AutoWoodOcrRule {
            enabled: config.auto_wood_config.wood_count_ocr_enabled,
            engine: "Paddle".to_string(),
            wood_count_rect: scale_rect(
                Rect {
                    x: 100,
                    y: 450,
                    width: 300,
                    height: 250,
                },
                config.asset_scale,
            ),
            first_ocr_timeout_ms: 3_500,
            first_ocr_interval_ms: 300,
            later_ocr_interval_ms: 100,
            empty_statistics_stop_count: 3,
            first_empty_disables_ocr_when_no_metrics: true,
            first_detection_requires_obtained_text: true,
            first_detection_requires_multiply_mark: true,
            later_detection_requires_obtained_text: true,
            parse_regex: r"([^\d\n]+)[×x](\d+)".to_string(),
            unknown_wood_discarded: true,
            best_first_ocr_prefers_longest_valid_result: true,
            later_ocr_reuses_first_metrics_when_match_count_not_greater: true,
            reached_max_uses_min_total_count: true,
            known_woods: known_woods(),
        },
        refresh_rule: AutoWoodRefreshRule {
            skips_refresh_on_last_round: true,
            default_strategy: refresh_strategy,
            wonderland_common_job_key: "WonderlandCycle".to_string(),
            fallback_strategy: AutoWoodRefreshStrategy::LegacyExitEnter,
            manual_gc_after_refresh: true,
        },
        legacy_exit_enter_rule: AutoWoodLegacyExitEnterRule {
            opens_menu_with_escape: true,
            menu_open_sleep_ms: 800,
            menu_bag_retry_interval_ms: 1_200,
            menu_bag_retry_attempts: 5,
            retries_escape_when_menu_bag_missing: true,
            exit_button_click: AutoWoodScaledClick {
                x_scale_offset_1080p: 50.0,
                y_from_bottom_scale_offset_1080p: 50.0,
            },
            after_exit_click_sleep_ms: 500,
            confirm_with_template: true,
            enter_game_loop_attempts: 50,
            enter_game_pre_check_sleep_ms: 1,
            enter_game_click_1080p: AutoWoodScreenPoint {
                x_1080p: 960.0,
                y_1080p: 630.0,
            },
            enter_game_loop_sleep_ms: 1_000,
            enter_game_after_seen_missing_min_clicks: 2,
            enter_game_after_seen_missing_sleep_ms: 5_000,
            throws_when_enter_game_never_seen: true,
        },
        third_party_login_rule: AutoWoodThirdPartyLoginRule {
            detects_bilibili_by_yuanshen_config_channel_14: true,
            login_retry_attempts_before_give_up: 20,
            login_retry_interval_ms: 500,
            agreement_window_title_contains: "协议".to_string(),
            login_window_title_contains: "登录".to_string(),
            agreement_click_relative_to_center: (70, 75),
            login_click_relative_to_center: (0, 90),
            login_click_pre_sleep_ms: 2_000,
            login_click_post_sleep_ms: 2_000,
        },
        loop_rule: AutoWoodLoopRule {
            checks_ocr_before_each_round: config.auto_wood_config.wood_count_ocr_enabled,
            breaks_when_ocr_empty_count_reached: true,
            breaks_when_daily_max_reached: true,
            cancellation_checked_before_round: true,
            felling_sequence: vec![
                "PressZ".to_string(),
                "OptionalWoodCountOcr".to_string(),
                "RefreshCooldownUnlessLastRound".to_string(),
                "ManualGc".to_string(),
            ],
        },
        steps: auto_wood_steps(),
        executor_ready: false,
        pending_native: vec![
            "TaskRunner StartGameTask/main-UI wait and solo-task lock".to_string(),
            "live capture loop, cancellation-aware sleep, SystemControl focus, and power-state calls"
                .to_string(),
            "AutoWood asset template matching and DrawContent overlay cleanup".to_string(),
            "GIActions.QuickUseGadget, Escape, and click input dispatch".to_string(),
            "Paddle OCR for wood statistics and user-facing NormalEnd/Retry exceptions".to_string(),
            "WonderlandCycle common-job execution and legacy exit-enter relogin flow".to_string(),
            "Bilibili third-party login window detection and clicks".to_string(),
        ],
    }
}

pub fn normalize_round_num(value: u64) -> u64 {
    if value == 0 {
        AUTO_WOOD_UNLIMITED_COUNT
    } else {
        value
    }
}

pub fn normalize_daily_max_count(value: u64) -> u64 {
    if value == 0 || value >= AUTO_WOOD_UNLIMITED_COUNT {
        AUTO_WOOD_UNLIMITED_COUNT
    } else {
        value
    }
}

fn auto_wood_locators(capture_size: Size, asset_scale: f64) -> AutoWoodLocators {
    AutoWoodLocators {
        wood_count_upper_rect: scale_rect(
            Rect {
                x: 100,
                y: 450,
                width: 300,
                height: 250,
            },
            asset_scale,
        ),
        the_boon_of_the_elder_tree: AutoWoodTemplateLocator {
            name: "TheBoonOfTheElderTree".to_string(),
            asset: AUTO_WOOD_THE_BOON_ASSET.to_string(),
            roi: Some(Rect {
                x: (capture_size.width - capture_size.width / 4) as i32,
                y: (capture_size.height / 2) as i32,
                width: (capture_size.width / 4) as i32,
                height: (capture_size.height - capture_size.height / 2) as i32,
            }),
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
        menu_bag: AutoWoodTemplateLocator {
            name: "MenuBag".to_string(),
            asset: AUTO_WOOD_MENU_BAG_ASSET.to_string(),
            roi: Some(Rect {
                x: 0,
                y: 0,
                width: (capture_size.width / 2) as i32,
                height: capture_size.height as i32,
            }),
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
        confirm: AutoWoodTemplateLocator {
            name: "AutoWoodConfirm".to_string(),
            asset: AUTO_WOOD_CONFIRM_ASSET.to_string(),
            roi: None,
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
        enter_game: AutoWoodTemplateLocator {
            name: "EnterGame".to_string(),
            asset: AUTO_WOOD_ENTER_GAME_ASSET.to_string(),
            roi: Some(Rect {
                x: 0,
                y: (capture_size.height / 2) as i32,
                width: capture_size.width as i32,
                height: (capture_size.height - capture_size.height / 2) as i32,
            }),
            threshold: 0.8,
            match_mode: TemplateMatchMode::CCoeffNormed,
            draw_on_window: false,
        },
    }
}

fn auto_wood_steps() -> Vec<AutoWoodTaskStep> {
    vec![
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::Startup,
            action: AutoWoodTaskAction::PreventSystemSleep,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::Startup,
            action: AutoWoodTaskAction::DetectThirdPartyLoginMode,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::Startup,
            action: AutoWoodTaskAction::ActivateGameWindow,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::LoopGuard,
            action: AutoWoodTaskAction::CheckWoodStatisticsEmpty,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::LoopGuard,
            action: AutoWoodTaskAction::CheckDailyMaxCount,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::PressGadget,
            action: AutoWoodTaskAction::ProbeBoonTemplate,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::PressGadget,
            action: AutoWoodTaskAction::QuickUseGadget,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::PressGadget,
            action: AutoWoodTaskAction::SleepAfterGadget,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::Ocr,
            action: AutoWoodTaskAction::OcrWoodCount,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::Ocr,
            action: AutoWoodTaskAction::ParseWoodStatistics,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::Refresh,
            action: AutoWoodTaskAction::RunWonderlandCycle,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::LegacyExitEnter,
            action: AutoWoodTaskAction::PressEscape,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::LegacyExitEnter,
            action: AutoWoodTaskAction::ClickExitButton,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::LegacyExitEnter,
            action: AutoWoodTaskAction::ClickConfirm,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::LegacyExitEnter,
            action: AutoWoodTaskAction::ClickEnterGame,
        },
        AutoWoodTaskStep {
            phase: AutoWoodTaskPhase::Cleanup,
            action: AutoWoodTaskAction::ClearOverlayAndSleep,
        },
    ]
}

fn known_woods() -> Vec<String> {
    [
        "悬铃木",
        "白梣木",
        "炬木",
        "椴木",
        "香柏木",
        "刺葵木",
        "柽木",
        "辉木",
        "业果木",
        "证悟木",
        "枫木",
        "垂香木",
        "杉木",
        "竹节",
        "却砂木",
        "松木",
        "萃华木",
        "桦木",
        "孔雀木",
        "梦见木",
        "御伽木",
        "燃爆木",
        "桃椰子木",
        "灰灰楼林木",
        "白栗栎木",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn scale_rect(rect: Rect, asset_scale: f64) -> Rect {
    Rect {
        x: scale_i32(rect.x, asset_scale),
        y: scale_i32(rect.y, asset_scale),
        width: scale_i32(rect.width, asset_scale),
        height: scale_i32(rect.height, asset_scale),
    }
}

fn scale_i32(value: i32, asset_scale: f64) -> i32 {
    (value as f64 * asset_scale) as i32
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    let capture = value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .unwrap_or(value);
    let width = u32_member(capture, ["width", "Width", "captureWidth", "CaptureWidth"])?;
    let height = u32_member(
        capture,
        ["height", "Height", "captureHeight", "CaptureHeight"],
    )?;
    Some(Size::new(width, height))
}

fn u64_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<u64> {
    keys.into_iter()
        .filter_map(|key| value.get(key))
        .find_map(Value::as_u64)
}

fn u32_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<u32> {
    keys.into_iter()
        .filter_map(|key| value.get(key))
        .find_map(|value| value.as_u64().and_then(|value| u32::try_from(value).ok()))
}

fn f64_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<f64> {
    keys.into_iter()
        .filter_map(|key| value.get(key))
        .find_map(Value::as_f64)
}
