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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoDomainExecutionStatus {
    Completed,
    StartupFailed,
    EntryFailed,
    CombatFailed,
    RewardSkipped,
    RewardFailed,
    ContinueFailed,
    RetryLimitReached,
    Cancelled,
    CleanupFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoDomainRuntimeActionStatus {
    Succeeded,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoDomainRuntimeActionKind {
    Startup,
    NotifyStart,
    TeleportToDomain,
    EnterDomain,
    CloseDomainTip,
    InitializeTeam,
    SelectCombatScript,
    WalkToFightKey,
    RunAutoFight,
    WaitAfterFight,
    MoveToPetrifiedTree,
    UseResin,
    RecognizeReward,
    ContinueOrExit,
    WaitMainUi,
    ArtifactSalvage,
    NotifyEnd,
    Cleanup,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoDomainRewardDecision {
    Claim,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoDomainSkipReason {
    ClaimDisabled,
    OriginalResinInsufficient,
    SpecifiedResinUnavailable,
    RewardPromptMissing,
    RuntimeRequestedStop,
    RoundLimitReached,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainRuntimeRoundContext {
    pub round_index: u32,
    pub total_rounds: u32,
    pub is_first_round: bool,
    pub is_last_round: bool,
    pub claimed_rewards: u32,
    pub selected_resin: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainStartupOutcome {
    pub completed: bool,
    pub assets_initialized: bool,
    pub combat_strategy_parsed: bool,
    pub auto_eat_trigger_registered: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainNotificationOutcome {
    pub sent: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainTeleportOutcome {
    pub attempted: bool,
    pub completed: bool,
    pub domain_name: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainEntryOutcome {
    pub completed: bool,
    pub matched: bool,
    pub team_selected: bool,
    pub started: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainBasicOutcome {
    pub completed: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainCombatOutcome {
    pub completed: bool,
    pub challenge_completed: bool,
    pub auto_leaving_detected: bool,
    pub duration_ms: Option<u64>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainTreeOutcome {
    pub completed: bool,
    pub tree_detected: bool,
    pub prompt_found: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainResinSelection {
    pub decision: AutoDomainRewardDecision,
    pub resin_name: Option<String>,
    pub available_count: Option<i32>,
    pub skip_reason: Option<AutoDomainSkipReason>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainRewardOutcome {
    pub claimed: bool,
    pub resin_name: Option<String>,
    pub stop_after_claim: bool,
    pub skip_reason: Option<AutoDomainSkipReason>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainRewardRecognitionOutcome {
    pub attempted: bool,
    pub recognized: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainContinuationOutcome {
    pub completed: bool,
    pub continue_next_round: bool,
    pub exited_domain: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainArtifactSalvageOutcome {
    pub attempted: bool,
    pub completed: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainCleanupOutcome {
    pub completed: bool,
    pub inputs_released: bool,
    pub overlays_cleared: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoDomainRuntimeActionOutcome {
    Startup(AutoDomainStartupOutcome),
    Notification(AutoDomainNotificationOutcome),
    Teleport(AutoDomainTeleportOutcome),
    Entry(AutoDomainEntryOutcome),
    Basic(AutoDomainBasicOutcome),
    Combat(AutoDomainCombatOutcome),
    Tree(AutoDomainTreeOutcome),
    ResinSelection(AutoDomainResinSelection),
    Reward(AutoDomainRewardOutcome),
    RewardRecognition(AutoDomainRewardRecognitionOutcome),
    Continuation(AutoDomainContinuationOutcome),
    ArtifactSalvage(AutoDomainArtifactSalvageOutcome),
    Cleanup(AutoDomainCleanupOutcome),
    Skipped(AutoDomainSkipReason),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainRuntimeActionReport {
    pub phase: AutoDomainTaskPhase,
    pub action_kind: AutoDomainRuntimeActionKind,
    pub status: AutoDomainRuntimeActionStatus,
    pub round_index: Option<u32>,
    pub detail: String,
    pub outcome: AutoDomainRuntimeActionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainSkippedStep {
    pub action_kind: AutoDomainRuntimeActionKind,
    pub round_index: Option<u32>,
    pub reason: AutoDomainSkipReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainExecutorState {
    pub startup_completed: bool,
    pub current_round: u32,
    pub target_rounds: u32,
    pub entered_domain: bool,
    pub team_initialized: bool,
    pub combat_script_selected: bool,
    pub fights_attempted: u32,
    pub fights_succeeded: u32,
    pub rewards_claimed: u32,
    pub rewards_skipped: u32,
    pub reward_recognition_attempts: u32,
    pub resin_records: Vec<AutoDomainResinUseRecord>,
    pub selected_resin: Option<String>,
    pub retries_used: u32,
    pub cancelled: bool,
    pub cleanup_completed: bool,
    pub last_skip_reason: Option<AutoDomainSkipReason>,
}

impl AutoDomainExecutorState {
    fn new(plan: &AutoDomainExecutionPlan) -> Self {
        Self {
            startup_completed: false,
            current_round: 0,
            target_rounds: auto_domain_target_rounds(plan),
            entered_domain: false,
            team_initialized: false,
            combat_script_selected: false,
            fights_attempted: 0,
            fights_succeeded: 0,
            rewards_claimed: 0,
            rewards_skipped: 0,
            reward_recognition_attempts: 0,
            resin_records: plan.resin_rule.specified_records.clone(),
            selected_resin: None,
            retries_used: 0,
            cancelled: false,
            cleanup_completed: false,
            last_skip_reason: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDomainExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub status: AutoDomainExecutionStatus,
    pub state: AutoDomainExecutorState,
    pub executed_actions: Vec<AutoDomainRuntimeActionReport>,
    pub skipped_steps: Vec<AutoDomainSkippedStep>,
}

pub trait AutoDomainRuntime {
    fn start_auto_domain(
        &mut self,
        plan: &AutoDomainExecutionPlan,
    ) -> Result<AutoDomainStartupOutcome>;

    fn notify_auto_domain_start(
        &mut self,
        plan: &AutoDomainExecutionPlan,
    ) -> Result<AutoDomainNotificationOutcome>;

    fn teleport_auto_domain_to_domain(
        &mut self,
        plan: &AutoDomainExecutionPlan,
    ) -> Result<AutoDomainTeleportOutcome>;

    fn enter_auto_domain_challenge(
        &mut self,
        plan: &AutoDomainExecutionPlan,
        context: &AutoDomainRuntimeRoundContext,
    ) -> Result<AutoDomainEntryOutcome>;

    fn close_auto_domain_tip(
        &mut self,
        plan: &AutoDomainExecutionPlan,
        context: &AutoDomainRuntimeRoundContext,
    ) -> Result<AutoDomainBasicOutcome>;

    fn initialize_auto_domain_team(
        &mut self,
        plan: &AutoDomainExecutionPlan,
        context: &AutoDomainRuntimeRoundContext,
    ) -> Result<AutoDomainBasicOutcome>;

    fn select_auto_domain_combat_script(
        &mut self,
        plan: &AutoDomainExecutionPlan,
        context: &AutoDomainRuntimeRoundContext,
    ) -> Result<AutoDomainBasicOutcome>;

    fn walk_auto_domain_to_fight_key(
        &mut self,
        plan: &AutoDomainExecutionPlan,
        context: &AutoDomainRuntimeRoundContext,
    ) -> Result<AutoDomainBasicOutcome>;

    fn run_auto_domain_fight(
        &mut self,
        plan: &AutoDomainExecutionPlan,
        context: &AutoDomainRuntimeRoundContext,
    ) -> Result<AutoDomainCombatOutcome>;

    fn wait_auto_domain_after_fight(
        &mut self,
        plan: &AutoDomainExecutionPlan,
        context: &AutoDomainRuntimeRoundContext,
    ) -> Result<AutoDomainBasicOutcome>;

    fn move_auto_domain_to_petrified_tree(
        &mut self,
        plan: &AutoDomainExecutionPlan,
        context: &AutoDomainRuntimeRoundContext,
    ) -> Result<AutoDomainTreeOutcome>;

    fn select_auto_domain_resin(
        &mut self,
        plan: &AutoDomainExecutionPlan,
        context: &AutoDomainRuntimeRoundContext,
        records: &[AutoDomainResinUseRecord],
    ) -> Result<AutoDomainResinSelection>;

    fn claim_auto_domain_reward(
        &mut self,
        plan: &AutoDomainExecutionPlan,
        context: &AutoDomainRuntimeRoundContext,
        selection: &AutoDomainResinSelection,
    ) -> Result<AutoDomainRewardOutcome>;

    fn recognize_auto_domain_reward(
        &mut self,
        plan: &AutoDomainExecutionPlan,
        context: &AutoDomainRuntimeRoundContext,
    ) -> Result<AutoDomainRewardRecognitionOutcome>;

    fn continue_or_exit_auto_domain(
        &mut self,
        plan: &AutoDomainExecutionPlan,
        context: &AutoDomainRuntimeRoundContext,
        should_continue: bool,
    ) -> Result<AutoDomainContinuationOutcome>;

    fn wait_auto_domain_main_ui(
        &mut self,
        plan: &AutoDomainExecutionPlan,
    ) -> Result<AutoDomainBasicOutcome>;

    fn run_auto_domain_artifact_salvage(
        &mut self,
        plan: &AutoDomainExecutionPlan,
    ) -> Result<AutoDomainArtifactSalvageOutcome>;

    fn notify_auto_domain_end(
        &mut self,
        plan: &AutoDomainExecutionPlan,
        status: AutoDomainExecutionStatus,
    ) -> Result<AutoDomainNotificationOutcome>;

    fn cleanup_auto_domain(
        &mut self,
        plan: &AutoDomainExecutionPlan,
    ) -> Result<AutoDomainCleanupOutcome>;

    fn is_auto_domain_cancelled(&mut self) -> bool {
        false
    }
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
        executor_ready: true,
        pending_native: vec![
            "Rust AutoDomain executor boundary is ready and testable through AutoDomainRuntime injection; desktop live adapters for TaskRunner lock, cancellation-aware delay, notifications, and game-window input are not wired yet".to_string(),
            "domain teleport/entry, live BV/template/OCR helpers, resin prompt OCR/clicks, BgiTree YOLO movement, reward recognition, and artifact salvage are represented as injectable runtime calls; desktop live adapters remain native-pending".to_string(),
            "full AutoFight/CombatScenes live adapters for team recognition, combat script execution, and domain-end OCR are still not connected to this Rust boundary".to_string(),
        ],
        param,
    })
}

pub fn execute_auto_domain_plan<R>(
    plan: &AutoDomainExecutionPlan,
    runtime: &mut R,
) -> Result<AutoDomainExecutionReport>
where
    R: AutoDomainRuntime,
{
    let mut state = AutoDomainExecutorState::new(plan);
    let mut executed_actions = Vec::new();
    let mut skipped_steps = Vec::new();

    let execution_result = execute_auto_domain_plan_inner(
        plan,
        runtime,
        &mut state,
        &mut executed_actions,
        &mut skipped_steps,
    );
    let status = match execution_result {
        Ok(status) => status,
        Err(error) => {
            let cleanup_error = execute_auto_domain_cleanup(
                plan,
                runtime,
                AutoDomainExecutionStatus::CleanupFailed,
                &mut state,
                &mut executed_actions,
            )
            .err();
            return Err(cleanup_error.unwrap_or(error));
        }
    };

    let cleanup_status =
        execute_auto_domain_cleanup(plan, runtime, status, &mut state, &mut executed_actions)?;
    let status = if cleanup_status == AutoDomainExecutionStatus::CleanupFailed {
        AutoDomainExecutionStatus::CleanupFailed
    } else {
        status
    };

    Ok(auto_domain_report(
        plan,
        status,
        state,
        executed_actions,
        skipped_steps,
    ))
}

fn execute_auto_domain_plan_inner<R>(
    plan: &AutoDomainExecutionPlan,
    runtime: &mut R,
    state: &mut AutoDomainExecutorState,
    executed_actions: &mut Vec<AutoDomainRuntimeActionReport>,
    skipped_steps: &mut Vec<AutoDomainSkippedStep>,
) -> Result<AutoDomainExecutionStatus>
where
    R: AutoDomainRuntime,
{
    let startup = runtime.start_auto_domain(plan)?;
    state.startup_completed = startup.completed;
    executed_actions.push(auto_domain_action_report(
        AutoDomainTaskPhase::Startup,
        AutoDomainRuntimeActionKind::Startup,
        if startup.completed {
            AutoDomainRuntimeActionStatus::Succeeded
        } else {
            AutoDomainRuntimeActionStatus::Failed
        },
        None,
        startup
            .message
            .clone()
            .unwrap_or_else(|| "auto domain startup boundary completed".to_string()),
        AutoDomainRuntimeActionOutcome::Startup(startup.clone()),
    ));
    if !startup.completed {
        return Ok(AutoDomainExecutionStatus::StartupFailed);
    }

    let start_notification = runtime.notify_auto_domain_start(plan)?;
    executed_actions.push(auto_domain_action_report(
        AutoDomainTaskPhase::Startup,
        AutoDomainRuntimeActionKind::NotifyStart,
        if start_notification.sent {
            AutoDomainRuntimeActionStatus::Succeeded
        } else {
            AutoDomainRuntimeActionStatus::Skipped
        },
        None,
        start_notification
            .message
            .clone()
            .unwrap_or_else(|| "start notification boundary completed".to_string()),
        AutoDomainRuntimeActionOutcome::Notification(start_notification),
    ));

    if runtime.is_auto_domain_cancelled() {
        state.cancelled = true;
        return Ok(auto_domain_skip(
            state,
            executed_actions,
            skipped_steps,
            AutoDomainTaskPhase::Startup,
            AutoDomainRuntimeActionKind::Skip,
            None,
            AutoDomainSkipReason::Cancelled,
            AutoDomainExecutionStatus::Cancelled,
        ));
    }

    if plan.domain_entry_rule.teleports_when_domain_name_configured {
        let teleport = runtime.teleport_auto_domain_to_domain(plan)?;
        let completed = teleport.completed;
        executed_actions.push(auto_domain_action_report(
            AutoDomainTaskPhase::Teleport,
            AutoDomainRuntimeActionKind::TeleportToDomain,
            if completed {
                AutoDomainRuntimeActionStatus::Succeeded
            } else {
                AutoDomainRuntimeActionStatus::Failed
            },
            None,
            teleport
                .message
                .clone()
                .unwrap_or_else(|| "domain teleport boundary completed".to_string()),
            AutoDomainRuntimeActionOutcome::Teleport(teleport),
        ));
        if !completed {
            return Ok(AutoDomainExecutionStatus::EntryFailed);
        }
    } else {
        executed_actions.push(auto_domain_action_report(
            AutoDomainTaskPhase::Teleport,
            AutoDomainRuntimeActionKind::TeleportToDomain,
            AutoDomainRuntimeActionStatus::Skipped,
            None,
            "domain name is not configured; teleport skipped".to_string(),
            AutoDomainRuntimeActionOutcome::Skipped(AutoDomainSkipReason::RuntimeRequestedStop),
        ));
    }

    loop {
        if state.current_round >= state.target_rounds {
            return Ok(AutoDomainExecutionStatus::Completed);
        }
        if runtime.is_auto_domain_cancelled() {
            state.cancelled = true;
            return Ok(auto_domain_skip(
                state,
                executed_actions,
                skipped_steps,
                AutoDomainTaskPhase::RoundLoop,
                AutoDomainRuntimeActionKind::Skip,
                None,
                AutoDomainSkipReason::Cancelled,
                AutoDomainExecutionStatus::Cancelled,
            ));
        }

        state.current_round += 1;
        let round_index = state.current_round;
        let mut context = auto_domain_round_context(plan, state, None);

        if !state.entered_domain {
            let entry = runtime.enter_auto_domain_challenge(plan, &context)?;
            let completed = entry.completed && entry.matched && entry.started;
            state.entered_domain = completed;
            executed_actions.push(auto_domain_action_report(
                AutoDomainTaskPhase::EnterDomain,
                AutoDomainRuntimeActionKind::EnterDomain,
                if completed {
                    AutoDomainRuntimeActionStatus::Succeeded
                } else {
                    AutoDomainRuntimeActionStatus::Failed
                },
                Some(round_index),
                entry
                    .message
                    .clone()
                    .unwrap_or_else(|| "domain entry boundary completed".to_string()),
                AutoDomainRuntimeActionOutcome::Entry(entry),
            ));
            if !completed {
                if auto_domain_retry_available(plan, state) {
                    state.retries_used += 1;
                    state.current_round = state.current_round.saturating_sub(1);
                    continue;
                }
                return Ok(AutoDomainExecutionStatus::EntryFailed);
            }
        }

        let close_tip = runtime.close_auto_domain_tip(plan, &context)?;
        executed_actions.push(auto_domain_basic_report(
            AutoDomainTaskPhase::RoundLoop,
            AutoDomainRuntimeActionKind::CloseDomainTip,
            Some(round_index),
            "close domain tip boundary completed",
            close_tip,
        ));

        if plan.combat_rule.initializes_team_on_first_round && !state.team_initialized {
            let team = runtime.initialize_auto_domain_team(plan, &context)?;
            state.team_initialized = team.completed;
            let completed = team.completed;
            executed_actions.push(auto_domain_basic_report(
                AutoDomainTaskPhase::RoundLoop,
                AutoDomainRuntimeActionKind::InitializeTeam,
                Some(round_index),
                "team initialization boundary completed",
                team,
            ));
            if !completed {
                if auto_domain_retry_available(plan, state) {
                    state.retries_used += 1;
                    state.current_round = state.current_round.saturating_sub(1);
                    state.entered_domain = false;
                    continue;
                }
                return Ok(AutoDomainExecutionStatus::EntryFailed);
            }
        }

        if !state.combat_script_selected {
            let script = runtime.select_auto_domain_combat_script(plan, &context)?;
            state.combat_script_selected = script.completed;
            let completed = script.completed;
            executed_actions.push(auto_domain_basic_report(
                AutoDomainTaskPhase::RoundLoop,
                AutoDomainRuntimeActionKind::SelectCombatScript,
                Some(round_index),
                "combat script selection boundary completed",
                script,
            ));
            if !completed {
                return Ok(AutoDomainExecutionStatus::EntryFailed);
            }
        }

        let walk = runtime.walk_auto_domain_to_fight_key(plan, &context)?;
        let walk_completed = walk.completed;
        executed_actions.push(auto_domain_basic_report(
            AutoDomainTaskPhase::Fight,
            AutoDomainRuntimeActionKind::WalkToFightKey,
            Some(round_index),
            "walk to fight key boundary completed",
            walk,
        ));
        if !walk_completed {
            return Ok(AutoDomainExecutionStatus::EntryFailed);
        }

        state.fights_attempted += 1;
        let combat = runtime.run_auto_domain_fight(plan, &context)?;
        let combat_completed = combat.completed && combat.challenge_completed;
        if combat_completed {
            state.fights_succeeded += 1;
        }
        executed_actions.push(auto_domain_action_report(
            AutoDomainTaskPhase::Fight,
            AutoDomainRuntimeActionKind::RunAutoFight,
            if combat_completed {
                AutoDomainRuntimeActionStatus::Succeeded
            } else {
                AutoDomainRuntimeActionStatus::Failed
            },
            Some(round_index),
            combat
                .message
                .clone()
                .unwrap_or_else(|| "auto fight boundary completed".to_string()),
            AutoDomainRuntimeActionOutcome::Combat(combat),
        ));
        if !combat_completed {
            return Ok(AutoDomainExecutionStatus::CombatFailed);
        }

        let wait = runtime.wait_auto_domain_after_fight(plan, &context)?;
        executed_actions.push(auto_domain_basic_report(
            AutoDomainTaskPhase::Fight,
            AutoDomainRuntimeActionKind::WaitAfterFight,
            Some(round_index),
            "post-fight wait boundary completed",
            wait,
        ));

        let tree = runtime.move_auto_domain_to_petrified_tree(plan, &context)?;
        let tree_completed = tree.completed && tree.prompt_found;
        executed_actions.push(auto_domain_action_report(
            AutoDomainTaskPhase::PetrifiedTree,
            AutoDomainRuntimeActionKind::MoveToPetrifiedTree,
            if tree_completed {
                AutoDomainRuntimeActionStatus::Succeeded
            } else {
                AutoDomainRuntimeActionStatus::Failed
            },
            Some(round_index),
            tree.message
                .clone()
                .unwrap_or_else(|| "petrified tree boundary completed".to_string()),
            AutoDomainRuntimeActionOutcome::Tree(tree),
        ));
        if !tree_completed {
            return Ok(auto_domain_skip(
                state,
                executed_actions,
                skipped_steps,
                AutoDomainTaskPhase::Reward,
                AutoDomainRuntimeActionKind::Skip,
                Some(round_index),
                AutoDomainSkipReason::RewardPromptMissing,
                AutoDomainExecutionStatus::RewardSkipped,
            ));
        }

        let selection = runtime.select_auto_domain_resin(plan, &context, &state.resin_records)?;
        context.selected_resin = selection.resin_name.clone();
        state.selected_resin = selection.resin_name.clone();
        let selection_status = match selection.decision {
            AutoDomainRewardDecision::Claim => AutoDomainRuntimeActionStatus::Succeeded,
            AutoDomainRewardDecision::Skip => AutoDomainRuntimeActionStatus::Skipped,
        };
        executed_actions.push(auto_domain_action_report(
            AutoDomainTaskPhase::Reward,
            AutoDomainRuntimeActionKind::UseResin,
            selection_status,
            Some(round_index),
            selection
                .message
                .clone()
                .unwrap_or_else(|| "resin selection boundary completed".to_string()),
            AutoDomainRuntimeActionOutcome::ResinSelection(selection.clone()),
        ));
        if selection.decision == AutoDomainRewardDecision::Skip {
            let reason = selection
                .skip_reason
                .unwrap_or(AutoDomainSkipReason::RuntimeRequestedStop);
            return Ok(auto_domain_skip(
                state,
                executed_actions,
                skipped_steps,
                AutoDomainTaskPhase::Reward,
                AutoDomainRuntimeActionKind::UseResin,
                Some(round_index),
                reason,
                AutoDomainExecutionStatus::RewardSkipped,
            ));
        }

        let reward = runtime.claim_auto_domain_reward(plan, &context, &selection)?;
        if reward.claimed {
            state.rewards_claimed += 1;
            auto_domain_consume_resin_record(
                &mut state.resin_records,
                reward.resin_name.as_deref(),
            );
        } else {
            state.rewards_skipped += 1;
            state.last_skip_reason = reward.skip_reason;
        }
        executed_actions.push(auto_domain_action_report(
            AutoDomainTaskPhase::Reward,
            AutoDomainRuntimeActionKind::UseResin,
            if reward.claimed {
                AutoDomainRuntimeActionStatus::Succeeded
            } else {
                AutoDomainRuntimeActionStatus::Skipped
            },
            Some(round_index),
            reward
                .message
                .clone()
                .unwrap_or_else(|| "reward claim boundary completed".to_string()),
            AutoDomainRuntimeActionOutcome::Reward(reward.clone()),
        ));
        if !reward.claimed {
            let reason = reward
                .skip_reason
                .unwrap_or(AutoDomainSkipReason::RuntimeRequestedStop);
            return Ok(auto_domain_skip(
                state,
                executed_actions,
                skipped_steps,
                AutoDomainTaskPhase::Reward,
                AutoDomainRuntimeActionKind::UseResin,
                Some(round_index),
                reason,
                AutoDomainExecutionStatus::RewardSkipped,
            ));
        }

        if plan.reward_rule.reward_recognition_enabled {
            let recognition = runtime.recognize_auto_domain_reward(plan, &context)?;
            state.reward_recognition_attempts += u32::from(recognition.attempted);
            executed_actions.push(auto_domain_action_report(
                AutoDomainTaskPhase::Reward,
                AutoDomainRuntimeActionKind::RecognizeReward,
                if !recognition.attempted {
                    AutoDomainRuntimeActionStatus::Skipped
                } else if recognition.recognized {
                    AutoDomainRuntimeActionStatus::Succeeded
                } else {
                    AutoDomainRuntimeActionStatus::Failed
                },
                Some(round_index),
                recognition
                    .message
                    .clone()
                    .unwrap_or_else(|| "reward recognition boundary completed".to_string()),
                AutoDomainRuntimeActionOutcome::RewardRecognition(recognition),
            ));
        }

        let should_continue = !reward.stop_after_claim && state.current_round < state.target_rounds;
        let continuation = runtime.continue_or_exit_auto_domain(plan, &context, should_continue)?;
        let continuation_completed = continuation.completed
            && continuation.continue_next_round == should_continue
            && (should_continue || continuation.exited_domain);
        executed_actions.push(auto_domain_action_report(
            AutoDomainTaskPhase::Reward,
            AutoDomainRuntimeActionKind::ContinueOrExit,
            if continuation_completed {
                AutoDomainRuntimeActionStatus::Succeeded
            } else {
                AutoDomainRuntimeActionStatus::Failed
            },
            Some(round_index),
            continuation
                .message
                .clone()
                .unwrap_or_else(|| "continue or exit boundary completed".to_string()),
            AutoDomainRuntimeActionOutcome::Continuation(continuation.clone()),
        ));
        if !continuation_completed {
            return Ok(AutoDomainExecutionStatus::ContinueFailed);
        }

        if continuation.exited_domain {
            state.entered_domain = false;
        }
        if !should_continue {
            return Ok(AutoDomainExecutionStatus::Completed);
        }
    }
}

fn execute_auto_domain_cleanup<R>(
    plan: &AutoDomainExecutionPlan,
    runtime: &mut R,
    execution_status: AutoDomainExecutionStatus,
    state: &mut AutoDomainExecutorState,
    executed_actions: &mut Vec<AutoDomainRuntimeActionReport>,
) -> Result<AutoDomainExecutionStatus>
where
    R: AutoDomainRuntime,
{
    let cleanup = runtime.cleanup_auto_domain(plan)?;
    state.cleanup_completed = cleanup.completed;
    let cleanup_completed = cleanup.completed;
    executed_actions.push(auto_domain_action_report(
        AutoDomainTaskPhase::Finish,
        AutoDomainRuntimeActionKind::Cleanup,
        if cleanup_completed {
            AutoDomainRuntimeActionStatus::Succeeded
        } else {
            AutoDomainRuntimeActionStatus::Failed
        },
        None,
        cleanup
            .message
            .clone()
            .unwrap_or_else(|| "auto domain cleanup boundary completed".to_string()),
        AutoDomainRuntimeActionOutcome::Cleanup(cleanup),
    ));

    let wait = runtime.wait_auto_domain_main_ui(plan)?;
    executed_actions.push(auto_domain_basic_report(
        AutoDomainTaskPhase::Finish,
        AutoDomainRuntimeActionKind::WaitMainUi,
        None,
        "wait main UI boundary completed",
        wait,
    ));

    if plan.artifact_salvage_rule.enabled {
        let salvage = runtime.run_auto_domain_artifact_salvage(plan)?;
        executed_actions.push(auto_domain_action_report(
            AutoDomainTaskPhase::Finish,
            AutoDomainRuntimeActionKind::ArtifactSalvage,
            if salvage.completed {
                AutoDomainRuntimeActionStatus::Succeeded
            } else if salvage.attempted {
                AutoDomainRuntimeActionStatus::Failed
            } else {
                AutoDomainRuntimeActionStatus::Skipped
            },
            None,
            salvage
                .message
                .clone()
                .unwrap_or_else(|| "artifact salvage boundary completed".to_string()),
            AutoDomainRuntimeActionOutcome::ArtifactSalvage(salvage),
        ));
    }

    let notification_status = if cleanup_completed {
        execution_status
    } else {
        AutoDomainExecutionStatus::CleanupFailed
    };
    let end_notification = runtime.notify_auto_domain_end(plan, notification_status)?;
    executed_actions.push(auto_domain_action_report(
        AutoDomainTaskPhase::Finish,
        AutoDomainRuntimeActionKind::NotifyEnd,
        if end_notification.sent {
            AutoDomainRuntimeActionStatus::Succeeded
        } else {
            AutoDomainRuntimeActionStatus::Skipped
        },
        None,
        end_notification
            .message
            .clone()
            .unwrap_or_else(|| "end notification boundary completed".to_string()),
        AutoDomainRuntimeActionOutcome::Notification(end_notification),
    ));

    if cleanup_completed {
        Ok(AutoDomainExecutionStatus::Completed)
    } else {
        Ok(AutoDomainExecutionStatus::CleanupFailed)
    }
}

fn auto_domain_target_rounds(plan: &AutoDomainExecutionPlan) -> u32 {
    plan.param
        .domain_round_num
        .max(1)
        .try_into()
        .unwrap_or(AUTO_DOMAIN_UNLIMITED_ROUNDS as u32)
}

fn auto_domain_retry_available(
    plan: &AutoDomainExecutionPlan,
    state: &AutoDomainExecutorState,
) -> bool {
    !plan.param.domain_name.trim().is_empty()
        && state.retries_used < plan.retry_rule.revive_retry_count as u32
}

fn auto_domain_round_context(
    plan: &AutoDomainExecutionPlan,
    state: &AutoDomainExecutorState,
    selected_resin: Option<String>,
) -> AutoDomainRuntimeRoundContext {
    AutoDomainRuntimeRoundContext {
        round_index: state.current_round,
        total_rounds: state.target_rounds,
        is_first_round: state.current_round <= 1,
        is_last_round: state.current_round >= auto_domain_target_rounds(plan),
        claimed_rewards: state.rewards_claimed,
        selected_resin,
    }
}

fn auto_domain_consume_resin_record(
    records: &mut [AutoDomainResinUseRecord],
    resin_name: Option<&str>,
) {
    let Some(resin_name) = resin_name else {
        return;
    };
    if let Some(record) = records
        .iter_mut()
        .find(|record| record.name == resin_name && record.remain_count > 0)
    {
        record.remain_count -= 1;
    }
}

fn auto_domain_skip(
    state: &mut AutoDomainExecutorState,
    executed_actions: &mut Vec<AutoDomainRuntimeActionReport>,
    skipped_steps: &mut Vec<AutoDomainSkippedStep>,
    phase: AutoDomainTaskPhase,
    action_kind: AutoDomainRuntimeActionKind,
    round_index: Option<u32>,
    reason: AutoDomainSkipReason,
    status: AutoDomainExecutionStatus,
) -> AutoDomainExecutionStatus {
    state.rewards_skipped += u32::from(matches!(
        reason,
        AutoDomainSkipReason::ClaimDisabled
            | AutoDomainSkipReason::OriginalResinInsufficient
            | AutoDomainSkipReason::SpecifiedResinUnavailable
            | AutoDomainSkipReason::RewardPromptMissing
            | AutoDomainSkipReason::RuntimeRequestedStop
    ));
    state.last_skip_reason = Some(reason);
    skipped_steps.push(AutoDomainSkippedStep {
        action_kind,
        round_index,
        reason,
    });
    executed_actions.push(auto_domain_action_report(
        phase,
        action_kind,
        AutoDomainRuntimeActionStatus::Skipped,
        round_index,
        format!("skipped AutoDomain step: {:?}", reason),
        AutoDomainRuntimeActionOutcome::Skipped(reason),
    ));
    status
}

fn auto_domain_report(
    plan: &AutoDomainExecutionPlan,
    status: AutoDomainExecutionStatus,
    state: AutoDomainExecutorState,
    executed_actions: Vec<AutoDomainRuntimeActionReport>,
    skipped_steps: Vec<AutoDomainSkippedStep>,
) -> AutoDomainExecutionReport {
    AutoDomainExecutionReport {
        task_key: plan.task_key.clone(),
        completed: status == AutoDomainExecutionStatus::Completed,
        status,
        state,
        executed_actions,
        skipped_steps,
    }
}

fn auto_domain_basic_report(
    phase: AutoDomainTaskPhase,
    action_kind: AutoDomainRuntimeActionKind,
    round_index: Option<u32>,
    default_detail: &str,
    outcome: AutoDomainBasicOutcome,
) -> AutoDomainRuntimeActionReport {
    let status = if outcome.completed {
        AutoDomainRuntimeActionStatus::Succeeded
    } else {
        AutoDomainRuntimeActionStatus::Failed
    };
    auto_domain_action_report(
        phase,
        action_kind,
        status,
        round_index,
        outcome
            .message
            .clone()
            .unwrap_or_else(|| default_detail.to_string()),
        AutoDomainRuntimeActionOutcome::Basic(outcome),
    )
}

fn auto_domain_action_report(
    phase: AutoDomainTaskPhase,
    action_kind: AutoDomainRuntimeActionKind,
    status: AutoDomainRuntimeActionStatus,
    round_index: Option<u32>,
    detail: impl Into<String>,
    outcome: AutoDomainRuntimeActionOutcome,
) -> AutoDomainRuntimeActionReport {
    AutoDomainRuntimeActionReport {
        phase,
        action_kind,
        status,
        round_index,
        detail: detail.into(),
        outcome,
    }
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

#[cfg(test)]
mod auto_domain_executor_tests {
    use super::*;

    #[derive(Debug)]
    struct MockAutoDomainRuntime {
        calls: Vec<AutoDomainRuntimeActionKind>,
        startup: AutoDomainStartupOutcome,
        fight: AutoDomainCombatOutcome,
        selection: AutoDomainResinSelection,
        reward: AutoDomainRewardOutcome,
        cleanup: AutoDomainCleanupOutcome,
        entry_failures_remaining: u32,
        entry_calls: u32,
        claim_calls: u32,
        cleanup_calls: u32,
    }

    impl Default for MockAutoDomainRuntime {
        fn default() -> Self {
            Self {
                calls: Vec::new(),
                startup: AutoDomainStartupOutcome {
                    completed: true,
                    assets_initialized: true,
                    combat_strategy_parsed: true,
                    auto_eat_trigger_registered: false,
                    message: None,
                },
                fight: AutoDomainCombatOutcome {
                    completed: true,
                    challenge_completed: true,
                    auto_leaving_detected: false,
                    duration_ms: Some(12_000),
                    message: None,
                },
                selection: AutoDomainResinSelection {
                    decision: AutoDomainRewardDecision::Claim,
                    resin_name: Some("浓缩树脂".to_string()),
                    available_count: Some(1),
                    skip_reason: None,
                    message: None,
                },
                reward: AutoDomainRewardOutcome {
                    claimed: true,
                    resin_name: Some("浓缩树脂".to_string()),
                    stop_after_claim: false,
                    skip_reason: None,
                    message: None,
                },
                cleanup: AutoDomainCleanupOutcome {
                    completed: true,
                    inputs_released: true,
                    overlays_cleared: true,
                    message: None,
                },
                entry_failures_remaining: 0,
                entry_calls: 0,
                claim_calls: 0,
                cleanup_calls: 0,
            }
        }
    }

    impl AutoDomainRuntime for MockAutoDomainRuntime {
        fn start_auto_domain(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
        ) -> Result<AutoDomainStartupOutcome> {
            self.calls.push(AutoDomainRuntimeActionKind::Startup);
            Ok(self.startup.clone())
        }

        fn notify_auto_domain_start(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
        ) -> Result<AutoDomainNotificationOutcome> {
            self.calls.push(AutoDomainRuntimeActionKind::NotifyStart);
            Ok(notification_outcome())
        }

        fn teleport_auto_domain_to_domain(
            &mut self,
            plan: &AutoDomainExecutionPlan,
        ) -> Result<AutoDomainTeleportOutcome> {
            self.calls
                .push(AutoDomainRuntimeActionKind::TeleportToDomain);
            Ok(AutoDomainTeleportOutcome {
                attempted: true,
                completed: true,
                domain_name: plan.param.domain_name.clone(),
                message: None,
            })
        }

        fn enter_auto_domain_challenge(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
            _context: &AutoDomainRuntimeRoundContext,
        ) -> Result<AutoDomainEntryOutcome> {
            self.calls.push(AutoDomainRuntimeActionKind::EnterDomain);
            self.entry_calls += 1;
            if self.entry_failures_remaining > 0 {
                self.entry_failures_remaining -= 1;
                return Ok(AutoDomainEntryOutcome {
                    completed: false,
                    matched: false,
                    team_selected: false,
                    started: false,
                    message: Some("entry failed in test".to_string()),
                });
            }
            Ok(AutoDomainEntryOutcome {
                completed: true,
                matched: true,
                team_selected: true,
                started: true,
                message: None,
            })
        }

        fn close_auto_domain_tip(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
            _context: &AutoDomainRuntimeRoundContext,
        ) -> Result<AutoDomainBasicOutcome> {
            self.calls.push(AutoDomainRuntimeActionKind::CloseDomainTip);
            Ok(basic_outcome())
        }

        fn initialize_auto_domain_team(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
            _context: &AutoDomainRuntimeRoundContext,
        ) -> Result<AutoDomainBasicOutcome> {
            self.calls.push(AutoDomainRuntimeActionKind::InitializeTeam);
            Ok(basic_outcome())
        }

        fn select_auto_domain_combat_script(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
            _context: &AutoDomainRuntimeRoundContext,
        ) -> Result<AutoDomainBasicOutcome> {
            self.calls
                .push(AutoDomainRuntimeActionKind::SelectCombatScript);
            Ok(basic_outcome())
        }

        fn walk_auto_domain_to_fight_key(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
            _context: &AutoDomainRuntimeRoundContext,
        ) -> Result<AutoDomainBasicOutcome> {
            self.calls.push(AutoDomainRuntimeActionKind::WalkToFightKey);
            Ok(basic_outcome())
        }

        fn run_auto_domain_fight(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
            _context: &AutoDomainRuntimeRoundContext,
        ) -> Result<AutoDomainCombatOutcome> {
            self.calls.push(AutoDomainRuntimeActionKind::RunAutoFight);
            Ok(self.fight.clone())
        }

        fn wait_auto_domain_after_fight(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
            _context: &AutoDomainRuntimeRoundContext,
        ) -> Result<AutoDomainBasicOutcome> {
            self.calls.push(AutoDomainRuntimeActionKind::WaitAfterFight);
            Ok(basic_outcome())
        }

        fn move_auto_domain_to_petrified_tree(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
            _context: &AutoDomainRuntimeRoundContext,
        ) -> Result<AutoDomainTreeOutcome> {
            self.calls
                .push(AutoDomainRuntimeActionKind::MoveToPetrifiedTree);
            Ok(AutoDomainTreeOutcome {
                completed: true,
                tree_detected: true,
                prompt_found: true,
                message: None,
            })
        }

        fn select_auto_domain_resin(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
            _context: &AutoDomainRuntimeRoundContext,
            _records: &[AutoDomainResinUseRecord],
        ) -> Result<AutoDomainResinSelection> {
            self.calls.push(AutoDomainRuntimeActionKind::UseResin);
            Ok(self.selection.clone())
        }

        fn claim_auto_domain_reward(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
            _context: &AutoDomainRuntimeRoundContext,
            _selection: &AutoDomainResinSelection,
        ) -> Result<AutoDomainRewardOutcome> {
            self.calls.push(AutoDomainRuntimeActionKind::UseResin);
            self.claim_calls += 1;
            Ok(self.reward.clone())
        }

        fn recognize_auto_domain_reward(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
            _context: &AutoDomainRuntimeRoundContext,
        ) -> Result<AutoDomainRewardRecognitionOutcome> {
            self.calls
                .push(AutoDomainRuntimeActionKind::RecognizeReward);
            Ok(AutoDomainRewardRecognitionOutcome {
                attempted: true,
                recognized: true,
                message: None,
            })
        }

        fn continue_or_exit_auto_domain(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
            _context: &AutoDomainRuntimeRoundContext,
            should_continue: bool,
        ) -> Result<AutoDomainContinuationOutcome> {
            self.calls.push(AutoDomainRuntimeActionKind::ContinueOrExit);
            Ok(AutoDomainContinuationOutcome {
                completed: true,
                continue_next_round: should_continue,
                exited_domain: !should_continue,
                message: None,
            })
        }

        fn wait_auto_domain_main_ui(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
        ) -> Result<AutoDomainBasicOutcome> {
            self.calls.push(AutoDomainRuntimeActionKind::WaitMainUi);
            Ok(basic_outcome())
        }

        fn run_auto_domain_artifact_salvage(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
        ) -> Result<AutoDomainArtifactSalvageOutcome> {
            self.calls
                .push(AutoDomainRuntimeActionKind::ArtifactSalvage);
            Ok(AutoDomainArtifactSalvageOutcome {
                attempted: false,
                completed: true,
                message: None,
            })
        }

        fn notify_auto_domain_end(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
            _status: AutoDomainExecutionStatus,
        ) -> Result<AutoDomainNotificationOutcome> {
            self.calls.push(AutoDomainRuntimeActionKind::NotifyEnd);
            Ok(notification_outcome())
        }

        fn cleanup_auto_domain(
            &mut self,
            _plan: &AutoDomainExecutionPlan,
        ) -> Result<AutoDomainCleanupOutcome> {
            self.calls.push(AutoDomainRuntimeActionKind::Cleanup);
            self.cleanup_calls += 1;
            Ok(self.cleanup.clone())
        }
    }

    #[test]
    fn execute_auto_domain_plan_single_round_success() {
        let plan = test_plan_with_specified_resin(1);
        let mut runtime = MockAutoDomainRuntime::default();

        let report = execute_auto_domain_plan(&plan, &mut runtime).unwrap();

        assert!(plan.executor_ready);
        assert!(report.completed);
        assert_eq!(report.status, AutoDomainExecutionStatus::Completed);
        assert_eq!(report.state.current_round, 1);
        assert_eq!(report.state.fights_attempted, 1);
        assert_eq!(report.state.fights_succeeded, 1);
        assert_eq!(report.state.rewards_claimed, 1);
        assert_eq!(report.state.resin_records[0].remain_count, 0);
        assert!(report.state.cleanup_completed);
        assert_eq!(runtime.cleanup_calls, 1);
        assert!(runtime
            .calls
            .contains(&AutoDomainRuntimeActionKind::EnterDomain));
        assert!(runtime
            .calls
            .contains(&AutoDomainRuntimeActionKind::RunAutoFight));
        assert!(runtime
            .calls
            .contains(&AutoDomainRuntimeActionKind::ContinueOrExit));
    }

    #[test]
    fn execute_auto_domain_plan_skips_when_resin_unavailable() {
        let plan = test_plan(1);
        let mut runtime = MockAutoDomainRuntime {
            selection: AutoDomainResinSelection {
                decision: AutoDomainRewardDecision::Skip,
                resin_name: None,
                available_count: Some(0),
                skip_reason: Some(AutoDomainSkipReason::OriginalResinInsufficient),
                message: Some("resin unavailable".to_string()),
            },
            ..MockAutoDomainRuntime::default()
        };

        let report = execute_auto_domain_plan(&plan, &mut runtime).unwrap();

        assert!(!report.completed);
        assert_eq!(report.status, AutoDomainExecutionStatus::RewardSkipped);
        assert_eq!(report.state.rewards_claimed, 0);
        assert_eq!(report.state.rewards_skipped, 1);
        assert_eq!(
            report.state.last_skip_reason,
            Some(AutoDomainSkipReason::OriginalResinInsufficient)
        );
        assert_eq!(runtime.claim_calls, 0);
        assert_eq!(runtime.cleanup_calls, 1);
    }

    #[test]
    fn execute_auto_domain_plan_stops_on_fight_failure() {
        let plan = test_plan(1);
        let mut runtime = MockAutoDomainRuntime {
            fight: AutoDomainCombatOutcome {
                completed: false,
                challenge_completed: false,
                auto_leaving_detected: false,
                duration_ms: Some(3_000),
                message: Some("fight failed".to_string()),
            },
            ..MockAutoDomainRuntime::default()
        };

        let report = execute_auto_domain_plan(&plan, &mut runtime).unwrap();

        assert!(!report.completed);
        assert_eq!(report.status, AutoDomainExecutionStatus::CombatFailed);
        assert_eq!(report.state.fights_attempted, 1);
        assert_eq!(report.state.fights_succeeded, 0);
        assert_eq!(report.state.rewards_claimed, 0);
        assert!(!runtime
            .calls
            .contains(&AutoDomainRuntimeActionKind::MoveToPetrifiedTree));
        assert_eq!(runtime.cleanup_calls, 1);
    }

    #[test]
    fn execute_auto_domain_plan_cleanup_runs_after_startup_failure() {
        let plan = test_plan(1);
        let mut runtime = MockAutoDomainRuntime {
            startup: AutoDomainStartupOutcome {
                completed: false,
                assets_initialized: false,
                combat_strategy_parsed: false,
                auto_eat_trigger_registered: false,
                message: Some("startup failed".to_string()),
            },
            ..MockAutoDomainRuntime::default()
        };

        let report = execute_auto_domain_plan(&plan, &mut runtime).unwrap();

        assert!(!report.completed);
        assert_eq!(report.status, AutoDomainExecutionStatus::StartupFailed);
        assert!(!report.state.startup_completed);
        assert!(report.state.cleanup_completed);
        assert_eq!(runtime.cleanup_calls, 1);
        assert!(runtime
            .calls
            .contains(&AutoDomainRuntimeActionKind::Cleanup));
        assert!(runtime
            .calls
            .contains(&AutoDomainRuntimeActionKind::WaitMainUi));
        assert!(runtime
            .calls
            .contains(&AutoDomainRuntimeActionKind::NotifyEnd));
    }

    #[test]
    fn execute_auto_domain_plan_retries_entry_before_success() {
        let plan = test_plan(1);
        let mut runtime = MockAutoDomainRuntime {
            entry_failures_remaining: 1,
            ..MockAutoDomainRuntime::default()
        };

        let report = execute_auto_domain_plan(&plan, &mut runtime).unwrap();

        assert!(report.completed);
        assert_eq!(report.status, AutoDomainExecutionStatus::Completed);
        assert_eq!(report.state.retries_used, 1);
        assert_eq!(runtime.entry_calls, 2);
        assert_eq!(runtime.cleanup_calls, 1);
    }

    fn test_plan(rounds: i32) -> AutoDomainExecutionPlan {
        let mut config = AutoDomainExecutionConfig::default();
        config.param.domain_round_num = rounds;
        config.param.domain_name = "太山府".to_string();
        config.auto_domain_config.revive_retry_count = 1;
        plan_auto_domain(config).unwrap()
    }

    fn test_plan_with_specified_resin(rounds: i32) -> AutoDomainExecutionPlan {
        let mut config = AutoDomainExecutionConfig::default();
        config.param.domain_round_num = rounds;
        config.param.domain_name = "太山府".to_string();
        config.param.specify_resin_use = true;
        config.param.condensed_resin_use_count = 1;
        config.auto_domain_config.revive_retry_count = 1;
        plan_auto_domain(config).unwrap()
    }

    fn basic_outcome() -> AutoDomainBasicOutcome {
        AutoDomainBasicOutcome {
            completed: true,
            message: None,
        }
    }

    fn notification_outcome() -> AutoDomainNotificationOutcome {
        AutoDomainNotificationOutcome {
            sent: true,
            message: None,
        }
    }
}
