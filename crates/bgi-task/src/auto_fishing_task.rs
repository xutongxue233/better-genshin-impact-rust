use bgi_core::AutoFishingConfig;
use bgi_input::{InputEvent, MouseButton};
use bgi_vision::{Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;

pub const AUTO_FISHING_TASK_KEY: &str = "AutoFishing";
pub const AUTO_FISHING_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_FISHING_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_FISHING_FISH_MODEL_NAME: &str = "BgiFish";
pub const AUTO_FISHING_FISH_MODEL_PATH: &str = "Assets/Model/Fish/bgi_fish.onnx";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingTaskExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub config_rule: AutoFishingTaskConfigRule,
    pub startup_rule: AutoFishingStartupRule,
    pub blackboard_rule: AutoFishingBlackboardRule,
    pub behavior_tree_rule: AutoFishingBehaviorTreeRule,
    pub time_policy_rule: AutoFishingTimePolicyRule,
    pub find_fish_rule: AutoFishingFindFishRule,
    pub enter_mode_rule: AutoFishingEnterModeRule,
    pub fish_model_rule: AutoFishingFishModelRule,
    pub bait_rule: AutoFishingBaitRule,
    pub choose_bait_rule: AutoFishingChooseBaitRule,
    pub throw_rod_rule: AutoFishingThrowRodRule,
    pub check_result_rule: AutoFishingCheckResultRule,
    pub rod_net_rule: AutoFishingRodNetRule,
    pub quit_rule: AutoFishingQuitRule,
    pub steps: Vec<AutoFishingTaskStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoFishingTaskExecutionConfig {
    pub capture_size: Size,
    pub auto_fishing_config: AutoFishingConfig,
    pub whole_process_timeout_seconds: Option<u64>,
    pub throw_rod_timeout_seconds: Option<u64>,
    pub fishing_time_policy: Option<Value>,
    pub save_screenshot_on_key_tick: bool,
}

impl Default for AutoFishingTaskExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_FISHING_DEFAULT_CAPTURE_WIDTH,
                AUTO_FISHING_DEFAULT_CAPTURE_HEIGHT,
            ),
            auto_fishing_config: AutoFishingConfig::default(),
            whole_process_timeout_seconds: None,
            throw_rod_timeout_seconds: None,
            fishing_time_policy: None,
            save_screenshot_on_key_tick: false,
        }
    }
}

