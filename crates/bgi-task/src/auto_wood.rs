use bgi_core::AutoWoodConfig;
use bgi_vision::{Rect, Size, TemplateMatchMode};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::Result;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoWoodRefreshStrategy {
    WonderlandCycle,
    LegacyExitEnter,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoWoodScaledClick {
    pub x_scale_offset_1080p: f64,
    pub y_from_bottom_scale_offset_1080p: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoWoodTaskPhase {
    Startup,
    LoopGuard,
    PressGadget,
    Ocr,
    Refresh,
    LegacyExitEnter,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoWoodExecutionStatus {
    Completed,
    StartupFailed,
    WindowActivationFailed,
    Cancelled,
    GadgetMissing,
    FirstOcrEmpty,
    EmptyStatisticsLimitReached,
    DailyMaxReached,
    CleanupFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoWoodRuntimeActionStatus {
    Succeeded,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoWoodRuntimeActionKind {
    Startup,
    ActivateGameWindow,
    LoopGuard,
    EnsureGadget,
    QuickUseGadget,
    Delay,
    OcrWoodCount,
    ParseWoodStatistics,
    WonderlandRefresh,
    LegacyRefresh,
    ThirdPartyLogin,
    ManualGarbageCollection,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoWoodSkipReason {
    Cancelled,
    DailyMaxReached,
    EmptyStatisticsLimitReached,
    FirstOcrEmpty,
    GadgetMissing,
    LastRound,
    OcrDisabled,
    RefreshSkippedOnLastRound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoWoodDelayReason {
    RetryBeforeCapture,
    RetryInterval,
    AfterQuickUseGadget,
    AfterLaterRoundQuickUseGadget,
    AfterLegacyMenuOpen,
    AfterLegacyExitClick,
    BeforeEnterGameProbe,
    EnterGameLoop,
    AfterEnterGameMissing,
    PostRound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoWoodThirdPartyLoginMode {
    None,
    Bilibili,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodCountEntry {
    pub wood_name: String,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodStartupOutcome {
    pub completed: bool,
    pub assets_initialized: bool,
    pub system_sleep_prevented: bool,
    pub third_party_login_mode: AutoWoodThirdPartyLoginMode,
    pub initial_wood_totals: Vec<AutoWoodCountEntry>,
    pub initial_empty_statistics_count: u64,
    pub message: Option<String>,
}

impl AutoWoodStartupOutcome {
    pub fn completed() -> Self {
        Self {
            completed: true,
            assets_initialized: true,
            system_sleep_prevented: true,
            third_party_login_mode: AutoWoodThirdPartyLoginMode::None,
            initial_wood_totals: Vec::new(),
            initial_empty_statistics_count: 0,
            message: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodWindowOutcome {
    pub activated: bool,
    pub message: Option<String>,
}

impl AutoWoodWindowOutcome {
    pub fn activated() -> Self {
        Self {
            activated: true,
            message: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodGadgetOutcome {
    pub ready: bool,
    pub switched_or_equipped: bool,
    pub message: Option<String>,
}

impl AutoWoodGadgetOutcome {
    pub fn ready() -> Self {
        Self {
            ready: true,
            switched_or_equipped: false,
            message: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodInputOutcome {
    pub dispatched: bool,
    pub action: AutoWoodInputAction,
    pub message: Option<String>,
}

impl AutoWoodInputOutcome {
    pub fn dispatched(action: AutoWoodInputAction) -> Self {
        Self {
            dispatched: true,
            action,
            message: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodDelayOutcome {
    pub duration_ms: u64,
    pub reason: AutoWoodDelayReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodOcrOutcome {
    pub text: String,
    pub message: Option<String>,
}

impl AutoWoodOcrOutcome {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            message: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodOcrParseReport {
    pub raw_text: String,
    pub entries: Vec<AutoWoodCountEntry>,
    pub unknown_woods: Vec<AutoWoodCountEntry>,
    pub invalid_fragments: Vec<String>,
    pub used_cached_first_metrics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodRefreshOutcome {
    pub completed: bool,
    pub strategy: AutoWoodRefreshStrategy,
    pub fallback_used: bool,
    pub message: Option<String>,
}

impl AutoWoodRefreshOutcome {
    pub fn completed(strategy: AutoWoodRefreshStrategy) -> Self {
        Self {
            completed: true,
            strategy,
            fallback_used: false,
            message: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodThirdPartyLoginOutcome {
    pub attempted: bool,
    pub completed: bool,
    pub mode: AutoWoodThirdPartyLoginMode,
    pub message: Option<String>,
}

impl AutoWoodThirdPartyLoginOutcome {
    pub fn skipped(mode: AutoWoodThirdPartyLoginMode) -> Self {
        Self {
            attempted: false,
            completed: true,
            mode,
            message: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodGarbageCollectionOutcome {
    pub completed: bool,
    pub message: Option<String>,
}

impl AutoWoodGarbageCollectionOutcome {
    pub fn completed() -> Self {
        Self {
            completed: true,
            message: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodCleanupOutcome {
    pub completed: bool,
    pub overlay_cleared: bool,
    pub power_state_restored: bool,
    pub message: Option<String>,
}

impl AutoWoodCleanupOutcome {
    pub fn completed() -> Self {
        Self {
            completed: true,
            overlay_cleared: true,
            power_state_restored: true,
            message: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoWoodRuntimeActionOutcome {
    Startup(AutoWoodStartupOutcome),
    Window(AutoWoodWindowOutcome),
    Gadget(AutoWoodGadgetOutcome),
    Input(AutoWoodInputOutcome),
    Delay(AutoWoodDelayOutcome),
    Ocr(AutoWoodOcrOutcome),
    OcrParse(AutoWoodOcrParseReport),
    Refresh(AutoWoodRefreshOutcome),
    ThirdPartyLogin(AutoWoodThirdPartyLoginOutcome),
    GarbageCollection(AutoWoodGarbageCollectionOutcome),
    Cleanup(AutoWoodCleanupOutcome),
    Skipped(AutoWoodSkipReason),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodRuntimeActionReport {
    pub phase: AutoWoodTaskPhase,
    pub action_kind: AutoWoodRuntimeActionKind,
    pub status: AutoWoodRuntimeActionStatus,
    pub round_index: Option<u64>,
    pub detail: String,
    pub outcome: AutoWoodRuntimeActionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodSkippedStep {
    pub action_kind: AutoWoodRuntimeActionKind,
    pub round_index: Option<u64>,
    pub reason: AutoWoodSkipReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodRuntimeRoundContext {
    pub round_index: u64,
    pub is_first_round: bool,
    pub is_last_round: bool,
    pub total_rounds: u64,
    pub empty_statistics_count: u64,
    pub reached_daily_max: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodExecutorState {
    pub startup_completed: bool,
    pub game_window_activated: bool,
    pub current_round: u64,
    pub rounds_started: u64,
    pub rounds_completed: u64,
    pub quick_use_gadget_count: u64,
    pub ocr_enabled: bool,
    pub ocr_disabled_after_first_empty: bool,
    pub ocr_attempts: u64,
    pub ocr_parse_count: u64,
    pub empty_statistics_count: u64,
    pub wood_totals: BTreeMap<String, u64>,
    pub first_ocr_text: Option<String>,
    pub first_ocr_completed: bool,
    pub baseline_wood_counts: BTreeMap<String, u64>,
    pub reached_daily_max: bool,
    pub refresh_count: u64,
    pub wonderland_refresh_count: u64,
    pub legacy_refresh_count: u64,
    pub manual_gc_count: u64,
    pub cancelled: bool,
    pub cleanup_completed: bool,
    pub last_skip_reason: Option<AutoWoodSkipReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoWoodExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub status: AutoWoodExecutionStatus,
    pub state: AutoWoodExecutorState,
    pub executed_actions: Vec<AutoWoodRuntimeActionReport>,
    pub skipped_steps: Vec<AutoWoodSkippedStep>,
}

pub trait AutoWoodRuntime {
    fn start_auto_wood(&mut self, plan: &AutoWoodExecutionPlan) -> Result<AutoWoodStartupOutcome>;

    fn activate_auto_wood_game_window(
        &mut self,
        plan: &AutoWoodExecutionPlan,
    ) -> Result<AutoWoodWindowOutcome>;

    fn ensure_auto_wood_gadget(
        &mut self,
        plan: &AutoWoodExecutionPlan,
        context: &AutoWoodRuntimeRoundContext,
    ) -> Result<AutoWoodGadgetOutcome>;

    fn dispatch_auto_wood_input(
        &mut self,
        action: AutoWoodInputAction,
        context: &AutoWoodRuntimeRoundContext,
    ) -> Result<AutoWoodInputOutcome>;

    fn delay_auto_wood(
        &mut self,
        duration_ms: u64,
        reason: AutoWoodDelayReason,
        context: Option<&AutoWoodRuntimeRoundContext>,
    ) -> Result<AutoWoodDelayOutcome>;

    fn ocr_auto_wood_count(
        &mut self,
        plan: &AutoWoodExecutionPlan,
        context: &AutoWoodRuntimeRoundContext,
        attempt_index: u64,
    ) -> Result<AutoWoodOcrOutcome>;

    fn run_auto_wood_wonderland_cycle(
        &mut self,
        plan: &AutoWoodExecutionPlan,
        context: &AutoWoodRuntimeRoundContext,
    ) -> Result<AutoWoodRefreshOutcome>;

    fn probe_auto_wood_legacy_template(
        &mut self,
        locator: &AutoWoodTemplateLocator,
        context: &AutoWoodRuntimeRoundContext,
    ) -> Result<bool>;

    fn handle_auto_wood_third_party_login(
        &mut self,
        plan: &AutoWoodExecutionPlan,
        context: &AutoWoodRuntimeRoundContext,
        mode: AutoWoodThirdPartyLoginMode,
    ) -> Result<AutoWoodThirdPartyLoginOutcome>;

    fn collect_auto_wood_garbage(
        &mut self,
        plan: &AutoWoodExecutionPlan,
        context: &AutoWoodRuntimeRoundContext,
        strategy: AutoWoodRefreshStrategy,
    ) -> Result<AutoWoodGarbageCollectionOutcome>;

    fn cleanup_auto_wood(
        &mut self,
        plan: &AutoWoodExecutionPlan,
        state: &AutoWoodExecutorState,
    ) -> Result<AutoWoodCleanupOutcome>;

    fn is_auto_wood_cancelled(&mut self) -> bool {
        false
    }
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
        executor_ready: true,
        pending_native: vec![
            "Rust AutoWood now has an injectable executor boundary for startup/preflight, gadget readiness, QuickUseGadget dispatch, OCR/counting, Wonderland or legacy refresh, third-party login gate, manual GC, and cleanup".to_string(),
            "desktop live adapters now wire capture/template matching, input dispatch, WinRT wood-count OCR fallback, Wonderland/legacy refresh dispatch, window activation, power-state calls, Bilibili third-party login through the Relogin platform driver, and no-op overlay cleanup for draw-disabled AutoWood locators; Paddle OCR parity and real-game regression remain pending".to_string(),
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

pub fn parse_auto_wood_ocr_text(
    text: &str,
    known_woods: &[String],
) -> Result<AutoWoodOcrParseReport> {
    let regex = Regex::new(r"([^\d\n]+)[×xX](\d+)").map_err(|error| {
        crate::TaskError::VisionPlan(format!("invalid AutoWood OCR regex: {error}"))
    })?;
    let mut entries = Vec::new();
    let mut unknown_woods = Vec::new();
    let mut invalid_fragments = Vec::new();

    for capture in regex.captures_iter(text) {
        let wood_name = capture
            .get(1)
            .map(|match_| match_.as_str().trim())
            .unwrap_or_default()
            .to_string();
        let count_text = capture
            .get(2)
            .map(|match_| match_.as_str().trim())
            .unwrap_or_default();
        match count_text.parse::<u64>() {
            Ok(count) if known_woods.iter().any(|known| known == &wood_name) => {
                entries.push(AutoWoodCountEntry { wood_name, count });
            }
            Ok(count) => unknown_woods.push(AutoWoodCountEntry { wood_name, count }),
            Err(_) => invalid_fragments.push(capture[0].to_string()),
        }
    }

    Ok(AutoWoodOcrParseReport {
        raw_text: text.to_string(),
        entries,
        unknown_woods,
        invalid_fragments,
        used_cached_first_metrics: false,
    })
}

pub fn execute_auto_wood_plan<R>(
    plan: &AutoWoodExecutionPlan,
    runtime: &mut R,
) -> Result<AutoWoodExecutionReport>
where
    R: AutoWoodRuntime,
{
    let mut state = AutoWoodExecutorState {
        ocr_enabled: plan.ocr_rule.enabled,
        ..AutoWoodExecutorState::default()
    };
    let mut executed_actions = Vec::new();
    let mut skipped_steps = Vec::new();

    let execution_result = execute_auto_wood_plan_inner(
        plan,
        runtime,
        &mut state,
        &mut executed_actions,
        &mut skipped_steps,
    );

    match execution_result {
        Ok(status) => {
            let cleanup_status =
                execute_auto_wood_cleanup(plan, runtime, &mut state, &mut executed_actions)?;
            let status = if cleanup_status == AutoWoodExecutionStatus::CleanupFailed {
                AutoWoodExecutionStatus::CleanupFailed
            } else {
                status
            };
            Ok(auto_wood_report(
                plan,
                status,
                state,
                executed_actions,
                skipped_steps,
            ))
        }
        Err(error) => {
            let cleanup_error =
                execute_auto_wood_cleanup(plan, runtime, &mut state, &mut executed_actions).err();
            Err(cleanup_error.unwrap_or(error))
        }
    }
}

fn execute_auto_wood_plan_inner<R>(
    plan: &AutoWoodExecutionPlan,
    runtime: &mut R,
    state: &mut AutoWoodExecutorState,
    executed_actions: &mut Vec<AutoWoodRuntimeActionReport>,
    skipped_steps: &mut Vec<AutoWoodSkippedStep>,
) -> Result<AutoWoodExecutionStatus>
where
    R: AutoWoodRuntime,
{
    let startup = runtime.start_auto_wood(plan)?;
    state.startup_completed = startup.completed;
    state.empty_statistics_count = startup.initial_empty_statistics_count;
    state.wood_totals = startup
        .initial_wood_totals
        .iter()
        .map(|entry| (entry.wood_name.clone(), entry.count))
        .collect();
    state.reached_daily_max = auto_wood_reached_daily_max(
        &state.wood_totals,
        plan.param_rule.normalized_daily_max_count,
    );
    let startup_completed = startup.completed;
    let third_party_login_mode = startup.third_party_login_mode;
    executed_actions.push(auto_wood_action_report(
        AutoWoodTaskPhase::Startup,
        AutoWoodRuntimeActionKind::Startup,
        if startup_completed {
            AutoWoodRuntimeActionStatus::Succeeded
        } else {
            AutoWoodRuntimeActionStatus::Failed
        },
        None,
        startup
            .message
            .clone()
            .unwrap_or_else(|| "auto wood startup boundary completed".to_string()),
        AutoWoodRuntimeActionOutcome::Startup(startup),
    ));
    if !startup_completed {
        return Ok(AutoWoodExecutionStatus::StartupFailed);
    }

    let window = runtime.activate_auto_wood_game_window(plan)?;
    state.game_window_activated = window.activated;
    let window_activated = window.activated;
    executed_actions.push(auto_wood_action_report(
        AutoWoodTaskPhase::Startup,
        AutoWoodRuntimeActionKind::ActivateGameWindow,
        if window_activated {
            AutoWoodRuntimeActionStatus::Succeeded
        } else {
            AutoWoodRuntimeActionStatus::Failed
        },
        None,
        window
            .message
            .clone()
            .unwrap_or_else(|| "game window activation boundary completed".to_string()),
        AutoWoodRuntimeActionOutcome::Window(window),
    ));
    if !window_activated {
        return Ok(AutoWoodExecutionStatus::WindowActivationFailed);
    }

    for round_index in 0..plan.param_rule.normalized_round_num {
        if runtime.is_auto_wood_cancelled() {
            state.cancelled = true;
            record_auto_wood_skip(
                skipped_steps,
                executed_actions,
                AutoWoodTaskPhase::LoopGuard,
                AutoWoodRuntimeActionKind::LoopGuard,
                Some(round_index),
                AutoWoodSkipReason::Cancelled,
            );
            return Ok(AutoWoodExecutionStatus::Cancelled);
        }

        if state.ocr_enabled
            && state.empty_statistics_count >= plan.ocr_rule.empty_statistics_stop_count
        {
            state.last_skip_reason = Some(AutoWoodSkipReason::EmptyStatisticsLimitReached);
            record_auto_wood_skip(
                skipped_steps,
                executed_actions,
                AutoWoodTaskPhase::LoopGuard,
                AutoWoodRuntimeActionKind::LoopGuard,
                Some(round_index),
                AutoWoodSkipReason::EmptyStatisticsLimitReached,
            );
            return Ok(AutoWoodExecutionStatus::EmptyStatisticsLimitReached);
        }

        if state.ocr_enabled && state.reached_daily_max {
            state.last_skip_reason = Some(AutoWoodSkipReason::DailyMaxReached);
            record_auto_wood_skip(
                skipped_steps,
                executed_actions,
                AutoWoodTaskPhase::LoopGuard,
                AutoWoodRuntimeActionKind::LoopGuard,
                Some(round_index),
                AutoWoodSkipReason::DailyMaxReached,
            );
            return Ok(AutoWoodExecutionStatus::DailyMaxReached);
        }

        state.current_round = round_index;
        state.rounds_started += 1;
        let context = auto_wood_context(plan, state, round_index);

        let gadget = ensure_auto_wood_gadget_with_retry(plan, runtime, &context, executed_actions)?;
        if !gadget.ready {
            state.last_skip_reason = Some(AutoWoodSkipReason::GadgetMissing);
            record_auto_wood_skip(
                skipped_steps,
                executed_actions,
                AutoWoodTaskPhase::PressGadget,
                AutoWoodRuntimeActionKind::EnsureGadget,
                Some(round_index),
                AutoWoodSkipReason::GadgetMissing,
            );
            return Ok(AutoWoodExecutionStatus::GadgetMissing);
        }

        let input =
            runtime.dispatch_auto_wood_input(AutoWoodInputAction::QuickUseGadget, &context)?;
        let input_dispatched = input.dispatched;
        executed_actions.push(auto_wood_action_report(
            AutoWoodTaskPhase::PressGadget,
            AutoWoodRuntimeActionKind::QuickUseGadget,
            if input_dispatched {
                AutoWoodRuntimeActionStatus::Succeeded
            } else {
                AutoWoodRuntimeActionStatus::Failed
            },
            Some(round_index),
            input
                .message
                .clone()
                .unwrap_or_else(|| "quick-use gadget input boundary completed".to_string()),
            AutoWoodRuntimeActionOutcome::Input(input),
        ));
        if input_dispatched {
            state.quick_use_gadget_count += 1;
        }

        delay_auto_wood_and_record(
            runtime,
            executed_actions,
            plan.press_gadget_rule.post_press_base_sleep_ms,
            AutoWoodDelayReason::AfterQuickUseGadget,
            Some(&context),
        )?;
        delay_auto_wood_and_record(
            runtime,
            executed_actions,
            plan.press_gadget_rule.post_press_extra_sleep_ms,
            AutoWoodDelayReason::AfterQuickUseGadget,
            Some(&context),
        )?;

        if context.is_last_round {
            state.rounds_completed += 1;
            delay_auto_wood_and_record(
                runtime,
                executed_actions,
                plan.startup_rule.post_round_sleep_ms,
                AutoWoodDelayReason::PostRound,
                Some(&context),
            )?;
            continue;
        }

        if state.ocr_enabled {
            let ocr_status =
                execute_auto_wood_ocr(plan, runtime, state, &context, executed_actions)?;
            match ocr_status {
                AutoWoodExecutionStatus::Completed => {}
                status => return Ok(status),
            }
        } else {
            record_auto_wood_skip(
                skipped_steps,
                executed_actions,
                AutoWoodTaskPhase::Ocr,
                AutoWoodRuntimeActionKind::OcrWoodCount,
                Some(round_index),
                AutoWoodSkipReason::OcrDisabled,
            );
        }

        if state.ocr_enabled
            && state.empty_statistics_count >= plan.ocr_rule.empty_statistics_stop_count
        {
            state.last_skip_reason = Some(AutoWoodSkipReason::EmptyStatisticsLimitReached);
            record_auto_wood_skip(
                skipped_steps,
                executed_actions,
                AutoWoodTaskPhase::Refresh,
                AutoWoodRuntimeActionKind::LoopGuard,
                Some(round_index),
                AutoWoodSkipReason::EmptyStatisticsLimitReached,
            );
            return Ok(AutoWoodExecutionStatus::EmptyStatisticsLimitReached);
        }

        if state.ocr_enabled && state.reached_daily_max {
            state.last_skip_reason = Some(AutoWoodSkipReason::DailyMaxReached);
            record_auto_wood_skip(
                skipped_steps,
                executed_actions,
                AutoWoodTaskPhase::Refresh,
                AutoWoodRuntimeActionKind::LoopGuard,
                Some(round_index),
                AutoWoodSkipReason::DailyMaxReached,
            );
            return Ok(AutoWoodExecutionStatus::DailyMaxReached);
        }

        let strategy = execute_auto_wood_refresh(
            plan,
            runtime,
            state,
            &context,
            third_party_login_mode,
            executed_actions,
        )?;
        if let Some(strategy) = strategy {
            let gc = runtime.collect_auto_wood_garbage(plan, &context, strategy)?;
            let gc_completed = gc.completed;
            executed_actions.push(auto_wood_action_report(
                AutoWoodTaskPhase::Refresh,
                AutoWoodRuntimeActionKind::ManualGarbageCollection,
                if gc_completed {
                    AutoWoodRuntimeActionStatus::Succeeded
                } else {
                    AutoWoodRuntimeActionStatus::Failed
                },
                Some(round_index),
                gc.message
                    .clone()
                    .unwrap_or_else(|| "manual garbage collection boundary completed".to_string()),
                AutoWoodRuntimeActionOutcome::GarbageCollection(gc),
            ));
            if gc_completed {
                state.manual_gc_count += 1;
            }
        }

        state.rounds_completed += 1;
        delay_auto_wood_and_record(
            runtime,
            executed_actions,
            plan.startup_rule.post_round_sleep_ms,
            AutoWoodDelayReason::PostRound,
            Some(&context),
        )?;
    }

    Ok(AutoWoodExecutionStatus::Completed)
}

fn ensure_auto_wood_gadget_with_retry<R>(
    plan: &AutoWoodExecutionPlan,
    runtime: &mut R,
    context: &AutoWoodRuntimeRoundContext,
    executed_actions: &mut Vec<AutoWoodRuntimeActionReport>,
) -> Result<AutoWoodGadgetOutcome>
where
    R: AutoWoodRuntime,
{
    let attempts = if context.is_first_round {
        1
    } else {
        plan.press_gadget_rule.later_round_retry_attempts.max(1)
    };

    let mut last = AutoWoodGadgetOutcome {
        ready: false,
        switched_or_equipped: false,
        message: Some("gadget readiness was not checked".to_string()),
    };
    for attempt in 0..attempts {
        if !context.is_first_round && plan.press_gadget_rule.retry_pre_capture_sleep_ms > 0 {
            delay_auto_wood_and_record(
                runtime,
                executed_actions,
                plan.press_gadget_rule.retry_pre_capture_sleep_ms,
                AutoWoodDelayReason::RetryBeforeCapture,
                Some(context),
            )?;
        }

        last = runtime.ensure_auto_wood_gadget(plan, context)?;
        let ready = last.ready;
        executed_actions.push(auto_wood_action_report(
            AutoWoodTaskPhase::PressGadget,
            AutoWoodRuntimeActionKind::EnsureGadget,
            if ready {
                AutoWoodRuntimeActionStatus::Succeeded
            } else {
                AutoWoodRuntimeActionStatus::Skipped
            },
            Some(context.round_index),
            last.message.clone().unwrap_or_else(|| {
                if ready {
                    "auto wood gadget is ready".to_string()
                } else {
                    "auto wood gadget is not ready".to_string()
                }
            }),
            AutoWoodRuntimeActionOutcome::Gadget(last.clone()),
        ));
        if ready || context.is_first_round {
            return Ok(last);
        }
        if attempt + 1 < attempts {
            delay_auto_wood_and_record(
                runtime,
                executed_actions,
                plan.press_gadget_rule.later_round_retry_interval_ms,
                AutoWoodDelayReason::RetryInterval,
                Some(context),
            )?;
        }
    }
    Ok(last)
}

fn execute_auto_wood_ocr<R>(
    plan: &AutoWoodExecutionPlan,
    runtime: &mut R,
    state: &mut AutoWoodExecutorState,
    context: &AutoWoodRuntimeRoundContext,
    executed_actions: &mut Vec<AutoWoodRuntimeActionReport>,
) -> Result<AutoWoodExecutionStatus>
where
    R: AutoWoodRuntime,
{
    let text = if !state.first_ocr_completed {
        let best_text =
            collect_first_auto_wood_ocr_text(plan, runtime, state, context, executed_actions)?;
        if best_text.is_empty() {
            state.empty_statistics_count += 1;
            if plan.ocr_rule.first_empty_disables_ocr_when_no_metrics
                && state.baseline_wood_counts.is_empty()
            {
                state.ocr_enabled = false;
                state.ocr_disabled_after_first_empty = true;
                state.last_skip_reason = Some(AutoWoodSkipReason::FirstOcrEmpty);
                return Ok(AutoWoodExecutionStatus::FirstOcrEmpty);
            }
            return Ok(AutoWoodExecutionStatus::Completed);
        }
        state.first_ocr_text = Some(best_text.clone());
        best_text
    } else {
        loop {
            state.ocr_attempts += 1;
            let attempt_index = state.ocr_attempts;
            let ocr = runtime.ocr_auto_wood_count(plan, context, attempt_index)?;
            let has_text = auto_wood_detected_text(&ocr.text, false);
            let text = ocr.text.clone();
            executed_actions.push(auto_wood_action_report(
                AutoWoodTaskPhase::Ocr,
                AutoWoodRuntimeActionKind::OcrWoodCount,
                if has_text {
                    AutoWoodRuntimeActionStatus::Succeeded
                } else {
                    AutoWoodRuntimeActionStatus::Skipped
                },
                Some(context.round_index),
                ocr.message
                    .clone()
                    .unwrap_or_else(|| "wood-count OCR boundary completed".to_string()),
                AutoWoodRuntimeActionOutcome::Ocr(ocr),
            ));
            if has_text {
                state.empty_statistics_count = 0;
                break state.first_ocr_text.clone().unwrap_or(text);
            }
            delay_auto_wood_and_record(
                runtime,
                executed_actions,
                plan.ocr_rule.later_ocr_interval_ms,
                AutoWoodDelayReason::RetryInterval,
                Some(context),
            )?;
            state.empty_statistics_count += 1;
            if state.empty_statistics_count >= plan.ocr_rule.empty_statistics_stop_count {
                return Ok(AutoWoodExecutionStatus::EmptyStatisticsLimitReached);
            }
        }
    };

    let parse_report = auto_wood_parse_for_state(plan, state, &text)?;
    apply_auto_wood_parse_report(plan, state, &parse_report);
    state.ocr_parse_count += 1;
    let entry_count = parse_report.entries.len();
    let used_cached_first_metrics = parse_report.used_cached_first_metrics;
    executed_actions.push(auto_wood_action_report(
        AutoWoodTaskPhase::Ocr,
        AutoWoodRuntimeActionKind::ParseWoodStatistics,
        if entry_count > 0 {
            AutoWoodRuntimeActionStatus::Succeeded
        } else {
            AutoWoodRuntimeActionStatus::Skipped
        },
        Some(context.round_index),
        if used_cached_first_metrics {
            "wood-count OCR reused first-round metrics".to_string()
        } else {
            format!("wood-count OCR parsed {entry_count} known wood entries")
        },
        AutoWoodRuntimeActionOutcome::OcrParse(parse_report),
    ));
    if state.reached_daily_max {
        state.last_skip_reason = Some(AutoWoodSkipReason::DailyMaxReached);
        return Ok(AutoWoodExecutionStatus::DailyMaxReached);
    }
    Ok(AutoWoodExecutionStatus::Completed)
}

fn collect_first_auto_wood_ocr_text<R>(
    plan: &AutoWoodExecutionPlan,
    runtime: &mut R,
    state: &mut AutoWoodExecutorState,
    context: &AutoWoodRuntimeRoundContext,
    executed_actions: &mut Vec<AutoWoodRuntimeActionReport>,
) -> Result<String>
where
    R: AutoWoodRuntime,
{
    let attempts =
        (plan.ocr_rule.first_ocr_timeout_ms / plan.ocr_rule.first_ocr_interval_ms.max(1)).max(1);
    let mut candidates = Vec::new();
    let mut previous_found = false;

    for _ in 0..attempts {
        state.ocr_attempts += 1;
        let attempt_index = state.ocr_attempts;
        let ocr = runtime.ocr_auto_wood_count(plan, context, attempt_index)?;
        let found = auto_wood_detected_text(&ocr.text, true);
        if found {
            candidates.push(ocr.text.clone());
        }
        executed_actions.push(auto_wood_action_report(
            AutoWoodTaskPhase::Ocr,
            AutoWoodRuntimeActionKind::OcrWoodCount,
            if found {
                AutoWoodRuntimeActionStatus::Succeeded
            } else {
                AutoWoodRuntimeActionStatus::Skipped
            },
            Some(context.round_index),
            ocr.message
                .clone()
                .unwrap_or_else(|| "first wood-count OCR boundary completed".to_string()),
            AutoWoodRuntimeActionOutcome::Ocr(ocr),
        ));
        if previous_found && !found {
            break;
        }
        previous_found = found;
        delay_auto_wood_and_record(
            runtime,
            executed_actions,
            plan.ocr_rule.first_ocr_interval_ms,
            AutoWoodDelayReason::RetryInterval,
            Some(context),
        )?;
    }

    Ok(best_first_auto_wood_ocr_result(plan, &candidates))
}

fn auto_wood_parse_for_state(
    plan: &AutoWoodExecutionPlan,
    state: &AutoWoodExecutorState,
    text: &str,
) -> Result<AutoWoodOcrParseReport> {
    let mut parse_report = parse_auto_wood_ocr_text(text, &plan.ocr_rule.known_woods)?;
    if state.first_ocr_completed
        && plan
            .ocr_rule
            .later_ocr_reuses_first_metrics_when_match_count_not_greater
        && !state.baseline_wood_counts.is_empty()
        && !parse_report.entries.is_empty()
        && parse_report.entries.len() <= state.baseline_wood_counts.len()
    {
        parse_report.entries = state
            .baseline_wood_counts
            .iter()
            .filter(|(_, count)| **count <= plan.param_rule.normalized_daily_max_count)
            .map(|(wood_name, count)| AutoWoodCountEntry {
                wood_name: wood_name.clone(),
                count: *count,
            })
            .collect();
        parse_report.used_cached_first_metrics = true;
    }
    Ok(parse_report)
}

fn apply_auto_wood_parse_report(
    plan: &AutoWoodExecutionPlan,
    state: &mut AutoWoodExecutorState,
    parse_report: &AutoWoodOcrParseReport,
) {
    if parse_report.entries.is_empty() {
        state.empty_statistics_count += 1;
        state.reached_daily_max = false;
        return;
    }

    state.empty_statistics_count = 0;
    for entry in &parse_report.entries {
        *state
            .wood_totals
            .entry(entry.wood_name.clone())
            .or_default() += entry.count;
        if !state.first_ocr_completed {
            state
                .baseline_wood_counts
                .entry(entry.wood_name.clone())
                .or_insert(entry.count);
        }
    }
    state.first_ocr_completed = true;
    state.reached_daily_max = auto_wood_reached_daily_max(
        &state.wood_totals,
        plan.param_rule.normalized_daily_max_count,
    );
}

fn execute_auto_wood_refresh<R>(
    plan: &AutoWoodExecutionPlan,
    runtime: &mut R,
    state: &mut AutoWoodExecutorState,
    context: &AutoWoodRuntimeRoundContext,
    third_party_login_mode: AutoWoodThirdPartyLoginMode,
    executed_actions: &mut Vec<AutoWoodRuntimeActionReport>,
) -> Result<Option<AutoWoodRefreshStrategy>>
where
    R: AutoWoodRuntime,
{
    if plan.refresh_rule.skips_refresh_on_last_round && context.is_last_round {
        executed_actions.push(auto_wood_action_report(
            AutoWoodTaskPhase::Refresh,
            AutoWoodRuntimeActionKind::WonderlandRefresh,
            AutoWoodRuntimeActionStatus::Skipped,
            Some(context.round_index),
            "refresh skipped on last round".to_string(),
            AutoWoodRuntimeActionOutcome::Skipped(AutoWoodSkipReason::RefreshSkippedOnLastRound),
        ));
        return Ok(None);
    }

    match plan.refresh_rule.default_strategy {
        AutoWoodRefreshStrategy::WonderlandCycle => {
            let refresh = runtime.run_auto_wood_wonderland_cycle(plan, context)?;
            let completed = refresh.completed;
            let fallback_used = refresh.fallback_used;
            executed_actions.push(auto_wood_action_report(
                AutoWoodTaskPhase::Refresh,
                AutoWoodRuntimeActionKind::WonderlandRefresh,
                if completed {
                    AutoWoodRuntimeActionStatus::Succeeded
                } else {
                    AutoWoodRuntimeActionStatus::Failed
                },
                Some(context.round_index),
                refresh
                    .message
                    .clone()
                    .unwrap_or_else(|| "WonderlandCycle refresh boundary completed".to_string()),
                AutoWoodRuntimeActionOutcome::Refresh(refresh),
            ));
            if completed {
                state.refresh_count += 1;
                state.wonderland_refresh_count += 1;
                Ok(Some(if fallback_used {
                    plan.refresh_rule.fallback_strategy
                } else {
                    AutoWoodRefreshStrategy::WonderlandCycle
                }))
            } else if plan.refresh_rule.fallback_strategy
                == AutoWoodRefreshStrategy::LegacyExitEnter
            {
                execute_auto_wood_legacy_refresh(
                    plan,
                    runtime,
                    state,
                    context,
                    third_party_login_mode,
                    true,
                    executed_actions,
                )?;
                Ok(Some(AutoWoodRefreshStrategy::LegacyExitEnter))
            } else {
                Ok(None)
            }
        }
        AutoWoodRefreshStrategy::LegacyExitEnter => {
            execute_auto_wood_legacy_refresh(
                plan,
                runtime,
                state,
                context,
                third_party_login_mode,
                false,
                executed_actions,
            )?;
            Ok(Some(AutoWoodRefreshStrategy::LegacyExitEnter))
        }
    }
}

fn execute_auto_wood_legacy_refresh<R>(
    plan: &AutoWoodExecutionPlan,
    runtime: &mut R,
    state: &mut AutoWoodExecutorState,
    context: &AutoWoodRuntimeRoundContext,
    third_party_login_mode: AutoWoodThirdPartyLoginMode,
    fallback_used: bool,
    executed_actions: &mut Vec<AutoWoodRuntimeActionReport>,
) -> Result<()>
where
    R: AutoWoodRuntime,
{
    let escape = runtime.dispatch_auto_wood_input(AutoWoodInputAction::Escape, context)?;
    let escape_dispatched = escape.dispatched;
    executed_actions.push(auto_wood_action_report(
        AutoWoodTaskPhase::LegacyExitEnter,
        AutoWoodRuntimeActionKind::LegacyRefresh,
        if escape_dispatched {
            AutoWoodRuntimeActionStatus::Succeeded
        } else {
            AutoWoodRuntimeActionStatus::Failed
        },
        Some(context.round_index),
        escape
            .message
            .clone()
            .unwrap_or_else(|| "legacy refresh escape input dispatched".to_string()),
        AutoWoodRuntimeActionOutcome::Input(escape),
    ));
    delay_auto_wood_and_record(
        runtime,
        executed_actions,
        plan.legacy_exit_enter_rule.menu_open_sleep_ms,
        AutoWoodDelayReason::AfterLegacyMenuOpen,
        Some(context),
    )?;

    for attempt in 0..plan.legacy_exit_enter_rule.menu_bag_retry_attempts.max(1) {
        delay_auto_wood_and_record(
            runtime,
            executed_actions,
            plan.legacy_exit_enter_rule.retry_pre_check_sleep_ms(),
            AutoWoodDelayReason::RetryBeforeCapture,
            Some(context),
        )?;
        let menu_found =
            runtime.probe_auto_wood_legacy_template(&plan.locators.menu_bag, context)?;
        executed_actions.push(auto_wood_action_report(
            AutoWoodTaskPhase::LegacyExitEnter,
            AutoWoodRuntimeActionKind::LegacyRefresh,
            if menu_found {
                AutoWoodRuntimeActionStatus::Succeeded
            } else {
                AutoWoodRuntimeActionStatus::Skipped
            },
            Some(context.round_index),
            if menu_found {
                "legacy refresh menu bag template detected".to_string()
            } else {
                "legacy refresh menu bag template missing".to_string()
            },
            AutoWoodRuntimeActionOutcome::Refresh(AutoWoodRefreshOutcome {
                completed: menu_found,
                strategy: AutoWoodRefreshStrategy::LegacyExitEnter,
                fallback_used,
                message: Some("menu bag template probe".to_string()),
            }),
        ));
        if menu_found {
            break;
        }
        if plan
            .legacy_exit_enter_rule
            .retries_escape_when_menu_bag_missing
        {
            let retry_escape =
                runtime.dispatch_auto_wood_input(AutoWoodInputAction::Escape, context)?;
            executed_actions.push(auto_wood_action_report(
                AutoWoodTaskPhase::LegacyExitEnter,
                AutoWoodRuntimeActionKind::LegacyRefresh,
                if retry_escape.dispatched {
                    AutoWoodRuntimeActionStatus::Succeeded
                } else {
                    AutoWoodRuntimeActionStatus::Failed
                },
                Some(context.round_index),
                retry_escape
                    .message
                    .clone()
                    .unwrap_or_else(|| "legacy refresh retry escape input dispatched".to_string()),
                AutoWoodRuntimeActionOutcome::Input(retry_escape),
            ));
        }
        if attempt + 1 < plan.legacy_exit_enter_rule.menu_bag_retry_attempts.max(1) {
            delay_auto_wood_and_record(
                runtime,
                executed_actions,
                plan.legacy_exit_enter_rule.menu_bag_retry_interval_ms,
                AutoWoodDelayReason::RetryInterval,
                Some(context),
            )?;
        }
    }

    let exit = runtime.dispatch_auto_wood_input(AutoWoodInputAction::ClickExitButton, context)?;
    executed_actions.push(auto_wood_action_report(
        AutoWoodTaskPhase::LegacyExitEnter,
        AutoWoodRuntimeActionKind::LegacyRefresh,
        if exit.dispatched {
            AutoWoodRuntimeActionStatus::Succeeded
        } else {
            AutoWoodRuntimeActionStatus::Failed
        },
        Some(context.round_index),
        exit.message
            .clone()
            .unwrap_or_else(|| "legacy refresh exit button click dispatched".to_string()),
        AutoWoodRuntimeActionOutcome::Input(exit),
    ));
    delay_auto_wood_and_record(
        runtime,
        executed_actions,
        plan.legacy_exit_enter_rule.after_exit_click_sleep_ms,
        AutoWoodDelayReason::AfterLegacyExitClick,
        Some(context),
    )?;

    if plan.legacy_exit_enter_rule.confirm_with_template
        && runtime.probe_auto_wood_legacy_template(&plan.locators.confirm, context)?
    {
        let confirm =
            runtime.dispatch_auto_wood_input(AutoWoodInputAction::ClickConfirm, context)?;
        executed_actions.push(auto_wood_action_report(
            AutoWoodTaskPhase::LegacyExitEnter,
            AutoWoodRuntimeActionKind::LegacyRefresh,
            if confirm.dispatched {
                AutoWoodRuntimeActionStatus::Succeeded
            } else {
                AutoWoodRuntimeActionStatus::Failed
            },
            Some(context.round_index),
            confirm
                .message
                .clone()
                .unwrap_or_else(|| "legacy refresh confirm click dispatched".to_string()),
            AutoWoodRuntimeActionOutcome::Input(confirm),
        ));
    }

    let third_party =
        runtime.handle_auto_wood_third_party_login(plan, context, third_party_login_mode)?;
    executed_actions.push(auto_wood_action_report(
        AutoWoodTaskPhase::LegacyExitEnter,
        AutoWoodRuntimeActionKind::ThirdPartyLogin,
        if third_party.completed {
            AutoWoodRuntimeActionStatus::Succeeded
        } else {
            AutoWoodRuntimeActionStatus::Skipped
        },
        Some(context.round_index),
        third_party
            .message
            .clone()
            .unwrap_or_else(|| "third-party login boundary completed".to_string()),
        AutoWoodRuntimeActionOutcome::ThirdPartyLogin(third_party),
    ));

    let mut click_count = 0;
    let mut saw_enter_game = false;
    for _ in 0..plan.legacy_exit_enter_rule.enter_game_loop_attempts {
        delay_auto_wood_and_record(
            runtime,
            executed_actions,
            plan.legacy_exit_enter_rule.enter_game_pre_check_sleep_ms,
            AutoWoodDelayReason::BeforeEnterGameProbe,
            Some(context),
        )?;
        let enter_game_found =
            runtime.probe_auto_wood_legacy_template(&plan.locators.enter_game, context)?;
        if enter_game_found {
            saw_enter_game = true;
            click_count += 1;
            let enter =
                runtime.dispatch_auto_wood_input(AutoWoodInputAction::ClickEnterGame, context)?;
            executed_actions.push(auto_wood_action_report(
                AutoWoodTaskPhase::LegacyExitEnter,
                AutoWoodRuntimeActionKind::LegacyRefresh,
                if enter.dispatched {
                    AutoWoodRuntimeActionStatus::Succeeded
                } else {
                    AutoWoodRuntimeActionStatus::Failed
                },
                Some(context.round_index),
                enter
                    .message
                    .clone()
                    .unwrap_or_else(|| "legacy refresh enter-game click dispatched".to_string()),
                AutoWoodRuntimeActionOutcome::Input(enter),
            ));
        } else if click_count
            > plan
                .legacy_exit_enter_rule
                .enter_game_after_seen_missing_min_clicks
        {
            delay_auto_wood_and_record(
                runtime,
                executed_actions,
                plan.legacy_exit_enter_rule
                    .enter_game_after_seen_missing_sleep_ms,
                AutoWoodDelayReason::AfterEnterGameMissing,
                Some(context),
            )?;
            break;
        }
        delay_auto_wood_and_record(
            runtime,
            executed_actions,
            plan.legacy_exit_enter_rule.enter_game_loop_sleep_ms,
            AutoWoodDelayReason::EnterGameLoop,
            Some(context),
        )?;
    }

    if plan
        .legacy_exit_enter_rule
        .throws_when_enter_game_never_seen
        && !saw_enter_game
    {
        return Err(crate::TaskError::CommonJobExecution(
            "AutoWood legacy refresh never detected enter-game screen".to_string(),
        ));
    }

    state.refresh_count += 1;
    state.legacy_refresh_count += 1;
    executed_actions.push(auto_wood_action_report(
        AutoWoodTaskPhase::LegacyExitEnter,
        AutoWoodRuntimeActionKind::LegacyRefresh,
        AutoWoodRuntimeActionStatus::Succeeded,
        Some(context.round_index),
        "legacy exit-enter refresh boundary completed".to_string(),
        AutoWoodRuntimeActionOutcome::Refresh(AutoWoodRefreshOutcome {
            completed: true,
            strategy: AutoWoodRefreshStrategy::LegacyExitEnter,
            fallback_used,
            message: None,
        }),
    ));
    Ok(())
}

fn execute_auto_wood_cleanup<R>(
    plan: &AutoWoodExecutionPlan,
    runtime: &mut R,
    state: &mut AutoWoodExecutorState,
    executed_actions: &mut Vec<AutoWoodRuntimeActionReport>,
) -> Result<AutoWoodExecutionStatus>
where
    R: AutoWoodRuntime,
{
    let cleanup = runtime.cleanup_auto_wood(plan, state)?;
    state.cleanup_completed = cleanup.completed;
    let cleanup_completed = cleanup.completed;
    executed_actions.push(auto_wood_action_report(
        AutoWoodTaskPhase::Cleanup,
        AutoWoodRuntimeActionKind::Cleanup,
        if cleanup_completed {
            AutoWoodRuntimeActionStatus::Succeeded
        } else {
            AutoWoodRuntimeActionStatus::Failed
        },
        None,
        cleanup
            .message
            .clone()
            .unwrap_or_else(|| "auto wood cleanup boundary completed".to_string()),
        AutoWoodRuntimeActionOutcome::Cleanup(cleanup),
    ));
    if cleanup_completed {
        Ok(AutoWoodExecutionStatus::Completed)
    } else {
        Ok(AutoWoodExecutionStatus::CleanupFailed)
    }
}

fn delay_auto_wood_and_record<R>(
    runtime: &mut R,
    executed_actions: &mut Vec<AutoWoodRuntimeActionReport>,
    duration_ms: u64,
    reason: AutoWoodDelayReason,
    context: Option<&AutoWoodRuntimeRoundContext>,
) -> Result<()>
where
    R: AutoWoodRuntime,
{
    if duration_ms == 0 {
        return Ok(());
    }
    let delay = runtime.delay_auto_wood(duration_ms, reason, context)?;
    let round_index = context.map(|context| context.round_index);
    executed_actions.push(auto_wood_action_report(
        match reason {
            AutoWoodDelayReason::AfterLegacyMenuOpen
            | AutoWoodDelayReason::AfterLegacyExitClick
            | AutoWoodDelayReason::BeforeEnterGameProbe
            | AutoWoodDelayReason::EnterGameLoop
            | AutoWoodDelayReason::AfterEnterGameMissing => AutoWoodTaskPhase::LegacyExitEnter,
            AutoWoodDelayReason::PostRound => AutoWoodTaskPhase::Cleanup,
            AutoWoodDelayReason::RetryInterval if round_index.is_some() => AutoWoodTaskPhase::Ocr,
            _ => AutoWoodTaskPhase::PressGadget,
        },
        AutoWoodRuntimeActionKind::Delay,
        AutoWoodRuntimeActionStatus::Succeeded,
        round_index,
        format!("delay {} ms for {:?}", delay.duration_ms, delay.reason),
        AutoWoodRuntimeActionOutcome::Delay(delay),
    ));
    Ok(())
}

fn auto_wood_context(
    plan: &AutoWoodExecutionPlan,
    state: &AutoWoodExecutorState,
    round_index: u64,
) -> AutoWoodRuntimeRoundContext {
    AutoWoodRuntimeRoundContext {
        round_index,
        is_first_round: round_index == 0,
        is_last_round: round_index + 1 >= plan.param_rule.normalized_round_num,
        total_rounds: plan.param_rule.normalized_round_num,
        empty_statistics_count: state.empty_statistics_count,
        reached_daily_max: state.reached_daily_max,
    }
}

fn auto_wood_detected_text(text: &str, first_ocr: bool) -> bool {
    let has_obtained = text.contains("获得");
    let has_multiply = text.contains('×') || text.contains('x') || text.contains('X');
    if first_ocr {
        !text.is_empty() && has_obtained && has_multiply
    } else {
        !text.is_empty() && has_obtained
    }
}

fn best_first_auto_wood_ocr_result(plan: &AutoWoodExecutionPlan, candidates: &[String]) -> String {
    let mut sorted = candidates.to_vec();
    sorted.sort_by_key(|text| std::cmp::Reverse(text.len()));
    let mut target_length = None;
    for text in sorted {
        if let Some(length) = target_length {
            if text.len() != length {
                continue;
            }
        } else {
            target_length = Some(text.len());
        }

        if let Ok(parse_report) = parse_auto_wood_ocr_text(&text, &plan.ocr_rule.known_woods) {
            let known_and_unknown_count =
                parse_report.entries.len() + parse_report.unknown_woods.len();
            if known_and_unknown_count > 0 && parse_report.unknown_woods.is_empty() {
                return text;
            }
        }
    }
    String::new()
}

fn auto_wood_reached_daily_max(wood_totals: &BTreeMap<String, u64>, daily_max_count: u64) -> bool {
    !wood_totals.is_empty() && wood_totals.values().all(|count| *count >= daily_max_count)
}

fn auto_wood_report(
    plan: &AutoWoodExecutionPlan,
    status: AutoWoodExecutionStatus,
    state: AutoWoodExecutorState,
    executed_actions: Vec<AutoWoodRuntimeActionReport>,
    skipped_steps: Vec<AutoWoodSkippedStep>,
) -> AutoWoodExecutionReport {
    AutoWoodExecutionReport {
        task_key: plan.task_key.clone(),
        completed: matches!(status, AutoWoodExecutionStatus::Completed),
        status,
        state,
        executed_actions,
        skipped_steps,
    }
}

fn auto_wood_action_report(
    phase: AutoWoodTaskPhase,
    action_kind: AutoWoodRuntimeActionKind,
    status: AutoWoodRuntimeActionStatus,
    round_index: Option<u64>,
    detail: impl Into<String>,
    outcome: AutoWoodRuntimeActionOutcome,
) -> AutoWoodRuntimeActionReport {
    AutoWoodRuntimeActionReport {
        phase,
        action_kind,
        status,
        round_index,
        detail: detail.into(),
        outcome,
    }
}

fn record_auto_wood_skip(
    skipped_steps: &mut Vec<AutoWoodSkippedStep>,
    executed_actions: &mut Vec<AutoWoodRuntimeActionReport>,
    phase: AutoWoodTaskPhase,
    action_kind: AutoWoodRuntimeActionKind,
    round_index: Option<u64>,
    reason: AutoWoodSkipReason,
) {
    skipped_steps.push(AutoWoodSkippedStep {
        action_kind,
        round_index,
        reason,
    });
    executed_actions.push(auto_wood_action_report(
        phase,
        action_kind,
        AutoWoodRuntimeActionStatus::Skipped,
        round_index,
        format!("auto wood skipped: {:?}", reason),
        AutoWoodRuntimeActionOutcome::Skipped(reason),
    ));
}

trait AutoWoodLegacyExitEnterRuleExt {
    fn retry_pre_check_sleep_ms(&self) -> u64;
}

impl AutoWoodLegacyExitEnterRuleExt for AutoWoodLegacyExitEnterRule {
    fn retry_pre_check_sleep_ms(&self) -> u64 {
        1
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskError;
    use std::collections::VecDeque;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum RuntimeCall {
        Start,
        ActivateWindow,
        EnsureGadget(u64),
        Input(AutoWoodInputAction, u64),
        Delay(u64, AutoWoodDelayReason, Option<u64>),
        Ocr(u64, u64),
        Wonderland(u64),
        Probe(String, u64),
        ThirdPartyLogin(u64),
        GarbageCollection(u64, AutoWoodRefreshStrategy),
        Cleanup,
    }

    struct FakeAutoWoodRuntime {
        calls: Vec<RuntimeCall>,
        startup: AutoWoodStartupOutcome,
        window: AutoWoodWindowOutcome,
        gadget_ready: VecDeque<bool>,
        ocr_texts: VecDeque<String>,
        wonderland_outcomes: VecDeque<AutoWoodRefreshOutcome>,
        menu_bag_matches: VecDeque<bool>,
        confirm_matches: VecDeque<bool>,
        enter_game_matches: VecDeque<bool>,
        fail_on_wonderland: bool,
        cleanup_count: u64,
    }

    impl Default for FakeAutoWoodRuntime {
        fn default() -> Self {
            Self {
                calls: Vec::new(),
                startup: AutoWoodStartupOutcome::completed(),
                window: AutoWoodWindowOutcome::activated(),
                gadget_ready: VecDeque::new(),
                ocr_texts: VecDeque::new(),
                wonderland_outcomes: VecDeque::new(),
                menu_bag_matches: VecDeque::new(),
                confirm_matches: VecDeque::new(),
                enter_game_matches: VecDeque::new(),
                fail_on_wonderland: false,
                cleanup_count: 0,
            }
        }
    }

    impl FakeAutoWoodRuntime {
        fn with_ocr_texts(mut self, texts: impl IntoIterator<Item = &'static str>) -> Self {
            self.ocr_texts = texts.into_iter().map(str::to_string).collect();
            self
        }

        fn with_initial_wood(mut self, wood_name: &str, count: u64) -> Self {
            self.startup.initial_wood_totals.push(AutoWoodCountEntry {
                wood_name: wood_name.to_string(),
                count,
            });
            self
        }

        fn with_legacy_matches(
            mut self,
            menu_bag: impl IntoIterator<Item = bool>,
            confirm: impl IntoIterator<Item = bool>,
            enter_game: impl IntoIterator<Item = bool>,
        ) -> Self {
            self.menu_bag_matches = menu_bag.into_iter().collect();
            self.confirm_matches = confirm.into_iter().collect();
            self.enter_game_matches = enter_game.into_iter().collect();
            self
        }

        fn ready_gadget(mut self, count: usize) -> Self {
            self.gadget_ready = std::iter::repeat(true).take(count).collect();
            self
        }
    }

    impl AutoWoodRuntime for FakeAutoWoodRuntime {
        fn start_auto_wood(
            &mut self,
            _plan: &AutoWoodExecutionPlan,
        ) -> Result<AutoWoodStartupOutcome> {
            self.calls.push(RuntimeCall::Start);
            Ok(self.startup.clone())
        }

        fn activate_auto_wood_game_window(
            &mut self,
            _plan: &AutoWoodExecutionPlan,
        ) -> Result<AutoWoodWindowOutcome> {
            self.calls.push(RuntimeCall::ActivateWindow);
            Ok(self.window.clone())
        }

        fn ensure_auto_wood_gadget(
            &mut self,
            _plan: &AutoWoodExecutionPlan,
            context: &AutoWoodRuntimeRoundContext,
        ) -> Result<AutoWoodGadgetOutcome> {
            self.calls
                .push(RuntimeCall::EnsureGadget(context.round_index));
            Ok(AutoWoodGadgetOutcome {
                ready: self.gadget_ready.pop_front().unwrap_or(true),
                switched_or_equipped: false,
                message: None,
            })
        }

        fn dispatch_auto_wood_input(
            &mut self,
            action: AutoWoodInputAction,
            context: &AutoWoodRuntimeRoundContext,
        ) -> Result<AutoWoodInputOutcome> {
            self.calls
                .push(RuntimeCall::Input(action, context.round_index));
            Ok(AutoWoodInputOutcome::dispatched(action))
        }

        fn delay_auto_wood(
            &mut self,
            duration_ms: u64,
            reason: AutoWoodDelayReason,
            context: Option<&AutoWoodRuntimeRoundContext>,
        ) -> Result<AutoWoodDelayOutcome> {
            self.calls.push(RuntimeCall::Delay(
                duration_ms,
                reason,
                context.map(|context| context.round_index),
            ));
            Ok(AutoWoodDelayOutcome {
                duration_ms,
                reason,
            })
        }

        fn ocr_auto_wood_count(
            &mut self,
            _plan: &AutoWoodExecutionPlan,
            context: &AutoWoodRuntimeRoundContext,
            attempt_index: u64,
        ) -> Result<AutoWoodOcrOutcome> {
            self.calls
                .push(RuntimeCall::Ocr(context.round_index, attempt_index));
            Ok(AutoWoodOcrOutcome::text(
                self.ocr_texts.pop_front().unwrap_or_default(),
            ))
        }

        fn run_auto_wood_wonderland_cycle(
            &mut self,
            _plan: &AutoWoodExecutionPlan,
            context: &AutoWoodRuntimeRoundContext,
        ) -> Result<AutoWoodRefreshOutcome> {
            self.calls
                .push(RuntimeCall::Wonderland(context.round_index));
            if self.fail_on_wonderland {
                return Err(TaskError::CommonJobExecution(
                    "WonderlandCycle failed".to_string(),
                ));
            }
            Ok(self.wonderland_outcomes.pop_front().unwrap_or_else(|| {
                AutoWoodRefreshOutcome::completed(AutoWoodRefreshStrategy::WonderlandCycle)
            }))
        }

        fn probe_auto_wood_legacy_template(
            &mut self,
            locator: &AutoWoodTemplateLocator,
            context: &AutoWoodRuntimeRoundContext,
        ) -> Result<bool> {
            self.calls.push(RuntimeCall::Probe(
                locator.name.clone(),
                context.round_index,
            ));
            if locator.name == "MenuBag" {
                return Ok(self.menu_bag_matches.pop_front().unwrap_or(true));
            }
            if locator.name == "AutoWoodConfirm" {
                return Ok(self.confirm_matches.pop_front().unwrap_or(true));
            }
            if locator.name == "EnterGame" {
                return Ok(self.enter_game_matches.pop_front().unwrap_or(true));
            }
            Ok(true)
        }

        fn handle_auto_wood_third_party_login(
            &mut self,
            _plan: &AutoWoodExecutionPlan,
            context: &AutoWoodRuntimeRoundContext,
            mode: AutoWoodThirdPartyLoginMode,
        ) -> Result<AutoWoodThirdPartyLoginOutcome> {
            self.calls
                .push(RuntimeCall::ThirdPartyLogin(context.round_index));
            Ok(AutoWoodThirdPartyLoginOutcome::skipped(mode))
        }

        fn collect_auto_wood_garbage(
            &mut self,
            _plan: &AutoWoodExecutionPlan,
            context: &AutoWoodRuntimeRoundContext,
            strategy: AutoWoodRefreshStrategy,
        ) -> Result<AutoWoodGarbageCollectionOutcome> {
            self.calls.push(RuntimeCall::GarbageCollection(
                context.round_index,
                strategy,
            ));
            Ok(AutoWoodGarbageCollectionOutcome::completed())
        }

        fn cleanup_auto_wood(
            &mut self,
            _plan: &AutoWoodExecutionPlan,
            _state: &AutoWoodExecutorState,
        ) -> Result<AutoWoodCleanupOutcome> {
            self.calls.push(RuntimeCall::Cleanup);
            self.cleanup_count += 1;
            Ok(AutoWoodCleanupOutcome::completed())
        }
    }

    fn auto_wood_plan(
        rounds: u64,
        daily_max: u64,
        ocr_enabled: bool,
        wonderland: bool,
    ) -> AutoWoodExecutionPlan {
        plan_auto_wood(AutoWoodExecutionConfig {
            auto_wood_config: AutoWoodConfig {
                wood_count_ocr_enabled: ocr_enabled,
                use_wonderland_refresh: wonderland,
                ..AutoWoodConfig::default()
            },
            wood_round_num: rounds,
            wood_daily_max_count: daily_max,
            ..AutoWoodExecutionConfig::default()
        })
    }

    #[test]
    fn auto_wood_plan_is_executor_ready_with_live_adapters_pending() {
        let plan = auto_wood_plan(1, 2000, false, true);

        assert!(plan.executor_ready);
        assert!(plan
            .pending_native
            .iter()
            .any(|item| item.contains("injectable executor boundary")));
        assert!(plan
            .pending_native
            .iter()
            .any(|item| item.contains("desktop live adapters")));
        assert!(plan.pending_native.iter().any(
            |item| item.contains("Paddle OCR parity") && item.contains("real-game regression")
        ));
    }

    #[test]
    fn auto_wood_execute_normal_felling_success() {
        let plan = auto_wood_plan(2, 2000, true, true);
        let mut runtime = FakeAutoWoodRuntime::default()
            .ready_gadget(2)
            .with_ocr_texts(["获得\n竹节×30\n杉木×20", ""]);

        let report = execute_auto_wood_plan(&plan, &mut runtime).unwrap();

        assert_eq!(report.status, AutoWoodExecutionStatus::Completed);
        assert!(report.completed);
        assert_eq!(report.state.rounds_started, 2);
        assert_eq!(report.state.rounds_completed, 2);
        assert_eq!(report.state.quick_use_gadget_count, 2);
        assert_eq!(report.state.wonderland_refresh_count, 1);
        assert_eq!(report.state.manual_gc_count, 1);
        assert_eq!(report.state.wood_totals.get("竹节"), Some(&30));
        assert_eq!(report.state.wood_totals.get("杉木"), Some(&20));
        assert_eq!(runtime.cleanup_count, 1);
        assert!(runtime.calls.contains(&RuntimeCall::Wonderland(0)));
        assert_eq!(runtime.calls.last(), Some(&RuntimeCall::Cleanup));
    }

    #[test]
    fn auto_wood_execute_skips_when_count_already_full() {
        let plan = auto_wood_plan(3, 2000, true, true);
        let mut runtime = FakeAutoWoodRuntime::default().with_initial_wood("竹节", 2000);

        let report = execute_auto_wood_plan(&plan, &mut runtime).unwrap();

        assert_eq!(report.status, AutoWoodExecutionStatus::DailyMaxReached);
        assert!(!report.completed);
        assert_eq!(report.state.rounds_started, 0);
        assert_eq!(report.state.quick_use_gadget_count, 0);
        assert_eq!(runtime.cleanup_count, 1);
        assert!(report.skipped_steps.iter().any(|step| {
            step.action_kind == AutoWoodRuntimeActionKind::LoopGuard
                && step.reason == AutoWoodSkipReason::DailyMaxReached
        }));
        assert!(!runtime.calls.iter().any(|call| matches!(
            call,
            RuntimeCall::Input(AutoWoodInputAction::QuickUseGadget, _)
        )));
    }

    #[test]
    fn auto_wood_execute_legacy_refresh_branch() {
        let plan = auto_wood_plan(2, 2000, false, false);
        let mut runtime = FakeAutoWoodRuntime::default()
            .ready_gadget(2)
            .with_legacy_matches([true], [true], [true, false, false, false]);

        let report = execute_auto_wood_plan(&plan, &mut runtime).unwrap();

        assert_eq!(report.status, AutoWoodExecutionStatus::Completed);
        assert_eq!(report.state.legacy_refresh_count, 1);
        assert_eq!(report.state.wonderland_refresh_count, 0);
        assert_eq!(report.state.manual_gc_count, 1);
        assert!(runtime
            .calls
            .contains(&RuntimeCall::Input(AutoWoodInputAction::Escape, 0)));
        assert!(runtime
            .calls
            .contains(&RuntimeCall::Input(AutoWoodInputAction::ClickExitButton, 0)));
        assert!(runtime
            .calls
            .contains(&RuntimeCall::Input(AutoWoodInputAction::ClickConfirm, 0)));
        assert!(runtime
            .calls
            .contains(&RuntimeCall::Input(AutoWoodInputAction::ClickEnterGame, 0)));
        assert!(runtime.calls.contains(&RuntimeCall::GarbageCollection(
            0,
            AutoWoodRefreshStrategy::LegacyExitEnter
        )));
        assert_eq!(runtime.cleanup_count, 1);
    }

    #[test]
    fn auto_wood_execute_runtime_error_still_runs_cleanup() {
        let plan = auto_wood_plan(2, 2000, false, true);
        let mut runtime = FakeAutoWoodRuntime {
            fail_on_wonderland: true,
            ..FakeAutoWoodRuntime::default().ready_gadget(1)
        };

        let error = execute_auto_wood_plan(&plan, &mut runtime).unwrap_err();

        assert!(
            matches!(error, TaskError::CommonJobExecution(message) if message.contains("WonderlandCycle"))
        );
        assert_eq!(runtime.cleanup_count, 1);
        assert_eq!(runtime.calls.last(), Some(&RuntimeCall::Cleanup));
        assert!(runtime.calls.contains(&RuntimeCall::Wonderland(0)));
    }
}
