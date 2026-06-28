use bgi_core::{AutoGeniusInvokationConfig, RectConfig};
use bgi_vision::{Rect, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::{Result, TaskError};

pub const AUTO_GENIUS_INVOKATION_TASK_KEY: &str = "AutoGeniusInvokation";
pub const AUTO_GENIUS_INVOKATION_DISPLAY_NAME: &str = "自动七圣召唤";
pub const AUTO_GENIUS_INVOKATION_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_GENIUS_INVOKATION_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_GENIUS_INVOKATION_USER_STRATEGY_DIR: &str = "User/AutoGeniusInvokation";
pub const AUTO_GENIUS_INVOKATION_DEFAULT_CARD_ASSET: &str =
    "AutoGeniusInvokation:tcg_character_card.json";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusInvokationExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub asset_scale: f64,
    pub config_rule: AutoGeniusInvokationConfigRule,
    pub strategy_source: AutoGeniusInvokationStrategySource,
    pub strategy: AutoGeniusStrategyPlan,
    pub startup_rule: AutoGeniusInvokationStartupRule,
    pub locators: AutoGeniusInvokationLocators,
    pub dice_rule: AutoGeniusInvokationDiceRule,
    pub action_rule: AutoGeniusInvokationActionRule,
    pub ocr_rule: AutoGeniusInvokationOcrRule,
    pub wait_rule: AutoGeniusInvokationWaitRule,
    pub exception_rule: AutoGeniusInvokationExceptionRule,
    pub steps: Vec<AutoGeniusInvokationStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationExecutionConfig {
    pub capture_size: Size,
    pub asset_scale: f64,
    pub strategy_name: Option<String>,
    pub strategy: Option<String>,
    pub auto_genius_invokation_config: AutoGeniusInvokationConfig,
}

impl Default for AutoGeniusInvokationExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_GENIUS_INVOKATION_DEFAULT_CAPTURE_WIDTH,
                AUTO_GENIUS_INVOKATION_DEFAULT_CAPTURE_HEIGHT,
            ),
            asset_scale: 1.0,
            strategy_name: None,
            strategy: None,
            auto_genius_invokation_config: AutoGeniusInvokationConfig::default(),
        }
    }
}

impl AutoGeniusInvokationExecutionConfig {
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
        if let Some(strategy) = string_member(value, ["strategy", "Strategy", "script", "Script"]) {
            config.strategy = Some(strategy);
        }

        let auto_genius_value = value
            .get("autoGeniusInvokationConfig")
            .or_else(|| value.get("AutoGeniusInvokationConfig"))
            .or_else(|| value.get("auto_genius_invokation_config"))
            .unwrap_or(value);
        config.auto_genius_invokation_config =
            serde_json::from_value(auto_genius_value.clone()).unwrap_or_default();