impl AutoFishingTaskExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };
        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }

        let auto_fishing_value = value
            .get("autoFishingConfig")
            .or_else(|| value.get("AutoFishingConfig"))
            .or_else(|| value.get("auto_fishing_config"))
            .unwrap_or(value);
        config.auto_fishing_config =
            serde_json::from_value(auto_fishing_value.clone()).unwrap_or_default();

        config.whole_process_timeout_seconds = u64_member(
            value,
            [
                "wholeProcessTimeoutSeconds",
                "WholeProcessTimeoutSeconds",
                "whole_process_timeout_seconds",
            ],
        );
        config.throw_rod_timeout_seconds = u64_member(
            value,
            [
                "throwRodTimeOutTimeoutSeconds",
                "ThrowRodTimeOutTimeoutSeconds",
                "throwRodTimeoutSeconds",
                "throw_rod_timeout_seconds",
            ],
        );
        config.fishing_time_policy = value
            .get("fishingTimePolicy")
            .or_else(|| value.get("FishingTimePolicy"))
            .or_else(|| value.get("fishing_time_policy"))
            .cloned();
        config.save_screenshot_on_key_tick = bool_member(
            value,
            [
                "saveScreenshotOnKeyTick",
                "SaveScreenshotOnKeyTick",
                "save_screenshot_on_key_tick",
            ],
        )
        .unwrap_or(false);
        config
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingTaskConfigRule {
    pub whole_process_timeout_seconds: u64,
    pub throw_rod_timeout_seconds: u64,
    pub fishing_time_policy: AutoFishingTimePolicy,
    pub fishing_time_policy_raw: Value,
    pub save_screenshot_on_key_tick: bool,
    pub torch_dll_full_path: String,
    pub use_torch_probe_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AutoFishingTimePolicy {
    All,
    Daytime,
    Nighttime,
    DontChange,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingStartupRule {
    pub task_name: String,
    pub disables_realtime_auto_fishing_config: bool,
    pub warns_about_pets: bool,
    pub requires_active_genshin_window: bool,
    pub captures_with_no_retry: bool,
    pub manual_gc_interval_seconds: u64,
    pub always_quits_fishing_mode_at_end: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingBlackboardRule {
    pub tracks_abort: bool,
    pub tracks_selected_bait: bool,
    pub tracks_fishpond: bool,
    pub tracks_throw_rod_no_target: bool,
    pub tracks_throw_rod_no_target_times: bool,
    pub tracks_throw_rod_no_bait_fish: bool,
    pub tracks_throw_rod_no_bait_fish_failures: bool,
    pub tracks_fish_box_rect: bool,
    pub choose_bait_ui_blocks_fishing_ui_detection: bool,
    pub tracks_choose_bait_failures: bool,
    pub pitch_reset_initial_value: bool,
    pub reset_preserves_fishpond_and_throw_rod_flags: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingBehaviorTreeRule {
    pub root_sequence: String,
    pub whole_process_parallel_policy: String,
    pub initial_find_fish_timeout_seconds: u64,
    pub loop_find_fish_timeout_seconds: u64,
    pub throw_rod_parallel_policy: String,
    pub bite_parallel_policy: String,
    pub pull_bar_parallel_policy: String,
    pub ordered_stages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingTimePolicyRule {
    pub skips_time_setting_in_multiplayer: bool,
    pub dont_change_runs_once: bool,
    pub daytime_hours: Vec<u8>,
    pub nighttime_hours: Vec<u8>,
    pub all_policy_hours: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingTimePolicyRound {
    pub set_time_hour: Option<u8>,
    pub set_time_minute: Option<u8>,
    pub run_tick_around: bool,
    pub resets_blackboard_before_round: bool,
    pub resets_manual_gc_timer_before_round: bool,
    pub reason: AutoFishingTimePolicyRoundReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingTimePolicyRoundReason {
    MultiplayerSkipsTimeSetting,
    DontChange,
    Daytime,
    Nighttime,
    AllOrFallback,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingFindFishRule {
    pub move_viewpoint_down_dy: i32,
    pub move_viewpoint_down_sleep_ms: u64,
    pub turn_no_fish_dx: i32,
    pub turn_after_move_sleep_ms: u64,
    pub fishpond_detected_overlay_sleep_ms: u64,
    pub fishpond_right_threshold_ratio: f64,
    pub fishpond_left_threshold_ratio: f64,
    pub align_backward_vk: u16,
    pub align_forward_vk: u16,
    pub align_key_hold_ms: u64,
    pub align_between_key_sleep_ms: u64,
    pub align_final_sleep_ms: u64,
    pub initial_state_theta_step_radians: f64,
    pub initial_state_rho_base: f64,
    pub initial_state_rho_theta_multiplier: f64,
    pub initial_state_move_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingEnterModeRule {
    pub localized_fishing_text_default: String,
    pub overall_wait_seconds: u64,
    pub press_f_retry_seconds: u64,
    pub click_white_confirm_retry_seconds: u64,
    pub white_confirm_pre_click_delay_ms: u64,
    pub interact_vk: u16,
    pub initial_bait_icon_crop_ratio: AutoFishingRatioRect,
    pub grid_icon_input_size: Size,
    pub marks_pitch_reset_after_confirm: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingFishModelRule {
    pub yolo_model_name: String,
    pub yolo_model: String,
    pub grid_icon_model_loaded_for_bait: bool,
    pub confidence_min: f64,
    pub ignore_obtained_left_tip_rect_ratio: AutoFishingRatioRect,
    pub ignore_obtained_center_tip_rect_ratio: AutoFishingRatioRect,
    pub fish_types: Vec<AutoFishingBigFishTypeRule>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingRatioRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingBigFishTypeRule {
    pub name: String,
    pub bait: AutoFishingBaitType,
    pub chinese_name: String,
    pub net_index: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingBaitType {
    FruitPasteBait,
    RedrotBait,
    FalseWormBait,
    FakeFlyBait,
    SugardewBait,
    SourBait,
    FlashingMaintenanceMekBait,
    SpinelgrainBait,
    EmberglowBait,
    BerryBait,
    RefreshingLakkaBait,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingBaitRule {
    pub bait_types: Vec<AutoFishingBaitType>,
    pub selected_bait_tracked_in_blackboard: bool,
    pub failed_baits_tracked_in_blackboard: bool,
    pub no_bait_fish_failures_before_switch: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingTargetFishCandidate {
    pub fish_type_name: String,
    pub bait: AutoFishingBaitType,
    pub net_index: u8,
    pub rect: Rect,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingTargetFishSelectionObservation {
    pub selected_bait: Option<AutoFishingBaitType>,
    pub throw_rod_no_bait_fish_failures: Vec<AutoFishingBaitType>,
    pub target_rect: Rect,
    pub fishes: Vec<AutoFishingTargetFishCandidate>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingTargetFishSelectionReport {
    pub ignored_baits: Vec<AutoFishingBaitType>,
    pub eligible_indices: Vec<usize>,
    pub selected_index: Option<usize>,
    pub selected_fish: Option<AutoFishingTargetFishCandidate>,
    pub selected_distance: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingFishpondSnapshot {
    pub fishpond_rect: Option<Rect>,
    pub fishes: Vec<AutoFishingTargetFishCandidate>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingBlackboardState {
    pub abort: bool,
    pub selected_bait: Option<AutoFishingBaitType>,
    pub fishpond: Option<AutoFishingFishpondSnapshot>,
    pub throw_rod_no_target: bool,
    pub throw_rod_no_target_times: u64,
    pub throw_rod_no_bait_fish: bool,
    pub throw_rod_no_bait_fish_failures: Vec<AutoFishingBaitType>,
    pub fish_box_rect: Option<Rect>,
    pub choose_bait_ui_opening: bool,
    pub choose_bait_failures: Vec<AutoFishingBaitType>,
    pub pitch_reset: bool,
}

impl Default for AutoFishingBlackboardState {
    fn default() -> Self {
        Self {
            abort: false,
            selected_bait: None,
            fishpond: None,
            throw_rod_no_target: false,
            throw_rod_no_target_times: 0,
            throw_rod_no_bait_fish: false,
            throw_rod_no_bait_fish_failures: Vec::new(),
            fish_box_rect: None,
            choose_bait_ui_opening: false,
            choose_bait_failures: Vec::new(),
            pitch_reset: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingBlackboardResetReport {
    pub next_state: AutoFishingBlackboardState,
    pub cleared_abort_was: bool,
    pub cleared_selected_bait: Option<AutoFishingBaitType>,
    pub cleared_throw_rod_no_target_times: u64,
    pub cleared_throw_rod_no_bait_fish_failures_len: usize,
    pub cleared_fish_box_rect_was_non_empty: bool,
    pub cleared_choose_bait_ui_opening_was: bool,
    pub cleared_choose_bait_failures_len: usize,
    pub pitch_reset_was: bool,
    pub preserved_fishpond: bool,
    pub preserved_throw_rod_no_target: bool,
    pub preserved_throw_rod_no_bait_fish: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingThrowRodInitializeReport {
    pub next_state: AutoFishingBlackboardState,
    pub cleared_stale_throw_rod_no_target: bool,
    pub cleared_stale_throw_rod_no_bait_fish: bool,
    pub sets_pitch_reset: bool,
    pub input_events: Vec<InputEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingBaitCount {
    pub bait: AutoFishingBaitType,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingFishpondAvailabilityObservation {
    pub choose_bait_failures: Vec<AutoFishingBaitType>,
    pub throw_rod_no_bait_fish_failures: Vec<AutoFishingBaitType>,
    pub fishes: Vec<AutoFishingTargetFishCandidate>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingFishpondAvailabilityReport {
    pub choose_bait_ignored_baits: Vec<AutoFishingBaitType>,
    pub throw_rod_ignored_baits: Vec<AutoFishingBaitType>,
    pub available_baits: Vec<AutoFishingBaitType>,
    pub has_available_fish: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingBaitSelectionObservation {
    pub current_selected_bait: Option<AutoFishingBaitType>,
    pub choose_bait_failures: Vec<AutoFishingBaitType>,
    pub fishes: Vec<AutoFishingTargetFishCandidate>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingBaitSelectionReport {
    pub keeps_current_bait: bool,
    pub ignored_baits: Vec<AutoFishingBaitType>,
    pub eligible_bait_counts: Vec<AutoFishingBaitCount>,
    pub selected_bait: Option<AutoFishingBaitType>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingChooseBaitRule {
    pub max_failed_times_before_ignore: u64,
    pub opens_ui_with_right_click: bool,
    pub ui_open_wait_seconds: u64,
    pub post_right_click_sleep_ms: u64,
    pub mouse_move_after_open_dx: i32,
    pub mouse_move_after_open_dy: i32,
    pub post_mouse_move_sleep_ms: u64,
    pub grid_crop_ratio: AutoFishingRatioRect,
    pub canny_low_threshold: f64,
    pub canny_high_threshold: f64,
    pub min_contour_width_screen_ratio: f64,
    pub min_contour_width_multiplier: f64,
    pub bait_icon_aspect_ratio: f64,
    pub bait_icon_aspect_tolerance: f64,
    pub fixed_post_select_click_x_ratio: f64,
    pub fixed_post_select_click_y_ratio: f64,
    pub post_bait_click_sleep_ms: u64,
    pub post_fixed_click_sleep_ms: u64,
    pub close_after_success_sleep_ms: u64,
    pub confirm_template_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingThrowRodRule {
    pub hold_left_button_on_initialize: bool,
    pub sets_pitch_reset_on_initialize: bool,
    pub ignore_obtained_seconds: u64,
    pub find_target_timeout_seconds: u64,
    pub initial_mouse_sweep_step_pixels: i32,
    pub initial_mouse_sweep_angle_step_radians: f64,
    pub no_drop_point_failures_before_abort: u64,
    pub no_placement_retries_before_restart: u64,
    pub no_target_fish_retries_before_switch_bait: u64,
    pub max_no_bait_fish_failures_before_ignore: u64,
    pub random_move_base_pixels: i32,
    pub retry_release_sleep_ms: u64,
    pub retry_click_after_release_sleep_ms: u64,
    pub mid_cast_retry_sleep_ms: u64,
    pub rod_input_normalized_size: Size,
    pub state_failed_random_move: i8,
    pub too_close_minimum_step: f64,
    pub too_close_dx_divisor: f64,
    pub too_close_dy_multiplier: f64,
    pub too_far_dx_divisor: f64,
    pub too_far_dy_multiplier: f64,
    pub sleep_after_adjustment_uses_distance: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingCheckResultRule {
    pub check_throw_rod_delay_seconds: u64,
    pub check_throw_rod_failure_template: String,
    pub fish_bite_timeout_seconds: u64,
    pub fish_bite_timeout_clicks_left_button: bool,
    pub fish_bite_timeout_after_click_seconds: u64,
    pub check_raise_hook_delay_seconds: u64,
    pub check_raise_hook_failure_template: String,
    pub get_fish_box_area_timeout_seconds: u64,
    pub fish_box_rect_max_height_delta: i32,
    pub fishing_no_detection_finish_grace_seconds: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingBehaviorStatus {
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingTimeoutObservation {
    pub timeout_elapsed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingTimeoutReport {
    pub status: AutoFishingBehaviorStatus,
    pub sets_abort: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingTickAroundObservation {
    pub cancellation_requested: bool,
    pub genshin_active: bool,
    pub capture_succeeded: bool,
    pub behavior_tree_status: AutoFishingBehaviorStatus,
    pub manual_gc_due: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingTickAroundStopReason {
    CancellationRequested,
    InactiveGameWindow,
    BehaviorTreeStopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingTickAroundReport {
    pub continue_loop: bool,
    pub should_tick_behavior_tree: bool,
    pub stop_reason: Option<AutoFishingTickAroundStopReason>,
    pub finished_behavior_status: Option<AutoFishingBehaviorStatus>,
    pub collect_gc: bool,
    pub updates_manual_gc_timestamp: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingMoveViewpointDownObservation {
    pub pitch_reset: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingMoveViewpointDownReport {
    pub next_pitch_reset: bool,
    pub status: AutoFishingBehaviorStatus,
    pub input_events: Vec<InputEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct AutoFishingInitialStateObservation {
    pub bait_button_present: bool,
    pub theta: f64,
    pub move_interval_elapsed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct AutoFishingInitialStateReport {
    pub next_theta: f64,
    pub status: AutoFishingBehaviorStatus,
    pub reschedule_after_ms: Option<u64>,
    pub mouse_move: Option<(i32, i32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingTurnAroundObservation {
    pub capture_size: Size,
    pub fishpond_rect: Option<Rect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingTurnAroundDecisionKind {
    NoFishSweep,
    FishpondTooFarRight,
    FishpondTooFarLeft,
    AlignCharacterAndCamera,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingTurnAroundReport {
    pub status: AutoFishingBehaviorStatus,
    pub decision: AutoFishingTurnAroundDecisionKind,
    pub clears_overlay: bool,
    pub input_events: Vec<InputEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingEnterModeState {
    pub initialized: bool,
    pub selected_bait: Option<AutoFishingBaitType>,
    pub pitch_reset: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingEnterModeObservation {
    pub state: AutoFishingEnterModeState,
    pub press_f_cooldown_elapsed: bool,
    pub click_white_confirm_cooldown_elapsed: bool,
    pub f_fishing_text_visible: bool,
    pub white_confirm_button_present: bool,
    pub exit_fishing_button_present: bool,
    pub overall_timeout_elapsed: bool,
    pub inferred_bait_after_confirm: Option<AutoFishingBaitType>,
    pub capture_size: Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingEnterModeDecisionKind {
    StartOverallWait,
    PressFishingKey,
    ClickWhiteConfirm,
    Entered,
    WaitingForExitButton,
    FailedTimeout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingEnterModeReport {
    pub next_state: AutoFishingEnterModeState,
    pub status: AutoFishingBehaviorStatus,
    pub decision: AutoFishingEnterModeDecisionKind,
    pub input_events: Vec<InputEvent>,
    pub clicks_white_confirm: bool,
    pub white_confirm_pre_click_delay_ms: Option<u64>,
    pub bait_icon_crop: Option<AutoFishingBaitIconCropPlan>,
    pub starts_overall_timeout_ms: Option<u64>,
    pub reschedule_press_f_after_ms: Option<u64>,
    pub reschedule_click_white_confirm_after_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingBaitIconCropPlan {
    pub crop_rect: Rect,
    pub resize_size: Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingEnterModeError {
    InvalidCaptureSize,
    InvalidCropRect,
    NonFiniteComputation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingDelayedTemplateCheckObservation {
    pub delay_elapsed: bool,
    pub has_checked: bool,
    pub failure_template_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingDelayedTemplateCheckReport {
    pub next_has_checked: bool,
    pub status: AutoFishingBehaviorStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingFishBiteTimeoutObservation {
    pub timeout_elapsed: bool,
    pub left_button_clicked: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingFishBiteTimeoutReport {
    pub next_left_button_clicked: bool,
    pub status: AutoFishingBehaviorStatus,
    pub input_events: Vec<InputEvent>,
    pub reschedule_after_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingFishBiteMethod {
    WordBlock,
    LiftRodButton,
    Ocr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingFishBiteObservation {
    pub word_block_detected: bool,
    pub lift_rod_button_present: bool,
    pub ocr_text: Option<String>,
    pub localized_get_bite_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingFishBiteReport {
    pub status: AutoFishingBehaviorStatus,
    pub method: Option<AutoFishingFishBiteMethod>,
    pub removes_fish_bite_tips_overlay: bool,
    pub input_events: Vec<InputEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingPullBarState {
    pub previous_left_button_down: bool,
    pub no_detection_armed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingPullBarObservation {
    pub state: AutoFishingPullBarState,
    pub fish_bar_rects: Vec<Rect>,
    pub no_detection_grace_elapsed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingPullBarDecisionKind {
    PressLeftButton,
    ReleaseLeftButton,
    KeepLeftButtonDown,
    KeepLeftButtonUp,
    InvalidTwoRectTarget,
    IgnoreUnexpectedRectCount,
    ArmNoDetectionGrace,
    WaitNoDetectionGrace,
    CompleteNoDetection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingPullBarReport {
    pub next_state: AutoFishingPullBarState,
    pub status: AutoFishingBehaviorStatus,
    pub decision: AutoFishingPullBarDecisionKind,
    pub considered_rects: Vec<Rect>,
    pub removes_fish_box_overlay: bool,
    pub clears_bar_overlay: bool,
    pub input_events: Vec<InputEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingFishBoxAreaObservation {
    pub timeout_elapsed: bool,
    pub capture_size: Size,
    pub top_rects: Vec<Rect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingFishBoxAreaDecisionKind {
    Timeout,
    WaitingForTwoRects,
    HeightMismatch,
    InvalidGeometry,
    Detected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingFishBoxAreaReport {
    pub status: AutoFishingBehaviorStatus,
    pub decision: AutoFishingFishBoxAreaDecisionKind,
    pub fish_box_rect: Option<Rect>,
    pub cursor_rect: Option<Rect>,
    pub target_rect: Option<Rect>,
    pub error_screenshot_recommended: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingQuitModeObservation {
    pub first_tick: bool,
    pub f_fishing_text_visible: bool,
    pub black_confirm_button_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingQuitModeDecisionKind {
    InitialWait,
    AlreadyExited,
    ClickBlackConfirm,
    PressEscape,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingQuitModeReport {
    pub status: AutoFishingBehaviorStatus,
    pub decision: AutoFishingQuitModeDecisionKind,
    pub clicks_black_confirm: bool,
    pub input_events: Vec<InputEvent>,
    pub sleep_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingRodNetRule {
    pub model_input_size: Size,
    pub alpha: f64,
    pub label_count: usize,
    pub state_ready: u8,
    pub state_too_close: u8,
    pub state_too_far: u8,
    pub offset_values: Vec<f64>,
    pub dz_values: Vec<f64>,
    pub h_coeff_values: Vec<f64>,
    pub weight_values: Vec<[f64; 3]>,
    pub bias_values: Vec<[f64; 3]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct AutoFishingRodInput {
    pub rod_x1: f64,
    pub rod_x2: f64,
    pub rod_y1: f64,
    pub rod_y2: f64,
    pub fish_x1: f64,
    pub fish_x2: f64,
    pub fish_y1: f64,
    pub fish_y2: f64,
    pub fish_label: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct AutoFishingRodPreprocessResult {
    pub y0: f64,
    pub z0: f64,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub h: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingRodScoreReport {
    pub preprocess: AutoFishingRodPreprocessResult,
    pub dist: f64,
    pub logits: [f64; 3],
    pub scores: [f64; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingRodState {
    Ready,
    TooClose,
    TooFar,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AutoFishingRodNetError {
    InvalidFishLabel {
        fish_label: usize,
        label_count: usize,
    },
    InvalidParameterShape,
    NonFiniteComputation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingThrowRodClassification {
    Failed,
    Ready,
    TooClose,
    TooFar,
    Unknown(i32),
}

impl From<AutoFishingRodState> for AutoFishingThrowRodClassification {
    fn from(value: AutoFishingRodState) -> Self {
        match value {
            AutoFishingRodState::Ready => Self::Ready,
            AutoFishingRodState::TooClose => Self::TooClose,
            AutoFishingRodState::TooFar => Self::TooFar,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingRandomMoveSample {
    pub random_x: u32,
    pub random_y: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoFishingThrowRodGeometryObservation {
    pub capture_size: Size,
    pub rod_rect: Rect,
    pub fish_rect: Rect,
    pub fish_label: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct AutoFishingThrowRodGeometryReport {
    pub scale_size: Size,
    pub rod_input: AutoFishingRodInput,
    pub delta_x: f64,
    pub delta_y: f64,
    pub distance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct AutoFishingThrowRodAdjustmentObservation {
    pub rod_input: AutoFishingRodInput,
    pub classification: AutoFishingThrowRodClassification,
    pub capture_size: Size,
    pub random_sample: Option<AutoFishingRandomMoveSample>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingThrowRodAdjustmentKind {
    FailedRandomMove,
    ReleaseRod,
    MoveAwayFromFish,
    MoveTowardFish,
    Noop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingThrowRodAdjustmentStatus {
    Running,
    Succeeded,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishingThrowRodAdjustmentReport {
    pub kind: AutoFishingThrowRodAdjustmentKind,
    pub status: AutoFishingThrowRodAdjustmentStatus,
    pub delta_x: f64,
    pub delta_y: f64,
    pub distance: f64,
    pub input_events: Vec<InputEvent>,
    pub sleep_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AutoFishingThrowRodAdjustmentError {
    MissingRandomSample,
    InvalidCaptureSize,
    NonFiniteComputation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingQuitRule {
    pub localized_fishing_text_default: String,
    pub succeeds_when_find_f_fishing_text: bool,
    pub clicks_black_confirm_when_present: bool,
    pub black_confirm_sleep_ms: u64,
    pub escape_retry_sleep_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishingTaskStep {
    pub phase: AutoFishingTaskPhase,
    pub action: AutoFishingTaskAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingTaskPhase {
    Startup,
    TimePolicy,
    FindFish,
    EnterFishingMode,
    ChooseBait,
    ThrowRod,
    BiteAndRaiseRod,
    PullFishingBar,
    QuitFishingMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishingTaskAction {
    DisableRealtimeAutoFish,
    ApplyFishingTimePolicy,
    MoveViewpointDown,
    DetectFishpondWithYolo,
    EnterFishingMode,
    ChooseBaitWithGridIconModel,
    ThrowRodWithRodNet,
    CheckThrowRodResult,
    DetectBiteAndRaiseRod,
    WaitBiteTimeout,
    CheckRaiseHookResult,
    PullFishingBar,
    QuitFishingMode,
}

pub fn plan_auto_fishing_task(
    config: AutoFishingTaskExecutionConfig,
) -> AutoFishingTaskExecutionPlan {
    let auto_config = config.auto_fishing_config;
    let policy_raw = config
        .fishing_time_policy
        .unwrap_or_else(|| auto_config.fishing_time_policy.clone());
    let whole_process_timeout_seconds = config
        .whole_process_timeout_seconds
        .unwrap_or(auto_config.whole_process_timeout_seconds);
    let throw_rod_timeout_seconds = config
        .throw_rod_timeout_seconds
        .unwrap_or(auto_config.auto_throw_rod_time_out);
    AutoFishingTaskExecutionPlan {
        task_key: AUTO_FISHING_TASK_KEY.to_string(),
        display_name: "Auto Fishing Task".to_string(),
        capture_size: config.capture_size,
        config_rule: AutoFishingTaskConfigRule {
            whole_process_timeout_seconds,
            throw_rod_timeout_seconds,
            fishing_time_policy: fishing_time_policy(&policy_raw),
            fishing_time_policy_raw: policy_raw,
            save_screenshot_on_key_tick: config.save_screenshot_on_key_tick,
            torch_dll_full_path: auto_config.torch_dll_full_path,
            use_torch_probe_required: true,
        },
        startup_rule: AutoFishingStartupRule {
            task_name: "钓鱼独立任务".to_string(),
            disables_realtime_auto_fishing_config: true,
            warns_about_pets: true,
            requires_active_genshin_window: true,
            captures_with_no_retry: true,
            manual_gc_interval_seconds: 2,
            always_quits_fishing_mode_at_end: true,
        },
        blackboard_rule: AutoFishingBlackboardRule {
            tracks_abort: true,
            tracks_selected_bait: true,
            tracks_fishpond: true,
            tracks_throw_rod_no_target: true,
            tracks_throw_rod_no_target_times: true,
            tracks_throw_rod_no_bait_fish: true,
            tracks_throw_rod_no_bait_fish_failures: true,
            tracks_fish_box_rect: true,
            choose_bait_ui_blocks_fishing_ui_detection: true,
            tracks_choose_bait_failures: true,
            pitch_reset_initial_value: true,
            reset_preserves_fishpond_and_throw_rod_flags: true,
        },
        behavior_tree_rule: AutoFishingBehaviorTreeRule {
            root_sequence: "钓鱼并确保完成后退出钓鱼模式".to_string(),
            whole_process_parallel_policy: "OnlyOneMustSucceed".to_string(),
            initial_find_fish_timeout_seconds: 20,
            loop_find_fish_timeout_seconds: 10,
            throw_rod_parallel_policy: "OnlyOneMustSucceed".to_string(),
            bite_parallel_policy: "OnlyOneMustSucceed".to_string(),
            pull_bar_parallel_policy: "OnlyOneMustSucceed".to_string(),
            ordered_stages: vec![
                "MoveViewpointDown".to_string(),
                "FindFishTimeout(20)".to_string(),
                "EnterFishingMode".to_string(),
                "CheckInitalState".to_string(),
                "GetFishpond".to_string(),
                "ChooseBait".to_string(),
                "ThrowRod".to_string(),
                "CheckThrowRod".to_string(),
                "FishBite".to_string(),
                "FishBiteTimeout".to_string(),
                "CheckRaiseHook".to_string(),
                "GetFishBoxArea".to_string(),
                "Fishing".to_string(),
                "WholeProcessTimeout".to_string(),
                "QuitFishingMode".to_string(),
            ],
        },
        time_policy_rule: AutoFishingTimePolicyRule {
            skips_time_setting_in_multiplayer: true,
            dont_change_runs_once: true,
            daytime_hours: vec![7],
            nighttime_hours: vec![19],
            all_policy_hours: vec![7, 19],
        },
        find_fish_rule: AutoFishingFindFishRule {
            move_viewpoint_down_dy: 500,
            move_viewpoint_down_sleep_ms: 100,
            turn_no_fish_dx: 100,
            turn_after_move_sleep_ms: 100,
            fishpond_detected_overlay_sleep_ms: 1_000,
            fishpond_right_threshold_ratio: 0.75,
            fishpond_left_threshold_ratio: 0.25,
            align_backward_vk: 0x53,
            align_forward_vk: 0x57,
            align_key_hold_ms: 100,
            align_between_key_sleep_ms: 400,
            align_final_sleep_ms: 300,
            initial_state_theta_step_radians: std::f64::consts::PI / 10.0,
            initial_state_rho_base: 10.0,
            initial_state_rho_theta_multiplier: 2.0,
            initial_state_move_interval_ms: 100,
        },
        enter_mode_rule: AutoFishingEnterModeRule {
            localized_fishing_text_default: "钓鱼".to_string(),
            overall_wait_seconds: 10,
            press_f_retry_seconds: 3,
            click_white_confirm_retry_seconds: 3,
            white_confirm_pre_click_delay_ms: 500,
            interact_vk: 0x46,
            initial_bait_icon_crop_ratio: AutoFishingRatioRect {
                x: 0.824,
                y: 0.669,
                width: 0.065,
                height: 0.065,
            },
            grid_icon_input_size: Size::new(125, 125),
            marks_pitch_reset_after_confirm: true,
        },
        fish_model_rule: AutoFishingFishModelRule {
            yolo_model_name: AUTO_FISHING_FISH_MODEL_NAME.to_string(),
            yolo_model: AUTO_FISHING_FISH_MODEL_PATH.to_string(),
            grid_icon_model_loaded_for_bait: true,
            confidence_min: 0.4,
            ignore_obtained_left_tip_rect_ratio: AutoFishingRatioRect {
                x: 0.04375,
                y: 0.4666,
                width: 0.1,
                height: 0.1,
            },
            ignore_obtained_center_tip_rect_ratio: AutoFishingRatioRect {
                x: 0.4,
                y: 0.445,
                width: 0.2,
                height: 0.06125,
            },
            fish_types: big_fish_types(),
        },
        bait_rule: AutoFishingBaitRule {
            bait_types: bait_types(),
            selected_bait_tracked_in_blackboard: true,
            failed_baits_tracked_in_blackboard: true,
            no_bait_fish_failures_before_switch: 10,
        },
        choose_bait_rule: AutoFishingChooseBaitRule {
            max_failed_times_before_ignore: 2,
            opens_ui_with_right_click: true,
            ui_open_wait_seconds: 3,
            post_right_click_sleep_ms: 100,
            mouse_move_after_open_dx: 0,
            mouse_move_after_open_dy: 200,
            post_mouse_move_sleep_ms: 500,
            grid_crop_ratio: AutoFishingRatioRect {
                x: 0.28,
                y: 0.37,
                width: 0.45,
                height: 0.22,
            },
            canny_low_threshold: 20.0,
            canny_high_threshold: 40.0,
            min_contour_width_screen_ratio: 0.065,
            min_contour_width_multiplier: 0.80,
            bait_icon_aspect_ratio: 0.81,
            bait_icon_aspect_tolerance: 0.05,
            fixed_post_select_click_x_ratio: 0.675,
            fixed_post_select_click_y_ratio: 1.0 / 3.0,
            post_bait_click_sleep_ms: 700,
            post_fixed_click_sleep_ms: 200,
            close_after_success_sleep_ms: 500,
            confirm_template_name: "BtnWhiteConfirm".to_string(),
        },
        throw_rod_rule: AutoFishingThrowRodRule {
            hold_left_button_on_initialize: true,
            sets_pitch_reset_on_initialize: true,
            ignore_obtained_seconds: 6,
            find_target_timeout_seconds: 5,
            initial_mouse_sweep_step_pixels: 80,
            initial_mouse_sweep_angle_step_radians: std::f64::consts::PI / 16.0,
            no_drop_point_failures_before_abort: 2,
            no_placement_retries_before_restart: 25,
            no_target_fish_retries_before_switch_bait: 10,
            max_no_bait_fish_failures_before_ignore: 2,
            random_move_base_pixels: 100,
            retry_release_sleep_ms: 2_000,
            retry_click_after_release_sleep_ms: 800,
            mid_cast_retry_sleep_ms: 2_000,
            rod_input_normalized_size: Size::new(1024, 576),
            state_failed_random_move: -1,
            too_close_minimum_step: 30.0,
            too_close_dx_divisor: 1.5,
            too_close_dy_multiplier: 1.5,
            too_far_dx_divisor: 1.5,
            too_far_dy_multiplier: 1.5,
            sleep_after_adjustment_uses_distance: true,
        },
        check_result_rule: AutoFishingCheckResultRule {
            check_throw_rod_delay_seconds: 3,
            check_throw_rod_failure_template: "BaitButtonRo".to_string(),
            fish_bite_timeout_seconds: throw_rod_timeout_seconds,
            fish_bite_timeout_clicks_left_button: true,
            fish_bite_timeout_after_click_seconds: 2,
            check_raise_hook_delay_seconds: 3,
            check_raise_hook_failure_template: "WaitBiteButtonRo".to_string(),
            get_fish_box_area_timeout_seconds: 5,
            fish_box_rect_max_height_delta: 10,
            fishing_no_detection_finish_grace_seconds: 1,
        },
        rod_net_rule: AutoFishingRodNetRule {
            model_input_size: Size::new(1024, 576),
            alpha: 1734.34 / 2.5,
            label_count: 11,
            state_ready: 0,
            state_too_close: 1,
            state_too_far: 2,
            offset_values: vec![0.8, 0.4, 0.35, 0.35, 0.6, 0.3, 0.3, 0.8, 0.8, 0.8, 0.8],
            dz_values: vec![
                1.0307939, 1.5887239, 1.4377865, 0.8548809, 1.8640924, -0.1687729, 1.8621461,
                0.7167622, 1.7071064, 1.8727832, 0.5531539,
            ],
            h_coeff_values: vec![
                0.5840698, 0.8029298, 0.6090596, -0.1390072, 0.7214464, -0.6076725, 0.3286690,
                -0.2991239, 0.6072225, 0.7662407, -0.3689651,
            ],
            weight_values: vec![
                [0.7779633, -1.7124480, 2.7366412],
                [-0.0381155, -1.6536976, 3.5904298],
                [0.1947731, -0.0445049, 0.8416666],
                [-0.0331017, -1.3641578, 1.2834741],
                [1.0268835, -1.6553984, 2.9930501],
                [0.0108103, -0.8515291, 1.0032536],
                [-0.0746362, -0.9677668, 0.7450780],
                [0.7382144, -9.5275803, 2.6134675],
                [-0.3597502, -1.7422760, 1.4354013],
                [-0.0578425, -2.0274212, 1.7173727],
                [-0.1225260, -1.0630554, 1.2958838],
            ],
            bias_values: vec![
                [3.1733532, 9.3601589, -11.0612173],
                [6.4961057, 11.2683334, -13.7752209],
                [2.3662698, 2.4709859, -2.5402584],
                [2.4701204, 8.5112562, -7.6070199],
                [0.9597272, 8.9189463, -11.9037018],
                [2.1239815, 5.8446727, -5.7748013],
                [2.1403685, 5.5432696, -4.0048418],
                [-9.0128260, 28.4402637, -24.2205143],
                [5.2072763, 8.6428480, -9.2946615],
                [4.9253063, 11.4634714, -9.4336052],
                [5.2460732, 7.7711511, -7.5998945],
            ],
        },
        quit_rule: AutoFishingQuitRule {
            localized_fishing_text_default: "钓鱼".to_string(),
            succeeds_when_find_f_fishing_text: true,
            clicks_black_confirm_when_present: true,
            black_confirm_sleep_ms: 1_000,
            escape_retry_sleep_ms: 2_000,
        },
        steps: auto_fishing_task_steps(),
        executor_ready: false,
        pending_native: vec![
            "live capture loop, cancellation/sleep/active-window checks, SetTimeTask execution, and GC execution; pure time-policy rounds and tick loop control decisions are ported".to_string(),
            "BehaviourTree execution and mutable Blackboard orchestration; pure Blackboard reset/state contract is ported".to_string(),
            "BgiFish YOLO model inference and Fishpond construction from detections; pure find-fish timeout, viewpoint, initial-state, and turn/alignment decisions are ported".to_string(),
            "EnterFishingMode Bv observations, white-confirm click execution, and GridIcon initial bait inference; pure retry/timeout/crop/selected-bait/pitch-reset decisions are ported".to_string(),
            "ChooseBait GridIcon recognition and bait UI interaction".to_string(),
            "Torch/RodNet bridge for optional native parity; pure RodNet math, throw-rod initialization, and adjustment decisions are ported".to_string(),
            "mouse/keyboard input dispatch for entering, throwing, raising, pulling, and quitting".to_string(),
            "Paddle OCR, OpenCV fish-bar recognition/template execution, and DrawContent overlays"
                .to_string(),
        ],
    }
}

fn auto_fishing_task_steps() -> Vec<AutoFishingTaskStep> {
    vec![
        AutoFishingTaskStep {
            phase: AutoFishingTaskPhase::Startup,
            action: AutoFishingTaskAction::DisableRealtimeAutoFish,
        },
        AutoFishingTaskStep {
            phase: AutoFishingTaskPhase::TimePolicy,
            action: AutoFishingTaskAction::ApplyFishingTimePolicy,
        },
        AutoFishingTaskStep {
            phase: AutoFishingTaskPhase::FindFish,
            action: AutoFishingTaskAction::MoveViewpointDown,
        },
        AutoFishingTaskStep {
            phase: AutoFishingTaskPhase::FindFish,
            action: AutoFishingTaskAction::DetectFishpondWithYolo,
        },
        AutoFishingTaskStep {
            phase: AutoFishingTaskPhase::EnterFishingMode,
            action: AutoFishingTaskAction::EnterFishingMode,
        },
        AutoFishingTaskStep {
            phase: AutoFishingTaskPhase::ChooseBait,
            action: AutoFishingTaskAction::ChooseBaitWithGridIconModel,
        },
        AutoFishingTaskStep {
            phase: AutoFishingTaskPhase::ThrowRod,
            action: AutoFishingTaskAction::ThrowRodWithRodNet,
        },
        AutoFishingTaskStep {
            phase: AutoFishingTaskPhase::ThrowRod,
            action: AutoFishingTaskAction::CheckThrowRodResult,
        },
        AutoFishingTaskStep {
            phase: AutoFishingTaskPhase::BiteAndRaiseRod,
            action: AutoFishingTaskAction::DetectBiteAndRaiseRod,
        },
        AutoFishingTaskStep {
            phase: AutoFishingTaskPhase::BiteAndRaiseRod,
            action: AutoFishingTaskAction::WaitBiteTimeout,
        },
        AutoFishingTaskStep {
            phase: AutoFishingTaskPhase::BiteAndRaiseRod,
            action: AutoFishingTaskAction::CheckRaiseHookResult,
        },
        AutoFishingTaskStep {
            phase: AutoFishingTaskPhase::PullFishingBar,
            action: AutoFishingTaskAction::PullFishingBar,
        },
        AutoFishingTaskStep {
            phase: AutoFishingTaskPhase::QuitFishingMode,
            action: AutoFishingTaskAction::QuitFishingMode,
        },
    ]
}

pub fn preprocess_auto_fishing_rod_input(
    input: AutoFishingRodInput,
    rule: &AutoFishingRodNetRule,
) -> Result<AutoFishingRodPreprocessResult, AutoFishingRodNetError> {
    validate_auto_fishing_rod_label(input.fish_label, rule)?;

    let alpha = rule.alpha;
    let mut a = (input.rod_x2 - input.rod_x1) / 2.0 / alpha;
    let mut b = (input.rod_y2 - input.rod_y1) / 2.0 / alpha;
    let h = (input.fish_y2 - input.fish_y1) / 2.0 / alpha;

    if a < b {
        b = (a * b).sqrt();
        a = b + 1e-6;
    }

    let v0 =
        (rule.model_input_size.height as f64 / 2.0 - (input.rod_y1 + input.rod_y2) / 2.0) / alpha;
    let u = (input.fish_x1 + input.fish_x2 - input.rod_x1 - input.rod_x2) / 2.0 / alpha;
    let v =
        (rule.model_input_size.height as f64 / 2.0 - (input.fish_y1 + input.fish_y2) / 2.0) / alpha;

    let a2 = a * a;
    let b2 = b * b;
    let y0 = ((a2 * a2) - b2 + a2 * (1.0 - b2 + v0 * v0)).sqrt() / a2;
    let z0 = b / a2;
    let t = a2 * (y0 * b + v0) / (a2 - b2);

    let result = AutoFishingRodPreprocessResult { y0, z0, t, u, v, h };
    if [result.y0, result.z0, result.t, result.u, result.v, result.h]
        .into_iter()
        .all(f64::is_finite)
    {
        Ok(result)
    } else {
        Err(AutoFishingRodNetError::NonFiniteComputation)
    }
}

pub fn compute_auto_fishing_rod_scores(
    input: AutoFishingRodInput,
    rule: &AutoFishingRodNetRule,
) -> Result<AutoFishingRodScoreReport, AutoFishingRodNetError> {
    validate_auto_fishing_rod_shapes(rule)?;
    let preprocess = preprocess_auto_fishing_rod_input(input, rule)?;
    let label = input.fish_label;

    let v = preprocess.v - preprocess.h * rule.h_coeff_values[label];
    let z = preprocess.z0 + rule.dz_values[label];
    let denominator = preprocess.t - v;
    let x = preprocess.u * z * (1.0 + preprocess.t * preprocess.t).sqrt() / denominator;
    let y = z * (1.0 + preprocess.t * v) / denominator;
    let dist = (x * x + (y - preprocess.y0) * (y - preprocess.y0)).sqrt();

    let mut logits = [0.0; 3];
    for (index, logit) in logits.iter_mut().enumerate() {
        *logit = rule.weight_values[label][index] * dist + rule.bias_values[label][index];
    }

    let mut scores = softmax3(logits);
    scores[0] -= rule.offset_values[label];

    if [
        dist, logits[0], logits[1], logits[2], scores[0], scores[1], scores[2],
    ]
    .into_iter()
    .all(f64::is_finite)
    {
        Ok(AutoFishingRodScoreReport {
            preprocess,
            dist,
            logits,
            scores,
        })
    } else {
        Err(AutoFishingRodNetError::NonFiniteComputation)
    }
}

pub fn classify_auto_fishing_rod_state(
    input: AutoFishingRodInput,
    rule: &AutoFishingRodNetRule,
) -> Result<AutoFishingRodState, AutoFishingRodNetError> {
    let report = compute_auto_fishing_rod_scores(input, rule)?;
    let mut max_index = 0;
    for index in 1..report.scores.len() {
        if report.scores[index] > report.scores[max_index] {
            max_index = index;
        }
    }

    if max_index == rule.state_ready as usize {
        Ok(AutoFishingRodState::Ready)
    } else if max_index == rule.state_too_close as usize {
        Ok(AutoFishingRodState::TooClose)
    } else {
        Ok(AutoFishingRodState::TooFar)
    }
}

pub fn select_auto_fishing_throw_rod_target_fish(
    observation: AutoFishingTargetFishSelectionObservation,
    rule: &AutoFishingThrowRodRule,
) -> AutoFishingTargetFishSelectionReport {
    let ignored_baits = auto_fishing_ignored_baits(
        &observation.throw_rod_no_bait_fish_failures,
        rule.max_no_bait_fish_failures_before_ignore,
    );
    let Some(selected_bait) = observation.selected_bait else {
        return AutoFishingTargetFishSelectionReport {
            ignored_baits,
            eligible_indices: Vec::new(),
            selected_index: None,
            selected_fish: None,
            selected_distance: None,
        };
    };

    let mut eligible: Vec<(usize, &AutoFishingTargetFishCandidate)> = observation
        .fishes
        .iter()
        .enumerate()
        .filter(|(_, fish)| !ignored_baits.contains(&fish.bait))
        .filter(|(_, fish)| fish.bait == selected_bait)
        .collect();
    eligible.sort_by(|(_, left), (_, right)| right.confidence.total_cmp(&left.confidence));

    let eligible_indices = eligible.iter().map(|(index, _)| *index).collect::<Vec<_>>();
    let mut best: Option<(usize, usize, f64)> = None;
    for (ranked_index, (candidate_index, fish)) in eligible.iter().enumerate() {
        let distance = auto_fishing_rect_center_distance(fish.rect, observation.target_rect);
        let should_replace = match best {
            None => true,
            Some((best_ranked_index, best_candidate_index, best_distance)) => {
                match distance.total_cmp(&best_distance) {
                    Ordering::Less => true,
                    Ordering::Greater => false,
                    Ordering::Equal => {
                        let best_confidence = observation.fishes[best_candidate_index].confidence;
                        match fish.confidence.total_cmp(&best_confidence) {
                            Ordering::Greater => true,
                            Ordering::Less => false,
                            Ordering::Equal => ranked_index < best_ranked_index,
                        }
                    }
                }
            }
        };

        if should_replace {
            best = Some((ranked_index, *candidate_index, distance));
        }
    }

    let (selected_index, selected_distance) = best
        .map(|(_, candidate_index, distance)| (Some(candidate_index), Some(distance)))
        .unwrap_or((None, None));
    let selected_fish = selected_index.map(|index| observation.fishes[index].clone());

    AutoFishingTargetFishSelectionReport {
        ignored_baits,
        eligible_indices,
        selected_index,
        selected_fish,
        selected_distance,
    }
}

pub fn evaluate_auto_fishing_fishpond_availability(
    observation: AutoFishingFishpondAvailabilityObservation,
    choose_bait_rule: &AutoFishingChooseBaitRule,
    throw_rod_rule: &AutoFishingThrowRodRule,
) -> AutoFishingFishpondAvailabilityReport {
    let choose_bait_ignored_baits = auto_fishing_ignored_baits(
        &observation.choose_bait_failures,
        choose_bait_rule.max_failed_times_before_ignore,
    );
    let throw_rod_ignored_baits = auto_fishing_ignored_baits(
        &observation.throw_rod_no_bait_fish_failures,
        throw_rod_rule.max_no_bait_fish_failures_before_ignore,
    );
    let mut available_baits = Vec::new();
    for fish in &observation.fishes {
        if !choose_bait_ignored_baits.contains(&fish.bait)
            && !throw_rod_ignored_baits.contains(&fish.bait)
            && !available_baits.contains(&fish.bait)
        {
            available_baits.push(fish.bait);
        }
    }

    AutoFishingFishpondAvailabilityReport {
        choose_bait_ignored_baits,
        throw_rod_ignored_baits,
        has_available_fish: !available_baits.is_empty(),
        available_baits,
    }
}

pub fn choose_auto_fishing_bait_for_fishpond(
    observation: AutoFishingBaitSelectionObservation,
    rule: &AutoFishingChooseBaitRule,
) -> AutoFishingBaitSelectionReport {
    if let Some(current_selected_bait) = observation.current_selected_bait {
        if observation
            .fishes
            .iter()
            .any(|fish| fish.bait == current_selected_bait)
        {
            return AutoFishingBaitSelectionReport {
                keeps_current_bait: true,
                ignored_baits: Vec::new(),
                eligible_bait_counts: Vec::new(),
                selected_bait: Some(current_selected_bait),
            };
        }
    }

    let ignored_baits = auto_fishing_ignored_baits(
        &observation.choose_bait_failures,
        rule.max_failed_times_before_ignore,
    );
    let mut bait_counts: Vec<AutoFishingBaitCount> = Vec::new();
    for fish in &observation.fishes {
        if ignored_baits.contains(&fish.bait) {
            continue;
        }
        if let Some(count) = bait_counts.iter_mut().find(|count| count.bait == fish.bait) {
            count.count += 1;
        } else {
            bait_counts.push(AutoFishingBaitCount {
                bait: fish.bait,
                count: 1,
            });
        }
    }

    let mut selected_bait = None;
    let mut selected_count = 0;
    for count in &bait_counts {
        if selected_bait.is_none() || count.count > selected_count {
            selected_bait = Some(count.bait);
            selected_count = count.count;
        }
    }

    AutoFishingBaitSelectionReport {
        keeps_current_bait: false,
        ignored_baits,
        eligible_bait_counts: bait_counts,
        selected_bait,
    }
}

pub fn reset_auto_fishing_blackboard_state(
    state: AutoFishingBlackboardState,
) -> AutoFishingBlackboardResetReport {
    let preserved_fishpond = state.fishpond.is_some();
    let preserved_throw_rod_no_target = state.throw_rod_no_target;
    let preserved_throw_rod_no_bait_fish = state.throw_rod_no_bait_fish;
    let cleared_abort_was = state.abort;
    let cleared_selected_bait = state.selected_bait;
    let cleared_throw_rod_no_target_times = state.throw_rod_no_target_times;
    let cleared_throw_rod_no_bait_fish_failures_len = state.throw_rod_no_bait_fish_failures.len();
    let cleared_fish_box_rect_was_non_empty = state.fish_box_rect.is_some();
    let cleared_choose_bait_ui_opening_was = state.choose_bait_ui_opening;
    let cleared_choose_bait_failures_len = state.choose_bait_failures.len();
    let pitch_reset_was = state.pitch_reset;
    AutoFishingBlackboardResetReport {
        next_state: AutoFishingBlackboardState {
            abort: false,
            selected_bait: None,
            fishpond: state.fishpond,
            throw_rod_no_target: state.throw_rod_no_target,
            throw_rod_no_target_times: 0,
            throw_rod_no_bait_fish: state.throw_rod_no_bait_fish,
            throw_rod_no_bait_fish_failures: Vec::new(),
            fish_box_rect: None,
            choose_bait_ui_opening: false,
            choose_bait_failures: Vec::new(),
            pitch_reset: true,
        },
        cleared_abort_was,
        cleared_selected_bait,
        cleared_throw_rod_no_target_times,
        cleared_throw_rod_no_bait_fish_failures_len,
        cleared_fish_box_rect_was_non_empty,
        cleared_choose_bait_ui_opening_was,
        cleared_choose_bait_failures_len,
        pitch_reset_was,
        preserved_fishpond,
        preserved_throw_rod_no_target,
        preserved_throw_rod_no_bait_fish,
    }
}

pub fn initialize_auto_fishing_throw_rod_state(
    state: AutoFishingBlackboardState,
    rule: &AutoFishingThrowRodRule,
) -> AutoFishingThrowRodInitializeReport {
    let cleared_stale_throw_rod_no_target = state.throw_rod_no_target;
    let cleared_stale_throw_rod_no_bait_fish = state.throw_rod_no_bait_fish;
    AutoFishingThrowRodInitializeReport {
        next_state: AutoFishingBlackboardState {
            throw_rod_no_target: false,
            throw_rod_no_bait_fish: false,
            pitch_reset: rule.sets_pitch_reset_on_initialize,
            ..state
        },
        cleared_stale_throw_rod_no_target,
        cleared_stale_throw_rod_no_bait_fish,
        sets_pitch_reset: rule.sets_pitch_reset_on_initialize,
        input_events: if rule.hold_left_button_on_initialize {
            vec![InputEvent::MouseButtonDown {
                button: MouseButton::Left,
            }]
        } else {
            Vec::new()
        },
    }
}

pub fn plan_auto_fishing_time_policy_rounds(
    policy: &AutoFishingTimePolicy,
    is_multiplayer: bool,
    rule: &AutoFishingTimePolicyRule,
) -> Vec<AutoFishingTimePolicyRound> {
    if is_multiplayer && rule.skips_time_setting_in_multiplayer {
        return vec![auto_fishing_time_policy_round(
            None,
            AutoFishingTimePolicyRoundReason::MultiplayerSkipsTimeSetting,
        )];
    }

    match policy {
        AutoFishingTimePolicy::DontChange if rule.dont_change_runs_once => {
            vec![auto_fishing_time_policy_round(
                None,
                AutoFishingTimePolicyRoundReason::DontChange,
            )]
        }
        AutoFishingTimePolicy::Daytime => auto_fishing_set_time_rounds(
            &rule.daytime_hours,
            AutoFishingTimePolicyRoundReason::Daytime,
        ),
        AutoFishingTimePolicy::Nighttime => auto_fishing_set_time_rounds(
            &rule.nighttime_hours,
            AutoFishingTimePolicyRoundReason::Nighttime,
        ),
        AutoFishingTimePolicy::All | AutoFishingTimePolicy::Unknown(_) => {
            auto_fishing_set_time_rounds(
                &rule.all_policy_hours,
                AutoFishingTimePolicyRoundReason::AllOrFallback,
            )
        }
        AutoFishingTimePolicy::DontChange => Vec::new(),
    }
}

pub fn reduce_auto_fishing_tick_around_loop(
    observation: AutoFishingTickAroundObservation,
) -> AutoFishingTickAroundReport {
    if observation.cancellation_requested {
        return AutoFishingTickAroundReport {
            continue_loop: false,
            should_tick_behavior_tree: false,
            stop_reason: Some(AutoFishingTickAroundStopReason::CancellationRequested),
            finished_behavior_status: None,
            collect_gc: false,
            updates_manual_gc_timestamp: false,
        };
    }

    if !observation.genshin_active {
        return AutoFishingTickAroundReport {
            continue_loop: false,
            should_tick_behavior_tree: false,
            stop_reason: Some(AutoFishingTickAroundStopReason::InactiveGameWindow),
            finished_behavior_status: None,
            collect_gc: false,
            updates_manual_gc_timestamp: false,
        };
    }

    if !observation.capture_succeeded {
        return AutoFishingTickAroundReport {
            continue_loop: true,
            should_tick_behavior_tree: false,
            stop_reason: None,
            finished_behavior_status: None,
            collect_gc: false,
            updates_manual_gc_timestamp: false,
        };
    }

    if observation.behavior_tree_status != AutoFishingBehaviorStatus::Running {
        return AutoFishingTickAroundReport {
            continue_loop: false,
            should_tick_behavior_tree: true,
            stop_reason: Some(AutoFishingTickAroundStopReason::BehaviorTreeStopped),
            finished_behavior_status: Some(observation.behavior_tree_status),
            collect_gc: false,
            updates_manual_gc_timestamp: false,
        };
    }

    AutoFishingTickAroundReport {
        continue_loop: true,
        should_tick_behavior_tree: true,
        stop_reason: None,
        finished_behavior_status: None,
        collect_gc: observation.manual_gc_due,
        updates_manual_gc_timestamp: observation.manual_gc_due,
    }
}

pub fn reduce_auto_fishing_whole_process_timeout(
    observation: AutoFishingTimeoutObservation,
) -> AutoFishingTimeoutReport {
    AutoFishingTimeoutReport {
        status: if observation.timeout_elapsed {
            AutoFishingBehaviorStatus::Succeeded
        } else {
            AutoFishingBehaviorStatus::Running
        },
        sets_abort: false,
    }
}

pub fn reduce_auto_fishing_find_fish_timeout(
    observation: AutoFishingTimeoutObservation,
) -> AutoFishingTimeoutReport {
    AutoFishingTimeoutReport {
        status: if observation.timeout_elapsed {
            AutoFishingBehaviorStatus::Failed
        } else {
            AutoFishingBehaviorStatus::Running
        },
        sets_abort: observation.timeout_elapsed,
    }
}

pub fn reduce_auto_fishing_move_viewpoint_down(
    observation: AutoFishingMoveViewpointDownObservation,
    rule: &AutoFishingFindFishRule,
) -> AutoFishingMoveViewpointDownReport {
    if observation.pitch_reset {
        AutoFishingMoveViewpointDownReport {
            next_pitch_reset: false,
            status: AutoFishingBehaviorStatus::Running,
            input_events: vec![
                InputEvent::MouseMoveRelative {
                    dx: 0,
                    dy: rule.move_viewpoint_down_dy,
                },
                InputEvent::Delay {
                    milliseconds: rule.move_viewpoint_down_sleep_ms,
                },
            ],
        }
    } else {
        AutoFishingMoveViewpointDownReport {
            next_pitch_reset: false,
            status: AutoFishingBehaviorStatus::Succeeded,
            input_events: Vec::new(),
        }
    }
}

pub fn reduce_auto_fishing_initial_state(
    observation: AutoFishingInitialStateObservation,
    rule: &AutoFishingFindFishRule,
) -> AutoFishingInitialStateReport {
    if observation.bait_button_present {
        return AutoFishingInitialStateReport {
            next_theta: observation.theta,
            status: AutoFishingBehaviorStatus::Succeeded,
            reschedule_after_ms: None,
            mouse_move: None,
        };
    }

    if !observation.move_interval_elapsed {
        return AutoFishingInitialStateReport {
            next_theta: observation.theta,
            status: AutoFishingBehaviorStatus::Running,
            reschedule_after_ms: None,
            mouse_move: None,
        };
    }

    let theta = observation.theta + rule.initial_state_theta_step_radians;
    let rho = rule.initial_state_rho_base + rule.initial_state_rho_theta_multiplier * theta;
    let x = auto_fishing_legacy_f64_to_i32_lossy(rho * theta.cos());
    let y = auto_fishing_legacy_f64_to_i32_lossy(rho * theta.sin());
    AutoFishingInitialStateReport {
        next_theta: theta,
        status: AutoFishingBehaviorStatus::Running,
        reschedule_after_ms: Some(rule.initial_state_move_interval_ms),
        mouse_move: Some((x, y)),
    }
}

pub fn reduce_auto_fishing_turn_around(
    observation: AutoFishingTurnAroundObservation,
    rule: &AutoFishingFindFishRule,
) -> AutoFishingTurnAroundReport {
    let Some(fishpond_rect) = observation.fishpond_rect else {
        return AutoFishingTurnAroundReport {
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingTurnAroundDecisionKind::NoFishSweep,
            clears_overlay: false,
            input_events: vec![
                InputEvent::MouseMoveRelative {
                    dx: rule.turn_no_fish_dx,
                    dy: 0,
                },
                InputEvent::Delay {
                    milliseconds: rule.turn_after_move_sleep_ms,
                },
            ],
        };
    };

    let capture_width = observation.capture_size.width as f64;
    let one_fourth_x = capture_width * rule.fishpond_left_threshold_ratio;
    let three_fourth_x = capture_width * rule.fishpond_right_threshold_ratio;
    if fishpond_rect.x as f64 > three_fourth_x {
        return AutoFishingTurnAroundReport {
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingTurnAroundDecisionKind::FishpondTooFarRight,
            clears_overlay: true,
            input_events: vec![
                InputEvent::Delay {
                    milliseconds: rule.fishpond_detected_overlay_sleep_ms,
                },
                InputEvent::MouseMoveRelative {
                    dx: rule.turn_no_fish_dx,
                    dy: 0,
                },
                InputEvent::Delay {
                    milliseconds: rule.turn_after_move_sleep_ms,
                },
            ],
        };
    }

    if ((fishpond_rect.x + fishpond_rect.width) as f64) < one_fourth_x {
        return AutoFishingTurnAroundReport {
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingTurnAroundDecisionKind::FishpondTooFarLeft,
            clears_overlay: true,
            input_events: vec![
                InputEvent::Delay {
                    milliseconds: rule.fishpond_detected_overlay_sleep_ms,
                },
                InputEvent::MouseMoveRelative {
                    dx: -rule.turn_no_fish_dx,
                    dy: 0,
                },
                InputEvent::Delay {
                    milliseconds: rule.turn_after_move_sleep_ms,
                },
            ],
        };
    }

    AutoFishingTurnAroundReport {
        status: AutoFishingBehaviorStatus::Succeeded,
        decision: AutoFishingTurnAroundDecisionKind::AlignCharacterAndCamera,
        clears_overlay: true,
        input_events: auto_fishing_turn_around_alignment_events(rule),
    }
}

pub fn plan_auto_fishing_enter_mode_bait_icon_crop(
    capture_size: Size,
    rule: &AutoFishingEnterModeRule,
) -> Result<AutoFishingBaitIconCropPlan, AutoFishingEnterModeError> {
    if capture_size.width == 0 || capture_size.height == 0 {
        return Err(AutoFishingEnterModeError::InvalidCaptureSize);
    }

    let width = capture_size.width as f64;
    let height = capture_size.height as f64;
    let ratio = &rule.initial_bait_icon_crop_ratio;
    let values = [ratio.x, ratio.y, ratio.width, ratio.height, width, height];
    if values.into_iter().any(|value| !value.is_finite()) {
        return Err(AutoFishingEnterModeError::NonFiniteComputation);
    }

    let x = (ratio.x * width).trunc() as i32;
    let y = (ratio.y * height).trunc() as i32;
    let crop_width = (ratio.width * width).trunc() as i32;
    let crop_height = (ratio.height * width).trunc() as i32;
    if crop_width <= 0 || crop_height <= 0 {
        return Err(AutoFishingEnterModeError::InvalidCropRect);
    }

    let crop_rect = Rect::new(x, y, crop_width, crop_height)
        .map_err(|_| AutoFishingEnterModeError::InvalidCropRect)?;
    if crop_rect.right() > capture_size.width as i32
        || crop_rect.bottom() > capture_size.height as i32
    {
        return Err(AutoFishingEnterModeError::InvalidCropRect);
    }

    Ok(AutoFishingBaitIconCropPlan {
        crop_rect,
        resize_size: rule.grid_icon_input_size,
    })
}

pub fn reduce_auto_fishing_enter_mode(
    observation: AutoFishingEnterModeObservation,
    rule: &AutoFishingEnterModeRule,
) -> Result<AutoFishingEnterModeReport, AutoFishingEnterModeError> {
    if !observation.state.initialized {
        return Ok(AutoFishingEnterModeReport {
            next_state: AutoFishingEnterModeState {
                initialized: true,
                ..observation.state
            },
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingEnterModeDecisionKind::StartOverallWait,
            input_events: Vec::new(),
            clicks_white_confirm: false,
            white_confirm_pre_click_delay_ms: None,
            bait_icon_crop: None,
            starts_overall_timeout_ms: Some(rule.overall_wait_seconds * 1_000),
            reschedule_press_f_after_ms: None,
            reschedule_click_white_confirm_after_ms: None,
        });
    }

    if observation.press_f_cooldown_elapsed && observation.f_fishing_text_visible {
        return Ok(AutoFishingEnterModeReport {
            next_state: observation.state,
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingEnterModeDecisionKind::PressFishingKey,
            input_events: auto_fishing_key_press_events(rule.interact_vk),
            clicks_white_confirm: false,
            white_confirm_pre_click_delay_ms: None,
            bait_icon_crop: None,
            starts_overall_timeout_ms: None,
            reschedule_press_f_after_ms: Some(rule.press_f_retry_seconds * 1_000),
            reschedule_click_white_confirm_after_ms: None,
        });
    }

    if observation.click_white_confirm_cooldown_elapsed && observation.white_confirm_button_present
    {
        let crop_plan =
            plan_auto_fishing_enter_mode_bait_icon_crop(observation.capture_size, rule)?;
        return Ok(AutoFishingEnterModeReport {
            next_state: AutoFishingEnterModeState {
                selected_bait: observation.inferred_bait_after_confirm,
                pitch_reset: rule.marks_pitch_reset_after_confirm,
                ..observation.state
            },
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingEnterModeDecisionKind::ClickWhiteConfirm,
            input_events: Vec::new(),
            clicks_white_confirm: true,
            white_confirm_pre_click_delay_ms: Some(rule.white_confirm_pre_click_delay_ms),
            bait_icon_crop: Some(crop_plan),
            starts_overall_timeout_ms: None,
            reschedule_press_f_after_ms: None,
            reschedule_click_white_confirm_after_ms: Some(
                rule.click_white_confirm_retry_seconds * 1_000,
            ),
        });
    }

    if observation.exit_fishing_button_present {
        return Ok(AutoFishingEnterModeReport {
            next_state: observation.state,
            status: AutoFishingBehaviorStatus::Succeeded,
            decision: AutoFishingEnterModeDecisionKind::Entered,
            input_events: Vec::new(),
            clicks_white_confirm: false,
            white_confirm_pre_click_delay_ms: None,
            bait_icon_crop: None,
            starts_overall_timeout_ms: None,
            reschedule_press_f_after_ms: None,
            reschedule_click_white_confirm_after_ms: None,
        });
    }

    if observation.overall_timeout_elapsed {
        Ok(AutoFishingEnterModeReport {
            next_state: observation.state,
            status: AutoFishingBehaviorStatus::Failed,
            decision: AutoFishingEnterModeDecisionKind::FailedTimeout,
            input_events: Vec::new(),
            clicks_white_confirm: false,
            white_confirm_pre_click_delay_ms: None,
            bait_icon_crop: None,
            starts_overall_timeout_ms: None,
            reschedule_press_f_after_ms: None,
            reschedule_click_white_confirm_after_ms: None,
        })
    } else {
        Ok(AutoFishingEnterModeReport {
            next_state: observation.state,
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingEnterModeDecisionKind::WaitingForExitButton,
            input_events: Vec::new(),
            clicks_white_confirm: false,
            white_confirm_pre_click_delay_ms: None,
            bait_icon_crop: None,
            starts_overall_timeout_ms: None,
            reschedule_press_f_after_ms: None,
            reschedule_click_white_confirm_after_ms: None,
        })
    }
}

pub fn reduce_auto_fishing_delayed_template_check(
    observation: AutoFishingDelayedTemplateCheckObservation,
) -> AutoFishingDelayedTemplateCheckReport {
    if !observation.delay_elapsed || observation.has_checked {
        return AutoFishingDelayedTemplateCheckReport {
            next_has_checked: observation.has_checked,
            status: AutoFishingBehaviorStatus::Running,
        };
    }

    if observation.failure_template_present {
        AutoFishingDelayedTemplateCheckReport {
            next_has_checked: observation.has_checked,
            status: AutoFishingBehaviorStatus::Failed,
        }
    } else {
        AutoFishingDelayedTemplateCheckReport {
            next_has_checked: true,
            status: AutoFishingBehaviorStatus::Running,
        }
    }
}

pub fn reduce_auto_fishing_fish_bite_timeout(
    observation: AutoFishingFishBiteTimeoutObservation,
    rule: &AutoFishingCheckResultRule,
) -> AutoFishingFishBiteTimeoutReport {
    if !observation.timeout_elapsed {
        return AutoFishingFishBiteTimeoutReport {
            next_left_button_clicked: observation.left_button_clicked,
            status: AutoFishingBehaviorStatus::Running,
            input_events: Vec::new(),
            reschedule_after_ms: None,
        };
    }

    if observation.left_button_clicked {
        AutoFishingFishBiteTimeoutReport {
            next_left_button_clicked: true,
            status: AutoFishingBehaviorStatus::Failed,
            input_events: Vec::new(),
            reschedule_after_ms: None,
        }
    } else {
        AutoFishingFishBiteTimeoutReport {
            next_left_button_clicked: true,
            status: AutoFishingBehaviorStatus::Running,
            input_events: if rule.fish_bite_timeout_clicks_left_button {
                auto_fishing_left_click_events()
            } else {
                Vec::new()
            },
            reschedule_after_ms: Some(rule.fish_bite_timeout_after_click_seconds * 1_000),
        }
    }
}

pub fn decide_auto_fishing_fish_bite(
    observation: AutoFishingFishBiteObservation,
) -> AutoFishingFishBiteReport {
    let method = if observation.word_block_detected {
        Some(AutoFishingFishBiteMethod::WordBlock)
    } else if observation.lift_rod_button_present {
        Some(AutoFishingFishBiteMethod::LiftRodButton)
    } else if observation
        .ocr_text
        .as_deref()
        .map(auto_fishing_remove_all_space)
        .is_some_and(|text| text.contains(&observation.localized_get_bite_text))
    {
        Some(AutoFishingFishBiteMethod::Ocr)
    } else {
        None
    };

    if let Some(method) = method {
        AutoFishingFishBiteReport {
            status: AutoFishingBehaviorStatus::Succeeded,
            method: Some(method),
            removes_fish_bite_tips_overlay: true,
            input_events: auto_fishing_left_click_events(),
        }
    } else {
        AutoFishingFishBiteReport {
            status: AutoFishingBehaviorStatus::Running,
            method: None,
            removes_fish_bite_tips_overlay: false,
            input_events: Vec::new(),
        }
    }
}

pub fn resolve_auto_fishing_fish_box_area(
    observation: AutoFishingFishBoxAreaObservation,
    rule: &AutoFishingCheckResultRule,
) -> AutoFishingFishBoxAreaReport {
    if observation.timeout_elapsed {
        return AutoFishingFishBoxAreaReport {
            status: AutoFishingBehaviorStatus::Failed,
            decision: AutoFishingFishBoxAreaDecisionKind::Timeout,
            fish_box_rect: None,
            cursor_rect: None,
            target_rect: None,
            error_screenshot_recommended: false,
        };
    }

    if observation.top_rects.len() != 2 {
        return AutoFishingFishBoxAreaReport {
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingFishBoxAreaDecisionKind::WaitingForTwoRects,
            fish_box_rect: None,
            cursor_rect: None,
            target_rect: None,
            error_screenshot_recommended: false,
        };
    }

    let left = observation.top_rects[0];
    let right = observation.top_rects[1];
    if (left.height - right.height).abs() > rule.fish_box_rect_max_height_delta {
        return AutoFishingFishBoxAreaReport {
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingFishBoxAreaDecisionKind::HeightMismatch,
            fish_box_rect: None,
            cursor_rect: None,
            target_rect: None,
            error_screenshot_recommended: true,
        };
    }

    let (cursor, target) = if left.width < right.width {
        (left, right)
    } else {
        (right, left)
    };
    let top_width = observation.capture_size.width as i32;
    let top_mid_x = top_width / 2;
    let cursor_right = cursor.x + cursor.width;
    let invalid = target.x < cursor.x
        || cursor.width > target.width
        || cursor_right > top_mid_x
        || cursor_right > target.x - target.width / 2
        || cursor_right > top_mid_x - target.width;
    if invalid {
        return AutoFishingFishBoxAreaReport {
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingFishBoxAreaDecisionKind::InvalidGeometry,
            fish_box_rect: None,
            cursor_rect: Some(cursor),
            target_rect: Some(target),
            error_screenshot_recommended: false,
        };
    }

    let h_extra = cursor.height;
    let v_extra = cursor.height / 4;
    let raw = Rect {
        x: cursor.x - h_extra,
        y: cursor.y - v_extra,
        width: (top_mid_x - cursor.x) * 2 + h_extra * 2,
        height: cursor.height + v_extra * 2,
    };
    let fish_box_rect = raw.clamp_to(observation.capture_size).ok();
    AutoFishingFishBoxAreaReport {
        status: if fish_box_rect.is_some() {
            AutoFishingBehaviorStatus::Succeeded
        } else {
            AutoFishingBehaviorStatus::Running
        },
        decision: if fish_box_rect.is_some() {
            AutoFishingFishBoxAreaDecisionKind::Detected
        } else {
            AutoFishingFishBoxAreaDecisionKind::InvalidGeometry
        },
        fish_box_rect,
        cursor_rect: Some(cursor),
        target_rect: Some(target),
        error_screenshot_recommended: false,
    }
}

pub fn decide_auto_fishing_quit_mode(
    observation: AutoFishingQuitModeObservation,
    rule: &AutoFishingQuitRule,
) -> AutoFishingQuitModeReport {
    if observation.first_tick {
        return AutoFishingQuitModeReport {
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingQuitModeDecisionKind::InitialWait,
            clicks_black_confirm: false,
            input_events: Vec::new(),
            sleep_ms: None,
        };
    }

    if rule.succeeds_when_find_f_fishing_text && observation.f_fishing_text_visible {
        return AutoFishingQuitModeReport {
            status: AutoFishingBehaviorStatus::Succeeded,
            decision: AutoFishingQuitModeDecisionKind::AlreadyExited,
            clicks_black_confirm: false,
            input_events: Vec::new(),
            sleep_ms: None,
        };
    }

    if rule.clicks_black_confirm_when_present && observation.black_confirm_button_present {
        return AutoFishingQuitModeReport {
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingQuitModeDecisionKind::ClickBlackConfirm,
            clicks_black_confirm: true,
            input_events: Vec::new(),
            sleep_ms: Some(rule.black_confirm_sleep_ms),
        };
    }

    AutoFishingQuitModeReport {
        status: AutoFishingBehaviorStatus::Running,
        decision: AutoFishingQuitModeDecisionKind::PressEscape,
        clicks_black_confirm: false,
        input_events: auto_fishing_escape_key_press_events(),
        sleep_ms: Some(rule.escape_retry_sleep_ms),
    }
}

pub fn reduce_auto_fishing_pull_bar(
    observation: AutoFishingPullBarObservation,
) -> AutoFishingPullBarReport {
    let mut rects = observation.fish_bar_rects;
    if rects.is_empty() {
        if !observation.state.no_detection_armed {
            return AutoFishingPullBarReport {
                next_state: AutoFishingPullBarState {
                    no_detection_armed: true,
                    ..observation.state
                },
                status: AutoFishingBehaviorStatus::Running,
                decision: AutoFishingPullBarDecisionKind::ArmNoDetectionGrace,
                considered_rects: Vec::new(),
                removes_fish_box_overlay: false,
                clears_bar_overlay: true,
                input_events: Vec::new(),
            };
        }

        if !observation.no_detection_grace_elapsed {
            return AutoFishingPullBarReport {
                next_state: observation.state,
                status: AutoFishingBehaviorStatus::Running,
                decision: AutoFishingPullBarDecisionKind::WaitNoDetectionGrace,
                considered_rects: Vec::new(),
                removes_fish_box_overlay: false,
                clears_bar_overlay: true,
                input_events: Vec::new(),
            };
        }

        return AutoFishingPullBarReport {
            next_state: AutoFishingPullBarState {
                previous_left_button_down: false,
                no_detection_armed: false,
            },
            status: AutoFishingBehaviorStatus::Succeeded,
            decision: AutoFishingPullBarDecisionKind::CompleteNoDetection,
            considered_rects: Vec::new(),
            removes_fish_box_overlay: true,
            clears_bar_overlay: true,
            input_events: vec![InputEvent::MouseButtonUp {
                button: MouseButton::Left,
            }],
        };
    }

    if rects.len() > 3 {
        rects.sort_by(|left, right| right.height.cmp(&left.height));
        rects.truncate(3);
    }

    if rects.len() == 2 {
        let (cursor, target) = if rects[0].width < rects[1].width {
            (rects[0], rects[1])
        } else {
            (rects[1], rects[0])
        };
        if target.width < cursor.width * 10 {
            return AutoFishingPullBarReport {
                next_state: observation.state,
                status: AutoFishingBehaviorStatus::Running,
                decision: AutoFishingPullBarDecisionKind::InvalidTwoRectTarget,
                considered_rects: vec![target, cursor],
                removes_fish_box_overlay: false,
                clears_bar_overlay: false,
                input_events: Vec::new(),
            };
        }

        let should_press = cursor.x < target.x;
        return auto_fishing_pull_bar_button_report(
            observation.state,
            vec![target, cursor],
            should_press,
        );
    }

    if rects.len() == 3 {
        rects.sort_by(|left, right| left.x.cmp(&right.x));
        let left = rects[0];
        let cursor = rects[1];
        let right = rects[2];
        let right_remaining = right.x + right.width - (cursor.x + cursor.width);
        let left_distance = cursor.x - left.x;
        let should_press = right_remaining > left_distance;
        return auto_fishing_pull_bar_button_report(
            observation.state,
            vec![left, cursor, right],
            should_press,
        );
    }

    AutoFishingPullBarReport {
        next_state: AutoFishingPullBarState {
            no_detection_armed: false,
            ..observation.state
        },
        status: AutoFishingBehaviorStatus::Running,
        decision: AutoFishingPullBarDecisionKind::IgnoreUnexpectedRectCount,
        considered_rects: Vec::new(),
        removes_fish_box_overlay: false,
        clears_bar_overlay: true,
        input_events: Vec::new(),
    }
}

fn auto_fishing_pull_bar_button_report(
    state: AutoFishingPullBarState,
    considered_rects: Vec<Rect>,
    should_press: bool,
) -> AutoFishingPullBarReport {
    if should_press {
        if state.previous_left_button_down {
            AutoFishingPullBarReport {
                next_state: AutoFishingPullBarState {
                    no_detection_armed: false,
                    ..state
                },
                status: AutoFishingBehaviorStatus::Running,
                decision: AutoFishingPullBarDecisionKind::KeepLeftButtonDown,
                considered_rects,
                removes_fish_box_overlay: false,
                clears_bar_overlay: false,
                input_events: Vec::new(),
            }
        } else {
            AutoFishingPullBarReport {
                next_state: AutoFishingPullBarState {
                    previous_left_button_down: true,
                    no_detection_armed: false,
                },
                status: AutoFishingBehaviorStatus::Running,
                decision: AutoFishingPullBarDecisionKind::PressLeftButton,
                considered_rects,
                removes_fish_box_overlay: false,
                clears_bar_overlay: false,
                input_events: vec![InputEvent::MouseButtonDown {
                    button: MouseButton::Left,
                }],
            }
        }
    } else if state.previous_left_button_down {
        AutoFishingPullBarReport {
            next_state: AutoFishingPullBarState {
                previous_left_button_down: false,
                no_detection_armed: false,
            },
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingPullBarDecisionKind::ReleaseLeftButton,
            considered_rects,
            removes_fish_box_overlay: false,
            clears_bar_overlay: false,
            input_events: vec![InputEvent::MouseButtonUp {
                button: MouseButton::Left,
            }],
        }
    } else {
        AutoFishingPullBarReport {
            next_state: AutoFishingPullBarState {
                no_detection_armed: false,
                ..state
            },
            status: AutoFishingBehaviorStatus::Running,
            decision: AutoFishingPullBarDecisionKind::KeepLeftButtonUp,
            considered_rects,
            removes_fish_box_overlay: false,
            clears_bar_overlay: false,
            input_events: Vec::new(),
        }
    }
}

fn auto_fishing_left_click_events() -> Vec<InputEvent> {
    vec![
        InputEvent::MouseButtonDown {
            button: MouseButton::Left,
        },
        InputEvent::MouseButtonUp {
            button: MouseButton::Left,
        },
    ]
}

fn auto_fishing_escape_key_press_events() -> Vec<InputEvent> {
    const VK_ESCAPE: u16 = 0x1B;
    auto_fishing_key_press_events(VK_ESCAPE)
}

fn auto_fishing_key_press_events(vk: u16) -> Vec<InputEvent> {
    vec![
        InputEvent::KeyDown { vk, extended: None },
        InputEvent::KeyUp { vk, extended: None },
    ]
}

fn auto_fishing_time_policy_round(
    hour: Option<u8>,
    reason: AutoFishingTimePolicyRoundReason,
) -> AutoFishingTimePolicyRound {
    AutoFishingTimePolicyRound {
        set_time_hour: hour,
        set_time_minute: hour.map(|_| 0),
        run_tick_around: true,
        resets_blackboard_before_round: true,
        resets_manual_gc_timer_before_round: true,
        reason,
    }
}

fn auto_fishing_set_time_rounds(
    hours: &[u8],
    reason: AutoFishingTimePolicyRoundReason,
) -> Vec<AutoFishingTimePolicyRound> {
    hours
        .iter()
        .map(|hour| auto_fishing_time_policy_round(Some(*hour), reason))
        .collect()
}

fn auto_fishing_turn_around_alignment_events(rule: &AutoFishingFindFishRule) -> Vec<InputEvent> {
    vec![
        InputEvent::Delay {
            milliseconds: rule.fishpond_detected_overlay_sleep_ms,
        },
        InputEvent::KeyDown {
            vk: rule.align_backward_vk,
            extended: None,
        },
        InputEvent::Delay {
            milliseconds: rule.align_key_hold_ms,
        },
        InputEvent::KeyUp {
            vk: rule.align_backward_vk,
            extended: None,
        },
        InputEvent::Delay {
            milliseconds: rule.align_between_key_sleep_ms,
        },
        InputEvent::KeyDown {
            vk: rule.align_forward_vk,
            extended: None,
        },
        InputEvent::Delay {
            milliseconds: rule.align_key_hold_ms,
        },
        InputEvent::KeyUp {
            vk: rule.align_forward_vk,
            extended: None,
        },
        InputEvent::Delay {
            milliseconds: rule.align_between_key_sleep_ms,
        },
        InputEvent::Delay {
            milliseconds: rule.align_final_sleep_ms,
        },
    ]
}

fn auto_fishing_remove_all_space(value: &str) -> String {
    value.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn auto_fishing_ignored_baits(
    failures: &[AutoFishingBaitType],
    failure_threshold: u64,
) -> Vec<AutoFishingBaitType> {
    let mut ignored = Vec::new();
    for bait in failures {
        if !ignored.contains(bait)
            && failures.iter().filter(|failure| *failure == bait).count() as u64
                >= failure_threshold
        {
            ignored.push(*bait);
        }
    }
    ignored
}

fn auto_fishing_rect_center_distance(left: Rect, right: Rect) -> f64 {
    let left_center = left.center();
    let right_center = right.center();
    let dx = i64::from(left_center.x) - i64::from(right_center.x);
    let dy = i64::from(left_center.y) - i64::from(right_center.y);
    ((dx * dx + dy * dy) as f64).sqrt()
}

pub fn resolve_auto_fishing_throw_rod_geometry(
    observation: AutoFishingThrowRodGeometryObservation,
    rule: &AutoFishingThrowRodRule,
) -> Result<AutoFishingThrowRodGeometryReport, AutoFishingThrowRodAdjustmentError> {
    let scale_size = auto_fishing_throw_rod_scale_size(observation.capture_size)?;
    if rule.rod_input_normalized_size.width == 0 || rule.rod_input_normalized_size.height == 0 {
        return Err(AutoFishingThrowRodAdjustmentError::InvalidCaptureSize);
    }

    let normalize_x =
        |x: i32| x as f64 / scale_size.width as f64 * rule.rod_input_normalized_size.width as f64;
    let normalize_y =
        |y: i32| y as f64 / scale_size.height as f64 * rule.rod_input_normalized_size.height as f64;

    let rod_input = AutoFishingRodInput {
        rod_x1: normalize_x(observation.rod_rect.x),
        rod_x2: normalize_x(observation.rod_rect.right()),
        rod_y1: normalize_y(observation.rod_rect.y),
        rod_y2: normalize_y(observation.rod_rect.bottom()),
        fish_x1: normalize_x(observation.fish_rect.x),
        fish_x2: normalize_x(observation.fish_rect.right()),
        fish_y1: normalize_y(observation.fish_rect.y),
        fish_y2: normalize_y(observation.fish_rect.bottom()),
        fish_label: observation.fish_label,
    };
    let (delta_x, delta_y) = auto_fishing_throw_rod_delta(rod_input);
    let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

    if [
        rod_input.rod_x1,
        rod_input.rod_x2,
        rod_input.rod_y1,
        rod_input.rod_y2,
        rod_input.fish_x1,
        rod_input.fish_x2,
        rod_input.fish_y1,
        rod_input.fish_y2,
        delta_x,
        delta_y,
        distance,
    ]
    .into_iter()
    .all(f64::is_finite)
    {
        Ok(AutoFishingThrowRodGeometryReport {
            scale_size,
            rod_input,
            delta_x,
            delta_y,
            distance,
        })
    } else {
        Err(AutoFishingThrowRodAdjustmentError::NonFiniteComputation)
    }
}

pub fn plan_auto_fishing_throw_rod_adjustment(
    observation: AutoFishingThrowRodAdjustmentObservation,
    rule: &AutoFishingThrowRodRule,
) -> Result<AutoFishingThrowRodAdjustmentReport, AutoFishingThrowRodAdjustmentError> {
    let (delta_x, delta_y) = auto_fishing_throw_rod_delta(observation.rod_input);
    let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();
    if !delta_x.is_finite() || !delta_y.is_finite() || !distance.is_finite() {
        return Err(AutoFishingThrowRodAdjustmentError::NonFiniteComputation);
    }

    match observation.classification {
        AutoFishingThrowRodClassification::Failed => {
            let sample = observation
                .random_sample
                .ok_or(AutoFishingThrowRodAdjustmentError::MissingRandomSample)?;
            let (move_x, move_y) = auto_fishing_failed_random_move(
                observation.capture_size,
                sample,
                rule.random_move_base_pixels,
            )?;
            Ok(AutoFishingThrowRodAdjustmentReport {
                kind: AutoFishingThrowRodAdjustmentKind::FailedRandomMove,
                status: AutoFishingThrowRodAdjustmentStatus::Running,
                delta_x,
                delta_y,
                distance,
                input_events: vec![InputEvent::MouseMoveRelative {
                    dx: move_x,
                    dy: move_y,
                }],
                sleep_ms: auto_fishing_adjustment_sleep_ms(distance, rule)?,
            })
        }
        AutoFishingThrowRodClassification::Ready => Ok(AutoFishingThrowRodAdjustmentReport {
            kind: AutoFishingThrowRodAdjustmentKind::ReleaseRod,
            status: AutoFishingThrowRodAdjustmentStatus::Succeeded,
            delta_x,
            delta_y,
            distance,
            input_events: vec![InputEvent::MouseButtonUp {
                button: MouseButton::Left,
            }],
            sleep_ms: None,
        }),
        AutoFishingThrowRodClassification::TooClose => {
            if distance <= f64::EPSILON {
                return Err(AutoFishingThrowRodAdjustmentError::NonFiniteComputation);
            }
            let adjusted_x = delta_x / distance * rule.too_close_minimum_step;
            let adjusted_y = delta_y / distance * rule.too_close_minimum_step;
            let move_x = auto_fishing_legacy_f64_to_i32(-adjusted_x / rule.too_close_dx_divisor)?;
            let move_y =
                auto_fishing_legacy_f64_to_i32(-adjusted_y * rule.too_close_dy_multiplier)?;
            Ok(AutoFishingThrowRodAdjustmentReport {
                kind: AutoFishingThrowRodAdjustmentKind::MoveAwayFromFish,
                status: AutoFishingThrowRodAdjustmentStatus::Running,
                delta_x,
                delta_y,
                distance,
                input_events: vec![InputEvent::MouseMoveRelative {
                    dx: move_x,
                    dy: move_y,
                }],
                sleep_ms: auto_fishing_adjustment_sleep_ms(distance, rule)?,
            })
        }
        AutoFishingThrowRodClassification::TooFar => {
            let move_x = auto_fishing_legacy_f64_to_i32(delta_x / rule.too_far_dx_divisor)?;
            let move_y = auto_fishing_legacy_f64_to_i32(delta_y * rule.too_far_dy_multiplier)?;
            Ok(AutoFishingThrowRodAdjustmentReport {
                kind: AutoFishingThrowRodAdjustmentKind::MoveTowardFish,
                status: AutoFishingThrowRodAdjustmentStatus::Running,
                delta_x,
                delta_y,
                distance,
                input_events: vec![InputEvent::MouseMoveRelative {
                    dx: move_x,
                    dy: move_y,
                }],
                sleep_ms: auto_fishing_adjustment_sleep_ms(distance, rule)?,
            })
        }
        AutoFishingThrowRodClassification::Unknown(_) => Ok(AutoFishingThrowRodAdjustmentReport {
            kind: AutoFishingThrowRodAdjustmentKind::Noop,
            status: AutoFishingThrowRodAdjustmentStatus::Running,
            delta_x,
            delta_y,
            distance,
            input_events: Vec::new(),
            sleep_ms: auto_fishing_adjustment_sleep_ms(distance, rule)?,
        }),
    }
}

fn auto_fishing_throw_rod_scale_size(
    capture_size: Size,
) -> Result<Size, AutoFishingThrowRodAdjustmentError> {
    if capture_size.width == 0 || capture_size.height == 0 {
        return Err(AutoFishingThrowRodAdjustmentError::InvalidCaptureSize);
    }

    if capture_size.width > 1920 {
        let scale = capture_size.width as f64 / 1920.0;
        let scaled_height = (capture_size.height as f64 / scale).trunc();
        if scaled_height < 1.0 || scaled_height > u32::MAX as f64 {
            return Err(AutoFishingThrowRodAdjustmentError::InvalidCaptureSize);
        }
        Ok(Size::new(1920, scaled_height as u32))
    } else {
        Ok(capture_size)
    }
}

fn auto_fishing_throw_rod_delta(input: AutoFishingRodInput) -> (f64, f64) {
    (
        (input.fish_x1 + input.fish_x2 - input.rod_x1 - input.rod_x2) / 2.0,
        (input.fish_y1 + input.fish_y2 - input.rod_y1 - input.rod_y2) / 2.0,
    )
}

fn auto_fishing_failed_random_move(
    capture_size: Size,
    sample: AutoFishingRandomMoveSample,
    base_pixels: i32,
) -> Result<(i32, i32), AutoFishingThrowRodAdjustmentError> {
    if capture_size.width == 0 || capture_size.height == 0 {
        return Err(AutoFishingThrowRodAdjustmentError::InvalidCaptureSize);
    }

    let width = i64::from(capture_size.width);
    let height = i64::from(capture_size.height);
    let center_x = width / 2;
    let center_y = height / 2;
    let base = i64::from(base_pixels);
    let move_x = base * (center_x - i64::from(sample.random_x)) / width;
    let move_y = base * (center_y - i64::from(sample.random_y)) / height;
    Ok((move_x as i32, move_y as i32))
}

fn auto_fishing_adjustment_sleep_ms(
    distance: f64,
    rule: &AutoFishingThrowRodRule,
) -> Result<Option<u64>, AutoFishingThrowRodAdjustmentError> {
    if rule.sleep_after_adjustment_uses_distance {
        auto_fishing_distance_sleep_ms(distance).map(Some)
    } else {
        Ok(None)
    }
}

fn auto_fishing_distance_sleep_ms(
    distance: f64,
) -> Result<u64, AutoFishingThrowRodAdjustmentError> {
    if distance.is_finite() && distance >= 0.0 && distance <= i32::MAX as f64 {
        Ok(distance.trunc() as u64)
    } else {
        Err(AutoFishingThrowRodAdjustmentError::NonFiniteComputation)
    }
}

fn auto_fishing_legacy_f64_to_i32(value: f64) -> Result<i32, AutoFishingThrowRodAdjustmentError> {
    if value.is_finite() && value >= i32::MIN as f64 && value <= i32::MAX as f64 {
        Ok(value.trunc() as i32)
    } else {
        Err(AutoFishingThrowRodAdjustmentError::NonFiniteComputation)
    }
}

fn auto_fishing_legacy_f64_to_i32_lossy(value: f64) -> i32 {
    value as i32
}

fn validate_auto_fishing_rod_label(
    fish_label: usize,
    rule: &AutoFishingRodNetRule,
) -> Result<(), AutoFishingRodNetError> {
    if fish_label < rule.label_count {
        Ok(())
    } else {
        Err(AutoFishingRodNetError::InvalidFishLabel {
            fish_label,
            label_count: rule.label_count,
        })
    }
}

fn validate_auto_fishing_rod_shapes(
    rule: &AutoFishingRodNetRule,
) -> Result<(), AutoFishingRodNetError> {
    let valid = rule.offset_values.len() == rule.label_count
        && rule.dz_values.len() == rule.label_count
        && rule.h_coeff_values.len() == rule.label_count
        && rule.weight_values.len() == rule.label_count
        && rule.bias_values.len() == rule.label_count
        && rule.state_ready as usize <= 2
        && rule.state_too_close as usize <= 2
        && rule.state_too_far as usize <= 2;
    if valid {
        Ok(())
    } else {
        Err(AutoFishingRodNetError::InvalidParameterShape)
    }
}

fn softmax3(logits: [f64; 3]) -> [f64; 3] {
    let exp0 = logits[0].exp();
    let exp1 = logits[1].exp();
    let exp2 = logits[2].exp();
    let sum = exp0 + exp1 + exp2;
    [exp0 / sum, exp1 / sum, exp2 / sum]
}

fn fishing_time_policy(value: &Value) -> AutoFishingTimePolicy {
    match value {
        Value::String(value) => match value.as_str() {
            "All" => AutoFishingTimePolicy::All,
            "Daytime" => AutoFishingTimePolicy::Daytime,
            "Nighttime" => AutoFishingTimePolicy::Nighttime,
            "DontChange" => AutoFishingTimePolicy::DontChange,
            other => AutoFishingTimePolicy::Unknown(other.to_string()),
        },
        Value::Number(value) => match value.as_u64() {
            Some(0) => AutoFishingTimePolicy::All,
            Some(1) => AutoFishingTimePolicy::Daytime,
            Some(2) => AutoFishingTimePolicy::Nighttime,
            Some(3) => AutoFishingTimePolicy::DontChange,
            _ => AutoFishingTimePolicy::Unknown(value.to_string()),
        },
        _ => AutoFishingTimePolicy::Unknown(value.to_string()),
    }
}

fn bait_types() -> Vec<AutoFishingBaitType> {
    vec![
        AutoFishingBaitType::FruitPasteBait,
        AutoFishingBaitType::RedrotBait,
        AutoFishingBaitType::FalseWormBait,
        AutoFishingBaitType::FakeFlyBait,
        AutoFishingBaitType::SugardewBait,
        AutoFishingBaitType::SourBait,
        AutoFishingBaitType::FlashingMaintenanceMekBait,
        AutoFishingBaitType::SpinelgrainBait,
        AutoFishingBaitType::EmberglowBait,
        AutoFishingBaitType::BerryBait,
        AutoFishingBaitType::RefreshingLakkaBait,
    ]
}

fn big_fish_types() -> Vec<AutoFishingBigFishTypeRule> {
    use AutoFishingBaitType::*;
    [
        ("medaka", FruitPasteBait, "花鳉", 0),
        ("large medaka", FruitPasteBait, "大花鳉", 1),
        ("stickleback", RedrotBait, "棘鱼", 2),
        ("koi", FakeFlyBait, "假龙", 3),
        ("koi head", FakeFlyBait, "假龙头", 3),
        ("butterflyfish", FalseWormBait, "蝶鱼", 4),
        ("pufferfish", FakeFlyBait, "炮鲀", 5),
        ("ray", FakeFlyBait, "鳐", 6),
        ("angler", SugardewBait, "角鲀", 7),
        ("axe marlin", SugardewBait, "斧枪鱼", 8),
        ("heartfeather bass", SourBait, "心羽鲈", 9),
        (
            "maintenance mek",
            FlashingMaintenanceMekBait,
            "维护机关",
            10,
        ),
        ("unihornfish", SpinelgrainBait, "独角鱼", 10),
        ("sunfish", SpinelgrainBait, "翻车鲀", 7),
        ("rapidfish", SpinelgrainBait, "斗士急流鱼", 9),
        ("phony unihornfish", EmberglowBait, "燃素独角鱼", 10),
        ("magma rapidfish", EmberglowBait, "炽岩斗士急流鱼", 9),
        ("secret source", EmberglowBait, "秘源机关・巡戒使", 9),
        ("mauler shark", RefreshingLakkaBait, "凶凶鲨", 9),
        ("crystal eye", RefreshingLakkaBait, "明眼鱼", 9),
        ("axehead", BerryBait, "巨斧鱼", 9),
    ]
    .into_iter()
    .map(
        |(name, bait, chinese_name, net_index)| AutoFishingBigFishTypeRule {
            name: name.to_string(),
            bait,
            chinese_name: chinese_name.to_string(),
            net_index,
        },
    )
    .collect()
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

fn bool_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<bool> {
    keys.into_iter()
        .filter_map(|key| value.get(key))
        .find_map(Value::as_bool)
}
