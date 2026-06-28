use bgi_core::config::{
    AutoLeyLineOutcropConfig, AutoLeyLineOutcropFightConfig, LeyLineFightFinishDetectConfig,
};
use bgi_vision::{Rect, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use crate::task_params::{
    combat_strategy_path, AutoLeyLineOutcropFightConfigParam, AutoLeyLineOutcropParam,
    FightFinishDetectParam, AUTO_STRATEGY_NAME,
};
use crate::{Result, TaskError};

pub const AUTO_LEY_LINE_OUTCROP_TASK_KEY: &str = "AutoLeyLineOutcrop";
pub const AUTO_LEY_LINE_OUTCROP_DISPLAY_NAME: &str = "自动地脉花";
pub const AUTO_LEY_LINE_OUTCROP_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_LEY_LINE_OUTCROP_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_LEY_LINE_OUTCROP_ASSET_DIR: &str = "GameTask/AutoLeyLineOutcrop/Assets";
pub const AUTO_LEY_LINE_OUTCROP_TASK_DIR: &str = "GameTask/AutoLeyLineOutcrop";
pub const AUTO_LEY_LINE_OUTCROP_CONFIG_JSON: &str =
    "GameTask/AutoLeyLineOutcrop/Assets/config.json";
pub const AUTO_LEY_LINE_OUTCROP_NODE_JSON: &str =
    "GameTask/AutoLeyLineOutcrop/Assets/LeyLineOutcropData.json";
pub const AUTO_LEY_LINE_ORIGINAL_RESIN_COST: i32 = 40;
pub const AUTO_LEY_LINE_HALF_ORIGINAL_RESIN_COST: i32 = 20;
pub const AUTO_LEY_LINE_MAX_RECHECK_COUNT: u64 = 3;
pub const AUTO_LEY_LINE_MAX_CONSECUTIVE_FAILURES: u64 = 5;
pub const AUTO_LEY_LINE_PROCESS_MAX_RETRIES: u64 = 3;
pub const AUTO_LEY_LINE_REWARD_MAX_RETRIES: u64 = 3;
pub const AUTO_LEY_LINE_REWARD_USE_ATTEMPTS: u64 = 2;
pub const AUTO_LEY_LINE_MAX_SCAN_DROPS_AFTER_REWARD_SECONDS: i32 = 60;
pub const AUTO_LEY_LINE_FIGHT_SEEK_INITIAL_DELAY_SECONDS: u64 = 2;
pub const AUTO_LEY_LINE_KAZUHA_PICKUP_POST_SKILL_WAIT_MS: u64 = 3_000;

pub const AUTO_LEY_LINE_VALID_TYPES: &[&str] = &["启示之花", "藏金之花"];
pub const AUTO_LEY_LINE_SUCCESS_KEYWORDS: &[&str] = &["挑战达成", "战斗胜利", "挑战成功"];
pub const AUTO_LEY_LINE_FAILURE_KEYWORDS: &[&str] = &["挑战失败"];
pub const AUTO_LEY_LINE_FIGHT_KEYWORDS: &[&str] = &["打倒", "所有", "敌人"];
pub const AUTO_LEY_LINE_REWARD_RESIN_PRIORITY_WITH_ORIGINAL: &[&str] =
    &["浓缩树脂", "须臾树脂", "原粹树脂", "脆弱树脂"];
pub const AUTO_LEY_LINE_REWARD_RESIN_PRIORITY_EMPTY_ORIGINAL: &[&str] =
    &["浓缩树脂", "须臾树脂", "脆弱树脂"];

pub const AUTO_LEY_LINE_OPEN_MARKS_ASSET: &str = "AutoLeyLineOutcrop:icon/open.png";
pub const AUTO_LEY_LINE_CLOSE_MARKS_ASSET: &str = "AutoLeyLineOutcrop:icon/close.png";
pub const AUTO_LEY_LINE_PAIMON_MENU_ASSET: &str = "AutoLeyLineOutcrop:icon/paimon_menu.png";
pub const AUTO_LEY_LINE_REWARD_BOX_ASSET: &str = "AutoLeyLineOutcrop:icon/box.png";
pub const AUTO_LEY_LINE_MAP_SETTING_ASSET: &str = "AutoLeyLineOutcrop:icon/map_setting_button.bmp";
pub const AUTO_LEY_LINE_HANDBOOK_TRACK_ACTION_ASSET: &str =
    "AutoLeyLineOutcrop:icon/handbook_track_action_left.png";
pub const AUTO_LEY_LINE_REVELATION_ASSET: &str =
    "AutoLeyLineOutcrop:icon/Blossom_of_Revelation.png";
pub const AUTO_LEY_LINE_WEALTH_ASSET: &str = "AutoLeyLineOutcrop:icon/Blossom_of_Wealth.png";
pub const AUTO_LEY_LINE_REWARD_SWITCH_ASSET: &str = "AutoLeyLineOutcrop:icon/switch_button.png";
pub const AUTO_LEY_LINE_REPLENISH_RESIN_ASSET: &str =
    "AutoLeyLineOutcrop:icon/replenish_resin_button.png";
pub const AUTO_LEY_LINE_ORIGINAL_RESIN_ASSET: &str =
    "AutoLeyLineOutcrop:1920x1080/original_resin.png";
pub const AUTO_LEY_LINE_CONDENSED_RESIN_ASSET: &str =
    "AutoLeyLineOutcrop:1920x1080/condensed_resin.png";
pub const AUTO_LEY_LINE_TRANSIENT_RESIN_ASSET: &str =
    "AutoLeyLineOutcrop:1920x1080/transient_resin.png";
pub const AUTO_LEY_LINE_FRAGILE_RESIN_ASSET: &str =
    "AutoLeyLineOutcrop:1920x1080/fragile_resin.png";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoLeyLineOutcropExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub asset_scale: f64,
    pub param: AutoLeyLineOutcropParam,
    pub data_rule: AutoLeyLineDataRule,
    pub validation_rule: AutoLeyLineValidationRule,
    pub startup_rule: AutoLeyLineStartupRule,
    pub discovery_rule: AutoLeyLineDiscoveryRule,
    pub pathing_rule: AutoLeyLinePathingRule,
    pub combat_rule: AutoLeyLineCombatRule,
    pub reward_navigation_rule: AutoLeyLineRewardNavigationRule,
    pub reward_rule: AutoLeyLineRewardRule,
    pub resin_rule: AutoLeyLineResinRule,
    pub locators: AutoLeyLineLocators,
    pub steps: Vec<AutoLeyLineTaskStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoLeyLineOutcropExecutionConfig {
    pub capture_size: Size,
    pub asset_scale: f64,
    pub param: AutoLeyLineOutcropParam,
    pub auto_ley_line_outcrop_config: AutoLeyLineOutcropConfig,
}

impl Default for AutoLeyLineOutcropExecutionConfig {
    fn default() -> Self {
        let auto_ley_line_outcrop_config = AutoLeyLineOutcropConfig::default();
        let mut param = AutoLeyLineOutcropParam::default();
        apply_auto_ley_line_config(&mut param, &auto_ley_line_outcrop_config);
        Self {
            capture_size: Size::new(
                AUTO_LEY_LINE_OUTCROP_DEFAULT_CAPTURE_WIDTH,
                AUTO_LEY_LINE_OUTCROP_DEFAULT_CAPTURE_HEIGHT,
            ),
            asset_scale: 1.0,
            param,
            auto_ley_line_outcrop_config,
        }
    }
}

impl AutoLeyLineOutcropExecutionConfig {
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

        let auto_ley_line_value = value
            .get("autoLeyLineOutcropConfig")
            .or_else(|| value.get("AutoLeyLineOutcropConfig"))
            .or_else(|| value.get("auto_ley_line_outcrop_config"))
            .unwrap_or(value);
        config.auto_ley_line_outcrop_config =
            serde_json::from_value(auto_ley_line_value.clone()).unwrap_or_default();

        let mut param = AutoLeyLineOutcropParam::default();
        apply_auto_ley_line_config(&mut param, &config.auto_ley_line_outcrop_config);
        overlay_auto_ley_line_param_members(&mut param, value);
        if let Some(param_value) = value
            .get("param")
            .or_else(|| value.get("Param"))
            .or_else(|| value.get("autoLeyLineOutcropParam"))
            .or_else(|| value.get("AutoLeyLineOutcropParam"))
            .or_else(|| value.get("auto_ley_line_outcrop_param"))
        {
            overlay_auto_ley_line_param_members(&mut param, param_value);
        }
        normalize_auto_ley_line_param(&mut param);
        config.param = param;
        config
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoLeyLineDataRule {
    pub task_directory: String,
    pub config_json: String,
    pub node_json: String,
    pub error_threshold: f64,
    pub supported_countries: Vec<String>,
    pub map_position_count: usize,
    pub ley_line_position_count: usize,
    pub teleport_count: usize,
    pub blossom_count: usize,
    pub edge_count: usize,
    pub node_index_groups: Vec<String>,
    pub selected_country_supported: bool,
    pub selected_country_map_positions: usize,
    pub selected_country_ley_line_positions: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoLeyLineValidationRule {
    pub valid_ley_line_types: Vec<String>,
    pub selected_type_valid: bool,
    pub requires_non_empty_country: bool,
    pub selected_country_supported_by_config: bool,
    pub normalized_count: i32,
    pub normalized_timeout_seconds: i32,
    pub friendship_team_requires_combat_team: bool,
    pub combat_strategy_path: Option<String>,
    pub combat_strategy_file_required: bool,
    pub static_config_files_required: Vec<String>,
    pub requires_16_to_9_resolution: bool,
    pub warns_below_1920x1080: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoLeyLineStartupRule {
    pub enables_mask_overlay_during_task: bool,
    pub restores_mask_overlay_on_finish: bool,
    pub ensures_exit_reward_page_before_start: bool,
    pub returns_main_ui_before_start: bool,
    pub teleports_to_statue_unless_one_dragon_mode: bool,
    pub switches_combat_team_when_configured: bool,
    pub use_adventurer_handbook_flag_means_manual_big_map_search: bool,
    pub closes_custom_marks_when_manual_big_map_search: bool,
    pub reopens_custom_marks_on_finish_if_closed: bool,
    pub registers_auto_pick_trigger: bool,
    pub sends_notification_when_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoLeyLineDiscoveryRule {
    pub handbook_flow_when_use_adventurer_handbook_false: AutoLeyLineHandbookRule,
    pub manual_map_flow_when_use_adventurer_handbook_true: AutoLeyLineManualMapRule,
    pub selected_manual_flow: bool,
    pub selected_country: String,
    pub selected_type: String,
    pub selected_blossom_asset: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoLeyLineHandbookRule {
    pub open_handbook_action: String,
    pub click_sequence_1080p: Vec<AutoLeyLineClickPoint>,
    pub revelation_type_click: AutoLeyLineClickPoint,
    pub wealth_type_click: AutoLeyLineClickPoint,
    pub country_ocr_special_case_nod_krai: String,
    pub track_action_template_attempts: u64,
    pub fallback_track_button_click: AutoLeyLineClickPoint,
    pub cancel_tracking_clicks_map_center_first: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoLeyLineManualMapRule {
    pub moves_big_map_to_country_scan_positions: bool,
    pub adjusts_zoom_to: f64,
    pub blossom_icon_center_offset_pixels: i32,
    pub coordinate_formula: String,
    pub map_positions_for_selected_country: Vec<AutoLeyLineMapPosition>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoLeyLinePathingRule {
    pub selected_position_plan: Option<AutoLeyLineSelectedPositionPlan>,
    pub uses_bfs_from_teleport_nodes: bool,
    pub uses_reverse_two_hop_fallback_when_no_forward_path: bool,
    pub branch_route_uses_nearest_detected_ley_line_node: bool,
    pub pathing_party_skip_party_switch: bool,
    pub target_route_derivation: String,
    pub rerun_route_derivation: String,
    pub process_max_retries: u64,
    pub max_consecutive_fight_failures: u64,
    pub required_route_files: Vec<String>,
    pub missing_route_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoLeyLineSelectedPositionPlan {
    pub strategy: String,
    pub order: i32,
    pub steps: i32,
    pub ley_line_position: AutoLeyLineNodePosition,
    pub start_node_id: i32,
    pub start_region: String,
    pub target_node_id: i32,
    pub target_region: String,
    pub from_teleport_start: bool,
    pub route_count: usize,
    pub routes: Vec<String>,
    pub target_route: String,
    pub rerun_route: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoLeyLineCombatRule {
    pub timeout_seconds: i32,
    pub strategy_name: String,
    pub strategy_path: Option<String>,
    pub blank_strategy_copies_global_auto_fight_config_at_runtime: bool,
    pub auto_fight_runs_without_finish_detect: bool,
    pub finish_detect_disabled_for_auto_fight_task: bool,
    pub ocr_finish_success_keywords: Vec<String>,
    pub ocr_finish_failure_keywords: Vec<String>,
    pub fight_text_keywords: Vec<String>,
    pub no_fight_text_count_before_failure: u64,
    pub poll_interval_ms: u64,
    pub seek_enemy_enabled: bool,
    pub seek_enemy_initial_delay_seconds: u64,
    pub seek_enemy_interval_seconds: i32,
    pub seek_enemy_rotary_factor: i32,
    pub resurrect_prompt_keyword: String,
    pub post_fight_collect_rule: AutoLeyLinePostFightCollectRule,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoLeyLinePostFightCollectRule {
    pub kazuha_pickup_enabled: bool,
    pub kazuha_hold_elemental_skill_ms: u64,
    pub kazuha_post_skill_wait_ms: u64,
    pub qin_double_pickup_enabled: bool,
    pub disables_auto_fight_builtin_pickup: bool,
    pub only_pick_elite_drops_mode: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoLeyLineRewardNavigationRule {
    pub max_retry: u64,
    pub navigation_timeout_ms: u64,
    pub middle_click_resets_camera: bool,
    pub reward_box_locator: AutoLeyLineTemplateLocator,
    pub screen_center: AutoLeyLineClickPoint,
    pub align_max_angle_degrees: f64,
    pub camera_move_x_clamp: i32,
    pub camera_move_down_when_icon_below_center: i32,
    pub forward_burst_ms: u64,
    pub recovery_key: String,
    pub backward_recovery_ms: u64,
    pub detects_reward_by_bv_keywords: Vec<String>,
    pub detects_reward_by_ocr_keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoLeyLineRewardRule {
    pub interact_action: String,
    pub interact_wait_ms: u64,
    pub verify_reward_prompt_before_resin: bool,
    pub reward_prompt_title_roi: AutoLeyLineRelativeRect,
    pub reward_prompt_content_roi: AutoLeyLineRelativeRect,
    pub title_keywords: Vec<String>,
    pub content_keywords: Vec<String>,
    pub action_keywords: Vec<String>,
    pub activation_clicks_title_before_use: bool,
    pub switch_double_reward_20_to_40: bool,
    pub reward_retry_count: u64,
    pub reward_use_attempts: u64,
    pub switches_back_to_combat_team_after_reward: bool,
    pub scan_drops_after_reward_enabled: bool,
    pub scan_drops_after_reward_seconds: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoLeyLineResinRule {
    pub resin_exhaustion_mode: bool,
    pub open_mode_count_min: bool,
    pub original_resin_cost: i32,
    pub half_original_resin_cost: i32,
    pub condensed_resin_counts_as_one_run: bool,
    pub transient_resin_enabled: bool,
    pub fragile_resin_enabled: bool,
    pub recheck_after_run_when_resin_exhaustion_mode: bool,
    pub max_recheck_count: u64,
    pub recheck_ignores_counts_above: i32,
    pub reward_priority_with_original_resin: Vec<String>,
    pub reward_priority_when_original_resin_empty: Vec<String>,
    pub double_reward_prefers_original_resin: bool,
    pub synthesizer_flag_configured: bool,
    pub synthesizer_flow_invoked_by_legacy_task: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoLeyLineLocators {
    pub open_custom_marks: AutoLeyLineTemplateLocator,
    pub close_custom_marks: AutoLeyLineTemplateLocator,
    pub paimon_menu: AutoLeyLineTemplateLocator,
    pub reward_box: AutoLeyLineTemplateLocator,
    pub map_setting_button: AutoLeyLineTemplateLocator,
    pub handbook_track_action: AutoLeyLineTemplateLocator,
    pub revelation_blossom: AutoLeyLineTemplateLocator,
    pub wealth_blossom: AutoLeyLineTemplateLocator,
    pub reward_switch_20_to_40: AutoLeyLineTemplateLocator,
    pub replenish_resin_button: AutoLeyLineTemplateLocator,
    pub original_resin: AutoLeyLineTemplateLocator,
    pub condensed_resin: AutoLeyLineTemplateLocator,
    pub transient_resin: AutoLeyLineTemplateLocator,
    pub fragile_resin: AutoLeyLineTemplateLocator,
    pub ocr_regions: AutoLeyLineOcrRegions,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoLeyLineTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Option<Rect>,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoLeyLineOcrRegions {
    pub fight_result: Rect,
    pub left_flow: Rect,
    pub right_flow: Rect,
    pub overlay_render_lead_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoLeyLineClickPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct AutoLeyLineRelativeRect {
    pub x_ratio: f64,
    pub y_ratio: f64,
    pub width_ratio: f64,
    pub height_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoLeyLineMapPosition {
    pub x: f64,
    pub y: f64,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AutoLeyLineNodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoLeyLineTaskStep {
    pub phase: AutoLeyLineTaskPhase,
    pub action: AutoLeyLineTaskAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoLeyLineTaskPhase {
    Startup,
    Resin,
    Discovery,
    Pathing,
    Combat,
    Reward,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoLeyLineTaskAction {
    ValidateConfigAndLoadStaticData,
    CountResinWhenExhaustionMode,
    PrepareMainUiTeamAndMarks,
    FindLeyLineByHandbookOrMap,
    MatchStaticStrategyAndRunPathing,
    ProcessLeyLineFightAndRecovery,
    NavigateToRewardFlower,
    ClaimRewardWithResinPriority,
    ScanDropsAfterRewardWhenEnabled,
    RestoreMarksOverlayAndInputs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoLeyLineOutcropExecutionStatus {
    Completed,
    StartupFailed,
    NoLeyLineFound,
    NavigationPreparationFailed,
    PathingFailed,
    FightFailedStopped,
    MaxConsecutiveFightFailures,
    ResinExhausted,
    RewardNavigationFailed,
    RewardSkipped,
    RuntimeError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoLeyLineOutcropRuntimeActionKind {
    PrepareStartup,
    CountResin,
    DiscoverLeyLine,
    PrepareNavigation,
    RunPathing,
    ExecuteFight,
    NavigateRewardFlower,
    ClaimReward,
    ScanDropsAfterReward,
    SwitchNextLeyLine,
    Cleanup,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoLeyLinePathingRunKind {
    Initial,
    Rerun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoLeyLineResinKind {
    Condensed,
    Transient,
    Original,
    Fragile,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLineResinInventory {
    pub original_resin: i32,
    pub condensed_resin: i32,
    pub transient_resin: i32,
    pub fragile_resin: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLineRewardClaimPlan {
    pub resin_kind: AutoLeyLineResinKind,
    pub priority_label: String,
    pub expected_original_resin_cost: i32,
    pub uses_double_reward: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLinePathingExecutionRequest {
    pub run_index: u32,
    pub run_kind: AutoLeyLinePathingRunKind,
    pub routes: Vec<String>,
    pub selected_start_node_id: Option<i32>,
    pub selected_target_node_id: Option<i32>,
    pub selected_strategy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLineStartupOutcome {
    pub prepared: bool,
    pub returned_main_ui: bool,
    pub combat_team_switched: bool,
    pub custom_marks_closed: bool,
    pub mask_overlay_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoLeyLineRuntimeDiscoveryOutcome {
    pub found: bool,
    pub position: Option<AutoLeyLineNodePosition>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLineNavigationPreparationOutcome {
    pub prepared: bool,
    pub teleported: bool,
    pub route_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLinePathingOutcome {
    pub completed: bool,
    pub executed_routes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoLeyLineFightOutcome {
    Succeeded { duration_ms: u64 },
    Failed { reason: String, stop: bool },
    TimedOut { timeout_seconds: i32, stop: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLineRewardNavigationOutcome {
    pub reached: bool,
    pub attempts: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoLeyLineRewardSkipReason {
    InsufficientResin,
    ClaimDisabled,
    RewardPromptMissing,
    RuntimeSkipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoLeyLineRewardClaimOutcome {
    Claimed {
        resin_kind: AutoLeyLineResinKind,
        consumed_original_resin: i32,
    },
    Skipped {
        reason: AutoLeyLineRewardSkipReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLineDropScanOutcome {
    pub enabled: bool,
    pub scanned: bool,
    pub picked_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLineSwitchNextOutcome {
    pub switched: bool,
    pub exhausted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLineCleanupOutcome {
    pub completed: bool,
    pub returned_main_ui: bool,
    pub custom_marks_restored: bool,
    pub mask_overlay_restored: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoLeyLineSkipReason {
    NoSelectedPath,
    ResinUnavailable(AutoLeyLineRewardSkipReason),
    RewardClaimSkipped(AutoLeyLineRewardSkipReason),
    FightFailed { reason: String },
    DropScanDisabled,
    NextLeyLineUnavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoLeyLineOutcropRuntimeActionOutcome {
    Startup(AutoLeyLineStartupOutcome),
    Resin(AutoLeyLineResinInventory),
    Discovery(AutoLeyLineRuntimeDiscoveryOutcome),
    NavigationPreparation(AutoLeyLineNavigationPreparationOutcome),
    Pathing(AutoLeyLinePathingOutcome),
    Fight(AutoLeyLineFightOutcome),
    RewardNavigation(AutoLeyLineRewardNavigationOutcome),
    RewardClaim(AutoLeyLineRewardClaimOutcome),
    DropScan(AutoLeyLineDropScanOutcome),
    SwitchNext(AutoLeyLineSwitchNextOutcome),
    Cleanup(AutoLeyLineCleanupOutcome),
    Skipped(AutoLeyLineSkipReason),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoLeyLineOutcropRuntimeActionReport {
    pub action_kind: AutoLeyLineOutcropRuntimeActionKind,
    pub outcome: AutoLeyLineOutcropRuntimeActionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLineOutcropSkippedStep {
    pub action_kind: AutoLeyLineOutcropRuntimeActionKind,
    pub reason: AutoLeyLineSkipReason,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AutoLeyLineOutcropExecutorState {
    pub target_runs: u32,
    pub attempted_runs: u32,
    pub fights_completed: u32,
    pub fight_failures: u32,
    pub consecutive_fight_failures: u32,
    pub reward_claims_completed: u32,
    pub reward_skipped_count: u32,
    pub resin_checks: u32,
    pub next_ley_line_switches: u32,
    pub drop_scans: u32,
    pub startup_prepared: bool,
    pub cleanup_completed: bool,
    pub last_routes: Vec<String>,
    pub last_resin_inventory: Option<AutoLeyLineResinInventory>,
    pub last_reward_claim_plan: Option<AutoLeyLineRewardClaimPlan>,
    pub last_reward_claim_outcome: Option<AutoLeyLineRewardClaimOutcome>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoLeyLineOutcropExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub status: AutoLeyLineOutcropExecutionStatus,
    pub state: AutoLeyLineOutcropExecutorState,
    pub executed_actions: Vec<AutoLeyLineOutcropRuntimeActionReport>,
    pub skipped_steps: Vec<AutoLeyLineOutcropSkippedStep>,
}

pub trait AutoLeyLineOutcropRuntime {
    fn prepare_auto_ley_line_outcrop(
        &mut self,
        plan: &AutoLeyLineOutcropExecutionPlan,
    ) -> Result<AutoLeyLineStartupOutcome>;

    fn count_auto_ley_line_outcrop_resin(
        &mut self,
        plan: &AutoLeyLineOutcropExecutionPlan,
        run_index: u32,
    ) -> Result<AutoLeyLineResinInventory>;

    fn discover_auto_ley_line_outcrop(
        &mut self,
        plan: &AutoLeyLineOutcropExecutionPlan,
        run_index: u32,
    ) -> Result<AutoLeyLineRuntimeDiscoveryOutcome>;

    fn prepare_auto_ley_line_outcrop_navigation(
        &mut self,
        plan: &AutoLeyLineOutcropExecutionPlan,
        request: &AutoLeyLinePathingExecutionRequest,
    ) -> Result<AutoLeyLineNavigationPreparationOutcome>;

    fn execute_auto_ley_line_outcrop_pathing(
        &mut self,
        plan: &AutoLeyLineOutcropExecutionPlan,
        request: &AutoLeyLinePathingExecutionRequest,
    ) -> Result<AutoLeyLinePathingOutcome>;

    fn execute_auto_ley_line_outcrop_fight(
        &mut self,
        plan: &AutoLeyLineOutcropExecutionPlan,
        run_index: u32,
    ) -> Result<AutoLeyLineFightOutcome>;

    fn navigate_auto_ley_line_outcrop_reward(
        &mut self,
        plan: &AutoLeyLineOutcropExecutionPlan,
        run_index: u32,
    ) -> Result<AutoLeyLineRewardNavigationOutcome>;

    fn claim_auto_ley_line_outcrop_reward(
        &mut self,
        plan: &AutoLeyLineOutcropExecutionPlan,
        claim_plan: &AutoLeyLineRewardClaimPlan,
    ) -> Result<AutoLeyLineRewardClaimOutcome>;

    fn scan_auto_ley_line_outcrop_drops(
        &mut self,
        plan: &AutoLeyLineOutcropExecutionPlan,
        run_index: u32,
    ) -> Result<AutoLeyLineDropScanOutcome>;

    fn switch_auto_ley_line_outcrop_next(
        &mut self,
        plan: &AutoLeyLineOutcropExecutionPlan,
        completed_runs: u32,
    ) -> Result<AutoLeyLineSwitchNextOutcome>;

    fn cleanup_auto_ley_line_outcrop(
        &mut self,
        plan: &AutoLeyLineOutcropExecutionPlan,
        status: AutoLeyLineOutcropExecutionStatus,
    ) -> Result<AutoLeyLineCleanupOutcome>;
}

pub fn plan_auto_ley_line_outcrop(
    working_directory: impl AsRef<Path>,
    config: AutoLeyLineOutcropExecutionConfig,
) -> Result<AutoLeyLineOutcropExecutionPlan> {
    let working_directory = working_directory.as_ref();
    let mut param = config.param;
    normalize_auto_ley_line_param(&mut param);
    validate_auto_ley_line_param(&param)?;
    let strategy_path = validate_auto_ley_line_strategy(working_directory, &param)?;
    let static_data = load_ley_line_static_data(working_directory)?;
    let selected_position_plan = selected_position_plan(&static_data, &param);
    let (required_route_files, missing_route_files) =
        route_file_status(&static_data.task_directory, selected_position_plan.as_ref());
    let selected_country = param.country.clone();
    let selected_type = param.ley_line_outcrop_type.clone();

    Ok(AutoLeyLineOutcropExecutionPlan {
        task_key: AUTO_LEY_LINE_OUTCROP_TASK_KEY.to_string(),
        display_name: AUTO_LEY_LINE_OUTCROP_DISPLAY_NAME.to_string(),
        capture_size: config.capture_size,
        asset_scale: config.asset_scale,
        data_rule: AutoLeyLineDataRule {
            task_directory: static_data.task_directory.display().to_string(),
            config_json: AUTO_LEY_LINE_OUTCROP_CONFIG_JSON.to_string(),
            node_json: AUTO_LEY_LINE_OUTCROP_NODE_JSON.to_string(),
            error_threshold: static_data.config.error_threshold,
            supported_countries: static_data.supported_countries(),
            map_position_count: static_data.map_position_count(),
            ley_line_position_count: static_data.ley_line_position_count(),
            teleport_count: static_data.raw_nodes.teleports.len(),
            blossom_count: static_data.raw_nodes.blossoms.len(),
            edge_count: static_data.raw_nodes.edges.len(),
            node_index_groups: static_data.node_index_groups(),
            selected_country_supported: static_data
                .config
                .ley_line_positions
                .contains_key(&selected_country),
            selected_country_map_positions: static_data
                .config
                .map_positions
                .get(&selected_country)
                .map_or(0, Vec::len),
            selected_country_ley_line_positions: static_data
                .config
                .ley_line_positions
                .get(&selected_country)
                .map_or(0, Vec::len),
        },
        validation_rule: AutoLeyLineValidationRule {
            valid_ley_line_types: AUTO_LEY_LINE_VALID_TYPES
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            selected_type_valid: AUTO_LEY_LINE_VALID_TYPES.contains(&selected_type.as_str()),
            requires_non_empty_country: true,
            selected_country_supported_by_config: static_data
                .config
                .ley_line_positions
                .contains_key(&selected_country),
            normalized_count: param.count,
            normalized_timeout_seconds: param.timeout,
            friendship_team_requires_combat_team: !param.friendship_team.trim().is_empty(),
            combat_strategy_path: strategy_path.clone(),
            combat_strategy_file_required: strategy_path.is_some(),
            static_config_files_required: vec![
                AUTO_LEY_LINE_OUTCROP_CONFIG_JSON.to_string(),
                AUTO_LEY_LINE_OUTCROP_NODE_JSON.to_string(),
            ],
            requires_16_to_9_resolution: true,
            warns_below_1920x1080: true,
        },
        startup_rule: AutoLeyLineStartupRule {
            enables_mask_overlay_during_task: true,
            restores_mask_overlay_on_finish: true,
            ensures_exit_reward_page_before_start: true,
            returns_main_ui_before_start: true,
            teleports_to_statue_unless_one_dragon_mode: true,
            switches_combat_team_when_configured: !param.team.trim().is_empty(),
            use_adventurer_handbook_flag_means_manual_big_map_search: true,
            closes_custom_marks_when_manual_big_map_search: param.use_adventurer_handbook,
            reopens_custom_marks_on_finish_if_closed: true,
            registers_auto_pick_trigger: true,
            sends_notification_when_enabled: param.is_notification,
        },
        discovery_rule: AutoLeyLineDiscoveryRule {
            handbook_flow_when_use_adventurer_handbook_false: handbook_rule(),
            manual_map_flow_when_use_adventurer_handbook_true: manual_map_rule(
                &static_data,
                &selected_country,
            ),
            selected_manual_flow: param.use_adventurer_handbook,
            selected_country,
            selected_type: selected_type.clone(),
            selected_blossom_asset: if selected_type == "启示之花" {
                AUTO_LEY_LINE_REVELATION_ASSET.to_string()
            } else {
                AUTO_LEY_LINE_WEALTH_ASSET.to_string()
            },
        },
        pathing_rule: AutoLeyLinePathingRule {
            selected_position_plan,
            uses_bfs_from_teleport_nodes: true,
            uses_reverse_two_hop_fallback_when_no_forward_path: true,
            branch_route_uses_nearest_detected_ley_line_node: true,
            pathing_party_skip_party_switch: true,
            target_route_derivation: "replace assets/pathing/ with assets/pathing/target/ and strip -rerun".to_string(),
            rerun_route_derivation: "replace pathing root with assets/pathing/rerun/ and append -rerun before .json when missing".to_string(),
            process_max_retries: AUTO_LEY_LINE_PROCESS_MAX_RETRIES,
            max_consecutive_fight_failures: AUTO_LEY_LINE_MAX_CONSECUTIVE_FAILURES,
            required_route_files,
            missing_route_files,
        },
        combat_rule: AutoLeyLineCombatRule {
            timeout_seconds: param.timeout,
            strategy_name: param.fight_config.strategy_name.clone(),
            strategy_path,
            blank_strategy_copies_global_auto_fight_config_at_runtime: param
                .fight_config
                .strategy_name
                .trim()
                .is_empty(),
            auto_fight_runs_without_finish_detect: true,
            finish_detect_disabled_for_auto_fight_task: true,
            ocr_finish_success_keywords: strings(AUTO_LEY_LINE_SUCCESS_KEYWORDS),
            ocr_finish_failure_keywords: strings(AUTO_LEY_LINE_FAILURE_KEYWORDS),
            fight_text_keywords: strings(AUTO_LEY_LINE_FIGHT_KEYWORDS),
            no_fight_text_count_before_failure: 10,
            poll_interval_ms: 1_000,
            seek_enemy_enabled: param.fight_config.seek_enemy_enabled,
            seek_enemy_initial_delay_seconds: AUTO_LEY_LINE_FIGHT_SEEK_INITIAL_DELAY_SECONDS,
            seek_enemy_interval_seconds: param.fight_config.seek_enemy_interval_seconds.clamp(1, 60),
            seek_enemy_rotary_factor: param.fight_config.seek_enemy_rotary_factor.clamp(1, 13),
            resurrect_prompt_keyword: "复苏".to_string(),
            post_fight_collect_rule: AutoLeyLinePostFightCollectRule {
                kazuha_pickup_enabled: param.fight_config.kazuha_pickup_enabled,
                kazuha_hold_elemental_skill_ms: 1_000,
                kazuha_post_skill_wait_ms: AUTO_LEY_LINE_KAZUHA_PICKUP_POST_SKILL_WAIT_MS,
                qin_double_pickup_enabled: param.fight_config.qin_double_pick_up,
                disables_auto_fight_builtin_pickup: true,
                only_pick_elite_drops_mode: "DisableAutoPickupForNonElite".to_string(),
            },
        },
        reward_navigation_rule: reward_navigation_rule(),
        reward_rule: AutoLeyLineRewardRule {
            interact_action: "PickUpOrInteract".to_string(),
            interact_wait_ms: 800,
            verify_reward_prompt_before_resin: true,
            reward_prompt_title_roi: AutoLeyLineRelativeRect {
                x_ratio: 0.25,
                y_ratio: 0.15,
                width_ratio: 0.5,
                height_ratio: 0.25,
            },
            reward_prompt_content_roi: AutoLeyLineRelativeRect {
                x_ratio: 0.25,
                y_ratio: 0.2,
                width_ratio: 0.5,
                height_ratio: 0.6,
            },
            title_keywords: strings(&["激活地脉之花", "选择激活方式", "地脉之花"]),
            content_keywords: strings(&[
                "原粹树脂",
                "浓缩树脂",
                "须臾树脂",
                "脆弱树脂",
                "激活地脉之花",
                "选择激活方式",
                "补充",
            ]),
            action_keywords: strings(&["使用"]),
            activation_clicks_title_before_use: true,
            switch_double_reward_20_to_40: true,
            reward_retry_count: AUTO_LEY_LINE_REWARD_MAX_RETRIES,
            reward_use_attempts: AUTO_LEY_LINE_REWARD_USE_ATTEMPTS,
            switches_back_to_combat_team_after_reward: !param.friendship_team.trim().is_empty()
                && !param.team.trim().is_empty(),
            scan_drops_after_reward_enabled: param.scan_drops_after_reward_enabled,
            scan_drops_after_reward_seconds: param
                .scan_drops_after_reward_seconds
                .clamp(0, AUTO_LEY_LINE_MAX_SCAN_DROPS_AFTER_REWARD_SECONDS),
        },
        resin_rule: AutoLeyLineResinRule {
            resin_exhaustion_mode: param.is_resin_exhaustion_mode,
            open_mode_count_min: param.open_mode_count_min,
            original_resin_cost: AUTO_LEY_LINE_ORIGINAL_RESIN_COST,
            half_original_resin_cost: AUTO_LEY_LINE_HALF_ORIGINAL_RESIN_COST,
            condensed_resin_counts_as_one_run: true,
            transient_resin_enabled: param.use_transient_resin,
            fragile_resin_enabled: param.use_fragile_resin,
            recheck_after_run_when_resin_exhaustion_mode: param.is_resin_exhaustion_mode,
            max_recheck_count: AUTO_LEY_LINE_MAX_RECHECK_COUNT,
            recheck_ignores_counts_above: 50,
            reward_priority_with_original_resin: strings(AUTO_LEY_LINE_REWARD_RESIN_PRIORITY_WITH_ORIGINAL),
            reward_priority_when_original_resin_empty: strings(
                AUTO_LEY_LINE_REWARD_RESIN_PRIORITY_EMPTY_ORIGINAL,
            ),
            double_reward_prefers_original_resin: true,
            synthesizer_flag_configured: param.is_go_to_synthesizer,
            synthesizer_flow_invoked_by_legacy_task: false,
        },
        locators: ley_line_locators(),
        steps: ley_line_steps(),
        param,
        executor_ready: true,
        pending_native: vec![
            "executor-ready Rust orchestration is available behind AutoLeyLineOutcropRuntime; desktop live adapters are not wired yet".to_string(),
            "desktop live capture/OCR/template/input adapters still need wiring for reward, resin, custom-mark, overlay, notification, and cleanup flows".to_string(),
            "full TpTask/PathExecutor pathing adapters and AutoFightTask/CombatScenes fight adapters remain pending native integration".to_string(),
        ],
    })
}

pub fn execute_auto_ley_line_outcrop_plan<R>(
    plan: &AutoLeyLineOutcropExecutionPlan,
    runtime: &mut R,
) -> Result<AutoLeyLineOutcropExecutionReport>
where
    R: AutoLeyLineOutcropRuntime,
{
    let mut state = AutoLeyLineOutcropExecutorState {
        target_runs: u32::try_from(plan.param.count.max(1)).unwrap_or(u32::MAX),
        ..AutoLeyLineOutcropExecutorState::default()
    };
    let mut executed_actions = Vec::new();
    let mut skipped_steps = Vec::new();

    let execution_result = execute_auto_ley_line_outcrop_steps(
        plan,
        runtime,
        &mut state,
        &mut executed_actions,
        &mut skipped_steps,
    );
    let status = match execution_result {
        Ok(status) => status,
        Err(error) => {
            let _ = execute_auto_ley_line_outcrop_cleanup(
                plan,
                runtime,
                AutoLeyLineOutcropExecutionStatus::RuntimeError,
                &mut state,
                &mut executed_actions,
            );
            return Err(error);
        }
    };

    execute_auto_ley_line_outcrop_cleanup(
        plan,
        runtime,
        status,
        &mut state,
        &mut executed_actions,
    )?;

    Ok(auto_ley_line_execution_report(
        plan,
        status,
        state,
        executed_actions,
        skipped_steps,
    ))
}

pub fn select_auto_ley_line_reward_claim_plan(
    plan: &AutoLeyLineOutcropExecutionPlan,
    inventory: &AutoLeyLineResinInventory,
) -> std::result::Result<AutoLeyLineRewardClaimPlan, AutoLeyLineRewardSkipReason> {
    let priority = if inventory.original_resin > 0 {
        &plan.resin_rule.reward_priority_with_original_resin
    } else {
        &plan.resin_rule.reward_priority_when_original_resin_empty
    };

    for label in priority {
        match label.as_str() {
            "浓缩树脂" if inventory.condensed_resin > 0 => {
                return Ok(auto_ley_line_claim_plan(
                    AutoLeyLineResinKind::Condensed,
                    label,
                    0,
                    true,
                ));
            }
            "须臾树脂"
                if plan.resin_rule.transient_resin_enabled && inventory.transient_resin > 0 =>
            {
                return Ok(auto_ley_line_claim_plan(
                    AutoLeyLineResinKind::Transient,
                    label,
                    0,
                    false,
                ));
            }
            "原粹树脂" => {
                let original_cost = if plan.reward_rule.switch_double_reward_20_to_40
                    && inventory.original_resin >= plan.resin_rule.original_resin_cost
                {
                    plan.resin_rule.original_resin_cost
                } else if inventory.original_resin >= plan.resin_rule.half_original_resin_cost {
                    plan.resin_rule.half_original_resin_cost
                } else {
                    continue;
                };
                return Ok(auto_ley_line_claim_plan(
                    AutoLeyLineResinKind::Original,
                    label,
                    original_cost,
                    original_cost == plan.resin_rule.original_resin_cost,
                ));
            }
            "脆弱树脂" if plan.resin_rule.fragile_resin_enabled && inventory.fragile_resin > 0 =>
            {
                return Ok(auto_ley_line_claim_plan(
                    AutoLeyLineResinKind::Fragile,
                    label,
                    0,
                    false,
                ));
            }
            _ => {}
        }
    }

    Err(AutoLeyLineRewardSkipReason::InsufficientResin)
}

fn execute_auto_ley_line_outcrop_steps<R>(
    plan: &AutoLeyLineOutcropExecutionPlan,
    runtime: &mut R,
    state: &mut AutoLeyLineOutcropExecutorState,
    executed_actions: &mut Vec<AutoLeyLineOutcropRuntimeActionReport>,
    skipped_steps: &mut Vec<AutoLeyLineOutcropSkippedStep>,
) -> Result<AutoLeyLineOutcropExecutionStatus>
where
    R: AutoLeyLineOutcropRuntime,
{
    let startup = runtime.prepare_auto_ley_line_outcrop(plan)?;
    state.startup_prepared = startup.prepared;
    executed_actions.push(auto_ley_line_action_report(
        AutoLeyLineOutcropRuntimeActionKind::PrepareStartup,
        AutoLeyLineOutcropRuntimeActionOutcome::Startup(startup.clone()),
    ));
    if !startup.prepared {
        return Ok(AutoLeyLineOutcropExecutionStatus::StartupFailed);
    }

    while state.reward_claims_completed < state.target_runs {
        let run_index = state.attempted_runs;
        state.attempted_runs += 1;

        let mut reward_claim_plan = None;
        if plan.resin_rule.resin_exhaustion_mode {
            let inventory = execute_auto_ley_line_resin_count(
                plan,
                runtime,
                run_index,
                state,
                executed_actions,
            )?;
            match select_auto_ley_line_reward_claim_plan(plan, &inventory) {
                Ok(plan) => reward_claim_plan = Some(plan),
                Err(reason) => {
                    state.reward_skipped_count += 1;
                    auto_ley_line_skip(
                        skipped_steps,
                        executed_actions,
                        AutoLeyLineOutcropRuntimeActionKind::ClaimReward,
                        AutoLeyLineSkipReason::ResinUnavailable(reason),
                    );
                    return Ok(AutoLeyLineOutcropExecutionStatus::ResinExhausted);
                }
            }
        }

        let discovery = runtime.discover_auto_ley_line_outcrop(plan, run_index)?;
        executed_actions.push(auto_ley_line_action_report(
            AutoLeyLineOutcropRuntimeActionKind::DiscoverLeyLine,
            AutoLeyLineOutcropRuntimeActionOutcome::Discovery(discovery.clone()),
        ));
        if !discovery.found {
            return Ok(AutoLeyLineOutcropExecutionStatus::NoLeyLineFound);
        }

        let Some(pathing_request) = auto_ley_line_pathing_request(plan, run_index) else {
            auto_ley_line_skip(
                skipped_steps,
                executed_actions,
                AutoLeyLineOutcropRuntimeActionKind::RunPathing,
                AutoLeyLineSkipReason::NoSelectedPath,
            );
            return Ok(AutoLeyLineOutcropExecutionStatus::PathingFailed);
        };
        state.last_routes = pathing_request.routes.clone();

        let navigation =
            runtime.prepare_auto_ley_line_outcrop_navigation(plan, &pathing_request)?;
        executed_actions.push(auto_ley_line_action_report(
            AutoLeyLineOutcropRuntimeActionKind::PrepareNavigation,
            AutoLeyLineOutcropRuntimeActionOutcome::NavigationPreparation(navigation.clone()),
        ));
        if !navigation.prepared {
            return Ok(AutoLeyLineOutcropExecutionStatus::NavigationPreparationFailed);
        }

        let pathing = runtime.execute_auto_ley_line_outcrop_pathing(plan, &pathing_request)?;
        executed_actions.push(auto_ley_line_action_report(
            AutoLeyLineOutcropRuntimeActionKind::RunPathing,
            AutoLeyLineOutcropRuntimeActionOutcome::Pathing(pathing.clone()),
        ));
        if !pathing.completed {
            return Ok(AutoLeyLineOutcropExecutionStatus::PathingFailed);
        }

        let fight = runtime.execute_auto_ley_line_outcrop_fight(plan, run_index)?;
        executed_actions.push(auto_ley_line_action_report(
            AutoLeyLineOutcropRuntimeActionKind::ExecuteFight,
            AutoLeyLineOutcropRuntimeActionOutcome::Fight(fight.clone()),
        ));
        if !auto_ley_line_fight_succeeded(&fight) {
            state.fight_failures += 1;
            state.consecutive_fight_failures += 1;
            let reason = auto_ley_line_fight_failure_reason(&fight);
            auto_ley_line_skip(
                skipped_steps,
                executed_actions,
                AutoLeyLineOutcropRuntimeActionKind::ExecuteFight,
                AutoLeyLineSkipReason::FightFailed { reason },
            );
            if u64::from(state.consecutive_fight_failures)
                >= plan.pathing_rule.max_consecutive_fight_failures.max(1)
            {
                return Ok(AutoLeyLineOutcropExecutionStatus::MaxConsecutiveFightFailures);
            }
            if auto_ley_line_fight_should_stop(&fight) {
                return Ok(AutoLeyLineOutcropExecutionStatus::FightFailedStopped);
            }
            if !execute_auto_ley_line_switch_next(
                plan,
                runtime,
                state,
                executed_actions,
                skipped_steps,
            )? {
                return Ok(AutoLeyLineOutcropExecutionStatus::NoLeyLineFound);
            }
            continue;
        }
        state.fights_completed += 1;
        state.consecutive_fight_failures = 0;

        let claim_plan = match reward_claim_plan {
            Some(plan) => plan,
            None => {
                let inventory = execute_auto_ley_line_resin_count(
                    plan,
                    runtime,
                    run_index,
                    state,
                    executed_actions,
                )?;
                match select_auto_ley_line_reward_claim_plan(plan, &inventory) {
                    Ok(plan) => plan,
                    Err(reason) => {
                        state.reward_skipped_count += 1;
                        auto_ley_line_skip(
                            skipped_steps,
                            executed_actions,
                            AutoLeyLineOutcropRuntimeActionKind::ClaimReward,
                            AutoLeyLineSkipReason::ResinUnavailable(reason),
                        );
                        return Ok(AutoLeyLineOutcropExecutionStatus::RewardSkipped);
                    }
                }
            }
        };
        state.last_reward_claim_plan = Some(claim_plan.clone());

        let reward_navigation = runtime.navigate_auto_ley_line_outcrop_reward(plan, run_index)?;
        executed_actions.push(auto_ley_line_action_report(
            AutoLeyLineOutcropRuntimeActionKind::NavigateRewardFlower,
            AutoLeyLineOutcropRuntimeActionOutcome::RewardNavigation(reward_navigation.clone()),
        ));
        if !reward_navigation.reached {
            return Ok(AutoLeyLineOutcropExecutionStatus::RewardNavigationFailed);
        }

        let claim_outcome = runtime.claim_auto_ley_line_outcrop_reward(plan, &claim_plan)?;
        state.last_reward_claim_outcome = Some(claim_outcome.clone());
        executed_actions.push(auto_ley_line_action_report(
            AutoLeyLineOutcropRuntimeActionKind::ClaimReward,
            AutoLeyLineOutcropRuntimeActionOutcome::RewardClaim(claim_outcome.clone()),
        ));
        match claim_outcome {
            AutoLeyLineRewardClaimOutcome::Claimed { .. } => {
                state.reward_claims_completed += 1;
            }
            AutoLeyLineRewardClaimOutcome::Skipped { reason } => {
                state.reward_skipped_count += 1;
                auto_ley_line_skip(
                    skipped_steps,
                    executed_actions,
                    AutoLeyLineOutcropRuntimeActionKind::ClaimReward,
                    AutoLeyLineSkipReason::RewardClaimSkipped(reason),
                );
                return Ok(
                    if reason == AutoLeyLineRewardSkipReason::InsufficientResin {
                        AutoLeyLineOutcropExecutionStatus::ResinExhausted
                    } else {
                        AutoLeyLineOutcropExecutionStatus::RewardSkipped
                    },
                );
            }
        }

        if plan.reward_rule.scan_drops_after_reward_enabled {
            let scan = runtime.scan_auto_ley_line_outcrop_drops(plan, run_index)?;
            state.drop_scans += 1;
            executed_actions.push(auto_ley_line_action_report(
                AutoLeyLineOutcropRuntimeActionKind::ScanDropsAfterReward,
                AutoLeyLineOutcropRuntimeActionOutcome::DropScan(scan),
            ));
        } else {
            auto_ley_line_skip(
                skipped_steps,
                executed_actions,
                AutoLeyLineOutcropRuntimeActionKind::ScanDropsAfterReward,
                AutoLeyLineSkipReason::DropScanDisabled,
            );
        }

        if state.reward_claims_completed >= state.target_runs {
            break;
        }

        if !execute_auto_ley_line_switch_next(
            plan,
            runtime,
            state,
            executed_actions,
            skipped_steps,
        )? {
            return Ok(AutoLeyLineOutcropExecutionStatus::NoLeyLineFound);
        }
    }

    Ok(AutoLeyLineOutcropExecutionStatus::Completed)
}

fn execute_auto_ley_line_resin_count<R>(
    plan: &AutoLeyLineOutcropExecutionPlan,
    runtime: &mut R,
    run_index: u32,
    state: &mut AutoLeyLineOutcropExecutorState,
    executed_actions: &mut Vec<AutoLeyLineOutcropRuntimeActionReport>,
) -> Result<AutoLeyLineResinInventory>
where
    R: AutoLeyLineOutcropRuntime,
{
    let inventory = runtime.count_auto_ley_line_outcrop_resin(plan, run_index)?;
    state.resin_checks += 1;
    state.last_resin_inventory = Some(inventory.clone());
    executed_actions.push(auto_ley_line_action_report(
        AutoLeyLineOutcropRuntimeActionKind::CountResin,
        AutoLeyLineOutcropRuntimeActionOutcome::Resin(inventory.clone()),
    ));
    Ok(inventory)
}

fn execute_auto_ley_line_switch_next<R>(
    plan: &AutoLeyLineOutcropExecutionPlan,
    runtime: &mut R,
    state: &mut AutoLeyLineOutcropExecutorState,
    executed_actions: &mut Vec<AutoLeyLineOutcropRuntimeActionReport>,
    skipped_steps: &mut Vec<AutoLeyLineOutcropSkippedStep>,
) -> Result<bool>
where
    R: AutoLeyLineOutcropRuntime,
{
    let switch = runtime.switch_auto_ley_line_outcrop_next(plan, state.reward_claims_completed)?;
    if switch.switched {
        state.next_ley_line_switches += 1;
    }
    executed_actions.push(auto_ley_line_action_report(
        AutoLeyLineOutcropRuntimeActionKind::SwitchNextLeyLine,
        AutoLeyLineOutcropRuntimeActionOutcome::SwitchNext(switch.clone()),
    ));
    if !switch.switched || switch.exhausted {
        auto_ley_line_skip(
            skipped_steps,
            executed_actions,
            AutoLeyLineOutcropRuntimeActionKind::SwitchNextLeyLine,
            AutoLeyLineSkipReason::NextLeyLineUnavailable,
        );
        return Ok(false);
    }
    Ok(true)
}

fn execute_auto_ley_line_outcrop_cleanup<R>(
    plan: &AutoLeyLineOutcropExecutionPlan,
    runtime: &mut R,
    status: AutoLeyLineOutcropExecutionStatus,
    state: &mut AutoLeyLineOutcropExecutorState,
    executed_actions: &mut Vec<AutoLeyLineOutcropRuntimeActionReport>,
) -> Result<()>
where
    R: AutoLeyLineOutcropRuntime,
{
    let cleanup = runtime.cleanup_auto_ley_line_outcrop(plan, status)?;
    state.cleanup_completed = cleanup.completed;
    executed_actions.push(auto_ley_line_action_report(
        AutoLeyLineOutcropRuntimeActionKind::Cleanup,
        AutoLeyLineOutcropRuntimeActionOutcome::Cleanup(cleanup),
    ));
    Ok(())
}

fn auto_ley_line_execution_report(
    plan: &AutoLeyLineOutcropExecutionPlan,
    status: AutoLeyLineOutcropExecutionStatus,
    state: AutoLeyLineOutcropExecutorState,
    executed_actions: Vec<AutoLeyLineOutcropRuntimeActionReport>,
    skipped_steps: Vec<AutoLeyLineOutcropSkippedStep>,
) -> AutoLeyLineOutcropExecutionReport {
    AutoLeyLineOutcropExecutionReport {
        task_key: plan.task_key.clone(),
        completed: matches!(
            status,
            AutoLeyLineOutcropExecutionStatus::Completed
                | AutoLeyLineOutcropExecutionStatus::ResinExhausted
        ),
        status,
        state,
        executed_actions,
        skipped_steps,
    }
}

fn auto_ley_line_action_report(
    action_kind: AutoLeyLineOutcropRuntimeActionKind,
    outcome: AutoLeyLineOutcropRuntimeActionOutcome,
) -> AutoLeyLineOutcropRuntimeActionReport {
    AutoLeyLineOutcropRuntimeActionReport {
        action_kind,
        outcome,
    }
}

fn auto_ley_line_skip(
    skipped_steps: &mut Vec<AutoLeyLineOutcropSkippedStep>,
    executed_actions: &mut Vec<AutoLeyLineOutcropRuntimeActionReport>,
    action_kind: AutoLeyLineOutcropRuntimeActionKind,
    reason: AutoLeyLineSkipReason,
) {
    skipped_steps.push(AutoLeyLineOutcropSkippedStep {
        action_kind,
        reason: reason.clone(),
    });
    executed_actions.push(auto_ley_line_action_report(
        AutoLeyLineOutcropRuntimeActionKind::Skip,
        AutoLeyLineOutcropRuntimeActionOutcome::Skipped(reason),
    ));
}

fn auto_ley_line_claim_plan(
    resin_kind: AutoLeyLineResinKind,
    priority_label: &str,
    expected_original_resin_cost: i32,
    uses_double_reward: bool,
) -> AutoLeyLineRewardClaimPlan {
    AutoLeyLineRewardClaimPlan {
        resin_kind,
        priority_label: priority_label.to_string(),
        expected_original_resin_cost,
        uses_double_reward,
    }
}

fn auto_ley_line_pathing_request(
    plan: &AutoLeyLineOutcropExecutionPlan,
    run_index: u32,
) -> Option<AutoLeyLinePathingExecutionRequest> {
    let selected = plan.pathing_rule.selected_position_plan.as_ref()?;
    let run_kind = if run_index == 0 {
        AutoLeyLinePathingRunKind::Initial
    } else {
        AutoLeyLinePathingRunKind::Rerun
    };
    let mut routes = if run_kind == AutoLeyLinePathingRunKind::Initial {
        selected.routes.clone()
    } else {
        Vec::new()
    };
    let terminal_route = if run_kind == AutoLeyLinePathingRunKind::Initial {
        &selected.target_route
    } else {
        &selected.rerun_route
    };
    if !terminal_route.trim().is_empty() && !routes.contains(terminal_route) {
        routes.push(terminal_route.clone());
    }
    if routes.is_empty() {
        return None;
    }

    Some(AutoLeyLinePathingExecutionRequest {
        run_index,
        run_kind,
        routes,
        selected_start_node_id: Some(selected.start_node_id),
        selected_target_node_id: Some(selected.target_node_id),
        selected_strategy: Some(selected.strategy.clone()),
    })
}

fn auto_ley_line_fight_succeeded(outcome: &AutoLeyLineFightOutcome) -> bool {
    matches!(outcome, AutoLeyLineFightOutcome::Succeeded { .. })
}

fn auto_ley_line_fight_should_stop(outcome: &AutoLeyLineFightOutcome) -> bool {
    match outcome {
        AutoLeyLineFightOutcome::Succeeded { .. } => false,
        AutoLeyLineFightOutcome::Failed { stop, .. }
        | AutoLeyLineFightOutcome::TimedOut { stop, .. } => *stop,
    }
}

fn auto_ley_line_fight_failure_reason(outcome: &AutoLeyLineFightOutcome) -> String {
    match outcome {
        AutoLeyLineFightOutcome::Succeeded { .. } => String::new(),
        AutoLeyLineFightOutcome::Failed { reason, .. } => reason.clone(),
        AutoLeyLineFightOutcome::TimedOut {
            timeout_seconds, ..
        } => format!("fight timed out after {timeout_seconds} seconds"),
    }
}

fn handbook_rule() -> AutoLeyLineHandbookRule {
    AutoLeyLineHandbookRule {
        open_handbook_action: "OpenAdventurerHandbook".to_string(),
        click_sequence_1080p: vec![
            AutoLeyLineClickPoint { x: 300, y: 550 },
            AutoLeyLineClickPoint { x: 500, y: 200 },
            AutoLeyLineClickPoint { x: 500, y: 500 },
            AutoLeyLineClickPoint { x: 1300, y: 800 },
        ],
        revelation_type_click: AutoLeyLineClickPoint { x: 700, y: 350 },
        wealth_type_click: AutoLeyLineClickPoint { x: 500, y: 350 },
        country_ocr_special_case_nod_krai: "挪德卡".to_string(),
        track_action_template_attempts: 2,
        fallback_track_button_click: AutoLeyLineClickPoint { x: 1500, y: 850 },
        cancel_tracking_clicks_map_center_first: true,
    }
}

fn manual_map_rule(
    static_data: &LoadedLeyLineStaticData,
    country: &str,
) -> AutoLeyLineManualMapRule {
    AutoLeyLineManualMapRule {
        moves_big_map_to_country_scan_positions: true,
        adjusts_zoom_to: 3.0,
        blossom_icon_center_offset_pixels: 25,
        coordinate_formula:
            "(screen_center - icon_top_left - 25) * map_zoom / map_scale_factor + map_center"
                .to_string(),
        map_positions_for_selected_country: static_data
            .config
            .map_positions
            .get(country)
            .map(|positions| {
                positions
                    .iter()
                    .map(|position| AutoLeyLineMapPosition {
                        x: position.x,
                        y: position.y,
                        name: position.name.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn reward_navigation_rule() -> AutoLeyLineRewardNavigationRule {
    AutoLeyLineRewardNavigationRule {
        max_retry: 3,
        navigation_timeout_ms: 60_000,
        middle_click_resets_camera: true,
        reward_box_locator: template_locator("LeyLineRewardBox", AUTO_LEY_LINE_REWARD_BOX_ASSET),
        screen_center: AutoLeyLineClickPoint { x: 960, y: 540 },
        align_max_angle_degrees: 10.0,
        camera_move_x_clamp: 300,
        camera_move_down_when_icon_below_center: 500,
        forward_burst_ms: 200,
        recovery_key: "X".to_string(),
        backward_recovery_ms: 1_000,
        detects_reward_by_bv_keywords: strings(&["接触", "地脉", "之花"]),
        detects_reward_by_ocr_keywords: strings(&["原粹树脂", "接触", "地脉", "之花"]),
    }
}

fn ley_line_locators() -> AutoLeyLineLocators {
    AutoLeyLineLocators {
        open_custom_marks: template_locator(
            "LeyLineOpenCustomMarks",
            AUTO_LEY_LINE_OPEN_MARKS_ASSET,
        ),
        close_custom_marks: template_locator(
            "LeyLineCloseCustomMarks",
            AUTO_LEY_LINE_CLOSE_MARKS_ASSET,
        ),
        paimon_menu: template_locator_with_roi(
            "LeyLinePaimonMenu",
            AUTO_LEY_LINE_PAIMON_MENU_ASSET,
            Some(Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 216,
            }),
            0.8,
        ),
        reward_box: template_locator("LeyLineRewardBox", AUTO_LEY_LINE_REWARD_BOX_ASSET),
        map_setting_button: template_locator(
            "LeyLineMapSettingButton",
            AUTO_LEY_LINE_MAP_SETTING_ASSET,
        ),
        handbook_track_action: template_locator_with_roi(
            "LeyLineHandbookTrackAction",
            AUTO_LEY_LINE_HANDBOOK_TRACK_ACTION_ASSET,
            Some(Rect {
                x: 1120,
                y: 680,
                width: 700,
                height: 320,
            }),
            0.72,
        ),
        revelation_blossom: template_locator(
            "LeyLineBlossomOfRevelation",
            AUTO_LEY_LINE_REVELATION_ASSET,
        ),
        wealth_blossom: template_locator("LeyLineBlossomOfWealth", AUTO_LEY_LINE_WEALTH_ASSET),
        reward_switch_20_to_40: template_locator_with_roi(
            "LeyLineRewardSwitch20To40",
            AUTO_LEY_LINE_REWARD_SWITCH_ASSET,
            None,
            0.7,
        ),
        replenish_resin_button: template_locator(
            "LeyLineReplenishResinButton",
            AUTO_LEY_LINE_REPLENISH_RESIN_ASSET,
        ),
        original_resin: template_locator(
            "LeyLineOriginalResin",
            AUTO_LEY_LINE_ORIGINAL_RESIN_ASSET,
        ),
        condensed_resin: template_locator(
            "LeyLineCondensedResin",
            AUTO_LEY_LINE_CONDENSED_RESIN_ASSET,
        ),
        transient_resin: template_locator(
            "LeyLineTransientResin",
            AUTO_LEY_LINE_TRANSIENT_RESIN_ASSET,
        ),
        fragile_resin: template_locator("LeyLineFragileResin", AUTO_LEY_LINE_FRAGILE_RESIN_ASSET),
        ocr_regions: AutoLeyLineOcrRegions {
            fight_result: Rect {
                x: 800,
                y: 200,
                width: 300,
                height: 100,
            },
            left_flow: Rect {
                x: 0,
                y: 200,
                width: 300,
                height: 300,
            },
            right_flow: Rect {
                x: 1200,
                y: 520,
                width: 300,
                height: 300,
            },
            overlay_render_lead_ms: 300,
        },
    }
}

fn template_locator(name: &str, asset: &str) -> AutoLeyLineTemplateLocator {
    template_locator_with_roi(name, asset, None, 0.8)
}

fn template_locator_with_roi(
    name: &str,
    asset: &str,
    roi: Option<Rect>,
    threshold: f64,
) -> AutoLeyLineTemplateLocator {
    AutoLeyLineTemplateLocator {
        name: name.to_string(),
        asset: asset.to_string(),
        roi,
        threshold,
        match_mode: TemplateMatchMode::CCoeffNormed,
        draw_on_window: false,
    }
}

fn ley_line_steps() -> Vec<AutoLeyLineTaskStep> {
    use AutoLeyLineTaskAction::*;
    use AutoLeyLineTaskPhase::*;
    vec![
        AutoLeyLineTaskStep {
            phase: Startup,
            action: ValidateConfigAndLoadStaticData,
        },
        AutoLeyLineTaskStep {
            phase: Resin,
            action: CountResinWhenExhaustionMode,
        },
        AutoLeyLineTaskStep {
            phase: Startup,
            action: PrepareMainUiTeamAndMarks,
        },
        AutoLeyLineTaskStep {
            phase: Discovery,
            action: FindLeyLineByHandbookOrMap,
        },
        AutoLeyLineTaskStep {
            phase: Pathing,
            action: MatchStaticStrategyAndRunPathing,
        },
        AutoLeyLineTaskStep {
            phase: Combat,
            action: ProcessLeyLineFightAndRecovery,
        },
        AutoLeyLineTaskStep {
            phase: Reward,
            action: NavigateToRewardFlower,
        },
        AutoLeyLineTaskStep {
            phase: Reward,
            action: ClaimRewardWithResinPriority,
        },
        AutoLeyLineTaskStep {
            phase: Reward,
            action: ScanDropsAfterRewardWhenEnabled,
        },
        AutoLeyLineTaskStep {
            phase: Cleanup,
            action: RestoreMarksOverlayAndInputs,
        },
    ]
}

fn apply_auto_ley_line_config(
    param: &mut AutoLeyLineOutcropParam,
    config: &AutoLeyLineOutcropConfig,
) {
    param.count = u64_to_i32(config.count);
    param.country = config.country.clone();
    param.ley_line_outcrop_type = config.ley_line_outcrop_type.clone();
    param.open_mode_count_min = config.open_mode_count_min;
    param.is_resin_exhaustion_mode = config.is_resin_exhaustion_mode;
    param.use_adventurer_handbook = config.use_adventurer_handbook;
    param.friendship_team = config.friendship_team.clone();
    param.team = config.team.clone();
    param.timeout = u64_to_i32(config.timeout);
    param.is_go_to_synthesizer = config.is_go_to_synthesizer;
    param.use_fragile_resin = config.use_fragile_resin;
    param.use_transient_resin = config.use_transient_resin;
    param.is_notification = config.is_notification;
    param.scan_drops_after_reward_enabled = config.scan_drops_after_reward_enabled;
    param.scan_drops_after_reward_seconds = u64_to_i32(config.scan_drops_after_reward_seconds);
    param.fight_config = fight_config_param_from_core(&config.fight_config);
}

fn fight_config_param_from_core(
    config: &AutoLeyLineOutcropFightConfig,
) -> AutoLeyLineOutcropFightConfigParam {
    AutoLeyLineOutcropFightConfigParam {
        strategy_name: config.strategy_name.clone(),
        team_names: config.team_names.clone(),
        fight_finish_detect_enabled: config.fight_finish_detect_enabled,
        action_scheduler_by_cd: config.action_scheduler_by_cd.clone(),
        finish_detect_config: finish_detect_param_from_core(&config.finish_detect_config),
        guardian_avatar: config.guardian_avatar.clone(),
        guardian_combat_skip: config.guardian_combat_skip,
        guardian_avatar_hold: config.guardian_avatar_hold,
        burst_enabled: config.burst_enabled,
        swimming_enabled: config.swimming_enabled,
        kazuha_pickup_enabled: config.kazuha_pickup_enabled,
        qin_double_pick_up: config.qin_double_pick_up,
        timeout: u64_to_i32(config.timeout),
        seek_enemy_enabled: config.seek_enemy_enabled,
        seek_enemy_interval_seconds: u64_to_i32(config.seek_enemy_interval_seconds),
        seek_enemy_rotary_factor: u64_to_i32(config.seek_enemy_rotary_factor),
    }
}

fn finish_detect_param_from_core(
    config: &LeyLineFightFinishDetectConfig,
) -> FightFinishDetectParam {
    FightFinishDetectParam {
        battle_end_progress_bar_color: config.battle_end_progress_bar_color.clone(),
        battle_end_progress_bar_color_tolerance: config
            .battle_end_progress_bar_color_tolerance
            .clone(),
        fast_check_enabled: config.fast_check_enabled,
        fast_check_params: config.fast_check_params.clone(),
        check_end_delay: config.check_end_delay.clone(),
        before_detect_delay: config.before_detect_delay.clone(),
        rotate_find_enemy_enabled: config.rotate_find_enemy_enabled,
        rotary_factor: u64_to_i32(config.rotary_factor),
        is_first_check: config.is_first_check,
        check_before_burst: config.check_before_burst,
    }
}

fn overlay_auto_ley_line_param_members(param: &mut AutoLeyLineOutcropParam, value: &Value) {
    if let Some(count) = i32_member(value, ["count", "Count"]) {
        param.count = count;
    }
    if let Some(country) = string_member(value, ["country", "Country"]) {
        param.country = country;
    }
    if let Some(value) = string_member(
        value,
        [
            "leyLineOutcropType",
            "LeyLineOutcropType",
            "ley_line_outcrop_type",
        ],
    ) {
        param.ley_line_outcrop_type = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "openModeCountMin",
            "OpenModeCountMin",
            "open_mode_count_min",
        ],
    ) {
        param.open_mode_count_min = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "isResinExhaustionMode",
            "IsResinExhaustionMode",
            "is_resin_exhaustion_mode",
        ],
    ) {
        param.is_resin_exhaustion_mode = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "useAdventurerHandbook",
            "UseAdventurerHandbook",
            "use_adventurer_handbook",
        ],
    ) {
        param.use_adventurer_handbook = value;
    }
    if let Some(value) = string_member(
        value,
        ["friendshipTeam", "FriendshipTeam", "friendship_team"],
    ) {
        param.friendship_team = value;
    }
    if let Some(value) = string_member(value, ["team", "Team"]) {
        param.team = value;
    }
    if let Some(value) = i32_member(value, ["timeout", "Timeout"]) {
        param.timeout = value;
        param.fight_config.timeout = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "isGoToSynthesizer",
            "IsGoToSynthesizer",
            "is_go_to_synthesizer",
        ],
    ) {
        param.is_go_to_synthesizer = value;
    }
    if let Some(value) = bool_member(
        value,
        ["useFragileResin", "UseFragileResin", "use_fragile_resin"],
    ) {
        param.use_fragile_resin = value;
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
        ["isNotification", "IsNotification", "is_notification"],
    ) {
        param.is_notification = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "scanDropsAfterRewardEnabled",
            "ScanDropsAfterRewardEnabled",
            "scan_drops_after_reward_enabled",
        ],
    ) {
        param.scan_drops_after_reward_enabled = value;
    }
    if let Some(value) = i32_member(
        value,
        [
            "scanDropsAfterRewardSeconds",
            "ScanDropsAfterRewardSeconds",
            "scan_drops_after_reward_seconds",
        ],
    ) {
        param.scan_drops_after_reward_seconds = value;
    }
    if let Some(fight_config) = value
        .get("fightConfig")
        .or_else(|| value.get("FightConfig"))
        .or_else(|| value.get("fight_config"))
    {
        overlay_fight_config_members(&mut param.fight_config, fight_config);
    }
}

fn overlay_fight_config_members(config: &mut AutoLeyLineOutcropFightConfigParam, value: &Value) {
    if let Some(value) = string_member(value, ["strategyName", "StrategyName", "strategy_name"]) {
        config.strategy_name = value;
    }
    if let Some(value) = string_member(value, ["teamNames", "TeamNames", "team_names"]) {
        config.team_names = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "fightFinishDetectEnabled",
            "FightFinishDetectEnabled",
            "fight_finish_detect_enabled",
        ],
    ) {
        config.fight_finish_detect_enabled = value;
    }
    if let Some(value) = string_member(
        value,
        [
            "actionSchedulerByCd",
            "ActionSchedulerByCd",
            "action_scheduler_by_cd",
        ],
    ) {
        config.action_scheduler_by_cd = value;
    }
    if let Some(value) = string_member(
        value,
        ["guardianAvatar", "GuardianAvatar", "guardian_avatar"],
    ) {
        config.guardian_avatar = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "guardianCombatSkip",
            "GuardianCombatSkip",
            "guardian_combat_skip",
        ],
    ) {
        config.guardian_combat_skip = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "guardianAvatarHold",
            "GuardianAvatarHold",
            "guardian_avatar_hold",
        ],
    ) {
        config.guardian_avatar_hold = value;
    }
    if let Some(value) = bool_member(value, ["burstEnabled", "BurstEnabled", "burst_enabled"]) {
        config.burst_enabled = value;
    }
    if let Some(value) = bool_member(
        value,
        ["swimmingEnabled", "SwimmingEnabled", "swimming_enabled"],
    ) {
        config.swimming_enabled = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "kazuhaPickupEnabled",
            "KazuhaPickupEnabled",
            "kazuha_pickup_enabled",
        ],
    ) {
        config.kazuha_pickup_enabled = value;
    }
    if let Some(value) = bool_member(
        value,
        ["qinDoublePickUp", "QinDoublePickUp", "qin_double_pick_up"],
    ) {
        config.qin_double_pick_up = value;
    }
    if let Some(value) = i32_member(value, ["timeout", "Timeout"]) {
        config.timeout = value;
    }
    if let Some(value) = bool_member(
        value,
        ["seekEnemyEnabled", "SeekEnemyEnabled", "seek_enemy_enabled"],
    ) {
        config.seek_enemy_enabled = value;
    }
    if let Some(value) = i32_member(
        value,
        [
            "seekEnemyIntervalSeconds",
            "SeekEnemyIntervalSeconds",
            "seek_enemy_interval_seconds",
        ],
    ) {
        config.seek_enemy_interval_seconds = value;
    }
    if let Some(value) = i32_member(
        value,
        [
            "seekEnemyRotaryFactor",
            "SeekEnemyRotaryFactor",
            "seek_enemy_rotary_factor",
        ],
    ) {
        config.seek_enemy_rotary_factor = value;
    }
}

fn normalize_auto_ley_line_param(param: &mut AutoLeyLineOutcropParam) {
    if param.count < 1 {
        param.count = 1;
    }
    if param.fight_config.strategy_name.trim().is_empty() && param.timeout > 0 {
        param.fight_config.timeout = param.timeout;
    }
    if param.fight_config.timeout <= 0 {
        param.fight_config.timeout = if param.timeout > 0 {
            param.timeout
        } else {
            120
        };
    } else {
        param.timeout = param.fight_config.timeout;
    }
    if param.scan_drops_after_reward_seconds < 0 {
        param.scan_drops_after_reward_seconds = 0;
    }
}

fn validate_auto_ley_line_param(param: &AutoLeyLineOutcropParam) -> Result<()> {
    if param.ley_line_outcrop_type.trim().is_empty() {
        return invalid_config("地脉花类型未选择");
    }
    if !AUTO_LEY_LINE_VALID_TYPES.contains(&param.ley_line_outcrop_type.as_str()) {
        return invalid_config("地脉花类型无效，请重新选择");
    }
    if param.country.trim().is_empty() {
        return invalid_config("国家未配置");
    }
    if !param.friendship_team.trim().is_empty() && param.team.trim().is_empty() {
        return invalid_config("配置好感队时必须配置战斗队伍");
    }
    Ok(())
}

fn validate_auto_ley_line_strategy(
    working_directory: &Path,
    param: &AutoLeyLineOutcropParam,
) -> Result<Option<String>> {
    let strategy_name = param.fight_config.strategy_name.trim();
    if strategy_name.is_empty() {
        return Ok(None);
    }

    let strategy_path = combat_strategy_path(Some(strategy_name));
    let full_path = working_directory.join(Path::new(&strategy_path));
    if !full_path.exists() {
        if strategy_name == AUTO_STRATEGY_NAME {
            return Err(TaskError::CombatStrategy(format!(
                "战斗策略目录不存在: {strategy_path}"
            )));
        }
        return Err(TaskError::CombatStrategy(format!(
            "战斗策略文件不存在: {strategy_path}"
        )));
    }
    Ok(Some(strategy_path))
}

fn invalid_config<T>(message: impl Into<String>) -> Result<T> {
    Err(TaskError::InvalidTaskConfig {
        key: AUTO_LEY_LINE_OUTCROP_TASK_KEY.to_string(),
        message: message.into(),
    })
}

#[derive(Debug, Clone)]
struct LoadedLeyLineStaticData {
    task_directory: PathBuf,
    config: AutoLeyLineConfigData,
    raw_nodes: RawNodeData,
    nodes: NodeData,
}

impl LoadedLeyLineStaticData {
    fn supported_countries(&self) -> Vec<String> {
        let mut countries = self
            .config
            .ley_line_positions
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        countries.sort();
        countries
    }

    fn map_position_count(&self) -> usize {
        self.config.map_positions.values().map(Vec::len).sum()
    }

    fn ley_line_position_count(&self) -> usize {
        self.config.ley_line_positions.values().map(Vec::len).sum()
    }

    fn node_index_groups(&self) -> Vec<String> {
        let mut groups = self.nodes.indexes.keys().cloned().collect::<Vec<_>>();
        groups.sort();
        groups
    }
}

fn load_ley_line_static_data(working_directory: &Path) -> Result<LoadedLeyLineStaticData> {
    let task_directory = resolve_ley_line_task_directory(working_directory)?;
    let config_path = task_directory.join("Assets").join("config.json");
    let node_path = task_directory
        .join("Assets")
        .join("LeyLineOutcropData.json");
    let config = read_json::<AutoLeyLineConfigData>(&config_path)?;
    let raw_nodes = read_json::<RawNodeData>(&node_path)?;
    let nodes = adapt_node_data(&raw_nodes);
    Ok(LoadedLeyLineStaticData {
        task_directory,
        config,
        raw_nodes,
        nodes,
    })
}

fn resolve_ley_line_task_directory(working_directory: &Path) -> Result<PathBuf> {
    let candidates = [
        working_directory.join(AUTO_LEY_LINE_OUTCROP_TASK_DIR),
        crate::task_asset_root().join(AUTO_LEY_LINE_OUTCROP_TASK_DIR),
    ];
    candidates
        .into_iter()
        .find(|candidate| {
            candidate.join("Assets").join("config.json").is_file()
                && candidate
                    .join("Assets")
                    .join("LeyLineOutcropData.json")
                    .is_file()
        })
        .ok_or_else(|| TaskError::InvalidTaskConfig {
            key: AUTO_LEY_LINE_OUTCROP_TASK_KEY.to_string(),
            message: "地脉花静态配置文件 config.json 或 LeyLineOutcropData.json 未找到".to_string(),
        })
}

fn read_json<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let text = std::fs::read_to_string(path).map_err(|source| TaskError::InvalidTaskConfig {
        key: AUTO_LEY_LINE_OUTCROP_TASK_KEY.to_string(),
        message: format!("读取 {} 失败: {source}", path.display()),
    })?;
    serde_json::from_str(&text).map_err(|source| TaskError::InvalidTaskConfig {
        key: AUTO_LEY_LINE_OUTCROP_TASK_KEY.to_string(),
        message: format!("解析 {} 失败: {source}", path.display()),
    })
}

fn selected_position_plan(
    static_data: &LoadedLeyLineStaticData,
    param: &AutoLeyLineOutcropParam,
) -> Option<AutoLeyLineSelectedPositionPlan> {
    let positions = static_data.config.ley_line_positions.get(&param.country)?;
    let position = positions.iter().min_by_key(|position| position.order)?;
    let target_node = find_target_node_by_position(&static_data.nodes, position.x, position.y)?;
    let paths = find_paths_to_target(&static_data.nodes, target_node);
    let optimal = paths.into_iter().min_by_key(|path| path.routes.len())?;
    let last_route = optimal.routes.last()?.clone();
    Some(AutoLeyLineSelectedPositionPlan {
        strategy: position.strategy.clone(),
        order: position.order,
        steps: position.steps,
        ley_line_position: AutoLeyLineNodePosition {
            x: position.x,
            y: position.y,
        },
        start_node_id: optimal.start_node.id,
        start_region: optimal.start_node.region.clone(),
        target_node_id: optimal.target_node.id,
        target_region: optimal.target_node.region.clone(),
        from_teleport_start: optimal.start_node.node_type == "teleport",
        route_count: optimal.routes.len(),
        routes: optimal.routes,
        target_route: build_target_route(&last_route),
        rerun_route: build_rerun_route(&last_route),
    })
}

fn route_file_status(
    task_directory: &Path,
    plan: Option<&AutoLeyLineSelectedPositionPlan>,
) -> (Vec<String>, Vec<String>) {
    let Some(plan) = plan else {
        return (Vec::new(), Vec::new());
    };
    let mut required = plan.routes.clone();
    required.push(plan.target_route.clone());
    required.push(plan.rerun_route.clone());
    required.sort();
    required.dedup();
    let missing = required
        .iter()
        .filter(|route| !route_to_file_path(task_directory, route).is_file())
        .cloned()
        .collect();
    (required, missing)
}

fn route_to_file_path(task_directory: &Path, route: &str) -> PathBuf {
    let normalized = if route.to_ascii_lowercase().starts_with("assets/") {
        format!("Assets/{}", &route[7..])
    } else {
        route.to_string()
    };
    task_directory.join(normalized.replace('/', std::path::MAIN_SEPARATOR_STR))
}

fn build_target_route(route_path: &str) -> String {
    route_path
        .replacen("assets/pathing/", "assets/pathing/target/", 1)
        .replace("-rerun", "")
}

fn build_rerun_route(route_path: &str) -> String {
    let mut route = route_path
        .replacen("assets/pathing/target/", "assets/pathing/rerun/", 1)
        .replacen("assets/pathing/", "assets/pathing/rerun/", 1);
    if !route.to_ascii_lowercase().contains("-rerun") {
        route = route.replace(".json", "-rerun.json");
    }
    route
}

fn adapt_node_data(raw: &RawNodeData) -> NodeData {
    let mut nodes = Vec::with_capacity(raw.teleports.len() + raw.blossoms.len());
    for teleport in &raw.teleports {
        nodes.push(Node {
            id: teleport.id,
            region: teleport.region.clone(),
            position: teleport.position,
            node_type: "teleport".to_string(),
            next: Vec::new(),
            prev: Vec::new(),
        });
    }
    for blossom in &raw.blossoms {
        nodes.push(Node {
            id: blossom.id,
            region: blossom.region.clone(),
            position: blossom.position,
            node_type: "blossom".to_string(),
            next: Vec::new(),
            prev: Vec::new(),
        });
    }

    let index_by_id = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id, index))
        .collect::<HashMap<_, _>>();
    for edge in &raw.edges {
        let Some(&source_index) = index_by_id.get(&edge.source) else {
            continue;
        };
        let Some(&target_index) = index_by_id.get(&edge.target) else {
            continue;
        };
        nodes[source_index].next.push(NodeRoute {
            target: edge.target,
            route: edge.route.clone(),
        });
        nodes[target_index].prev.push(edge.source);
    }

    NodeData {
        nodes,
        indexes: raw.indexes.clone(),
    }
}

fn find_target_node_by_position(node_data: &NodeData, x: f64, y: f64) -> Option<&Node> {
    const ERROR_THRESHOLD: f64 = 50.0;
    node_data.nodes.iter().find(|node| {
        node.node_type == "blossom"
            && (node.position.x - x).abs() <= ERROR_THRESHOLD
            && (node.position.y - y).abs() <= ERROR_THRESHOLD
    })
}

fn find_paths_to_target(node_data: &NodeData, target_node: &Node) -> Vec<PathInfo> {
    let mut valid_paths = breadth_first_path_search(node_data, target_node);
    if valid_paths.is_empty() {
        valid_paths.extend(find_reverse_paths_if_needed(node_data, target_node));
    }
    valid_paths
}

fn breadth_first_path_search(node_data: &NodeData, target_node: &Node) -> Vec<PathInfo> {
    let mut valid_paths = Vec::new();
    let node_map = node_data
        .nodes
        .iter()
        .map(|node| (node.id, node))
        .collect::<HashMap<_, _>>();

    for start_node in node_data
        .nodes
        .iter()
        .filter(|node| node.node_type == "teleport")
    {
        let mut queue = VecDeque::new();
        queue.push_back((
            start_node,
            PathInfo {
                start_node: start_node.clone(),
                target_node: target_node.clone(),
                routes: Vec::new(),
            },
            HashSet::from([start_node.id]),
        ));

        while let Some((current, path, visited)) = queue.pop_front() {
            if current.id == target_node.id {
                valid_paths.push(path);
                continue;
            }

            for next in &current.next {
                if visited.contains(&next.target) {
                    continue;
                }
                let Some(next_node) = node_map.get(&next.target).copied() else {
                    continue;
                };
                let mut new_routes = path.routes.clone();
                new_routes.push(next.route.clone());
                let mut new_visited = visited.clone();
                new_visited.insert(next.target);
                queue.push_back((
                    next_node,
                    PathInfo {
                        start_node: path.start_node.clone(),
                        target_node: target_node.clone(),
                        routes: new_routes,
                    },
                    new_visited,
                ));
            }
        }
    }

    valid_paths
}

fn find_reverse_paths_if_needed(node_data: &NodeData, target_node: &Node) -> Vec<PathInfo> {
    if target_node.prev.is_empty() {
        return Vec::new();
    }

    let node_map = node_data
        .nodes
        .iter()
        .map(|node| (node.id, node))
        .collect::<HashMap<_, _>>();
    let mut reverse_paths = Vec::new();

    for prev_node_id in &target_node.prev {
        let Some(prev_node) = node_map.get(prev_node_id).copied() else {
            continue;
        };
        for teleport_node in node_data.nodes.iter().filter(|node| {
            node.node_type == "teleport"
                && node.next.iter().any(|route| route.target == prev_node.id)
        }) {
            let route = teleport_node
                .next
                .iter()
                .find(|route| route.target == prev_node.id);
            let next_route = prev_node
                .next
                .iter()
                .find(|route| route.target == target_node.id);
            if let (Some(route), Some(next_route)) = (route, next_route) {
                reverse_paths.push(PathInfo {
                    start_node: teleport_node.clone(),
                    target_node: target_node.clone(),
                    routes: vec![route.route.clone(), next_route.route.clone()],
                });
            }
        }
    }

    reverse_paths
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn string_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<String> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(|value| value.as_str().map(str::to_string))
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

fn u64_to_i32(value: u64) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AutoLeyLineConfigData {
    #[serde(rename = "errorThreshold")]
    error_threshold: f64,
    map_positions: HashMap<String, Vec<MapPosition>>,
    ley_line_positions: HashMap<String, Vec<LeyLinePosition>>,
}

#[derive(Debug, Clone, Deserialize)]
struct MapPosition {
    x: f64,
    y: f64,
    name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct LeyLinePosition {
    x: f64,
    y: f64,
    strategy: String,
    steps: i32,
    order: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct RawNodeData {
    teleports: Vec<RawNode>,
    blossoms: Vec<RawNode>,
    edges: Vec<RawEdge>,
    indexes: HashMap<String, HashMap<String, Vec<i32>>>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawNode {
    id: i32,
    region: String,
    position: AutoLeyLineNodePosition,
}

#[derive(Debug, Clone, Deserialize)]
struct RawEdge {
    source: i32,
    target: i32,
    route: String,
}

#[derive(Debug, Clone)]
struct NodeData {
    nodes: Vec<Node>,
    indexes: HashMap<String, HashMap<String, Vec<i32>>>,
}

#[derive(Debug, Clone)]
struct Node {
    id: i32,
    region: String,
    position: AutoLeyLineNodePosition,
    node_type: String,
    next: Vec<NodeRoute>,
    prev: Vec<i32>,
}

#[derive(Debug, Clone)]
struct NodeRoute {
    target: i32,
    route: String,
}

#[derive(Debug, Clone)]
struct PathInfo {
    start_node: Node,
    target_node: Node,
    routes: Vec<String>,
}

#[cfg(test)]
mod auto_ley_line_outcrop_executor_tests {
    use super::*;

    fn test_plan(count: i32) -> AutoLeyLineOutcropExecutionPlan {
        let mut param = AutoLeyLineOutcropParam::new(count, "蒙德", "启示之花");
        param.scan_drops_after_reward_enabled = true;
        let selected_position_plan = AutoLeyLineSelectedPositionPlan {
            strategy: "测试策略".to_string(),
            order: 1,
            steps: 1,
            ley_line_position: AutoLeyLineNodePosition { x: 10.0, y: 20.0 },
            start_node_id: 1,
            start_region: "蒙德".to_string(),
            target_node_id: 1000,
            target_region: "蒙德".to_string(),
            from_teleport_start: true,
            route_count: 1,
            routes: vec!["assets/pathing/蒙德-test-1.json".to_string()],
            target_route: "assets/pathing/target/蒙德-test-1.json".to_string(),
            rerun_route: "assets/pathing/rerun/蒙德-test-1-rerun.json".to_string(),
        };

        AutoLeyLineOutcropExecutionPlan {
            task_key: AUTO_LEY_LINE_OUTCROP_TASK_KEY.to_string(),
            display_name: AUTO_LEY_LINE_OUTCROP_DISPLAY_NAME.to_string(),
            capture_size: Size::new(1920, 1080),
            asset_scale: 1.0,
            param,
            data_rule: AutoLeyLineDataRule {
                task_directory: "GameTask/AutoLeyLineOutcrop".to_string(),
                config_json: AUTO_LEY_LINE_OUTCROP_CONFIG_JSON.to_string(),
                node_json: AUTO_LEY_LINE_OUTCROP_NODE_JSON.to_string(),
                error_threshold: 40.0,
                supported_countries: vec!["蒙德".to_string()],
                map_position_count: 1,
                ley_line_position_count: 1,
                teleport_count: 1,
                blossom_count: 1,
                edge_count: 1,
                node_index_groups: vec!["edgesBySource".to_string()],
                selected_country_supported: true,
                selected_country_map_positions: 1,
                selected_country_ley_line_positions: 1,
            },
            validation_rule: AutoLeyLineValidationRule {
                valid_ley_line_types: strings(AUTO_LEY_LINE_VALID_TYPES),
                selected_type_valid: true,
                requires_non_empty_country: true,
                selected_country_supported_by_config: true,
                normalized_count: count,
                normalized_timeout_seconds: 120,
                friendship_team_requires_combat_team: false,
                combat_strategy_path: None,
                combat_strategy_file_required: false,
                static_config_files_required: vec![
                    AUTO_LEY_LINE_OUTCROP_CONFIG_JSON.to_string(),
                    AUTO_LEY_LINE_OUTCROP_NODE_JSON.to_string(),
                ],
                requires_16_to_9_resolution: true,
                warns_below_1920x1080: true,
            },
            startup_rule: AutoLeyLineStartupRule {
                enables_mask_overlay_during_task: true,
                restores_mask_overlay_on_finish: true,
                ensures_exit_reward_page_before_start: true,
                returns_main_ui_before_start: true,
                teleports_to_statue_unless_one_dragon_mode: true,
                switches_combat_team_when_configured: false,
                use_adventurer_handbook_flag_means_manual_big_map_search: true,
                closes_custom_marks_when_manual_big_map_search: false,
                reopens_custom_marks_on_finish_if_closed: true,
                registers_auto_pick_trigger: true,
                sends_notification_when_enabled: false,
            },
            discovery_rule: AutoLeyLineDiscoveryRule {
                handbook_flow_when_use_adventurer_handbook_false: handbook_rule(),
                manual_map_flow_when_use_adventurer_handbook_true: AutoLeyLineManualMapRule {
                    moves_big_map_to_country_scan_positions: true,
                    adjusts_zoom_to: 3.0,
                    blossom_icon_center_offset_pixels: 25,
                    coordinate_formula: "test".to_string(),
                    map_positions_for_selected_country: vec![AutoLeyLineMapPosition {
                        x: 0.0,
                        y: 0.0,
                        name: Some("蒙德测试点".to_string()),
                    }],
                },
                selected_manual_flow: false,
                selected_country: "蒙德".to_string(),
                selected_type: "启示之花".to_string(),
                selected_blossom_asset: AUTO_LEY_LINE_REVELATION_ASSET.to_string(),
            },
            pathing_rule: AutoLeyLinePathingRule {
                selected_position_plan: Some(selected_position_plan),
                uses_bfs_from_teleport_nodes: true,
                uses_reverse_two_hop_fallback_when_no_forward_path: true,
                branch_route_uses_nearest_detected_ley_line_node: true,
                pathing_party_skip_party_switch: true,
                target_route_derivation: "test target".to_string(),
                rerun_route_derivation: "test rerun".to_string(),
                process_max_retries: AUTO_LEY_LINE_PROCESS_MAX_RETRIES,
                max_consecutive_fight_failures: AUTO_LEY_LINE_MAX_CONSECUTIVE_FAILURES,
                required_route_files: vec![
                    "assets/pathing/蒙德-test-1.json".to_string(),
                    "assets/pathing/target/蒙德-test-1.json".to_string(),
                    "assets/pathing/rerun/蒙德-test-1-rerun.json".to_string(),
                ],
                missing_route_files: Vec::new(),
            },
            combat_rule: AutoLeyLineCombatRule {
                timeout_seconds: 120,
                strategy_name: String::new(),
                strategy_path: None,
                blank_strategy_copies_global_auto_fight_config_at_runtime: true,
                auto_fight_runs_without_finish_detect: true,
                finish_detect_disabled_for_auto_fight_task: true,
                ocr_finish_success_keywords: strings(AUTO_LEY_LINE_SUCCESS_KEYWORDS),
                ocr_finish_failure_keywords: strings(AUTO_LEY_LINE_FAILURE_KEYWORDS),
                fight_text_keywords: strings(AUTO_LEY_LINE_FIGHT_KEYWORDS),
                no_fight_text_count_before_failure: 10,
                poll_interval_ms: 1_000,
                seek_enemy_enabled: false,
                seek_enemy_initial_delay_seconds: AUTO_LEY_LINE_FIGHT_SEEK_INITIAL_DELAY_SECONDS,
                seek_enemy_interval_seconds: 3,
                seek_enemy_rotary_factor: 6,
                resurrect_prompt_keyword: "复苏".to_string(),
                post_fight_collect_rule: AutoLeyLinePostFightCollectRule {
                    kazuha_pickup_enabled: true,
                    kazuha_hold_elemental_skill_ms: 1_000,
                    kazuha_post_skill_wait_ms: AUTO_LEY_LINE_KAZUHA_PICKUP_POST_SKILL_WAIT_MS,
                    qin_double_pickup_enabled: false,
                    disables_auto_fight_builtin_pickup: true,
                    only_pick_elite_drops_mode: "DisableAutoPickupForNonElite".to_string(),
                },
            },
            reward_navigation_rule: reward_navigation_rule(),
            reward_rule: AutoLeyLineRewardRule {
                interact_action: "PickUpOrInteract".to_string(),
                interact_wait_ms: 800,
                verify_reward_prompt_before_resin: true,
                reward_prompt_title_roi: AutoLeyLineRelativeRect {
                    x_ratio: 0.25,
                    y_ratio: 0.15,
                    width_ratio: 0.5,
                    height_ratio: 0.25,
                },
                reward_prompt_content_roi: AutoLeyLineRelativeRect {
                    x_ratio: 0.25,
                    y_ratio: 0.2,
                    width_ratio: 0.5,
                    height_ratio: 0.6,
                },
                title_keywords: strings(&["激活地脉之花", "选择激活方式", "地脉之花"]),
                content_keywords: strings(&[
                    "原粹树脂",
                    "浓缩树脂",
                    "须臾树脂",
                    "脆弱树脂",
                    "激活地脉之花",
                    "选择激活方式",
                    "补充",
                ]),
                action_keywords: strings(&["使用"]),
                activation_clicks_title_before_use: true,
                switch_double_reward_20_to_40: true,
                reward_retry_count: AUTO_LEY_LINE_REWARD_MAX_RETRIES,
                reward_use_attempts: AUTO_LEY_LINE_REWARD_USE_ATTEMPTS,
                switches_back_to_combat_team_after_reward: false,
                scan_drops_after_reward_enabled: true,
                scan_drops_after_reward_seconds: 10,
            },
            resin_rule: AutoLeyLineResinRule {
                resin_exhaustion_mode: false,
                open_mode_count_min: false,
                original_resin_cost: AUTO_LEY_LINE_ORIGINAL_RESIN_COST,
                half_original_resin_cost: AUTO_LEY_LINE_HALF_ORIGINAL_RESIN_COST,
                condensed_resin_counts_as_one_run: true,
                transient_resin_enabled: true,
                fragile_resin_enabled: true,
                recheck_after_run_when_resin_exhaustion_mode: false,
                max_recheck_count: AUTO_LEY_LINE_MAX_RECHECK_COUNT,
                recheck_ignores_counts_above: 50,
                reward_priority_with_original_resin: strings(
                    AUTO_LEY_LINE_REWARD_RESIN_PRIORITY_WITH_ORIGINAL,
                ),
                reward_priority_when_original_resin_empty: strings(
                    AUTO_LEY_LINE_REWARD_RESIN_PRIORITY_EMPTY_ORIGINAL,
                ),
                double_reward_prefers_original_resin: true,
                synthesizer_flag_configured: false,
                synthesizer_flow_invoked_by_legacy_task: false,
            },
            locators: ley_line_locators(),
            steps: ley_line_steps(),
            executor_ready: true,
            pending_native: vec![
                "executor-ready Rust orchestration is available behind AutoLeyLineOutcropRuntime; desktop live adapters are not wired yet".to_string(),
                "full TpTask/PathExecutor pathing adapters and AutoFightTask/CombatScenes fight adapters remain pending native integration".to_string(),
            ],
        }
    }

    #[derive(Debug)]
    struct FakeAutoLeyLineOutcropRuntime {
        calls: Vec<AutoLeyLineOutcropRuntimeActionKind>,
        resin: VecDeque<AutoLeyLineResinInventory>,
        fight: VecDeque<AutoLeyLineFightOutcome>,
        claim: VecDeque<AutoLeyLineRewardClaimOutcome>,
        switch_next: VecDeque<AutoLeyLineSwitchNextOutcome>,
        cleanup_statuses: Vec<AutoLeyLineOutcropExecutionStatus>,
    }

    impl Default for FakeAutoLeyLineOutcropRuntime {
        fn default() -> Self {
            Self {
                calls: Vec::new(),
                resin: VecDeque::from([AutoLeyLineResinInventory {
                    original_resin: 40,
                    condensed_resin: 0,
                    transient_resin: 0,
                    fragile_resin: 0,
                }]),
                fight: VecDeque::from([AutoLeyLineFightOutcome::Succeeded { duration_ms: 1000 }]),
                claim: VecDeque::new(),
                switch_next: VecDeque::from([AutoLeyLineSwitchNextOutcome {
                    switched: true,
                    exhausted: false,
                }]),
                cleanup_statuses: Vec::new(),
            }
        }
    }

    impl FakeAutoLeyLineOutcropRuntime {
        fn with_resin(
            mut self,
            resin: impl IntoIterator<Item = AutoLeyLineResinInventory>,
        ) -> Self {
            self.resin = resin.into_iter().collect();
            self
        }

        fn with_fights(
            mut self,
            fights: impl IntoIterator<Item = AutoLeyLineFightOutcome>,
        ) -> Self {
            self.fight = fights.into_iter().collect();
            self
        }

        fn with_claims(
            mut self,
            claims: impl IntoIterator<Item = AutoLeyLineRewardClaimOutcome>,
        ) -> Self {
            self.claim = claims.into_iter().collect();
            self
        }
    }

    impl AutoLeyLineOutcropRuntime for FakeAutoLeyLineOutcropRuntime {
        fn prepare_auto_ley_line_outcrop(
            &mut self,
            _plan: &AutoLeyLineOutcropExecutionPlan,
        ) -> Result<AutoLeyLineStartupOutcome> {
            self.calls
                .push(AutoLeyLineOutcropRuntimeActionKind::PrepareStartup);
            Ok(AutoLeyLineStartupOutcome {
                prepared: true,
                returned_main_ui: true,
                combat_team_switched: true,
                custom_marks_closed: false,
                mask_overlay_enabled: true,
            })
        }

        fn count_auto_ley_line_outcrop_resin(
            &mut self,
            _plan: &AutoLeyLineOutcropExecutionPlan,
            _run_index: u32,
        ) -> Result<AutoLeyLineResinInventory> {
            self.calls
                .push(AutoLeyLineOutcropRuntimeActionKind::CountResin);
            Ok(self.resin.pop_front().unwrap_or_default())
        }

        fn discover_auto_ley_line_outcrop(
            &mut self,
            _plan: &AutoLeyLineOutcropExecutionPlan,
            _run_index: u32,
        ) -> Result<AutoLeyLineRuntimeDiscoveryOutcome> {
            self.calls
                .push(AutoLeyLineOutcropRuntimeActionKind::DiscoverLeyLine);
            Ok(AutoLeyLineRuntimeDiscoveryOutcome {
                found: true,
                position: Some(AutoLeyLineNodePosition { x: 10.0, y: 20.0 }),
                source: "fake".to_string(),
            })
        }

        fn prepare_auto_ley_line_outcrop_navigation(
            &mut self,
            _plan: &AutoLeyLineOutcropExecutionPlan,
            request: &AutoLeyLinePathingExecutionRequest,
        ) -> Result<AutoLeyLineNavigationPreparationOutcome> {
            self.calls
                .push(AutoLeyLineOutcropRuntimeActionKind::PrepareNavigation);
            Ok(AutoLeyLineNavigationPreparationOutcome {
                prepared: true,
                teleported: request.run_kind == AutoLeyLinePathingRunKind::Initial,
                route_count: request.routes.len() as u32,
            })
        }

        fn execute_auto_ley_line_outcrop_pathing(
            &mut self,
            _plan: &AutoLeyLineOutcropExecutionPlan,
            request: &AutoLeyLinePathingExecutionRequest,
        ) -> Result<AutoLeyLinePathingOutcome> {
            self.calls
                .push(AutoLeyLineOutcropRuntimeActionKind::RunPathing);
            Ok(AutoLeyLinePathingOutcome {
                completed: true,
                executed_routes: request.routes.clone(),
            })
        }

        fn execute_auto_ley_line_outcrop_fight(
            &mut self,
            _plan: &AutoLeyLineOutcropExecutionPlan,
            _run_index: u32,
        ) -> Result<AutoLeyLineFightOutcome> {
            self.calls
                .push(AutoLeyLineOutcropRuntimeActionKind::ExecuteFight);
            Ok(self
                .fight
                .pop_front()
                .unwrap_or(AutoLeyLineFightOutcome::Succeeded { duration_ms: 1000 }))
        }

        fn navigate_auto_ley_line_outcrop_reward(
            &mut self,
            _plan: &AutoLeyLineOutcropExecutionPlan,
            _run_index: u32,
        ) -> Result<AutoLeyLineRewardNavigationOutcome> {
            self.calls
                .push(AutoLeyLineOutcropRuntimeActionKind::NavigateRewardFlower);
            Ok(AutoLeyLineRewardNavigationOutcome {
                reached: true,
                attempts: 1,
            })
        }

        fn claim_auto_ley_line_outcrop_reward(
            &mut self,
            _plan: &AutoLeyLineOutcropExecutionPlan,
            claim_plan: &AutoLeyLineRewardClaimPlan,
        ) -> Result<AutoLeyLineRewardClaimOutcome> {
            self.calls
                .push(AutoLeyLineOutcropRuntimeActionKind::ClaimReward);
            Ok(self
                .claim
                .pop_front()
                .unwrap_or(AutoLeyLineRewardClaimOutcome::Claimed {
                    resin_kind: claim_plan.resin_kind,
                    consumed_original_resin: claim_plan.expected_original_resin_cost,
                }))
        }

        fn scan_auto_ley_line_outcrop_drops(
            &mut self,
            _plan: &AutoLeyLineOutcropExecutionPlan,
            _run_index: u32,
        ) -> Result<AutoLeyLineDropScanOutcome> {
            self.calls
                .push(AutoLeyLineOutcropRuntimeActionKind::ScanDropsAfterReward);
            Ok(AutoLeyLineDropScanOutcome {
                enabled: true,
                scanned: true,
                picked_count: 1,
            })
        }

        fn switch_auto_ley_line_outcrop_next(
            &mut self,
            _plan: &AutoLeyLineOutcropExecutionPlan,
            _completed_runs: u32,
        ) -> Result<AutoLeyLineSwitchNextOutcome> {
            self.calls
                .push(AutoLeyLineOutcropRuntimeActionKind::SwitchNextLeyLine);
            Ok(self
                .switch_next
                .pop_front()
                .unwrap_or(AutoLeyLineSwitchNextOutcome {
                    switched: true,
                    exhausted: false,
                }))
        }

        fn cleanup_auto_ley_line_outcrop(
            &mut self,
            _plan: &AutoLeyLineOutcropExecutionPlan,
            status: AutoLeyLineOutcropExecutionStatus,
        ) -> Result<AutoLeyLineCleanupOutcome> {
            self.calls
                .push(AutoLeyLineOutcropRuntimeActionKind::Cleanup);
            self.cleanup_statuses.push(status);
            Ok(AutoLeyLineCleanupOutcome {
                completed: true,
                returned_main_ui: true,
                custom_marks_restored: true,
                mask_overlay_restored: true,
            })
        }
    }

    #[test]
    fn auto_ley_line_outcrop_executor_single_round_success() {
        let plan = test_plan(1);
        let mut runtime = FakeAutoLeyLineOutcropRuntime::default();

        let report = execute_auto_ley_line_outcrop_plan(&plan, &mut runtime).unwrap();

        assert!(plan.executor_ready);
        assert!(plan
            .pending_native
            .iter()
            .any(|item| item.contains("desktop live adapters are not wired yet")));
        assert_eq!(report.status, AutoLeyLineOutcropExecutionStatus::Completed);
        assert!(report.completed);
        assert_eq!(report.state.fights_completed, 1);
        assert_eq!(report.state.reward_claims_completed, 1);
        assert_eq!(report.state.drop_scans, 1);
        assert_eq!(
            report
                .state
                .last_reward_claim_plan
                .as_ref()
                .unwrap()
                .resin_kind,
            AutoLeyLineResinKind::Original
        );
        assert!(runtime
            .calls
            .contains(&AutoLeyLineOutcropRuntimeActionKind::Cleanup));
    }

    #[test]
    fn auto_ley_line_outcrop_executor_skips_when_resin_is_insufficient() {
        let mut plan = test_plan(1);
        plan.reward_rule.scan_drops_after_reward_enabled = false;
        let mut runtime = FakeAutoLeyLineOutcropRuntime::default()
            .with_resin([AutoLeyLineResinInventory::default()]);

        let report = execute_auto_ley_line_outcrop_plan(&plan, &mut runtime).unwrap();

        assert_eq!(
            report.status,
            AutoLeyLineOutcropExecutionStatus::RewardSkipped
        );
        assert!(!report.completed);
        assert_eq!(report.state.reward_claims_completed, 0);
        assert_eq!(report.state.reward_skipped_count, 1);
        assert!(report.skipped_steps.iter().any(|step| matches!(
            step.reason,
            AutoLeyLineSkipReason::ResinUnavailable(AutoLeyLineRewardSkipReason::InsufficientResin)
        )));
        assert!(!runtime
            .calls
            .contains(&AutoLeyLineOutcropRuntimeActionKind::ClaimReward));
        assert_eq!(
            runtime.cleanup_statuses,
            vec![AutoLeyLineOutcropExecutionStatus::RewardSkipped]
        );
    }

    #[test]
    fn auto_ley_line_outcrop_executor_records_reward_not_claimed_skip() {
        let plan = test_plan(1);
        let mut runtime = FakeAutoLeyLineOutcropRuntime::default().with_claims([
            AutoLeyLineRewardClaimOutcome::Skipped {
                reason: AutoLeyLineRewardSkipReason::ClaimDisabled,
            },
        ]);

        let report = execute_auto_ley_line_outcrop_plan(&plan, &mut runtime).unwrap();

        assert_eq!(
            report.status,
            AutoLeyLineOutcropExecutionStatus::RewardSkipped
        );
        assert_eq!(
            report.state.last_reward_claim_outcome,
            Some(AutoLeyLineRewardClaimOutcome::Skipped {
                reason: AutoLeyLineRewardSkipReason::ClaimDisabled,
            })
        );
        assert!(report.skipped_steps.iter().any(|step| matches!(
            step.reason,
            AutoLeyLineSkipReason::RewardClaimSkipped(AutoLeyLineRewardSkipReason::ClaimDisabled)
        )));
        assert!(runtime
            .calls
            .contains(&AutoLeyLineOutcropRuntimeActionKind::Cleanup));
    }

    #[test]
    fn auto_ley_line_outcrop_executor_stops_on_fight_failure_when_requested() {
        let plan = test_plan(1);
        let mut runtime = FakeAutoLeyLineOutcropRuntime::default().with_fights([
            AutoLeyLineFightOutcome::Failed {
                reason: "no fight text".to_string(),
                stop: true,
            },
        ]);

        let report = execute_auto_ley_line_outcrop_plan(&plan, &mut runtime).unwrap();

        assert_eq!(
            report.status,
            AutoLeyLineOutcropExecutionStatus::FightFailedStopped
        );
        assert_eq!(report.state.fight_failures, 1);
        assert_eq!(report.state.reward_claims_completed, 0);
        assert!(report.skipped_steps.iter().any(|step| matches!(
            &step.reason,
            AutoLeyLineSkipReason::FightFailed { reason } if reason == "no fight text"
        )));
        assert!(!runtime
            .calls
            .contains(&AutoLeyLineOutcropRuntimeActionKind::NavigateRewardFlower));
        assert_eq!(
            runtime.cleanup_statuses,
            vec![AutoLeyLineOutcropExecutionStatus::FightFailedStopped]
        );
    }

    #[test]
    fn auto_ley_line_outcrop_executor_skips_failed_fight_and_runs_next_ley_line() {
        let plan = test_plan(1);
        let mut runtime = FakeAutoLeyLineOutcropRuntime::default().with_fights([
            AutoLeyLineFightOutcome::Failed {
                reason: "temporary failure".to_string(),
                stop: false,
            },
            AutoLeyLineFightOutcome::Succeeded { duration_ms: 900 },
        ]);

        let report = execute_auto_ley_line_outcrop_plan(&plan, &mut runtime).unwrap();

        assert_eq!(report.status, AutoLeyLineOutcropExecutionStatus::Completed);
        assert_eq!(report.state.fight_failures, 1);
        assert_eq!(report.state.fights_completed, 1);
        assert_eq!(report.state.next_ley_line_switches, 1);
        assert_eq!(report.state.reward_claims_completed, 1);
        assert_eq!(
            runtime
                .calls
                .iter()
                .filter(|kind| **kind == AutoLeyLineOutcropRuntimeActionKind::ExecuteFight)
                .count(),
            2
        );
        assert!(runtime
            .calls
            .contains(&AutoLeyLineOutcropRuntimeActionKind::Cleanup));
    }

    #[test]
    fn auto_ley_line_outcrop_executor_cleanup_runs_after_runtime_error() {
        #[derive(Debug, Default)]
        struct ErrorRuntime {
            cleanup_called: bool,
        }

        impl AutoLeyLineOutcropRuntime for ErrorRuntime {
            fn prepare_auto_ley_line_outcrop(
                &mut self,
                _plan: &AutoLeyLineOutcropExecutionPlan,
            ) -> Result<AutoLeyLineStartupOutcome> {
                Ok(AutoLeyLineStartupOutcome {
                    prepared: true,
                    returned_main_ui: true,
                    combat_team_switched: false,
                    custom_marks_closed: false,
                    mask_overlay_enabled: true,
                })
            }

            fn count_auto_ley_line_outcrop_resin(
                &mut self,
                _plan: &AutoLeyLineOutcropExecutionPlan,
                _run_index: u32,
            ) -> Result<AutoLeyLineResinInventory> {
                Ok(AutoLeyLineResinInventory {
                    original_resin: 40,
                    condensed_resin: 0,
                    transient_resin: 0,
                    fragile_resin: 0,
                })
            }

            fn discover_auto_ley_line_outcrop(
                &mut self,
                _plan: &AutoLeyLineOutcropExecutionPlan,
                _run_index: u32,
            ) -> Result<AutoLeyLineRuntimeDiscoveryOutcome> {
                Err(TaskError::CommonJobExecution(
                    "capture unavailable".to_string(),
                ))
            }

            fn prepare_auto_ley_line_outcrop_navigation(
                &mut self,
                _plan: &AutoLeyLineOutcropExecutionPlan,
                _request: &AutoLeyLinePathingExecutionRequest,
            ) -> Result<AutoLeyLineNavigationPreparationOutcome> {
                unreachable!()
            }

            fn execute_auto_ley_line_outcrop_pathing(
                &mut self,
                _plan: &AutoLeyLineOutcropExecutionPlan,
                _request: &AutoLeyLinePathingExecutionRequest,
            ) -> Result<AutoLeyLinePathingOutcome> {
                unreachable!()
            }

            fn execute_auto_ley_line_outcrop_fight(
                &mut self,
                _plan: &AutoLeyLineOutcropExecutionPlan,
                _run_index: u32,
            ) -> Result<AutoLeyLineFightOutcome> {
                unreachable!()
            }

            fn navigate_auto_ley_line_outcrop_reward(
                &mut self,
                _plan: &AutoLeyLineOutcropExecutionPlan,
                _run_index: u32,
            ) -> Result<AutoLeyLineRewardNavigationOutcome> {
                unreachable!()
            }

            fn claim_auto_ley_line_outcrop_reward(
                &mut self,
                _plan: &AutoLeyLineOutcropExecutionPlan,
                _claim_plan: &AutoLeyLineRewardClaimPlan,
            ) -> Result<AutoLeyLineRewardClaimOutcome> {
                unreachable!()
            }

            fn scan_auto_ley_line_outcrop_drops(
                &mut self,
                _plan: &AutoLeyLineOutcropExecutionPlan,
                _run_index: u32,
            ) -> Result<AutoLeyLineDropScanOutcome> {
                unreachable!()
            }

            fn switch_auto_ley_line_outcrop_next(
                &mut self,
                _plan: &AutoLeyLineOutcropExecutionPlan,
                _completed_runs: u32,
            ) -> Result<AutoLeyLineSwitchNextOutcome> {
                unreachable!()
            }

            fn cleanup_auto_ley_line_outcrop(
                &mut self,
                _plan: &AutoLeyLineOutcropExecutionPlan,
                status: AutoLeyLineOutcropExecutionStatus,
            ) -> Result<AutoLeyLineCleanupOutcome> {
                assert_eq!(status, AutoLeyLineOutcropExecutionStatus::RuntimeError);
                self.cleanup_called = true;
                Ok(AutoLeyLineCleanupOutcome {
                    completed: true,
                    returned_main_ui: true,
                    custom_marks_restored: true,
                    mask_overlay_restored: true,
                })
            }
        }

        let plan = test_plan(1);
        let mut runtime = ErrorRuntime::default();

        let error = execute_auto_ley_line_outcrop_plan(&plan, &mut runtime).unwrap_err();

        assert!(matches!(error, TaskError::CommonJobExecution(_)));
        assert!(runtime.cleanup_called);
    }
}