        config.strategy_name = string_member(
            value,
            [
                "strategyName",
                "StrategyName",
                "strategy_name",
                "autoGeniusInvokationStrategyName",
            ],
        )
        .or_else(|| {
            string_member(
                auto_genius_value,
                ["strategyName", "StrategyName", "strategy_name"],
            )
        });
        config
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusInvokationConfigRule {
    pub strategy_name: String,
    pub sleep_delay_ms: u64,
    pub sleep_delay_min_ms: u64,
    pub sleep_delay_max_ms: u64,
    pub default_character_card_rects: Vec<Rect>,
    pub active_character_card_space: i64,
    pub my_dice_count_rect: Rect,
    pub character_card_extend_hp_rect: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusInvokationStrategySource {
    pub strategy_name: String,
    pub user_strategy_directory: String,
    pub strategy_path: Option<String>,
    pub inline_strategy: bool,
    pub default_card_config_asset: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusStrategyPlan {
    pub characters: Vec<AutoGeniusCharacterPlan>,
    pub action_commands: Vec<AutoGeniusActionCommandPlan>,
    pub skipped_line_count: usize,
    pub stage_order: Vec<String>,
    pub preserves_legacy_stage_headers: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoGeniusCharacterPlan {
    pub index: u8,
    pub name: String,
    pub element: Option<AutoGeniusElementalType>,
    pub skills: Vec<AutoGeniusSkillPlan>,
    pub uses_default_card_config: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoGeniusSkillPlan {
    pub index: u8,
    pub element: AutoGeniusElementalType,
    pub specific_element_cost: u8,
    pub any_element_cost: u8,
    pub all_cost: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoGeniusActionCommandPlan {
    pub character_index: u8,
    pub character_name: String,
    pub action: AutoGeniusActionKind,
    pub target_index: u8,
    pub dice_delta: i8,
    pub all_cost: Option<i16>,
    pub dice_element: Option<AutoGeniusElementalType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoGeniusActionKind {
    UseSkill,
    SwitchLater,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoGeniusElementalType {
    Omni,
    Cryo,
    Hydro,
    Pyro,
    Electro,
    Dendro,
    Anemo,
    Geo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusInvokationStartupRule {
    pub skips_task_runner_main_ui_wait: bool,
    pub requires_exact_1920x1080: bool,
    pub sends_tcg_start_notification: bool,
    pub sends_tcg_end_notification: bool,
    pub destroys_asset_singleton_before_start: bool,
    pub initializes_control_with_cancellation_token: bool,
    pub prepares_initial_hand: bool,
    pub detects_character_rects_with_fallback: bool,
    pub chooses_first_action_character: bool,
    pub clears_draw_content_after_round: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusInvokationLocators {
    pub confirm_button: AutoGeniusTemplateLocator,
    pub round_end_button: AutoGeniusTemplateLocator,
    pub elemental_tuning_confirm_button: AutoGeniusTemplateLocator,
    pub exit_duel_button: AutoGeniusTemplateLocator,
    pub in_opponent_action: AutoGeniusTemplateLocator,
    pub end_phase: AutoGeniusTemplateLocator,
    pub elemental_dice_lack_warning: AutoGeniusTemplateLocator,
    pub character_taken_out: AutoGeniusTemplateLocator,
    pub in_character_pick: AutoGeniusTemplateLocator,
    pub character_hp_upper: AutoGeniusTemplateLocator,
    pub grayscale_assets: Vec<String>,
    pub roll_phase_dice_assets: Vec<AutoGeniusDiceAsset>,
    pub action_phase_dice_assets: Vec<AutoGeniusDiceAsset>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Option<Rect>,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusDiceAsset {
    pub element: AutoGeniusElementalType,
    pub asset: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusInvokationDiceRule {
    pub initial_dice_count: u8,
    pub roll_phase_threshold: f64,
    pub roll_phase_expected_count: u8,
    pub roll_phase_expected_upper_count: u8,
    pub roll_phase_expected_lower_count: u8,
    pub roll_phase_initial_wait_ms: u64,
    pub roll_phase_retry_interval_ms: u64,
    pub roll_phase_retry_attempts: u64,
    pub post_roll_confirm_sleep_ms: u64,
    pub opponent_reroll_wait_ms: u64,
    pub action_phase_threshold: f64,
    pub action_phase_roi_right_width_divisor: u8,
    pub action_phase_count_retry_attempts: u64,
    pub action_phase_count_retry_interval_ms: u64,
    pub action_phase_expected_8_actual_9_omni_retry_limit: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusInvokationActionRule {
    pub first_round_card_count: u8,
    pub next_round_card_increment: u8,
    pub switch_character_dice_cost: u8,
    pub switch_button_click_offset_from_right_1080p: f64,
    pub action_button_y_offset_from_bottom_1080p: f64,
    pub switch_button_double_click: bool,
    pub switch_animation_sleep_ms: u64,
    pub skill_click_offset_multiplier_1080p: f64,
    pub skill_popup_sleep_ms: u64,
    pub skill_confirm_sleep_ms: u64,
    pub skill_center_reset_before_click: bool,
    pub elemental_tuning_confirm_threshold: f64,
    pub elemental_tuning_hand_layouts: Vec<AutoGeniusHandLayout>,
    pub keqing_skill_2_alternates_card_count: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusHandLayout {
    pub card_count: u8,
    pub start_x_1080p: f64,
    pub spacing_1080p: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusInvokationOcrRule {
    pub dice_count_rect: Rect,
    pub dice_ocr_without_detector: bool,
    pub invalid_dice_count_sentinel: i32,
    pub replaces_circled_digit_text: bool,
    pub active_character_space_offset: i64,
    pub character_hp_empty_uses_active_offset: bool,
    pub active_character_fallback_by_exclusion: bool,
    pub active_character_fallback_by_template_shape: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusInvokationWaitRule {
    pub wait_my_turn_max_attempts: u64,
    pub wait_my_turn_interval_ms: u64,
    pub wait_my_turn_required_consecutive_hits: u64,
    pub wait_opponent_action_max_attempts: u64,
    pub wait_opponent_action_interval_ms: u64,
    pub default_after_action_wait_ms: u64,
    pub burst_after_action_wait_ms: u64,
    pub mona_switch_after_action_wait_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusInvokationExceptionRule {
    pub normal_end_is_logged_without_rethrow_in_duel: bool,
    pub task_cancelled_rethrows_from_duel: bool,
    pub outer_task_boundary_catches_all_and_logs: bool,
    pub check_task_verifies_game_foreground: bool,
    pub check_task_pause_retry_attempts: u64,
    pub check_task_pause_retry_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusInvokationStep {
    pub phase: AutoGeniusInvokationPhase,
    pub action: AutoGeniusInvokationStepAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoGeniusInvokationPhase {
    Startup,
    Prepare,
    RoundLoop,
    RollDice,
    MyTurn,
    Action,
    ElementalTuning,
    RoundEnd,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoGeniusInvokationStepAction {
    ValidateResolution,
    ParseStrategy,
    NotifyStart,
    PrepareInitialHand,
    ResolveCharacterRects,
    ChooseFirstCharacter,
    PredictDiceTypes,
    ReRollDice,
    WaitForMyTurn,
    DetectActiveCharacter,
    CalibrateDiceCountByOcr,
    SwitchCharacterIfNeeded,
    UseSkillOrTuneCards,
    RemoveExecutedCommand,
    ClickRoundEnd,
    WaitOpponentActionAndEndPhase,
    NotifyEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoGeniusInvokationExecutionStatus {
    Completed,
    StartupFailed,
    WindowCheckFailed,
    ResolutionUnsupported,
    InitialHandFailed,
    CharacterResolutionFailed,
    FirstCharacterSelectionFailed,
    DiceRollFailed,
    WaitMyTurnFailed,
    ActiveCharacterMissing,
    DiceCountUnavailable,
    DiceInsufficient,
    CardTuningFailed,
    SwitchCharacterFailed,
    SkillActionFailed,
    RoundEndFailed,
    OpponentWaitFailed,
    RuntimeExitFailed,
    Cancelled,
    CleanupFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoGeniusInvokationRuntimeActionStatus {
    Succeeded,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoGeniusInvokationRuntimeActionKind {
    Startup,
    ValidateWindow,
    NotifyStart,
    PrepareInitialHand,
    ResolveCharacterRects,
    ChooseFirstCharacter,
    PredictDiceTypes,
    ReRollDice,
    WaitForMyTurn,
    DetectActiveCharacter,
    CalibrateDiceCountByOcr,
    SwitchCharacter,
    TuneCards,
    UseSkill,
    WaitAfterAction,
    RemoveExecutedCommand,
    ClickRoundEnd,
    WaitOpponentActionAndEndPhase,
    ExitDuel,
    NotifyEnd,
    Cleanup,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoGeniusInvokationSkipReason {
    Cancelled,
    NoActionCommands,
    CharacterMissing,
    DiceInsufficient,
    RuntimeRequestedStop,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusElementalDiceCount {
    pub element: AutoGeniusElementalType,
    pub count: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationRuntimeContext {
    pub round_index: u32,
    pub command_index: Option<usize>,
    pub command: Option<AutoGeniusActionCommandPlan>,
    pub active_character_index: Option<u8>,
    pub dice_count: u8,
    pub hand_card_count: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationStartupOutcome {
    pub completed: bool,
    pub assets_initialized: bool,
    pub control_initialized: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationWindowOutcome {
    pub foreground: bool,
    pub resolution_supported: bool,
    pub capture_size: Size,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationNotificationOutcome {
    pub sent: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationInitialHandOutcome {
    pub completed: bool,
    pub card_count: Option<u8>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationCharacterRectsOutcome {
    pub completed: bool,
    pub detected_character_count: u8,
    pub character_rects: Vec<Rect>,
    pub used_fallback: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationCharacterSelectionOutcome {
    pub selected: bool,
    pub character_index: Option<u8>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationDicePredictionOutcome {
    pub completed: bool,
    pub preferred_elements: Vec<AutoGeniusElementalType>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationDiceRollOutcome {
    pub completed: bool,
    pub dice: Vec<AutoGeniusElementalDiceCount>,
    pub dice_count: u8,
    pub reroll_attempts: u64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationWaitOutcome {
    pub completed: bool,
    pub attempts: u64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationActiveCharacterOutcome {
    pub detected: bool,
    pub character_index: Option<u8>,
    pub defeated_character_indices: Vec<u8>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationDiceCountOutcome {
    pub dice_count: Option<u8>,
    pub dice: Vec<AutoGeniusElementalDiceCount>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationSwitchOutcome {
    pub completed: bool,
    pub from_character_index: Option<u8>,
    pub to_character_index: u8,
    pub dice_spent: u8,
    pub remaining_dice_count: Option<u8>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationCardTuningOutcome {
    pub attempted: bool,
    pub completed: bool,
    pub cards_tuned: u8,
    pub dice_gained: u8,
    pub remaining_dice_count: Option<u8>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationSkillOutcome {
    pub completed: bool,
    pub dice_spent: u8,
    pub remaining_dice_count: Option<u8>,
    pub dice_lack: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationRoundEndOutcome {
    pub completed: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationCommandOutcome {
    pub removed: bool,
    pub remaining_commands: usize,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationExitOutcome {
    pub attempted: bool,
    pub completed: bool,
    pub normal_end: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationCleanupOutcome {
    pub completed: bool,
    pub inputs_released: bool,
    pub overlays_cleared: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoGeniusInvokationRuntimeActionOutcome {
    Startup(AutoGeniusInvokationStartupOutcome),
    Window(AutoGeniusInvokationWindowOutcome),
    Notification(AutoGeniusInvokationNotificationOutcome),
    InitialHand(AutoGeniusInvokationInitialHandOutcome),
    CharacterRects(AutoGeniusInvokationCharacterRectsOutcome),
    CharacterSelection(AutoGeniusInvokationCharacterSelectionOutcome),
    DicePrediction(AutoGeniusInvokationDicePredictionOutcome),
    DiceRoll(AutoGeniusInvokationDiceRollOutcome),
    Wait(AutoGeniusInvokationWaitOutcome),
    ActiveCharacter(AutoGeniusInvokationActiveCharacterOutcome),
    DiceCount(AutoGeniusInvokationDiceCountOutcome),
    Switch(AutoGeniusInvokationSwitchOutcome),
    CardTuning(AutoGeniusInvokationCardTuningOutcome),
    Skill(AutoGeniusInvokationSkillOutcome),
    RoundEnd(AutoGeniusInvokationRoundEndOutcome),
    Command(AutoGeniusInvokationCommandOutcome),
    Exit(AutoGeniusInvokationExitOutcome),
    Cleanup(AutoGeniusInvokationCleanupOutcome),
    Skipped(AutoGeniusInvokationSkipReason),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationRuntimeActionReport {
    pub phase: AutoGeniusInvokationPhase,
    pub action_kind: AutoGeniusInvokationRuntimeActionKind,
    pub status: AutoGeniusInvokationRuntimeActionStatus,
    pub round_index: Option<u32>,
    pub command_index: Option<usize>,
    pub detail: String,
    pub outcome: AutoGeniusInvokationRuntimeActionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationSkippedStep {
    pub action_kind: AutoGeniusInvokationRuntimeActionKind,
    pub round_index: Option<u32>,
    pub command_index: Option<usize>,
    pub reason: AutoGeniusInvokationSkipReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationExecutorState {
    pub startup_completed: bool,
    pub window_foreground: bool,
    pub resolution_supported: bool,
    pub start_notification_sent: bool,
    pub current_round: u32,
    pub command_cursor: usize,
    pub commands_total: usize,
    pub commands_executed: usize,
    pub hand_card_count: u8,
    pub active_character_index: Option<u8>,
    pub character_rect_count: u8,
    pub used_character_rect_fallback: bool,
    pub dice_count: u8,
    pub reroll_attempts: u64,
    pub my_turn_waits: u64,
    pub switches_performed: u32,
    pub skills_used: u32,
    pub cards_tuned: u32,
    pub round_end_clicked: bool,
    pub opponent_wait_completed: bool,
    pub cancelled: bool,
    pub exit_completed: bool,
    pub cleanup_completed: bool,
    pub end_notification_sent: bool,
    pub last_skip_reason: Option<AutoGeniusInvokationSkipReason>,
}

impl AutoGeniusInvokationExecutorState {
    fn new(plan: &AutoGeniusInvokationExecutionPlan) -> Self {
        Self {
            startup_completed: false,
            window_foreground: false,
            resolution_supported: false,
            start_notification_sent: false,
            current_round: 0,
            command_cursor: 0,
            commands_total: plan.strategy.action_commands.len(),
            commands_executed: 0,
            hand_card_count: 0,
            active_character_index: None,
            character_rect_count: 0,
            used_character_rect_fallback: false,
            dice_count: 0,
            reroll_attempts: 0,
            my_turn_waits: 0,
            switches_performed: 0,
            skills_used: 0,
            cards_tuned: 0,
            round_end_clicked: false,
            opponent_wait_completed: false,
            cancelled: false,
            exit_completed: false,
            cleanup_completed: false,
            end_notification_sent: false,
            last_skip_reason: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub status: AutoGeniusInvokationExecutionStatus,
    pub state: AutoGeniusInvokationExecutorState,
    pub executed_actions: Vec<AutoGeniusInvokationRuntimeActionReport>,
    pub skipped_steps: Vec<AutoGeniusInvokationSkippedStep>,
}

pub trait AutoGeniusInvokationRuntime {
    fn start_auto_genius_invokation(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
    ) -> Result<AutoGeniusInvokationStartupOutcome>;

    fn check_auto_genius_invokation_window(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
    ) -> Result<AutoGeniusInvokationWindowOutcome>;

    fn notify_auto_genius_invokation_start(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
    ) -> Result<AutoGeniusInvokationNotificationOutcome>;

    fn prepare_auto_genius_initial_hand(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
    ) -> Result<AutoGeniusInvokationInitialHandOutcome>;

    fn resolve_auto_genius_character_rects(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
    ) -> Result<AutoGeniusInvokationCharacterRectsOutcome>;

    fn choose_auto_genius_initial_character(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        context: &AutoGeniusInvokationRuntimeContext,
        character: &AutoGeniusCharacterPlan,
    ) -> Result<AutoGeniusInvokationCharacterSelectionOutcome>;

    fn predict_auto_genius_roll_dice(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        context: &AutoGeniusInvokationRuntimeContext,
    ) -> Result<AutoGeniusInvokationDicePredictionOutcome>;

    fn reroll_auto_genius_dice(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        context: &AutoGeniusInvokationRuntimeContext,
        prediction: &AutoGeniusInvokationDicePredictionOutcome,
    ) -> Result<AutoGeniusInvokationDiceRollOutcome>;

    fn wait_auto_genius_my_turn(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        context: &AutoGeniusInvokationRuntimeContext,
    ) -> Result<AutoGeniusInvokationWaitOutcome>;

    fn detect_auto_genius_active_character(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        context: &AutoGeniusInvokationRuntimeContext,
    ) -> Result<AutoGeniusInvokationActiveCharacterOutcome>;

    fn calibrate_auto_genius_dice_count(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        context: &AutoGeniusInvokationRuntimeContext,
    ) -> Result<AutoGeniusInvokationDiceCountOutcome>;

    fn switch_auto_genius_character(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        context: &AutoGeniusInvokationRuntimeContext,
        command: &AutoGeniusActionCommandPlan,
    ) -> Result<AutoGeniusInvokationSwitchOutcome>;

    fn tune_auto_genius_cards_for_dice(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        context: &AutoGeniusInvokationRuntimeContext,
        command: &AutoGeniusActionCommandPlan,
        missing_dice: u8,
    ) -> Result<AutoGeniusInvokationCardTuningOutcome>;

    fn use_auto_genius_skill(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        context: &AutoGeniusInvokationRuntimeContext,
        command: &AutoGeniusActionCommandPlan,
    ) -> Result<AutoGeniusInvokationSkillOutcome>;

    fn wait_auto_genius_after_action(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        context: &AutoGeniusInvokationRuntimeContext,
        command: &AutoGeniusActionCommandPlan,
    ) -> Result<AutoGeniusInvokationWaitOutcome>;

    fn click_auto_genius_round_end(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        context: &AutoGeniusInvokationRuntimeContext,
    ) -> Result<AutoGeniusInvokationRoundEndOutcome>;

    fn wait_auto_genius_opponent_action_and_end_phase(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        context: &AutoGeniusInvokationRuntimeContext,
    ) -> Result<AutoGeniusInvokationWaitOutcome>;

    fn exit_auto_genius_invokation(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        status: AutoGeniusInvokationExecutionStatus,
    ) -> Result<AutoGeniusInvokationExitOutcome>;

    fn notify_auto_genius_invokation_end(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        status: AutoGeniusInvokationExecutionStatus,
    ) -> Result<AutoGeniusInvokationNotificationOutcome>;

    fn cleanup_auto_genius_invokation(
        &mut self,
        plan: &AutoGeniusInvokationExecutionPlan,
        state: &AutoGeniusInvokationExecutorState,
    ) -> Result<AutoGeniusInvokationCleanupOutcome>;

    fn is_auto_genius_invokation_cancelled(&mut self) -> Result<bool> {
        Ok(false)
    }
}

pub fn execute_auto_genius_invokation_plan<R>(
    plan: &AutoGeniusInvokationExecutionPlan,
    runtime: &mut R,
) -> Result<AutoGeniusInvokationExecutionReport>
where
    R: AutoGeniusInvokationRuntime,
{
    let mut state = AutoGeniusInvokationExecutorState::new(plan);
    let mut executed_actions = Vec::new();
    let mut skipped_steps = Vec::new();

    let execution_result = execute_auto_genius_invokation_plan_inner(
        plan,
        runtime,
        &mut state,
        &mut executed_actions,
        &mut skipped_steps,
    );

    match execution_result {
        Ok(status) => {
            let cleanup_status = execute_auto_genius_invokation_cleanup(
                plan,
                runtime,
                status,
                &mut state,
                &mut executed_actions,
            )?;
            let status = if cleanup_status == AutoGeniusInvokationExecutionStatus::CleanupFailed {
                AutoGeniusInvokationExecutionStatus::CleanupFailed
            } else if cleanup_status == AutoGeniusInvokationExecutionStatus::RuntimeExitFailed {
                AutoGeniusInvokationExecutionStatus::RuntimeExitFailed
            } else {
                status
            };
            Ok(auto_genius_invokation_report(
                plan,
                status,
                state,
                executed_actions,
                skipped_steps,
            ))
        }
        Err(error) => {
            let cleanup_error = execute_auto_genius_invokation_cleanup(
                plan,
                runtime,
                AutoGeniusInvokationExecutionStatus::CleanupFailed,
                &mut state,
                &mut executed_actions,
            )
            .err();
            Err(cleanup_error.unwrap_or(error))
        }
    }
}

fn execute_auto_genius_invokation_plan_inner<R>(
    plan: &AutoGeniusInvokationExecutionPlan,
    runtime: &mut R,
    state: &mut AutoGeniusInvokationExecutorState,
    executed_actions: &mut Vec<AutoGeniusInvokationRuntimeActionReport>,
    skipped_steps: &mut Vec<AutoGeniusInvokationSkippedStep>,
) -> Result<AutoGeniusInvokationExecutionStatus>
where
    R: AutoGeniusInvokationRuntime,
{
    let startup = runtime.start_auto_genius_invokation(plan)?;
    state.startup_completed = startup.completed;
    let startup_completed = startup.completed;
    executed_actions.push(auto_genius_invokation_action_report(
        AutoGeniusInvokationPhase::Startup,
        AutoGeniusInvokationRuntimeActionKind::Startup,
        if startup_completed {
            AutoGeniusInvokationRuntimeActionStatus::Succeeded
        } else {
            AutoGeniusInvokationRuntimeActionStatus::Failed
        },
        None,
        None,
        startup
            .message
            .clone()
            .unwrap_or_else(|| "auto genius invokation startup boundary completed".to_string()),
        AutoGeniusInvokationRuntimeActionOutcome::Startup(startup),
    ));
    if !startup_completed {
        return Ok(AutoGeniusInvokationExecutionStatus::StartupFailed);
    }

    if check_auto_genius_invokation_cancelled(
        runtime,
        state,
        executed_actions,
        skipped_steps,
        AutoGeniusInvokationPhase::Startup,
        AutoGeniusInvokationRuntimeActionKind::Skip,
        None,
        None,
    )? {
        return Ok(AutoGeniusInvokationExecutionStatus::Cancelled);
    }

    let window = runtime.check_auto_genius_invokation_window(plan)?;
    state.window_foreground = window.foreground;
    state.resolution_supported = window.resolution_supported;
    let window_ok = window.foreground && window.resolution_supported;
    let resolution_supported = window.resolution_supported;
    executed_actions.push(auto_genius_invokation_action_report(
        AutoGeniusInvokationPhase::Startup,
        AutoGeniusInvokationRuntimeActionKind::ValidateWindow,
        if window_ok {
            AutoGeniusInvokationRuntimeActionStatus::Succeeded
        } else {
            AutoGeniusInvokationRuntimeActionStatus::Failed
        },
        None,
        None,
        window
            .message
            .clone()
            .unwrap_or_else(|| "window and resolution boundary completed".to_string()),
        AutoGeniusInvokationRuntimeActionOutcome::Window(window),
    ));
    if !window_ok {
        return Ok(if resolution_supported {
            AutoGeniusInvokationExecutionStatus::WindowCheckFailed
        } else {
            AutoGeniusInvokationExecutionStatus::ResolutionUnsupported
        });
    }

    let start_notification = runtime.notify_auto_genius_invokation_start(plan)?;
    state.start_notification_sent = start_notification.sent;
    executed_actions.push(auto_genius_invokation_action_report(
        AutoGeniusInvokationPhase::Startup,
        AutoGeniusInvokationRuntimeActionKind::NotifyStart,
        if start_notification.sent {
            AutoGeniusInvokationRuntimeActionStatus::Succeeded
        } else {
            AutoGeniusInvokationRuntimeActionStatus::Skipped
        },
        None,
        None,
        start_notification
            .message
            .clone()
            .unwrap_or_else(|| "start notification boundary completed".to_string()),
        AutoGeniusInvokationRuntimeActionOutcome::Notification(start_notification),
    ));

    let initial_hand = runtime.prepare_auto_genius_initial_hand(plan)?;
    state.hand_card_count = initial_hand
        .card_count
        .unwrap_or(plan.action_rule.first_round_card_count);
    let initial_hand_completed = initial_hand.completed;
    executed_actions.push(auto_genius_invokation_action_report(
        AutoGeniusInvokationPhase::Prepare,
        AutoGeniusInvokationRuntimeActionKind::PrepareInitialHand,
        if initial_hand_completed {
            AutoGeniusInvokationRuntimeActionStatus::Succeeded
        } else {
            AutoGeniusInvokationRuntimeActionStatus::Failed
        },
        None,
        None,
        initial_hand
            .message
            .clone()
            .unwrap_or_else(|| "initial hand preparation boundary completed".to_string()),
        AutoGeniusInvokationRuntimeActionOutcome::InitialHand(initial_hand),
    ));
    if !initial_hand_completed {
        return Ok(AutoGeniusInvokationExecutionStatus::InitialHandFailed);
    }

    let character_rects = runtime.resolve_auto_genius_character_rects(plan)?;
    state.character_rect_count = character_rects.detected_character_count;
    state.used_character_rect_fallback = character_rects.used_fallback;
    let character_rects_completed = character_rects.completed
        && character_rects.detected_character_count >= plan.strategy.characters.len() as u8;
    executed_actions.push(auto_genius_invokation_action_report(
        AutoGeniusInvokationPhase::Prepare,
        AutoGeniusInvokationRuntimeActionKind::ResolveCharacterRects,
        if character_rects_completed {
            AutoGeniusInvokationRuntimeActionStatus::Succeeded
        } else {
            AutoGeniusInvokationRuntimeActionStatus::Failed
        },
        None,
        None,
        character_rects
            .message
            .clone()
            .unwrap_or_else(|| "character rect resolution boundary completed".to_string()),
        AutoGeniusInvokationRuntimeActionOutcome::CharacterRects(character_rects),
    ));
    if !character_rects_completed {
        return Ok(AutoGeniusInvokationExecutionStatus::CharacterResolutionFailed);
    }

    let Some(first_command) = plan.strategy.action_commands.first() else {
        return Ok(record_auto_genius_invokation_skip(
            state,
            executed_actions,
            skipped_steps,
            AutoGeniusInvokationPhase::RoundLoop,
            AutoGeniusInvokationRuntimeActionKind::Skip,
            None,
            None,
            AutoGeniusInvokationSkipReason::NoActionCommands,
            AutoGeniusInvokationExecutionStatus::Completed,
        ));
    };
    let Some(first_character) = plan
        .strategy
        .characters
        .iter()
        .find(|character| character.index == first_command.character_index)
    else {
        return Ok(record_auto_genius_invokation_skip(
            state,
            executed_actions,
            skipped_steps,
            AutoGeniusInvokationPhase::Prepare,
            AutoGeniusInvokationRuntimeActionKind::ChooseFirstCharacter,
            None,
            Some(0),
            AutoGeniusInvokationSkipReason::CharacterMissing,
            AutoGeniusInvokationExecutionStatus::CharacterResolutionFailed,
        ));
    };

    state.current_round = 1;
    let first_context = auto_genius_invokation_context(state, Some(0), Some(first_command.clone()));
    let selection =
        runtime.choose_auto_genius_initial_character(plan, &first_context, first_character)?;
    state.active_character_index = selection.character_index;
    let selected = selection.selected && selection.character_index == Some(first_character.index);
    executed_actions.push(auto_genius_invokation_action_report(
        AutoGeniusInvokationPhase::Prepare,
        AutoGeniusInvokationRuntimeActionKind::ChooseFirstCharacter,
        if selected {
            AutoGeniusInvokationRuntimeActionStatus::Succeeded
        } else {
            AutoGeniusInvokationRuntimeActionStatus::Failed
        },
        Some(state.current_round),
        Some(0),
        selection
            .message
            .clone()
            .unwrap_or_else(|| "first action character selection boundary completed".to_string()),
        AutoGeniusInvokationRuntimeActionOutcome::CharacterSelection(selection),
    ));
    if !selected {
        return Ok(AutoGeniusInvokationExecutionStatus::FirstCharacterSelectionFailed);
    }

    let dice_context = auto_genius_invokation_context(state, Some(0), Some(first_command.clone()));
    let prediction = runtime.predict_auto_genius_roll_dice(plan, &dice_context)?;
    let prediction_completed = prediction.completed;
    executed_actions.push(auto_genius_invokation_action_report(
        AutoGeniusInvokationPhase::RollDice,
        AutoGeniusInvokationRuntimeActionKind::PredictDiceTypes,
        if prediction_completed {
            AutoGeniusInvokationRuntimeActionStatus::Succeeded
        } else {
            AutoGeniusInvokationRuntimeActionStatus::Failed
        },
        Some(state.current_round),
        None,
        prediction
            .message
            .clone()
            .unwrap_or_else(|| "dice prediction boundary completed".to_string()),
        AutoGeniusInvokationRuntimeActionOutcome::DicePrediction(prediction.clone()),
    ));
    if !prediction_completed {
        return Ok(AutoGeniusInvokationExecutionStatus::DiceRollFailed);
    }

    let dice_roll = runtime.reroll_auto_genius_dice(plan, &dice_context, &prediction)?;
    state.dice_count = dice_roll.dice_count;
    state.reroll_attempts += dice_roll.reroll_attempts;
    let dice_roll_completed = dice_roll.completed;
    executed_actions.push(auto_genius_invokation_action_report(
        AutoGeniusInvokationPhase::RollDice,
        AutoGeniusInvokationRuntimeActionKind::ReRollDice,
        if dice_roll_completed {
            AutoGeniusInvokationRuntimeActionStatus::Succeeded
        } else {
            AutoGeniusInvokationRuntimeActionStatus::Failed
        },
        Some(state.current_round),
        None,
        dice_roll
            .message
            .clone()
            .unwrap_or_else(|| "dice reroll boundary completed".to_string()),
        AutoGeniusInvokationRuntimeActionOutcome::DiceRoll(dice_roll),
    ));
    if !dice_roll_completed {
        return Ok(AutoGeniusInvokationExecutionStatus::DiceRollFailed);
    }

    for (command_index, command) in plan.strategy.action_commands.iter().enumerate() {
        state.command_cursor = command_index;
        if check_auto_genius_invokation_cancelled(
            runtime,
            state,
            executed_actions,
            skipped_steps,
            AutoGeniusInvokationPhase::RoundLoop,
            AutoGeniusInvokationRuntimeActionKind::Skip,
            Some(state.current_round),
            Some(command_index),
        )? {
            return Ok(AutoGeniusInvokationExecutionStatus::Cancelled);
        }

        let mut context =
            auto_genius_invokation_context(state, Some(command_index), Some(command.clone()));
        let wait_turn = runtime.wait_auto_genius_my_turn(plan, &context)?;
        state.my_turn_waits += 1;
        let wait_turn_completed = wait_turn.completed;
        executed_actions.push(auto_genius_invokation_action_report(
            AutoGeniusInvokationPhase::MyTurn,
            AutoGeniusInvokationRuntimeActionKind::WaitForMyTurn,
            if wait_turn_completed {
                AutoGeniusInvokationRuntimeActionStatus::Succeeded
            } else {
                AutoGeniusInvokationRuntimeActionStatus::Failed
            },
            Some(state.current_round),
            Some(command_index),
            wait_turn
                .message
                .clone()
                .unwrap_or_else(|| "wait for my turn boundary completed".to_string()),
            AutoGeniusInvokationRuntimeActionOutcome::Wait(wait_turn),
        ));
        if !wait_turn_completed {
            return Ok(AutoGeniusInvokationExecutionStatus::WaitMyTurnFailed);
        }

        let active = runtime.detect_auto_genius_active_character(plan, &context)?;
        if active.detected {
            state.active_character_index = active.character_index;
        }
        let active_detected = active.detected && active.character_index.is_some();
        executed_actions.push(auto_genius_invokation_action_report(
            AutoGeniusInvokationPhase::Action,
            AutoGeniusInvokationRuntimeActionKind::DetectActiveCharacter,
            if active_detected {
                AutoGeniusInvokationRuntimeActionStatus::Succeeded
            } else {
                AutoGeniusInvokationRuntimeActionStatus::Failed
            },
            Some(state.current_round),
            Some(command_index),
            active
                .message
                .clone()
                .unwrap_or_else(|| "active character detection boundary completed".to_string()),
            AutoGeniusInvokationRuntimeActionOutcome::ActiveCharacter(active),
        ));
        if !active_detected {
            return Ok(AutoGeniusInvokationExecutionStatus::ActiveCharacterMissing);
        }

        context = auto_genius_invokation_context(state, Some(command_index), Some(command.clone()));
        let dice_count = runtime.calibrate_auto_genius_dice_count(plan, &context)?;
        let Some(calibrated_dice_count) = dice_count.dice_count else {
            executed_actions.push(auto_genius_invokation_action_report(
                AutoGeniusInvokationPhase::Action,
                AutoGeniusInvokationRuntimeActionKind::CalibrateDiceCountByOcr,
                AutoGeniusInvokationRuntimeActionStatus::Failed,
                Some(state.current_round),
                Some(command_index),
                dice_count
                    .message
                    .clone()
                    .unwrap_or_else(|| "dice count OCR did not return a usable count".to_string()),
                AutoGeniusInvokationRuntimeActionOutcome::DiceCount(dice_count),
            ));
            return Ok(AutoGeniusInvokationExecutionStatus::DiceCountUnavailable);
        };
        state.dice_count = calibrated_dice_count;
        executed_actions.push(auto_genius_invokation_action_report(
            AutoGeniusInvokationPhase::Action,
            AutoGeniusInvokationRuntimeActionKind::CalibrateDiceCountByOcr,
            AutoGeniusInvokationRuntimeActionStatus::Succeeded,
            Some(state.current_round),
            Some(command_index),
            dice_count
                .message
                .clone()
                .unwrap_or_else(|| "dice count OCR boundary completed".to_string()),
            AutoGeniusInvokationRuntimeActionOutcome::DiceCount(dice_count),
        ));

        if state.active_character_index != Some(command.character_index) {
            context =
                auto_genius_invokation_context(state, Some(command_index), Some(command.clone()));
            let switch = runtime.switch_auto_genius_character(plan, &context, command)?;
            let switch_completed = switch.completed;
            if switch_completed {
                state.switches_performed += 1;
                state.active_character_index = Some(switch.to_character_index);
                apply_auto_genius_dice_change(
                    &mut state.dice_count,
                    switch.dice_spent,
                    switch.remaining_dice_count,
                );
            }
            executed_actions.push(auto_genius_invokation_action_report(
                AutoGeniusInvokationPhase::Action,
                AutoGeniusInvokationRuntimeActionKind::SwitchCharacter,
                if switch_completed {
                    AutoGeniusInvokationRuntimeActionStatus::Succeeded
                } else {
                    AutoGeniusInvokationRuntimeActionStatus::Failed
                },
                Some(state.current_round),
                Some(command_index),
                switch
                    .message
                    .clone()
                    .unwrap_or_else(|| "character switch boundary completed".to_string()),
                AutoGeniusInvokationRuntimeActionOutcome::Switch(switch),
            ));
            if !switch_completed {
                return Ok(AutoGeniusInvokationExecutionStatus::SwitchCharacterFailed);
            }
        }

        if let Some(expected_cost) = auto_genius_command_expected_cost(command) {
            if state.dice_count < expected_cost {
                let missing_dice = expected_cost - state.dice_count;
                context = auto_genius_invokation_context(
                    state,
                    Some(command_index),
                    Some(command.clone()),
                );
                let tuning = runtime.tune_auto_genius_cards_for_dice(
                    plan,
                    &context,
                    command,
                    missing_dice,
                )?;
                let tuning_completed = tuning.completed;
                if tuning_completed {
                    state.cards_tuned += tuning.cards_tuned as u32;
                    state.hand_card_count =
                        state.hand_card_count.saturating_sub(tuning.cards_tuned);
                    state.dice_count = tuning
                        .remaining_dice_count
                        .unwrap_or_else(|| state.dice_count.saturating_add(tuning.dice_gained));
                }
                executed_actions.push(auto_genius_invokation_action_report(
                    AutoGeniusInvokationPhase::ElementalTuning,
                    AutoGeniusInvokationRuntimeActionKind::TuneCards,
                    if tuning_completed {
                        AutoGeniusInvokationRuntimeActionStatus::Succeeded
                    } else if tuning.attempted {
                        AutoGeniusInvokationRuntimeActionStatus::Failed
                    } else {
                        AutoGeniusInvokationRuntimeActionStatus::Skipped
                    },
                    Some(state.current_round),
                    Some(command_index),
                    tuning
                        .message
                        .clone()
                        .unwrap_or_else(|| "elemental tuning boundary completed".to_string()),
                    AutoGeniusInvokationRuntimeActionOutcome::CardTuning(tuning),
                ));
                if !tuning_completed {
                    return Ok(AutoGeniusInvokationExecutionStatus::CardTuningFailed);
                }
                if state.dice_count < expected_cost {
                    return Ok(record_auto_genius_invokation_skip(
                        state,
                        executed_actions,
                        skipped_steps,
                        AutoGeniusInvokationPhase::Action,
                        AutoGeniusInvokationRuntimeActionKind::UseSkill,
                        Some(state.current_round),
                        Some(command_index),
                        AutoGeniusInvokationSkipReason::DiceInsufficient,
                        AutoGeniusInvokationExecutionStatus::DiceInsufficient,
                    ));
                }
            }
        }

        context = auto_genius_invokation_context(state, Some(command_index), Some(command.clone()));
        let skill = runtime.use_auto_genius_skill(plan, &context, command)?;
        let skill_completed = skill.completed && !skill.dice_lack;
        if skill_completed {
            state.skills_used += 1;
            state.commands_executed += 1;
            state.command_cursor = command_index + 1;
            apply_auto_genius_dice_change(
                &mut state.dice_count,
                skill.dice_spent,
                skill.remaining_dice_count,
            );
        }
        executed_actions.push(auto_genius_invokation_action_report(
            AutoGeniusInvokationPhase::Action,
            AutoGeniusInvokationRuntimeActionKind::UseSkill,
            if skill_completed {
                AutoGeniusInvokationRuntimeActionStatus::Succeeded
            } else {
                AutoGeniusInvokationRuntimeActionStatus::Failed
            },
            Some(state.current_round),
            Some(command_index),
            skill
                .message
                .clone()
                .unwrap_or_else(|| "skill dispatch boundary completed".to_string()),
            AutoGeniusInvokationRuntimeActionOutcome::Skill(skill.clone()),
        ));
        if !skill_completed {
            if skill.dice_lack {
                return Ok(record_auto_genius_invokation_skip(
                    state,
                    executed_actions,
                    skipped_steps,
                    AutoGeniusInvokationPhase::Action,
                    AutoGeniusInvokationRuntimeActionKind::UseSkill,
                    Some(state.current_round),
                    Some(command_index),
                    AutoGeniusInvokationSkipReason::DiceInsufficient,
                    AutoGeniusInvokationExecutionStatus::DiceInsufficient,
                ));
            }
            return Ok(AutoGeniusInvokationExecutionStatus::SkillActionFailed);
        }

        executed_actions.push(auto_genius_invokation_action_report(
            AutoGeniusInvokationPhase::Action,
            AutoGeniusInvokationRuntimeActionKind::RemoveExecutedCommand,
            AutoGeniusInvokationRuntimeActionStatus::Succeeded,
            Some(state.current_round),
            Some(command_index),
            "strategy command was consumed by the Rust executor".to_string(),
            AutoGeniusInvokationRuntimeActionOutcome::Command(AutoGeniusInvokationCommandOutcome {
                removed: true,
                remaining_commands: plan
                    .strategy
                    .action_commands
                    .len()
                    .saturating_sub(state.commands_executed),
                message: None,
            }),
        ));

        context = auto_genius_invokation_context(state, Some(command_index), Some(command.clone()));
        let wait_after = runtime.wait_auto_genius_after_action(plan, &context, command)?;
        let wait_after_completed = wait_after.completed;
        executed_actions.push(auto_genius_invokation_action_report(
            AutoGeniusInvokationPhase::Action,
            AutoGeniusInvokationRuntimeActionKind::WaitAfterAction,
            if wait_after_completed {
                AutoGeniusInvokationRuntimeActionStatus::Succeeded
            } else {
                AutoGeniusInvokationRuntimeActionStatus::Failed
            },
            Some(state.current_round),
            Some(command_index),
            wait_after
                .message
                .clone()
                .unwrap_or_else(|| "post-action wait boundary completed".to_string()),
            AutoGeniusInvokationRuntimeActionOutcome::Wait(wait_after),
        ));
        if !wait_after_completed {
            return Ok(AutoGeniusInvokationExecutionStatus::WaitMyTurnFailed);
        }
    }

    let round_context = auto_genius_invokation_context(state, None, None);
    let round_end = runtime.click_auto_genius_round_end(plan, &round_context)?;
    state.round_end_clicked = round_end.completed;
    let round_end_completed = round_end.completed;
    executed_actions.push(auto_genius_invokation_action_report(
        AutoGeniusInvokationPhase::RoundEnd,
        AutoGeniusInvokationRuntimeActionKind::ClickRoundEnd,
        if round_end_completed {
            AutoGeniusInvokationRuntimeActionStatus::Succeeded
        } else {
            AutoGeniusInvokationRuntimeActionStatus::Failed
        },
        Some(state.current_round),
        None,
        round_end
            .message
            .clone()
            .unwrap_or_else(|| "round end click boundary completed".to_string()),
        AutoGeniusInvokationRuntimeActionOutcome::RoundEnd(round_end),
    ));
    if !round_end_completed {
        return Ok(AutoGeniusInvokationExecutionStatus::RoundEndFailed);
    }

    let opponent = runtime.wait_auto_genius_opponent_action_and_end_phase(plan, &round_context)?;
    state.opponent_wait_completed = opponent.completed;
    let opponent_completed = opponent.completed;
    executed_actions.push(auto_genius_invokation_action_report(
        AutoGeniusInvokationPhase::RoundEnd,
        AutoGeniusInvokationRuntimeActionKind::WaitOpponentActionAndEndPhase,
        if opponent_completed {
            AutoGeniusInvokationRuntimeActionStatus::Succeeded
        } else {
            AutoGeniusInvokationRuntimeActionStatus::Failed
        },
        Some(state.current_round),
        None,
        opponent
            .message
            .clone()
            .unwrap_or_else(|| "opponent action/end phase wait boundary completed".to_string()),
        AutoGeniusInvokationRuntimeActionOutcome::Wait(opponent),
    ));
    if !opponent_completed {
        return Ok(AutoGeniusInvokationExecutionStatus::OpponentWaitFailed);
    }

    Ok(AutoGeniusInvokationExecutionStatus::Completed)
}

fn execute_auto_genius_invokation_cleanup<R>(
    plan: &AutoGeniusInvokationExecutionPlan,
    runtime: &mut R,
    execution_status: AutoGeniusInvokationExecutionStatus,
    state: &mut AutoGeniusInvokationExecutorState,
    executed_actions: &mut Vec<AutoGeniusInvokationRuntimeActionReport>,
) -> Result<AutoGeniusInvokationExecutionStatus>
where
    R: AutoGeniusInvokationRuntime,
{
    let exit = runtime.exit_auto_genius_invokation(plan, execution_status)?;
    state.exit_completed = exit.completed;
    let exit_completed = exit.completed;
    executed_actions.push(auto_genius_invokation_action_report(
        AutoGeniusInvokationPhase::Cleanup,
        AutoGeniusInvokationRuntimeActionKind::ExitDuel,
        if exit_completed {
            AutoGeniusInvokationRuntimeActionStatus::Succeeded
        } else if exit.attempted {
            AutoGeniusInvokationRuntimeActionStatus::Failed
        } else {
            AutoGeniusInvokationRuntimeActionStatus::Skipped
        },
        None,
        None,
        exit.message
            .clone()
            .unwrap_or_else(|| "duel exit/exception boundary completed".to_string()),
        AutoGeniusInvokationRuntimeActionOutcome::Exit(exit),
    ));

    let cleanup = runtime.cleanup_auto_genius_invokation(plan, state)?;
    state.cleanup_completed = cleanup.completed;
    let cleanup_completed = cleanup.completed;
    executed_actions.push(auto_genius_invokation_action_report(
        AutoGeniusInvokationPhase::Cleanup,
        AutoGeniusInvokationRuntimeActionKind::Cleanup,
        if cleanup_completed {
            AutoGeniusInvokationRuntimeActionStatus::Succeeded
        } else {
            AutoGeniusInvokationRuntimeActionStatus::Failed
        },
        None,
        None,
        cleanup
            .message
            .clone()
            .unwrap_or_else(|| "auto genius invokation cleanup boundary completed".to_string()),
        AutoGeniusInvokationRuntimeActionOutcome::Cleanup(cleanup),
    ));

    let notification_status = if !cleanup_completed {
        AutoGeniusInvokationExecutionStatus::CleanupFailed
    } else if !exit_completed {
        AutoGeniusInvokationExecutionStatus::RuntimeExitFailed
    } else {
        execution_status
    };
    let end_notification = runtime.notify_auto_genius_invokation_end(plan, notification_status)?;
    state.end_notification_sent = end_notification.sent;
    executed_actions.push(auto_genius_invokation_action_report(
        AutoGeniusInvokationPhase::Cleanup,
        AutoGeniusInvokationRuntimeActionKind::NotifyEnd,
        if end_notification.sent {
            AutoGeniusInvokationRuntimeActionStatus::Succeeded
        } else {
            AutoGeniusInvokationRuntimeActionStatus::Skipped
        },
        None,
        None,
        end_notification
            .message
            .clone()
            .unwrap_or_else(|| "end notification boundary completed".to_string()),
        AutoGeniusInvokationRuntimeActionOutcome::Notification(end_notification),
    ));

    if !cleanup_completed {
        Ok(AutoGeniusInvokationExecutionStatus::CleanupFailed)
    } else if !exit_completed {
        Ok(AutoGeniusInvokationExecutionStatus::RuntimeExitFailed)
    } else {
        Ok(AutoGeniusInvokationExecutionStatus::Completed)
    }
}

fn check_auto_genius_invokation_cancelled<R>(
    runtime: &mut R,
    state: &mut AutoGeniusInvokationExecutorState,
    executed_actions: &mut Vec<AutoGeniusInvokationRuntimeActionReport>,
    skipped_steps: &mut Vec<AutoGeniusInvokationSkippedStep>,
    phase: AutoGeniusInvokationPhase,
    action_kind: AutoGeniusInvokationRuntimeActionKind,
    round_index: Option<u32>,
    command_index: Option<usize>,
) -> Result<bool>
where
    R: AutoGeniusInvokationRuntime,
{
    if runtime.is_auto_genius_invokation_cancelled()? {
        state.cancelled = true;
        record_auto_genius_invokation_skip(
            state,
            executed_actions,
            skipped_steps,
            phase,
            action_kind,
            round_index,
            command_index,
            AutoGeniusInvokationSkipReason::Cancelled,
            AutoGeniusInvokationExecutionStatus::Cancelled,
        );
        Ok(true)
    } else {
        Ok(false)
    }
}

fn auto_genius_invokation_context(
    state: &AutoGeniusInvokationExecutorState,
    command_index: Option<usize>,
    command: Option<AutoGeniusActionCommandPlan>,
) -> AutoGeniusInvokationRuntimeContext {
    AutoGeniusInvokationRuntimeContext {
        round_index: state.current_round,
        command_index,
        command,
        active_character_index: state.active_character_index,
        dice_count: state.dice_count,
        hand_card_count: state.hand_card_count,
    }
}

fn auto_genius_command_expected_cost(command: &AutoGeniusActionCommandPlan) -> Option<u8> {
    command
        .all_cost
        .map(|cost| cost.max(0).min(u8::MAX as i16) as u8)
}

fn apply_auto_genius_dice_change(
    dice_count: &mut u8,
    dice_spent: u8,
    remaining_dice_count: Option<u8>,
) {
    *dice_count = remaining_dice_count.unwrap_or_else(|| dice_count.saturating_sub(dice_spent));
}

fn record_auto_genius_invokation_skip(
    state: &mut AutoGeniusInvokationExecutorState,
    executed_actions: &mut Vec<AutoGeniusInvokationRuntimeActionReport>,
    skipped_steps: &mut Vec<AutoGeniusInvokationSkippedStep>,
    phase: AutoGeniusInvokationPhase,
    action_kind: AutoGeniusInvokationRuntimeActionKind,
    round_index: Option<u32>,
    command_index: Option<usize>,
    reason: AutoGeniusInvokationSkipReason,
    status: AutoGeniusInvokationExecutionStatus,
) -> AutoGeniusInvokationExecutionStatus {
    state.last_skip_reason = Some(reason);
    skipped_steps.push(AutoGeniusInvokationSkippedStep {
        action_kind,
        round_index,
        command_index,
        reason,
    });
    executed_actions.push(auto_genius_invokation_action_report(
        phase,
        action_kind,
        AutoGeniusInvokationRuntimeActionStatus::Skipped,
        round_index,
        command_index,
        format!("skipped AutoGeniusInvokation step: {:?}", reason),
        AutoGeniusInvokationRuntimeActionOutcome::Skipped(reason),
    ));
    status
}

fn auto_genius_invokation_report(
    plan: &AutoGeniusInvokationExecutionPlan,
    status: AutoGeniusInvokationExecutionStatus,
    state: AutoGeniusInvokationExecutorState,
    executed_actions: Vec<AutoGeniusInvokationRuntimeActionReport>,
    skipped_steps: Vec<AutoGeniusInvokationSkippedStep>,
) -> AutoGeniusInvokationExecutionReport {
    AutoGeniusInvokationExecutionReport {
        task_key: plan.task_key.clone(),
        completed: status == AutoGeniusInvokationExecutionStatus::Completed,
        status,
        state,
        executed_actions,
        skipped_steps,
    }
}

fn auto_genius_invokation_action_report(
    phase: AutoGeniusInvokationPhase,
    action_kind: AutoGeniusInvokationRuntimeActionKind,
    status: AutoGeniusInvokationRuntimeActionStatus,
    round_index: Option<u32>,
    command_index: Option<usize>,
    detail: impl Into<String>,
    outcome: AutoGeniusInvokationRuntimeActionOutcome,
) -> AutoGeniusInvokationRuntimeActionReport {
    AutoGeniusInvokationRuntimeActionReport {
        phase,
        action_kind,
        status,
        round_index,
        command_index,
        detail: detail.into(),
        outcome,
    }
}

pub fn plan_auto_genius_invokation(
    working_directory: impl AsRef<Path>,
    config: AutoGeniusInvokationExecutionConfig,
) -> Result<AutoGeniusInvokationExecutionPlan> {
    let working_directory = working_directory.as_ref();
    let strategy_name = config
        .strategy_name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| config.auto_genius_invokation_config.strategy_name.clone());
    let (strategy_source, strategy_text) =
        resolve_strategy_text(working_directory, &config, &strategy_name)?;
    let strategy = parse_auto_genius_strategy(&strategy_text)?;

    Ok(AutoGeniusInvokationExecutionPlan {
        task_key: AUTO_GENIUS_INVOKATION_TASK_KEY.to_string(),
        display_name: AUTO_GENIUS_INVOKATION_DISPLAY_NAME.to_string(),
        capture_size: config.capture_size,
        asset_scale: config.asset_scale,
        config_rule: AutoGeniusInvokationConfigRule {
            strategy_name,
            sleep_delay_ms: config.auto_genius_invokation_config.sleep_delay,
            sleep_delay_min_ms: 0,
            sleep_delay_max_ms: 5_000,
            default_character_card_rects: config
                .auto_genius_invokation_config
                .default_character_card_rects
                .iter()
                .map(|rect| scale_rect(rect_config_to_rect(*rect), config.asset_scale))
                .collect(),
            active_character_card_space: config
                .auto_genius_invokation_config
                .active_character_card_space,
            my_dice_count_rect: scale_rect(
                Rect {
                    x: 68,
                    y: 642,
                    width: 25,
                    height: 31,
                },
                config.asset_scale,
            ),
            character_card_extend_hp_rect: scale_rect(
                Rect {
                    x: -20,
                    y: 0,
                    width: 60,
                    height: 55,
                },
                config.asset_scale,
            ),
        },
        strategy_source,
        strategy,
        startup_rule: AutoGeniusInvokationStartupRule {
            skips_task_runner_main_ui_wait: true,
            requires_exact_1920x1080: true,
            sends_tcg_start_notification: true,
            sends_tcg_end_notification: true,
            destroys_asset_singleton_before_start: true,
            initializes_control_with_cancellation_token: true,
            prepares_initial_hand: true,
            detects_character_rects_with_fallback: true,
            chooses_first_action_character: true,
            clears_draw_content_after_round: true,
        },
        locators: auto_genius_locators(config.capture_size),
        dice_rule: AutoGeniusInvokationDiceRule {
            initial_dice_count: 8,
            roll_phase_threshold: 0.73,
            roll_phase_expected_count: 8,
            roll_phase_expected_upper_count: 4,
            roll_phase_expected_lower_count: 4,
            roll_phase_initial_wait_ms: 5_000,
            roll_phase_retry_interval_ms: 500,
            roll_phase_retry_attempts: 35,
            post_roll_confirm_sleep_ms: 1_000,
            opponent_reroll_wait_ms: 5_000,
            action_phase_threshold: 0.7,
            action_phase_roi_right_width_divisor: 5,
            action_phase_count_retry_attempts: 20,
            action_phase_count_retry_interval_ms: 1_000,
            action_phase_expected_8_actual_9_omni_retry_limit: 5,
        },
        action_rule: AutoGeniusInvokationActionRule {
            first_round_card_count: 5,
            next_round_card_increment: 2,
            switch_character_dice_cost: 1,
            switch_button_click_offset_from_right_1080p: 100.0,
            action_button_y_offset_from_bottom_1080p: 120.0,
            switch_button_double_click: true,
            switch_animation_sleep_ms: 800,
            skill_click_offset_multiplier_1080p: 100.0,
            skill_popup_sleep_ms: 1_200,
            skill_confirm_sleep_ms: 500,
            skill_center_reset_before_click: true,
            elemental_tuning_confirm_threshold: 0.9,
            elemental_tuning_hand_layouts: elemental_tuning_hand_layouts(),
            keqing_skill_2_alternates_card_count: true,
        },
        ocr_rule: AutoGeniusInvokationOcrRule {
            dice_count_rect: scale_rect(
                Rect {
                    x: 68,
                    y: 642,
                    width: 25,
                    height: 31,
                },
                config.asset_scale,
            ),
            dice_ocr_without_detector: true,
            invalid_dice_count_sentinel: -10,
            replaces_circled_digit_text: true,
            active_character_space_offset: config
                .auto_genius_invokation_config
                .active_character_card_space,
            character_hp_empty_uses_active_offset: true,
            active_character_fallback_by_exclusion: true,
            active_character_fallback_by_template_shape: true,
        },
        wait_rule: AutoGeniusInvokationWaitRule {
            wait_my_turn_max_attempts: 60,
            wait_my_turn_interval_ms: 1_000,
            wait_my_turn_required_consecutive_hits: 3,
            wait_opponent_action_max_attempts: 60,
            wait_opponent_action_interval_ms: 1_000,
            default_after_action_wait_ms: 10_000,
            burst_after_action_wait_ms: 15_000,
            mona_switch_after_action_wait_ms: 3_000,
        },
        exception_rule: AutoGeniusInvokationExceptionRule {
            normal_end_is_logged_without_rethrow_in_duel: true,
            task_cancelled_rethrows_from_duel: true,
            outer_task_boundary_catches_all_and_logs: true,
            check_task_verifies_game_foreground: true,
            check_task_pause_retry_attempts: 100,
            check_task_pause_retry_interval_ms: 1_000,
        },
        steps: auto_genius_steps(),
        executor_ready: true,
        pending_native: vec![
            "Rust AutoGeniusInvokation now has an injectable executor boundary for startup, exact-resolution/window checks, notifications, initial hand preparation, character rect resolution/selection, dice prediction/reroll, turn waits, active-character/dice OCR observations, switch/skill/card-tuning actions, round end, duel exit, cancellation, and cleanup".to_string(),
            "desktop live adapters still need to connect TaskRunner solo-task lock, trigger clearing, window activation/foreground checks, cancellation-aware pause handling, GeniusInvokationControl input dispatch, DrawContent cleanup, and notification routing".to_string(),
            "TCG live OCR/template/input/default-card adapters remain pending for capture, template matching, OpenCV masking, dice recognition, default tcg_character_card fallback loading, elemental tuning drag attempts, skill/switch clicks, character active/HP/status/energy recognition, and live duel-loop frame mutation".to_string(),
        ],
    })
}

pub fn parse_auto_genius_strategy(script: &str) -> Result<AutoGeniusStrategyPlan> {
    let mut stage = String::new();
    let mut stage_order = Vec::new();
    let mut skipped_line_count = 0;
    let mut characters: Vec<Option<AutoGeniusCharacterPlan>> = vec![None, None, None, None];
    let mut action_commands = Vec::new();

    for (line_index, raw_line) in script.lines().enumerate() {
        let line = raw_line.trim();
        if line.contains(':') {
            stage = line.to_string();
            if !stage_order.contains(&stage) {
                stage_order.push(stage.clone());
            }
            continue;
        }
        if line == "---" || line.starts_with("//") || line.is_empty() {
            skipped_line_count += 1;
            continue;
        }

        match stage.as_str() {
            "角色定义:" => {
                let character = parse_auto_genius_character(line, line_index + 1)?;
                let index = character.index as usize;
                characters[index] = Some(character);
            }
            "策略定义:" => {
                let defined_characters: Vec<_> = characters.iter().flatten().cloned().collect();
                if defined_characters.len() != 3 {
                    return Err(strategy_error(line_index + 1, "角色未定义"));
                }
                action_commands.push(parse_auto_genius_action(
                    line,
                    line_index + 1,
                    &defined_characters,
                )?);
            }
            _ => {
                return Err(strategy_error(
                    line_index + 1,
                    format!("未知的定义字段：{stage}"),
                ));
            }
        }
    }

    let characters: Vec<_> = characters.into_iter().flatten().collect();
    if characters.len() != 3 {
        return Err(strategy_error(
            script.lines().count().max(1),
            "角色未定义，请确认策略文本格式是否为UTF-8",
        ));
    }

    Ok(AutoGeniusStrategyPlan {
        characters,
        action_commands,
        skipped_line_count,
        stage_order,
        preserves_legacy_stage_headers: true,
    })
}

fn parse_auto_genius_character(line: &str, line_number: usize) -> Result<AutoGeniusCharacterPlan> {
    let (header, skill_block) = line
        .split_once('{')
        .map(|(header, rest)| (header, Some(rest.trim_end_matches('}'))))
        .unwrap_or((line, None));
    let (index_text, value) = header
        .split_once('=')
        .ok_or_else(|| strategy_error(line_number, "角色定义解析错误"))?;
    let index = digits_from(index_text)
        .parse::<u8>()
        .map_err(|_| strategy_error(line_number, "角色序号必须在1-3之间"))?;
    if !(1..=3).contains(&index) {
        return Err(strategy_error(line_number, "角色序号必须在1-3之间"));
    }

    if let Some((name, element)) = value.split_once('|') {
        let element = chinese_to_elemental_type(
            element
                .chars()
                .next()
                .ok_or_else(|| strategy_error(line_number, "角色元素解析错误"))?,
        )
        .map_err(|message| strategy_error(line_number, message))?;
        let skills = skill_block
            .ok_or_else(|| strategy_error(line_number, "角色技能定义缺失"))?
            .split(',')
            .filter(|part| !part.trim().is_empty())
            .map(|part| parse_auto_genius_skill(part.trim(), line_number))
            .collect::<Result<Vec<_>>>()?;
        Ok(AutoGeniusCharacterPlan {
            index,
            name: name.to_string(),
            element: Some(element),
            skills,
            uses_default_card_config: false,
        })
    } else {
        Ok(AutoGeniusCharacterPlan {
            index,
            name: value.to_string(),
            element: None,
            skills: Vec::new(),
            uses_default_card_config: true,
        })
    }
}

fn parse_auto_genius_skill(line: &str, line_number: usize) -> Result<AutoGeniusSkillPlan> {
    let (skill_name, cost_text) = line
        .split_once('=')
        .ok_or_else(|| strategy_error(line_number, "技能定义解析错误"))?;
    let index = digits_from(skill_name)
        .parse::<u8>()
        .map_err(|_| strategy_error(line_number, "技能序号必须在1-5之间"))?;
    if !(1..=5).contains(&index) {
        return Err(strategy_error(line_number, "技能序号必须在1-5之间"));
    }

    let mut parts = cost_text.split('+');
    let specific = parts
        .next()
        .ok_or_else(|| strategy_error(line_number, "技能消耗解析错误"))?;
    let specific_element_cost = specific
        .chars()
        .next()
        .ok_or_else(|| strategy_error(line_number, "技能消耗解析错误"))?
        .to_digit(10)
        .ok_or_else(|| strategy_error(line_number, "技能消耗解析错误"))?
        as u8;
    let element = chinese_to_elemental_type(
        specific
            .chars()
            .nth(1)
            .ok_or_else(|| strategy_error(line_number, "技能元素解析错误"))?,
    )
    .map_err(|message| strategy_error(line_number, message))?;
    let any_element_cost = parts
        .next()
        .and_then(|part| part.chars().next())
        .and_then(|value| value.to_digit(10))
        .unwrap_or(0) as u8;

    Ok(AutoGeniusSkillPlan {
        index,
        element,
        specific_element_cost,
        any_element_cost,
        all_cost: specific_element_cost + any_element_cost,
    })
}

fn parse_auto_genius_action(
    line: &str,
    line_number: usize,
    characters: &[AutoGeniusCharacterPlan],
) -> Result<AutoGeniusActionCommandPlan> {
    let parts: Vec<_> = line.split_whitespace().collect();
    if parts.len() < 3 || parts.len() > 4 || parts[1] != "使用" {
        return Err(strategy_error(line_number, "策略中的行动命令解析错误"));
    }
    let character = characters
        .iter()
        .find(|character| character.name == parts[0])
        .ok_or_else(|| {
            strategy_error(
                line_number,
                "策略中的行动命令解析错误：角色名称无法从角色定义中匹配到",
            )
        })?;
    let target_index = digits_from(parts[2])
        .parse::<u8>()
        .map_err(|_| strategy_error(line_number, "策略中的行动命令解析错误：技能编号错误"))?;
    if target_index >= 5 {
        return Err(strategy_error(
            line_number,
            "策略中的行动命令解析错误：技能编号错误",
        ));
    }
    let dice_delta = if let Some(delta) = parts.get(3) {
        if let Some(value) = delta.strip_prefix("骰子增加") {
            digits_from(value).parse::<i8>().map_err(|_| {
                strategy_error(
                    line_number,
                    "策略中的行动命令解析错误：骰子增减参数格式不正确",
                )
            })?
        } else if let Some(value) = delta.strip_prefix("骰子减少") {
            -digits_from(value).parse::<i8>().map_err(|_| {
                strategy_error(
                    line_number,
                    "策略中的行动命令解析错误：骰子增减参数格式不正确",
                )
            })?
        } else {
            return Err(strategy_error(
                line_number,
                format!(
                    "策略中的行动命令解析错误：骰子增减参数格式不正确（应为 骰子增加N 或 骰子减少N ），实际：{delta}"
                ),
            ));
        }
    } else {
        0
    };

    let skill = character
        .skills
        .iter()
        .find(|skill| skill.index == target_index);
    Ok(AutoGeniusActionCommandPlan {
        character_index: character.index,
        character_name: character.name.clone(),
        action: AutoGeniusActionKind::UseSkill,
        target_index,
        dice_delta,
        all_cost: skill.map(|skill| skill.all_cost as i16 + dice_delta as i16),
        dice_element: skill.map(|skill| skill.element),
    })
}

fn resolve_strategy_text(
    working_directory: &Path,
    config: &AutoGeniusInvokationExecutionConfig,
    strategy_name: &str,
) -> Result<(AutoGeniusInvokationStrategySource, String)> {
    if let Some(strategy) = config.strategy.clone() {
        return Ok((
            AutoGeniusInvokationStrategySource {
                strategy_name: strategy_name.to_string(),
                user_strategy_directory: AUTO_GENIUS_INVOKATION_USER_STRATEGY_DIR.to_string(),
                strategy_path: None,
                inline_strategy: true,
                default_card_config_asset: AUTO_GENIUS_INVOKATION_DEFAULT_CARD_ASSET.to_string(),
            },
            strategy,
        ));
    }

    let strategy_path = normalize_auto_genius_strategy_path(strategy_name)?;
    let absolute_strategy_path = working_directory.join(&strategy_path);
    let strategy = fs::read_to_string(&absolute_strategy_path).map_err(|error| {
        TaskError::InvalidTaskConfig {
            key: AUTO_GENIUS_INVOKATION_TASK_KEY.to_string(),
            message: format!(
                "failed to read AutoGeniusInvokation strategy {}: {error}",
                absolute_strategy_path.display()
            ),
        }
    })?;

    Ok((
        AutoGeniusInvokationStrategySource {
            strategy_name: strategy_name.to_string(),
            user_strategy_directory: AUTO_GENIUS_INVOKATION_USER_STRATEGY_DIR.to_string(),
            strategy_path: Some(strategy_path.to_string_lossy().replace('\\', "/")),
            inline_strategy: false,
            default_card_config_asset: AUTO_GENIUS_INVOKATION_DEFAULT_CARD_ASSET.to_string(),
        },
        strategy,
    ))
}

pub fn normalize_auto_genius_strategy_path(strategy_name: &str) -> Result<PathBuf> {
    let strategy_name = strategy_name.trim();
    if strategy_name.is_empty() {
        return Err(TaskError::InvalidTaskConfig {
            key: AUTO_GENIUS_INVOKATION_TASK_KEY.to_string(),
            message: "AutoGeniusInvokation strategy name is empty".to_string(),
        });
    }
    let mut relative = PathBuf::from(AUTO_GENIUS_INVOKATION_USER_STRATEGY_DIR);
    let mut name_path = PathBuf::from(strategy_name.replace('\\', "/"));
    if name_path.extension().is_none() {
        name_path.set_extension("txt");
    }
    for component in name_path.components() {
        match component {
            Component::Normal(value) => relative.push(value),
            _ => {
                return Err(TaskError::InvalidTaskConfig {
                    key: AUTO_GENIUS_INVOKATION_TASK_KEY.to_string(),
                    message: format!("invalid AutoGeniusInvokation strategy path: {strategy_name}"),
                });
            }
        }
    }
    Ok(relative)
}

fn auto_genius_locators(capture_size: Size) -> AutoGeniusInvokationLocators {
    AutoGeniusInvokationLocators {
        confirm_button: template(
            "ConfirmButton",
            "AutoGeniusInvokation:other/确定.png",
            None,
            0.8,
            false,
        ),
        round_end_button: template(
            "RoundEndButton",
            "AutoGeniusInvokation:other/回合结束.png",
            Some(Rect {
                x: 0,
                y: 0,
                width: (capture_size.width / 5) as i32,
                height: capture_size.height as i32,
            }),
            0.8,
            true,
        ),
        elemental_tuning_confirm_button: template(
            "ElementalTuningConfirmButton",
            "AutoGeniusInvokation:other/元素调和.png",
            Some(Rect {
                x: 0,
                y: (capture_size.height / 2) as i32,
                width: capture_size.width as i32,
                height: (capture_size.height / 2) as i32,
            }),
            0.9,
            false,
        ),
        exit_duel_button: template(
            "ExitDuelButton",
            "AutoGeniusInvokation:other/退出挑战.png",
            Some(Rect {
                x: 0,
                y: (capture_size.height / 2) as i32,
                width: (capture_size.width / 2) as i32,
                height: (capture_size.height / 2) as i32,
            }),
            0.8,
            true,
        ),
        in_opponent_action: template(
            "InOpponentAction",
            "AutoGeniusInvokation:other/对方行动中.png",
            Some(Rect {
                x: 0,
                y: 0,
                width: (capture_size.width / 5) as i32,
                height: capture_size.height as i32,
            }),
            0.8,
            true,
        ),
        end_phase: template(
            "EndPhase",
            "AutoGeniusInvokation:other/回合结算阶段.png",
            Some(Rect {
                x: 0,
                y: 0,
                width: (capture_size.width / 5) as i32,
                height: capture_size.height as i32,
            }),
            0.8,
            true,
        ),
        elemental_dice_lack_warning: template(
            "ElementalDiceLackWarning",
            "AutoGeniusInvokation:other/元素骰子不足.png",
            Some(Rect {
                x: (capture_size.width / 2) as i32,
                y: 0,
                width: (capture_size.width / 2) as i32,
                height: capture_size.height as i32,
            }),
            0.8,
            true,
        ),
        character_taken_out: template(
            "CharacterTakenOut",
            "AutoGeniusInvokation:other/角色死亡.png",
            None,
            0.8,
            true,
        ),
        in_character_pick: template(
            "InCharacterPick",
            "AutoGeniusInvokation:other/出战角色.png",
            Some(Rect {
                x: (capture_size.width / 2) as i32,
                y: (capture_size.height / 2) as i32,
                width: (capture_size.width / 2) as i32,
                height: (capture_size.height / 2) as i32,
            }),
            0.8,
            true,
        ),
        character_hp_upper: template(
            "CharacterHpUpper",
            "AutoGeniusInvokation:other/角色血量上方.png",
            None,
            0.8,
            true,
        ),
        grayscale_assets: vec![
            "AutoGeniusInvokation:other/角色被打败.png".to_string(),
            "AutoGeniusInvokation:other/角色状态_冻结.png".to_string(),
            "AutoGeniusInvokation:other/角色状态_水泡.png".to_string(),
            "AutoGeniusInvokation:other/满能量.png".to_string(),
        ],
        roll_phase_dice_assets: dice_assets("roll"),
        action_phase_dice_assets: dice_assets("action"),
    }
}

fn template(
    name: &str,
    asset: &str,
    roi: Option<Rect>,
    threshold: f64,
    draw_on_window: bool,
) -> AutoGeniusTemplateLocator {
    AutoGeniusTemplateLocator {
        name: name.to_string(),
        asset: asset.to_string(),
        roi,
        threshold,
        match_mode: TemplateMatchMode::CCoeffNormed,
        draw_on_window,
    }
}

fn dice_assets(prefix: &str) -> Vec<AutoGeniusDiceAsset> {
    [
        (AutoGeniusElementalType::Anemo, "anemo"),
        (AutoGeniusElementalType::Electro, "electro"),
        (AutoGeniusElementalType::Dendro, "dendro"),
        (AutoGeniusElementalType::Hydro, "hydro"),
        (AutoGeniusElementalType::Pyro, "pyro"),
        (AutoGeniusElementalType::Cryo, "cryo"),
        (AutoGeniusElementalType::Geo, "geo"),
        (AutoGeniusElementalType::Omni, "omni"),
    ]
    .into_iter()
    .map(|(element, name)| AutoGeniusDiceAsset {
        element,
        asset: format!("AutoGeniusInvokation:dice/{prefix}_{name}.png"),
    })
    .collect()
}

fn elemental_tuning_hand_layouts() -> Vec<AutoGeniusHandLayout> {
    [
        (10, 570.0, 120.0),
        (9, 570.0, 130.0),
        (8, 600.0, 145.0),
        (7, 630.0, 160.0),
        (6, 620.0, 200.0),
        (5, 720.0, 200.0),
        (4, 820.0, 200.0),
        (3, 920.0, 200.0),
        (2, 1020.0, 200.0),
        (1, 1120.0, 200.0),
    ]
    .into_iter()
    .map(
        |(card_count, start_x_1080p, spacing_1080p)| AutoGeniusHandLayout {
            card_count,
            start_x_1080p,
            spacing_1080p,
        },
    )
    .collect()
}

fn auto_genius_steps() -> Vec<AutoGeniusInvokationStep> {
    use AutoGeniusInvokationPhase::*;
    use AutoGeniusInvokationStepAction::*;
    vec![
        AutoGeniusInvokationStep {
            phase: Startup,
            action: ValidateResolution,
        },
        AutoGeniusInvokationStep {
            phase: Startup,
            action: ParseStrategy,
        },
        AutoGeniusInvokationStep {
            phase: Startup,
            action: NotifyStart,
        },
        AutoGeniusInvokationStep {
            phase: Prepare,
            action: PrepareInitialHand,
        },
        AutoGeniusInvokationStep {
            phase: Prepare,
            action: ResolveCharacterRects,
        },
        AutoGeniusInvokationStep {
            phase: Prepare,
            action: ChooseFirstCharacter,
        },
        AutoGeniusInvokationStep {
            phase: RollDice,
            action: PredictDiceTypes,
        },
        AutoGeniusInvokationStep {
            phase: RollDice,
            action: ReRollDice,
        },
        AutoGeniusInvokationStep {
            phase: MyTurn,
            action: WaitForMyTurn,
        },
        AutoGeniusInvokationStep {
            phase: Action,
            action: DetectActiveCharacter,
        },
        AutoGeniusInvokationStep {
            phase: Action,
            action: CalibrateDiceCountByOcr,
        },
        AutoGeniusInvokationStep {
            phase: Action,
            action: SwitchCharacterIfNeeded,
        },
        AutoGeniusInvokationStep {
            phase: Action,
            action: UseSkillOrTuneCards,
        },
        AutoGeniusInvokationStep {
            phase: Action,
            action: RemoveExecutedCommand,
        },
        AutoGeniusInvokationStep {
            phase: RoundEnd,
            action: ClickRoundEnd,
        },
        AutoGeniusInvokationStep {
            phase: RoundEnd,
            action: WaitOpponentActionAndEndPhase,
        },
        AutoGeniusInvokationStep {
            phase: Cleanup,
            action: NotifyEnd,
        },
    ]
}

fn rect_config_to_rect(rect: RectConfig) -> Rect {
    Rect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}

fn scale_rect(rect: Rect, scale: f64) -> Rect {
    Rect {
        x: (rect.x as f64 * scale) as i32,
        y: (rect.y as f64 * scale) as i32,
        width: (rect.width as f64 * scale) as i32,
        height: (rect.height as f64 * scale) as i32,
    }
}

fn chinese_to_elemental_type(value: char) -> std::result::Result<AutoGeniusElementalType, String> {
    match value {
        '全' => Ok(AutoGeniusElementalType::Omni),
        '冰' => Ok(AutoGeniusElementalType::Cryo),
        '水' => Ok(AutoGeniusElementalType::Hydro),
        '火' => Ok(AutoGeniusElementalType::Pyro),
        '雷' => Ok(AutoGeniusElementalType::Electro),
        '草' => Ok(AutoGeniusElementalType::Dendro),
        '风' => Ok(AutoGeniusElementalType::Anemo),
        '岩' => Ok(AutoGeniusElementalType::Geo),
        _ => Err(format!("unknown elemental type: {value}")),
    }
}

fn digits_from(value: &str) -> String {
    value.chars().filter(char::is_ascii_digit).collect()
}

fn strategy_error(line: usize, message: impl Into<String>) -> TaskError {
    TaskError::InvalidTaskConfig {
        key: AUTO_GENIUS_INVOKATION_TASK_KEY.to_string(),
        message: format!("strategy parse error at line {line}: {}", message.into()),
    }
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn f64_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<f64> {
    member_value(value, &names).and_then(|value| {
        value
            .as_f64()
            .or_else(|| value.as_str().and_then(|value| value.parse::<f64>().ok()))
    })
}

fn string_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<String> {
    member_value(value, &names).and_then(|value| value.as_str().map(str::to_string))
}

fn member_value<'a>(value: &'a Value, names: &[&str]) -> Option<&'a Value> {
    names.iter().find_map(|name| value.get(*name))
}

#[cfg(test)]
mod auto_genius_invokation_executor_tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum RuntimeCall {
        Start,
        CheckWindow,
        NotifyStart,
        PrepareInitialHand,
        ResolveCharacterRects,
        ChooseInitialCharacter(u8),
        PredictDice,
        RerollDice,
        WaitMyTurn(usize),
        DetectActiveCharacter(usize),
        CalibrateDice(usize),
        SwitchCharacter(usize, u8),
        TuneCards(usize, u8),
        UseSkill(usize, u8),
        WaitAfterAction(usize),
        RoundEnd,
        WaitOpponent,
        Exit(AutoGeniusInvokationExecutionStatus),
        NotifyEnd(AutoGeniusInvokationExecutionStatus),
        Cleanup,
    }

    #[derive(Debug)]
    struct FakeAutoGeniusInvokationRuntime {
        calls: Vec<RuntimeCall>,
        startup: AutoGeniusInvokationStartupOutcome,
        window: AutoGeniusInvokationWindowOutcome,
        initial_hand: AutoGeniusInvokationInitialHandOutcome,
        character_rects: AutoGeniusInvokationCharacterRectsOutcome,
        active_characters: VecDeque<Option<u8>>,
        dice_counts: VecDeque<Option<u8>>,
        switch_outcome: AutoGeniusInvokationSwitchOutcome,
        tune_outcomes: VecDeque<AutoGeniusInvokationCardTuningOutcome>,
        skill_outcomes: VecDeque<AutoGeniusInvokationSkillOutcome>,
        cancel_after_checks: Option<u32>,
        cancel_checks: u32,
        cleanup_calls: u32,
    }

    impl Default for FakeAutoGeniusInvokationRuntime {
        fn default() -> Self {
            Self {
                calls: Vec::new(),
                startup: AutoGeniusInvokationStartupOutcome {
                    completed: true,
                    assets_initialized: true,
                    control_initialized: true,
                    message: None,
                },
                window: AutoGeniusInvokationWindowOutcome {
                    foreground: true,
                    resolution_supported: true,
                    capture_size: Size::new(
                        AUTO_GENIUS_INVOKATION_DEFAULT_CAPTURE_WIDTH,
                        AUTO_GENIUS_INVOKATION_DEFAULT_CAPTURE_HEIGHT,
                    ),
                    message: None,
                },
                initial_hand: AutoGeniusInvokationInitialHandOutcome {
                    completed: true,
                    card_count: Some(5),
                    message: None,
                },
                character_rects: AutoGeniusInvokationCharacterRectsOutcome {
                    completed: true,
                    detected_character_count: 3,
                    character_rects: vec![
                        Rect {
                            x: 667,
                            y: 632,
                            width: 165,
                            height: 282,
                        },
                        Rect {
                            x: 878,
                            y: 632,
                            width: 165,
                            height: 282,
                        },
                        Rect {
                            x: 1090,
                            y: 632,
                            width: 165,
                            height: 282,
                        },
                    ],
                    used_fallback: false,
                    message: None,
                },
                active_characters: VecDeque::new(),
                dice_counts: VecDeque::new(),
                switch_outcome: AutoGeniusInvokationSwitchOutcome {
                    completed: true,
                    from_character_index: Some(1),
                    to_character_index: 2,
                    dice_spent: 1,
                    remaining_dice_count: Some(7),
                    message: None,
                },
                tune_outcomes: VecDeque::new(),
                skill_outcomes: VecDeque::new(),
                cancel_after_checks: None,
                cancel_checks: 0,
                cleanup_calls: 0,
            }
        }
    }

    impl FakeAutoGeniusInvokationRuntime {
        fn with_active_characters(mut self, values: impl IntoIterator<Item = u8>) -> Self {
            self.active_characters = values.into_iter().map(Some).collect();
            self
        }

        fn with_dice_counts(mut self, values: impl IntoIterator<Item = u8>) -> Self {
            self.dice_counts = values.into_iter().map(Some).collect();
            self
        }

        fn with_skill_outcomes(
            mut self,
            outcomes: impl IntoIterator<Item = AutoGeniusInvokationSkillOutcome>,
        ) -> Self {
            self.skill_outcomes = outcomes.into_iter().collect();
            self
        }

        fn with_tune_outcomes(
            mut self,
            outcomes: impl IntoIterator<Item = AutoGeniusInvokationCardTuningOutcome>,
        ) -> Self {
            self.tune_outcomes = outcomes.into_iter().collect();
            self
        }
    }

    impl AutoGeniusInvokationRuntime for FakeAutoGeniusInvokationRuntime {
        fn start_auto_genius_invokation(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
        ) -> Result<AutoGeniusInvokationStartupOutcome> {
            self.calls.push(RuntimeCall::Start);
            Ok(self.startup.clone())
        }

        fn check_auto_genius_invokation_window(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
        ) -> Result<AutoGeniusInvokationWindowOutcome> {
            self.calls.push(RuntimeCall::CheckWindow);
            Ok(self.window.clone())
        }

        fn notify_auto_genius_invokation_start(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
        ) -> Result<AutoGeniusInvokationNotificationOutcome> {
            self.calls.push(RuntimeCall::NotifyStart);
            Ok(notification_outcome())
        }

        fn prepare_auto_genius_initial_hand(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
        ) -> Result<AutoGeniusInvokationInitialHandOutcome> {
            self.calls.push(RuntimeCall::PrepareInitialHand);
            Ok(self.initial_hand.clone())
        }

        fn resolve_auto_genius_character_rects(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
        ) -> Result<AutoGeniusInvokationCharacterRectsOutcome> {
            self.calls.push(RuntimeCall::ResolveCharacterRects);
            Ok(self.character_rects.clone())
        }

        fn choose_auto_genius_initial_character(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            _context: &AutoGeniusInvokationRuntimeContext,
            character: &AutoGeniusCharacterPlan,
        ) -> Result<AutoGeniusInvokationCharacterSelectionOutcome> {
            self.calls
                .push(RuntimeCall::ChooseInitialCharacter(character.index));
            Ok(AutoGeniusInvokationCharacterSelectionOutcome {
                selected: true,
                character_index: Some(character.index),
                message: None,
            })
        }

        fn predict_auto_genius_roll_dice(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            _context: &AutoGeniusInvokationRuntimeContext,
        ) -> Result<AutoGeniusInvokationDicePredictionOutcome> {
            self.calls.push(RuntimeCall::PredictDice);
            Ok(AutoGeniusInvokationDicePredictionOutcome {
                completed: true,
                preferred_elements: vec![AutoGeniusElementalType::Electro],
                message: None,
            })
        }

        fn reroll_auto_genius_dice(
            &mut self,
            plan: &AutoGeniusInvokationExecutionPlan,
            _context: &AutoGeniusInvokationRuntimeContext,
            _prediction: &AutoGeniusInvokationDicePredictionOutcome,
        ) -> Result<AutoGeniusInvokationDiceRollOutcome> {
            self.calls.push(RuntimeCall::RerollDice);
            Ok(AutoGeniusInvokationDiceRollOutcome {
                completed: true,
                dice: vec![AutoGeniusElementalDiceCount {
                    element: AutoGeniusElementalType::Omni,
                    count: plan.dice_rule.initial_dice_count,
                }],
                dice_count: plan.dice_rule.initial_dice_count,
                reroll_attempts: 1,
                message: None,
            })
        }

        fn wait_auto_genius_my_turn(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            context: &AutoGeniusInvokationRuntimeContext,
        ) -> Result<AutoGeniusInvokationWaitOutcome> {
            self.calls
                .push(RuntimeCall::WaitMyTurn(context.command_index.unwrap()));
            Ok(AutoGeniusInvokationWaitOutcome {
                completed: true,
                attempts: 1,
                message: None,
            })
        }

        fn detect_auto_genius_active_character(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            context: &AutoGeniusInvokationRuntimeContext,
        ) -> Result<AutoGeniusInvokationActiveCharacterOutcome> {
            self.calls.push(RuntimeCall::DetectActiveCharacter(
                context.command_index.unwrap(),
            ));
            let character_index = self
                .active_characters
                .pop_front()
                .unwrap_or(context.active_character_index);
            Ok(AutoGeniusInvokationActiveCharacterOutcome {
                detected: character_index.is_some(),
                character_index,
                defeated_character_indices: Vec::new(),
                message: None,
            })
        }

        fn calibrate_auto_genius_dice_count(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            context: &AutoGeniusInvokationRuntimeContext,
        ) -> Result<AutoGeniusInvokationDiceCountOutcome> {
            self.calls
                .push(RuntimeCall::CalibrateDice(context.command_index.unwrap()));
            let dice_count = self
                .dice_counts
                .pop_front()
                .unwrap_or(Some(context.dice_count));
            Ok(AutoGeniusInvokationDiceCountOutcome {
                dice_count,
                dice: Vec::new(),
                message: None,
            })
        }

        fn switch_auto_genius_character(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            context: &AutoGeniusInvokationRuntimeContext,
            command: &AutoGeniusActionCommandPlan,
        ) -> Result<AutoGeniusInvokationSwitchOutcome> {
            self.calls.push(RuntimeCall::SwitchCharacter(
                context.command_index.unwrap(),
                command.character_index,
            ));
            let mut outcome = self.switch_outcome.clone();
            outcome.from_character_index = context.active_character_index;
            outcome.to_character_index = command.character_index;
            Ok(outcome)
        }

        fn tune_auto_genius_cards_for_dice(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            context: &AutoGeniusInvokationRuntimeContext,
            _command: &AutoGeniusActionCommandPlan,
            missing_dice: u8,
        ) -> Result<AutoGeniusInvokationCardTuningOutcome> {
            self.calls.push(RuntimeCall::TuneCards(
                context.command_index.unwrap(),
                missing_dice,
            ));
            Ok(self
                .tune_outcomes
                .pop_front()
                .unwrap_or(AutoGeniusInvokationCardTuningOutcome {
                    attempted: true,
                    completed: true,
                    cards_tuned: missing_dice,
                    dice_gained: missing_dice,
                    remaining_dice_count: Some(context.dice_count + missing_dice),
                    message: None,
                }))
        }

        fn use_auto_genius_skill(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            context: &AutoGeniusInvokationRuntimeContext,
            command: &AutoGeniusActionCommandPlan,
        ) -> Result<AutoGeniusInvokationSkillOutcome> {
            self.calls.push(RuntimeCall::UseSkill(
                context.command_index.unwrap(),
                command.target_index,
            ));
            Ok(self.skill_outcomes.pop_front().unwrap_or_else(|| {
                let dice_spent = auto_genius_command_expected_cost(command).unwrap_or(0);
                AutoGeniusInvokationSkillOutcome {
                    completed: true,
                    dice_spent,
                    remaining_dice_count: Some(context.dice_count.saturating_sub(dice_spent)),
                    dice_lack: false,
                    message: None,
                }
            }))
        }

        fn wait_auto_genius_after_action(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            context: &AutoGeniusInvokationRuntimeContext,
            _command: &AutoGeniusActionCommandPlan,
        ) -> Result<AutoGeniusInvokationWaitOutcome> {
            self.calls
                .push(RuntimeCall::WaitAfterAction(context.command_index.unwrap()));
            Ok(AutoGeniusInvokationWaitOutcome {
                completed: true,
                attempts: 1,
                message: None,
            })
        }

        fn click_auto_genius_round_end(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            _context: &AutoGeniusInvokationRuntimeContext,
        ) -> Result<AutoGeniusInvokationRoundEndOutcome> {
            self.calls.push(RuntimeCall::RoundEnd);
            Ok(AutoGeniusInvokationRoundEndOutcome {
                completed: true,
                message: None,
            })
        }

        fn wait_auto_genius_opponent_action_and_end_phase(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            _context: &AutoGeniusInvokationRuntimeContext,
        ) -> Result<AutoGeniusInvokationWaitOutcome> {
            self.calls.push(RuntimeCall::WaitOpponent);
            Ok(AutoGeniusInvokationWaitOutcome {
                completed: true,
                attempts: 1,
                message: None,
            })
        }

        fn exit_auto_genius_invokation(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            status: AutoGeniusInvokationExecutionStatus,
        ) -> Result<AutoGeniusInvokationExitOutcome> {
            self.calls.push(RuntimeCall::Exit(status));
            Ok(AutoGeniusInvokationExitOutcome {
                attempted: true,
                completed: true,
                normal_end: status == AutoGeniusInvokationExecutionStatus::Completed,
                message: None,
            })
        }

        fn notify_auto_genius_invokation_end(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            status: AutoGeniusInvokationExecutionStatus,
        ) -> Result<AutoGeniusInvokationNotificationOutcome> {
            self.calls.push(RuntimeCall::NotifyEnd(status));
            Ok(notification_outcome())
        }

        fn cleanup_auto_genius_invokation(
            &mut self,
            _plan: &AutoGeniusInvokationExecutionPlan,
            _state: &AutoGeniusInvokationExecutorState,
        ) -> Result<AutoGeniusInvokationCleanupOutcome> {
            self.calls.push(RuntimeCall::Cleanup);
            self.cleanup_calls += 1;
            Ok(AutoGeniusInvokationCleanupOutcome {
                completed: true,
                inputs_released: true,
                overlays_cleared: true,
                message: None,
            })
        }

        fn is_auto_genius_invokation_cancelled(&mut self) -> Result<bool> {
            self.cancel_checks += 1;
            Ok(self
                .cancel_after_checks
                .is_some_and(|threshold| self.cancel_checks >= threshold))
        }
    }

    #[test]
    fn auto_genius_invokation_plan_is_executor_ready_with_live_adapters_pending() {
        let plan = test_plan();

        assert!(plan.executor_ready);
        assert!(plan
            .pending_native
            .iter()
            .any(|item| item.contains("injectable executor boundary")));
        assert!(plan
            .pending_native
            .iter()
            .any(|item| item.contains("desktop live adapters")));
        assert!(plan
            .pending_native
            .iter()
            .any(|item| item.contains("default tcg_character_card fallback")));
        assert!(!plan
            .pending_native
            .iter()
            .any(|item| item.contains("no Rust executor")));
    }

    #[test]
    fn execute_auto_genius_invokation_normal_strategy_success() {
        let plan = test_plan();
        let mut runtime = FakeAutoGeniusInvokationRuntime::default()
            .with_active_characters([1, 1])
            .with_dice_counts([8, 6]);

        let report = execute_auto_genius_invokation_plan(&plan, &mut runtime).unwrap();

        assert_eq!(
            report.status,
            AutoGeniusInvokationExecutionStatus::Completed
        );
        assert!(report.completed);
        assert_eq!(report.state.commands_total, 2);
        assert_eq!(report.state.commands_executed, 2);
        assert_eq!(report.state.skills_used, 2);
        assert_eq!(report.state.switches_performed, 1);
        assert!(report.state.round_end_clicked);
        assert!(report.state.cleanup_completed);
        assert_eq!(runtime.cleanup_calls, 1);
        assert!(runtime.calls.contains(&RuntimeCall::SwitchCharacter(1, 2)));
        assert!(runtime.calls.contains(&RuntimeCall::UseSkill(0, 2)));
        assert!(runtime.calls.contains(&RuntimeCall::UseSkill(1, 1)));
        assert_eq!(
            runtime.calls.last(),
            Some(&RuntimeCall::NotifyEnd(
                AutoGeniusInvokationExecutionStatus::Completed
            ))
        );
    }

    #[test]
    fn execute_auto_genius_invokation_missing_character_rects_runs_cleanup() {
        let plan = test_plan();
        let mut runtime = FakeAutoGeniusInvokationRuntime {
            character_rects: AutoGeniusInvokationCharacterRectsOutcome {
                completed: false,
                detected_character_count: 2,
                character_rects: Vec::new(),
                used_fallback: true,
                message: Some("missing one character".to_string()),
            },
            ..FakeAutoGeniusInvokationRuntime::default()
        };

        let report = execute_auto_genius_invokation_plan(&plan, &mut runtime).unwrap();

        assert_eq!(
            report.status,
            AutoGeniusInvokationExecutionStatus::CharacterResolutionFailed
        );
        assert!(!report.completed);
        assert_eq!(report.state.character_rect_count, 2);
        assert!(report.state.used_character_rect_fallback);
        assert_eq!(report.state.commands_executed, 0);
        assert_eq!(runtime.cleanup_calls, 1);
        assert!(runtime.calls.contains(&RuntimeCall::Cleanup));
        assert!(runtime.calls.contains(&RuntimeCall::NotifyEnd(
            AutoGeniusInvokationExecutionStatus::CharacterResolutionFailed
        )));
        assert!(!runtime
            .calls
            .iter()
            .any(|call| matches!(call, RuntimeCall::UseSkill(_, _))));
    }

    #[test]
    fn execute_auto_genius_invokation_cancelled_after_startup_runs_cleanup() {
        let plan = test_plan();
        let mut runtime = FakeAutoGeniusInvokationRuntime {
            cancel_after_checks: Some(1),
            ..FakeAutoGeniusInvokationRuntime::default()
        };

        let report = execute_auto_genius_invokation_plan(&plan, &mut runtime).unwrap();

        assert_eq!(
            report.status,
            AutoGeniusInvokationExecutionStatus::Cancelled
        );
        assert!(report.state.cancelled);
        assert_eq!(report.state.commands_executed, 0);
        assert_eq!(
            report.state.last_skip_reason,
            Some(AutoGeniusInvokationSkipReason::Cancelled)
        );
        assert_eq!(runtime.cleanup_calls, 1);
        assert!(runtime.calls.contains(&RuntimeCall::Cleanup));
        assert!(!runtime.calls.contains(&RuntimeCall::CheckWindow));
    }

    #[test]
    fn execute_auto_genius_invokation_dice_insufficient_stops_before_skill() {
        let plan = test_plan();
        let mut runtime = FakeAutoGeniusInvokationRuntime::default()
            .with_active_characters([1])
            .with_dice_counts([1])
            .with_tune_outcomes([AutoGeniusInvokationCardTuningOutcome {
                attempted: true,
                completed: true,
                cards_tuned: 0,
                dice_gained: 0,
                remaining_dice_count: Some(1),
                message: Some("no usable cards".to_string()),
            }])
            .with_skill_outcomes([AutoGeniusInvokationSkillOutcome {
                completed: true,
                dice_spent: 2,
                remaining_dice_count: Some(0),
                dice_lack: false,
                message: None,
            }]);

        let report = execute_auto_genius_invokation_plan(&plan, &mut runtime).unwrap();

        assert_eq!(
            report.status,
            AutoGeniusInvokationExecutionStatus::DiceInsufficient
        );
        assert_eq!(report.state.commands_executed, 0);
        assert_eq!(
            report.state.last_skip_reason,
            Some(AutoGeniusInvokationSkipReason::DiceInsufficient)
        );
        assert_eq!(runtime.cleanup_calls, 1);
        assert!(runtime.calls.contains(&RuntimeCall::TuneCards(0, 1)));
        assert!(!runtime.calls.contains(&RuntimeCall::UseSkill(0, 2)));
    }

    fn test_plan() -> AutoGeniusInvokationExecutionPlan {
        let strategy = r#"
角色定义:
角色1=刻晴|雷{技能2消耗=2雷骰子,技能1消耗=1雷骰子}
角色2=莫娜|水{技能1消耗=3水骰子}
角色3=甘雨|冰{技能1消耗=3冰骰子}
策略定义:
刻晴 使用 技能2
莫娜 使用 技能1
"#;
        plan_auto_genius_invokation(
            ".",
            AutoGeniusInvokationExecutionConfig {
                strategy_name: Some("test".to_string()),
                strategy: Some(strategy.to_string()),
                ..AutoGeniusInvokationExecutionConfig::default()
            },
        )
        .unwrap()
    }

    fn notification_outcome() -> AutoGeniusInvokationNotificationOutcome {
        AutoGeniusInvokationNotificationOutcome {
            sent: true,
            message: None,
        }
    }
}
