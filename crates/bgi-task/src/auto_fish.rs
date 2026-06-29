use bgi_core::AutoFishingConfig;
use bgi_vision::{Rect, RgbPixel, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Result;

pub const AUTO_FISH_TASK_KEY: &str = "AutoFish";
pub const AUTO_FISH_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_FISH_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_FISH_SPACE_BUTTON: &str = "AutoFishing:space.png";
pub const AUTO_FISH_BAIT_BUTTON: &str = "AutoFishing:switch_bait.png";
pub const AUTO_FISH_WAIT_BITE_BUTTON: &str = "AutoFishing:wait_bite.png";
pub const AUTO_FISH_LIFT_ROD_BUTTON: &str = "AutoFishing:lift_rod.png";
pub const AUTO_FISH_EXIT_FISHING_BUTTON: &str = "AutoFishing:exit_fishing.png";
pub const AUTO_FISH_BITE_TIPS_OVERLAY: &str = "FishBiteTips";
pub const AUTO_FISH_BOX_OVERLAY: &str = "FishBox";
pub const AUTO_FISH_BAR_OVERLAY: &str = "FishingBarAll";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub config_rule: AutoFishConfigRule,
    pub trigger_rule: AutoFishTriggerRule,
    pub locators: AutoFishLocators,
    pub behavior_tree_rule: AutoFishBehaviorTreeRule,
    pub blackboard_rule: AutoFishBlackboardRule,
    pub bite_rule: AutoFishBiteRule,
    pub fish_bar_rule: AutoFishBarRule,
    pub full_task_rule: AutoFishFullTaskRule,
    pub fishing_input_rule: AutoFishInputRule,
    pub steps: Vec<AutoFishTickStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoFishExecutionConfig {
    pub capture_size: Size,
    pub auto_fishing_config: AutoFishingConfig,
}

impl Default for AutoFishExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_FISH_DEFAULT_CAPTURE_WIDTH,
                AUTO_FISH_DEFAULT_CAPTURE_HEIGHT,
            ),
            auto_fishing_config: AutoFishingConfig::default(),
        }
    }
}

impl AutoFishExecutionConfig {
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
        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishConfigRule {
    pub enabled: bool,
    pub auto_throw_rod_enabled: bool,
    pub auto_throw_rod_timeout_seconds: u64,
    pub whole_process_timeout_seconds: u64,
    pub fishing_time_policy: AutoFishTimePolicy,
    pub fishing_time_policy_raw: Value,
    pub torch_dll_full_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AutoFishTimePolicy {
    All,
    Daytime,
    Nighttime,
    DontChange,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishTriggerRule {
    pub priority: i32,
    pub initial_exclusive: bool,
    pub exclusive_when_exit_button_detected: bool,
    pub dynamic_add_trigger_supported_by_desktop_bridge: bool,
    pub tick_throttle_ms: u64,
    pub behavior_tree_policy: String,
    pub creates_bgi_fish_yolo_predictor: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishLocators {
    pub space_button: AutoFishTemplateLocator,
    pub bait_button: AutoFishTemplateLocator,
    pub wait_bite_button: AutoFishTemplateLocator,
    pub lift_rod_button: AutoFishTemplateLocator,
    pub exit_fishing_button: AutoFishTemplateLocator,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Rect,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub use_3_channels: bool,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishBehaviorTreeRule {
    pub root: String,
    pub parallel_policy: String,
    pub exclusive_gate_leaf: String,
    pub semi_auto_sequence: Vec<String>,
    pub auto_throw_rod_branch_present_but_disabled_in_trigger: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishBlackboardRule {
    pub tracks_selected_bait: bool,
    pub tracks_fishpond: bool,
    pub tracks_throw_rod_no_target: bool,
    pub tracks_throw_rod_no_bait_fish: bool,
    pub tracks_failure_lists: bool,
    pub tracks_fish_box_rect: bool,
    pub choose_bait_ui_blocks_fishing_ui_detection: bool,
    pub pitch_reset_initial_value: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishBiteRule {
    pub localized_bite_text_default: String,
    pub lifting_words_area: Rect,
    pub white_lower_rgb: RgbPixel,
    pub white_upper_rgb: RgbPixel,
    pub dilate_kernel: Size,
    pub min_word_aspect_ratio: f64,
    pub word_area_width_must_exceed_text_width_times: f64,
    pub center_x_must_intersect_text_rect: bool,
    pub detection_order: Vec<String>,
    pub action: AutoFishInputAction,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishBarRule {
    pub initial_search_region: Rect,
    pub wait_fish_box_appear_seconds: u64,
    pub hsv_full_base: AutoFishHsvColor,
    pub hsv_full_low_delta: AutoFishHsvColorDelta,
    pub hsv_full_high_delta: AutoFishHsvColorDelta,
    pub contour_angle_mod_45_max: f64,
    pub same_line_center_y_tolerance_divisor: f64,
    pub height_difference_tolerance_divisor: f64,
    pub min_width_height_divisor: f64,
    pub initial_rect_height_diff_max: i32,
    pub two_rect_target_min_cursor_width_multiplier: f64,
    pub max_rect_count_before_taking_highest_three: usize,
    pub no_detection_finish_grace_seconds: u64,
    pub fish_box_expansion: AutoFishBoxExpansionRule,
    pub overlay_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishHsvColor {
    pub h: f64,
    pub s: f64,
    pub v: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishHsvColorDelta {
    pub h: f64,
    pub s: f64,
    pub v: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishBoxExpansionRule {
    pub horizontal_extra_source: String,
    pub vertical_extra_source: String,
    pub right_edge_source: String,
    pub clamps_to_capture: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishFullTaskRule {
    pub independent_task_key: String,
    pub runtime_catalog_remains_native_pending: bool,
    pub disables_realtime_config_on_start: bool,
    pub daytime_hours: Vec<u8>,
    pub nighttime_hours: Vec<u8>,
    pub all_policy_hours: Vec<u8>,
    pub rod_net_rule: AutoFishRodNetRule,
    pub auto_throw_rod_rule: AutoFishThrowRodRule,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFishRodNetRule {
    pub model_input_size: Size,
    pub alpha: f64,
    pub state_ready: u8,
    pub state_too_close: u8,
    pub state_too_far: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishThrowRodRule {
    pub find_target_timeout_seconds: u64,
    pub ignore_obtained_seconds: u64,
    pub no_bait_fish_failures_before_switch: u64,
    pub no_target_retries_before_restart: u64,
    pub drop_point_failures_before_abort: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishInputRule {
    pub raise_rod_action: AutoFishInputAction,
    pub fishing_left_of_target_action: AutoFishInputAction,
    pub fishing_past_target_action: AutoFishInputAction,
    pub fishing_center_bias_action: AutoFishInputAction,
    pub completion_action: AutoFishInputAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoFishInputAction {
    LeftButtonClick,
    LeftButtonDown,
    LeftButtonUp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoFishTickStep {
    pub phase: AutoFishTickPhase,
    pub condition: AutoFishTickCondition,
    pub action: AutoFishTickAction,
}

impl AutoFishTickStep {
    fn new(
        phase: AutoFishTickPhase,
        condition: AutoFishTickCondition,
        action: AutoFishTickAction,
    ) -> Self {
        Self {
            phase,
            condition,
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishTickPhase {
    Throttle,
    ExclusiveGate,
    BiteDetection,
    FishBoxDetection,
    FishingBar,
    Completion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishTickCondition {
    WhenTickElapsedLessOrEqual67Ms,
    WhenNotExclusive,
    WhenExitFishingButtonDetected,
    WhenBiteTextBlockOrLiftButtonOrOcrDetected,
    WhenFishBoxRectsFound,
    WhenCursorBeforeTarget,
    WhenCursorPastTarget,
    WhenNoBarDetectedForGraceWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoFishTickAction {
    SkipTick,
    DetectFishingUiAndToggleExclusive,
    TickSemiAutoFishingTree,
    RaiseRod,
    CaptureFishBoxArea,
    HoldLeftButton,
    ReleaseLeftButton,
    ClearFishingOverlays,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoFishTickStage {
    #[default]
    WaitingForBite,
    WaitingForFishBox,
    Fishing,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFishTriggerState {
    pub prev_execute_ms: Option<u64>,
    pub exclusive: bool,
    pub stage: AutoFishTickStage,
    pub fish_box_rect: Option<Rect>,
    pub left_button_down: bool,
    pub no_bar_since_ms: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFishTickObservation {
    pub now_ms: u64,
    pub exit_fishing_button_detected: bool,
    pub bite_text_block_detected: bool,
    pub lift_rod_button_detected: bool,
    pub bite_ocr_text: Option<String>,
    pub fish_box_rects: Vec<Rect>,
    pub fishing_bar_rects: Vec<Rect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFishTickDecisionReport {
    pub processed: bool,
    pub skip_reason: Option<AutoFishTickSkipReason>,
    pub exclusive_before: bool,
    pub exclusive_after: bool,
    pub stage_before: AutoFishTickStage,
    pub stage_after: AutoFishTickStage,
    pub bite_method: Option<AutoFishBiteDetectionMethod>,
    pub fish_box: Option<AutoFishFishBoxDecisionReport>,
    pub bar: Option<AutoFishBarDecisionReport>,
    pub actions: Vec<AutoFishRuntimeAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoFishTickSkipReason {
    Disabled,
    TickThrottle,
    NotExclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoFishBiteDetectionMethod {
    WordBlock,
    LiftRodButton,
    Ocr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFishFishBoxDecisionReport {
    pub decision: AutoFishFishBoxDecisionKind,
    pub fish_box_rect: Option<Rect>,
    pub cursor_rect: Option<Rect>,
    pub target_rect: Option<Rect>,
    pub considered_rects: Vec<Rect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoFishFishBoxDecisionKind {
    WaitingForTwoRects,
    HeightMismatch,
    InvalidGeometry,
    Detected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFishBarDecisionReport {
    pub decision: AutoFishBarDecisionKind,
    pub relation: Option<AutoFishBarTargetRelation>,
    pub considered_rects: Vec<Rect>,
    pub no_bar_since_ms: Option<u64>,
    pub no_bar_grace_elapsed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoFishBarDecisionKind {
    PressLeftButton,
    ReleaseLeftButton,
    KeepLeftButtonDown,
    KeepLeftButtonUp,
    InvalidTwoRectTarget,
    IgnoreUnexpectedRectCount,
    ArmNoBarGrace,
    WaitNoBarGrace,
    CompleteNoBarGrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoFishBarTargetRelation {
    CursorBeforeTarget,
    CursorPastTarget,
    CenterBias,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoFishRuntimeAction {
    Input {
        action: AutoFishInputAction,
        reason: AutoFishRuntimeActionReason,
    },
    Overlay {
        action: AutoFishOverlayAction,
        reason: AutoFishRuntimeActionReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoFishRuntimeActionReason {
    RaiseRod,
    FishBoxDetected,
    FishingBarDetected,
    CursorBeforeTarget,
    CursorPastTarget,
    NoBarGrace,
    CompletionCleanup,
    ExclusiveLost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoFishOverlayAction {
    Clear { key: String },
    DrawFishBox { key: String, rect: Rect },
    DrawFishingBar { key: String, rects: Vec<Rect> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFishRuntimeActionReport {
    pub action: AutoFishRuntimeAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFishTickExecutionReport {
    pub task_key: String,
    pub observation: AutoFishTickObservation,
    pub decision: AutoFishTickDecisionReport,
    pub runtime_actions: Vec<AutoFishRuntimeActionReport>,
}

pub trait AutoFishRuntime {
    fn auto_fish_now_ms(&mut self, plan: &AutoFishExecutionPlan) -> Result<u64>;

    fn detect_auto_fish_template(&mut self, locator: &AutoFishTemplateLocator) -> Result<bool>;

    fn detect_auto_fish_bite_text_block(&mut self, rule: &AutoFishBiteRule) -> Result<bool>;

    fn ocr_auto_fish_bite_text(&mut self, rule: &AutoFishBiteRule) -> Result<Option<String>>;

    fn detect_auto_fish_fish_box_rects(
        &mut self,
        plan: &AutoFishExecutionPlan,
    ) -> Result<Vec<Rect>>;

    fn detect_auto_fish_fishing_bar_rects(
        &mut self,
        plan: &AutoFishExecutionPlan,
        fish_box_rect: Rect,
    ) -> Result<Vec<Rect>>;

    fn dispatch_auto_fish_input(&mut self, action: AutoFishInputAction) -> Result<()>;

    fn update_auto_fish_overlay(&mut self, action: &AutoFishOverlayAction) -> Result<()>;
}

pub fn plan_auto_fish(config: AutoFishExecutionConfig) -> AutoFishExecutionPlan {
    let capture_size = config.capture_size;
    let auto_fishing_config = config.auto_fishing_config;
    AutoFishExecutionPlan {
        task_key: AUTO_FISH_TASK_KEY.to_string(),
        display_name: "Auto Fishing Trigger".to_string(),
        capture_size,
        config_rule: AutoFishConfigRule {
            enabled: auto_fishing_config.enabled,
            auto_throw_rod_enabled: auto_fishing_config.auto_throw_rod_enabled,
            auto_throw_rod_timeout_seconds: auto_fishing_config.auto_throw_rod_time_out,
            whole_process_timeout_seconds: auto_fishing_config.whole_process_timeout_seconds,
            fishing_time_policy: fishing_time_policy(&auto_fishing_config.fishing_time_policy),
            fishing_time_policy_raw: auto_fishing_config.fishing_time_policy,
            torch_dll_full_path: auto_fishing_config.torch_dll_full_path,
        },
        trigger_rule: AutoFishTriggerRule {
            priority: 15,
            initial_exclusive: false,
            exclusive_when_exit_button_detected: true,
            dynamic_add_trigger_supported_by_desktop_bridge: false,
            tick_throttle_ms: 67,
            behavior_tree_policy: "MySimpleParallel(root, OnlyOneMustSucceed)".to_string(),
            creates_bgi_fish_yolo_predictor: true,
        },
        locators: auto_fish_locators(capture_size),
        behavior_tree_rule: AutoFishBehaviorTreeRule {
            root: "root".to_string(),
            parallel_policy: "OnlyOneMustSucceed".to_string(),
            exclusive_gate_leaf: "CheckFishingUserInterface".to_string(),
            semi_auto_sequence: vec![
                "FishBite".to_string(),
                "GetFishBoxArea".to_string(),
                "Fishing".to_string(),
            ],
            auto_throw_rod_branch_present_but_disabled_in_trigger: true,
        },
        blackboard_rule: AutoFishBlackboardRule {
            tracks_selected_bait: true,
            tracks_fishpond: true,
            tracks_throw_rod_no_target: true,
            tracks_throw_rod_no_bait_fish: true,
            tracks_failure_lists: true,
            tracks_fish_box_rect: true,
            choose_bait_ui_blocks_fishing_ui_detection: true,
            pitch_reset_initial_value: true,
        },
        bite_rule: AutoFishBiteRule {
            localized_bite_text_default: "上钩".to_string(),
            lifting_words_area: Rect {
                x: capture_size.width as i32 / 3,
                y: 0,
                width: capture_size.width as i32 / 3,
                height: capture_size.height as i32 / 2,
            },
            white_lower_rgb: RgbPixel {
                r: 253,
                g: 253,
                b: 253,
            },
            white_upper_rgb: RgbPixel {
                r: 255,
                g: 255,
                b: 255,
            },
            dilate_kernel: Size::new(20, 20),
            min_word_aspect_ratio: 3.0,
            word_area_width_must_exceed_text_width_times: 3.0,
            center_x_must_intersect_text_rect: true,
            detection_order: vec![
                "white text block".to_string(),
                "LiftRodButton template".to_string(),
                "Paddle OCR contains localized bite text".to_string(),
            ],
            action: AutoFishInputAction::LeftButtonClick,
        },
        fish_bar_rule: AutoFishBarRule {
            initial_search_region: Rect {
                x: 0,
                y: 0,
                width: capture_size.width as i32,
                height: capture_size.height as i32 / 2,
            },
            wait_fish_box_appear_seconds: 5,
            hsv_full_base: AutoFishHsvColor {
                h: 60.0,
                s: 0.25,
                v: 1.0,
            },
            hsv_full_low_delta: AutoFishHsvColorDelta {
                h: -3.0,
                s: -20.0,
                v: -10.0,
            },
            hsv_full_high_delta: AutoFishHsvColorDelta {
                h: 3.5,
                s: 40.0,
                v: 0.0,
            },
            contour_angle_mod_45_max: 1.0,
            same_line_center_y_tolerance_divisor: 5.0,
            height_difference_tolerance_divisor: 3.0,
            min_width_height_divisor: 4.0,
            initial_rect_height_diff_max: 10,
            two_rect_target_min_cursor_width_multiplier: 10.0,
            max_rect_count_before_taking_highest_three: 3,
            no_detection_finish_grace_seconds: 1,
            fish_box_expansion: AutoFishBoxExpansionRule {
                horizontal_extra_source: "cursor height".to_string(),
                vertical_extra_source: "cursor height / 4".to_string(),
                right_edge_source: "(top width / 2 - cursor.x) * 2 + horizontal extra * 2"
                    .to_string(),
                clamps_to_capture: true,
            },
            overlay_keys: vec![
                "FishBiteTips".to_string(),
                "FishBox".to_string(),
                "FishingBarAll".to_string(),
            ],
        },
        full_task_rule: AutoFishFullTaskRule {
            independent_task_key: "AutoFishing".to_string(),
            runtime_catalog_remains_native_pending: true,
            disables_realtime_config_on_start: true,
            daytime_hours: vec![7],
            nighttime_hours: vec![19],
            all_policy_hours: vec![7, 19],
            rod_net_rule: AutoFishRodNetRule {
                model_input_size: Size::new(1024, 576),
                alpha: 1734.34 / 2.5,
                state_ready: 0,
                state_too_close: 1,
                state_too_far: 2,
            },
            auto_throw_rod_rule: AutoFishThrowRodRule {
                find_target_timeout_seconds: 5,
                ignore_obtained_seconds: 6,
                no_bait_fish_failures_before_switch: 10,
                no_target_retries_before_restart: 25,
                drop_point_failures_before_abort: 2,
            },
        },
        fishing_input_rule: AutoFishInputRule {
            raise_rod_action: AutoFishInputAction::LeftButtonClick,
            fishing_left_of_target_action: AutoFishInputAction::LeftButtonDown,
            fishing_past_target_action: AutoFishInputAction::LeftButtonUp,
            fishing_center_bias_action: AutoFishInputAction::LeftButtonUp,
            completion_action: AutoFishInputAction::LeftButtonUp,
        },
        steps: auto_fish_steps(),
        executor_ready: true,
        pending_native: vec![
            "desktop live adapter now covers BitBlt capture, fishing UI/lift-rod template matching, SendInput left-button actions, and no-op overlay cleanup; white text block detection, Paddle OCR, fish-box contour, and fishing-bar contour adapters remain pending".to_string(),
            "BgiFish YOLO predictor construction and bait selection remain in full AutoFishingTask"
                .to_string(),
            "RodNet inference and full AutoFishingTask throw-rod branch remain native-pending"
                .to_string(),
        ],
    }
}

pub fn execute_auto_fish_tick_plan<R>(
    plan: &AutoFishExecutionPlan,
    state: &mut AutoFishTriggerState,
    runtime: &mut R,
) -> Result<AutoFishTickExecutionReport>
where
    R: AutoFishRuntime,
{
    let observation = observe_auto_fish_tick(plan, state, runtime)?;
    let decision = decide_auto_fish_tick(plan, state, observation.clone());
    let mut runtime_actions = Vec::new();
    for action in &decision.actions {
        dispatch_auto_fish_runtime_action(runtime, action)?;
        runtime_actions.push(AutoFishRuntimeActionReport {
            action: action.clone(),
        });
    }

    Ok(AutoFishTickExecutionReport {
        task_key: plan.task_key.clone(),
        observation,
        decision,
        runtime_actions,
    })
}

pub fn observe_auto_fish_tick<R>(
    plan: &AutoFishExecutionPlan,
    state: &AutoFishTriggerState,
    runtime: &mut R,
) -> Result<AutoFishTickObservation>
where
    R: AutoFishRuntime,
{
    let now_ms = runtime.auto_fish_now_ms(plan)?;
    let exit_fishing_button_detected =
        runtime.detect_auto_fish_template(&plan.locators.exit_fishing_button)?;

    if !plan.config_rule.enabled
        || tick_elapsed_ms(state.prev_execute_ms, now_ms) <= plan.trigger_rule.tick_throttle_ms
    {
        return Ok(AutoFishTickObservation {
            now_ms,
            exit_fishing_button_detected,
            ..AutoFishTickObservation::default()
        });
    }

    if !exit_fishing_button_detected {
        return Ok(AutoFishTickObservation {
            now_ms,
            exit_fishing_button_detected,
            ..AutoFishTickObservation::default()
        });
    }

    let bite_text_block_detected = matches!(state.stage, AutoFishTickStage::WaitingForBite)
        && runtime.detect_auto_fish_bite_text_block(&plan.bite_rule)?;
    let lift_rod_button_detected = matches!(state.stage, AutoFishTickStage::WaitingForBite)
        && runtime.detect_auto_fish_template(&plan.locators.lift_rod_button)?;
    let bite_ocr_text = if matches!(state.stage, AutoFishTickStage::WaitingForBite)
        && !bite_text_block_detected
        && !lift_rod_button_detected
    {
        runtime.ocr_auto_fish_bite_text(&plan.bite_rule)?
    } else {
        None
    };

    let fish_box_rects = if matches!(state.stage, AutoFishTickStage::WaitingForFishBox) {
        runtime.detect_auto_fish_fish_box_rects(plan)?
    } else {
        Vec::new()
    };
    let fishing_bar_rects = if matches!(state.stage, AutoFishTickStage::Fishing) {
        if let Some(fish_box_rect) = state.fish_box_rect {
            runtime.detect_auto_fish_fishing_bar_rects(plan, fish_box_rect)?
        } else {
            runtime.detect_auto_fish_fish_box_rects(plan)?
        }
    } else {
        Vec::new()
    };

    Ok(AutoFishTickObservation {
        now_ms,
        exit_fishing_button_detected,
        bite_text_block_detected,
        lift_rod_button_detected,
        bite_ocr_text,
        fish_box_rects,
        fishing_bar_rects,
    })
}

pub fn decide_auto_fish_tick(
    plan: &AutoFishExecutionPlan,
    state: &mut AutoFishTriggerState,
    observation: AutoFishTickObservation,
) -> AutoFishTickDecisionReport {
    let exclusive_before = state.exclusive;
    let stage_before = state.stage;

    if !plan.config_rule.enabled {
        return auto_fish_skip_report(
            AutoFishTickSkipReason::Disabled,
            exclusive_before,
            stage_before,
            state,
        );
    }

    if tick_elapsed_ms(state.prev_execute_ms, observation.now_ms)
        <= plan.trigger_rule.tick_throttle_ms
    {
        return auto_fish_skip_report(
            AutoFishTickSkipReason::TickThrottle,
            exclusive_before,
            stage_before,
            state,
        );
    }

    state.prev_execute_ms = Some(observation.now_ms);
    state.exclusive = observation.exit_fishing_button_detected;

    if !state.exclusive {
        let mut actions = Vec::new();
        if state.left_button_down {
            actions.push(AutoFishRuntimeAction::Input {
                action: plan.fishing_input_rule.completion_action,
                reason: AutoFishRuntimeActionReason::ExclusiveLost,
            });
        }
        actions.extend(auto_fish_completion_overlay_actions(
            &plan.fish_bar_rule.overlay_keys,
            AutoFishRuntimeActionReason::ExclusiveLost,
        ));
        state.stage = AutoFishTickStage::WaitingForBite;
        state.fish_box_rect = None;
        state.left_button_down = false;
        state.no_bar_since_ms = None;

        return AutoFishTickDecisionReport {
            processed: false,
            skip_reason: Some(AutoFishTickSkipReason::NotExclusive),
            exclusive_before,
            exclusive_after: state.exclusive,
            stage_before,
            stage_after: state.stage,
            bite_method: None,
            fish_box: None,
            bar: None,
            actions,
        };
    }

    match state.stage {
        AutoFishTickStage::WaitingForBite => {
            let Some(method) = detect_auto_fish_bite_method(&observation, &plan.bite_rule) else {
                return AutoFishTickDecisionReport {
                    processed: true,
                    skip_reason: None,
                    exclusive_before,
                    exclusive_after: state.exclusive,
                    stage_before,
                    stage_after: state.stage,
                    bite_method: None,
                    fish_box: None,
                    bar: None,
                    actions: Vec::new(),
                };
            };

            state.stage = AutoFishTickStage::WaitingForFishBox;
            state.fish_box_rect = None;
            state.no_bar_since_ms = None;
            state.left_button_down = false;
            AutoFishTickDecisionReport {
                processed: true,
                skip_reason: None,
                exclusive_before,
                exclusive_after: state.exclusive,
                stage_before,
                stage_after: state.stage,
                bite_method: Some(method),
                fish_box: None,
                bar: None,
                actions: vec![
                    AutoFishRuntimeAction::Overlay {
                        action: AutoFishOverlayAction::Clear {
                            key: AUTO_FISH_BITE_TIPS_OVERLAY.to_string(),
                        },
                        reason: AutoFishRuntimeActionReason::RaiseRod,
                    },
                    AutoFishRuntimeAction::Input {
                        action: plan.fishing_input_rule.raise_rod_action,
                        reason: AutoFishRuntimeActionReason::RaiseRod,
                    },
                ],
            }
        }
        AutoFishTickStage::WaitingForFishBox => {
            let fish_box = resolve_auto_fish_fish_box(
                observation.fish_box_rects.clone(),
                plan.capture_size,
                &plan.fish_bar_rule,
            );
            let mut actions = Vec::new();
            if let Some(rect) = fish_box.fish_box_rect {
                state.stage = AutoFishTickStage::Fishing;
                state.fish_box_rect = Some(rect);
                state.no_bar_since_ms = None;
                actions.push(AutoFishRuntimeAction::Overlay {
                    action: AutoFishOverlayAction::DrawFishBox {
                        key: AUTO_FISH_BOX_OVERLAY.to_string(),
                        rect,
                    },
                    reason: AutoFishRuntimeActionReason::FishBoxDetected,
                });
            }

            AutoFishTickDecisionReport {
                processed: true,
                skip_reason: None,
                exclusive_before,
                exclusive_after: state.exclusive,
                stage_before,
                stage_after: state.stage,
                bite_method: None,
                fish_box: Some(fish_box),
                bar: None,
                actions,
            }
        }
        AutoFishTickStage::Fishing => {
            let bar = resolve_auto_fish_bar(
                observation.fishing_bar_rects.clone(),
                state.left_button_down,
                state.no_bar_since_ms,
                observation.now_ms,
                &plan.fish_bar_rule,
            );

            state.left_button_down = match bar.decision {
                AutoFishBarDecisionKind::PressLeftButton
                | AutoFishBarDecisionKind::KeepLeftButtonDown => true,
                AutoFishBarDecisionKind::ReleaseLeftButton
                | AutoFishBarDecisionKind::KeepLeftButtonUp
                | AutoFishBarDecisionKind::CompleteNoBarGrace => false,
                AutoFishBarDecisionKind::InvalidTwoRectTarget
                | AutoFishBarDecisionKind::IgnoreUnexpectedRectCount
                | AutoFishBarDecisionKind::ArmNoBarGrace
                | AutoFishBarDecisionKind::WaitNoBarGrace => state.left_button_down,
            };
            state.no_bar_since_ms = bar.no_bar_since_ms;

            let mut actions = auto_fish_bar_actions(plan, &bar);
            if matches!(bar.decision, AutoFishBarDecisionKind::CompleteNoBarGrace) {
                state.stage = AutoFishTickStage::WaitingForBite;
                state.fish_box_rect = None;
                state.no_bar_since_ms = None;
                actions.extend(auto_fish_completion_overlay_actions(
                    &plan.fish_bar_rule.overlay_keys,
                    AutoFishRuntimeActionReason::CompletionCleanup,
                ));
            }

            AutoFishTickDecisionReport {
                processed: true,
                skip_reason: None,
                exclusive_before,
                exclusive_after: state.exclusive,
                stage_before,
                stage_after: state.stage,
                bite_method: None,
                fish_box: None,
                bar: Some(bar),
                actions,
            }
        }
    }
}

fn auto_fish_skip_report(
    reason: AutoFishTickSkipReason,
    exclusive_before: bool,
    stage_before: AutoFishTickStage,
    state: &AutoFishTriggerState,
) -> AutoFishTickDecisionReport {
    AutoFishTickDecisionReport {
        processed: false,
        skip_reason: Some(reason),
        exclusive_before,
        exclusive_after: state.exclusive,
        stage_before,
        stage_after: state.stage,
        bite_method: None,
        fish_box: None,
        bar: None,
        actions: Vec::new(),
    }
}

fn detect_auto_fish_bite_method(
    observation: &AutoFishTickObservation,
    rule: &AutoFishBiteRule,
) -> Option<AutoFishBiteDetectionMethod> {
    if observation.bite_text_block_detected {
        return Some(AutoFishBiteDetectionMethod::WordBlock);
    }
    if observation.lift_rod_button_detected {
        return Some(AutoFishBiteDetectionMethod::LiftRodButton);
    }
    if observation
        .bite_ocr_text
        .as_deref()
        .is_some_and(|text| text.contains(&rule.localized_bite_text_default))
    {
        return Some(AutoFishBiteDetectionMethod::Ocr);
    }
    None
}

fn resolve_auto_fish_fish_box(
    rects: Vec<Rect>,
    capture_size: Size,
    rule: &AutoFishBarRule,
) -> AutoFishFishBoxDecisionReport {
    if rects.len() != 2 {
        return AutoFishFishBoxDecisionReport {
            decision: AutoFishFishBoxDecisionKind::WaitingForTwoRects,
            fish_box_rect: None,
            cursor_rect: None,
            target_rect: None,
            considered_rects: rects,
        };
    }

    let left = rects[0];
    let right = rects[1];
    if (left.height - right.height).abs() > rule.initial_rect_height_diff_max {
        return AutoFishFishBoxDecisionReport {
            decision: AutoFishFishBoxDecisionKind::HeightMismatch,
            fish_box_rect: None,
            cursor_rect: None,
            target_rect: None,
            considered_rects: rects,
        };
    }

    let (cursor, target) = if left.width < right.width {
        (left, right)
    } else {
        (right, left)
    };
    let top_width = capture_size.width as i32;
    let top_mid_x = top_width / 2;
    let cursor_right = cursor.x + cursor.width;
    let invalid = target.x < cursor.x
        || cursor.width > target.width
        || cursor_right > top_mid_x
        || cursor_right > target.x - target.width / 2
        || cursor_right > top_mid_x - target.width;
    if invalid {
        return AutoFishFishBoxDecisionReport {
            decision: AutoFishFishBoxDecisionKind::InvalidGeometry,
            fish_box_rect: None,
            cursor_rect: Some(cursor),
            target_rect: Some(target),
            considered_rects: rects,
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

    match raw.clamp_to(capture_size).ok() {
        Some(fish_box_rect) => AutoFishFishBoxDecisionReport {
            decision: AutoFishFishBoxDecisionKind::Detected,
            fish_box_rect: Some(fish_box_rect),
            cursor_rect: Some(cursor),
            target_rect: Some(target),
            considered_rects: rects,
        },
        None => AutoFishFishBoxDecisionReport {
            decision: AutoFishFishBoxDecisionKind::InvalidGeometry,
            fish_box_rect: None,
            cursor_rect: Some(cursor),
            target_rect: Some(target),
            considered_rects: rects,
        },
    }
}

fn resolve_auto_fish_bar(
    mut rects: Vec<Rect>,
    left_button_down: bool,
    no_bar_since_ms: Option<u64>,
    now_ms: u64,
    rule: &AutoFishBarRule,
) -> AutoFishBarDecisionReport {
    if rects.is_empty() {
        let Some(armed_since_ms) = no_bar_since_ms else {
            return AutoFishBarDecisionReport {
                decision: AutoFishBarDecisionKind::ArmNoBarGrace,
                relation: None,
                considered_rects: Vec::new(),
                no_bar_since_ms: Some(now_ms),
                no_bar_grace_elapsed: false,
            };
        };

        let grace_elapsed =
            now_ms.saturating_sub(armed_since_ms) >= rule.no_detection_finish_grace_seconds * 1_000;
        if !grace_elapsed {
            return AutoFishBarDecisionReport {
                decision: AutoFishBarDecisionKind::WaitNoBarGrace,
                relation: None,
                considered_rects: Vec::new(),
                no_bar_since_ms: Some(armed_since_ms),
                no_bar_grace_elapsed: false,
            };
        }

        return AutoFishBarDecisionReport {
            decision: AutoFishBarDecisionKind::CompleteNoBarGrace,
            relation: None,
            considered_rects: Vec::new(),
            no_bar_since_ms: None,
            no_bar_grace_elapsed: true,
        };
    }

    if rects.len() > rule.max_rect_count_before_taking_highest_three {
        rects.sort_by_key(|rect| std::cmp::Reverse(rect.height));
        rects.truncate(rule.max_rect_count_before_taking_highest_three);
    }

    if rects.len() == 2 {
        let (cursor, target) = if rects[0].width < rects[1].width {
            (rects[0], rects[1])
        } else {
            (rects[1], rects[0])
        };
        if (target.width as f64)
            < cursor.width as f64 * rule.two_rect_target_min_cursor_width_multiplier
        {
            return AutoFishBarDecisionReport {
                decision: AutoFishBarDecisionKind::InvalidTwoRectTarget,
                relation: None,
                considered_rects: vec![target, cursor],
                no_bar_since_ms: None,
                no_bar_grace_elapsed: false,
            };
        }

        let relation = if cursor.x < target.x {
            AutoFishBarTargetRelation::CursorBeforeTarget
        } else {
            AutoFishBarTargetRelation::CursorPastTarget
        };
        return auto_fish_bar_button_report(left_button_down, vec![target, cursor], relation);
    }

    if rects.len() == 3 {
        rects.sort_by_key(|rect| rect.x);
        let left = rects[0];
        let cursor = rects[1];
        let right = rects[2];
        let right_remaining = right.x + right.width - (cursor.x + cursor.width);
        let left_distance = cursor.x - left.x;
        let relation = if right_remaining > left_distance {
            AutoFishBarTargetRelation::CursorBeforeTarget
        } else {
            AutoFishBarTargetRelation::CursorPastTarget
        };
        return auto_fish_bar_button_report(left_button_down, vec![left, cursor, right], relation);
    }

    AutoFishBarDecisionReport {
        decision: AutoFishBarDecisionKind::IgnoreUnexpectedRectCount,
        relation: None,
        considered_rects: Vec::new(),
        no_bar_since_ms: None,
        no_bar_grace_elapsed: false,
    }
}

fn auto_fish_bar_button_report(
    left_button_down: bool,
    considered_rects: Vec<Rect>,
    relation: AutoFishBarTargetRelation,
) -> AutoFishBarDecisionReport {
    let should_press = matches!(relation, AutoFishBarTargetRelation::CursorBeforeTarget);
    let decision = if should_press {
        if left_button_down {
            AutoFishBarDecisionKind::KeepLeftButtonDown
        } else {
            AutoFishBarDecisionKind::PressLeftButton
        }
    } else if left_button_down {
        AutoFishBarDecisionKind::ReleaseLeftButton
    } else {
        AutoFishBarDecisionKind::KeepLeftButtonUp
    };

    AutoFishBarDecisionReport {
        decision,
        relation: Some(relation),
        considered_rects,
        no_bar_since_ms: None,
        no_bar_grace_elapsed: false,
    }
}

fn auto_fish_bar_actions(
    plan: &AutoFishExecutionPlan,
    bar: &AutoFishBarDecisionReport,
) -> Vec<AutoFishRuntimeAction> {
    let mut actions = Vec::new();
    if !bar.considered_rects.is_empty() {
        actions.push(AutoFishRuntimeAction::Overlay {
            action: AutoFishOverlayAction::DrawFishingBar {
                key: AUTO_FISH_BAR_OVERLAY.to_string(),
                rects: bar.considered_rects.clone(),
            },
            reason: AutoFishRuntimeActionReason::FishingBarDetected,
        });
    }

    match bar.decision {
        AutoFishBarDecisionKind::PressLeftButton => actions.push(AutoFishRuntimeAction::Input {
            action: plan.fishing_input_rule.fishing_left_of_target_action,
            reason: AutoFishRuntimeActionReason::CursorBeforeTarget,
        }),
        AutoFishBarDecisionKind::ReleaseLeftButton => {
            actions.push(AutoFishRuntimeAction::Input {
                action: plan.fishing_input_rule.fishing_past_target_action,
                reason: AutoFishRuntimeActionReason::CursorPastTarget,
            });
        }
        AutoFishBarDecisionKind::CompleteNoBarGrace => {
            actions.push(AutoFishRuntimeAction::Input {
                action: plan.fishing_input_rule.completion_action,
                reason: AutoFishRuntimeActionReason::NoBarGrace,
            });
        }
        AutoFishBarDecisionKind::ArmNoBarGrace | AutoFishBarDecisionKind::WaitNoBarGrace => actions
            .push(AutoFishRuntimeAction::Overlay {
                action: AutoFishOverlayAction::Clear {
                    key: AUTO_FISH_BAR_OVERLAY.to_string(),
                },
                reason: AutoFishRuntimeActionReason::NoBarGrace,
            }),
        AutoFishBarDecisionKind::KeepLeftButtonDown
        | AutoFishBarDecisionKind::KeepLeftButtonUp
        | AutoFishBarDecisionKind::InvalidTwoRectTarget
        | AutoFishBarDecisionKind::IgnoreUnexpectedRectCount => {}
    }

    actions
}

fn auto_fish_completion_overlay_actions(
    overlay_keys: &[String],
    reason: AutoFishRuntimeActionReason,
) -> Vec<AutoFishRuntimeAction> {
    overlay_keys
        .iter()
        .map(|key| AutoFishRuntimeAction::Overlay {
            action: AutoFishOverlayAction::Clear { key: key.clone() },
            reason,
        })
        .collect()
}

fn dispatch_auto_fish_runtime_action<R>(
    runtime: &mut R,
    action: &AutoFishRuntimeAction,
) -> Result<()>
where
    R: AutoFishRuntime,
{
    match action {
        AutoFishRuntimeAction::Input { action, .. } => runtime.dispatch_auto_fish_input(*action),
        AutoFishRuntimeAction::Overlay { action, .. } => runtime.update_auto_fish_overlay(action),
    }
}

fn tick_elapsed_ms(previous_ms: Option<u64>, now_ms: u64) -> u64 {
    previous_ms
        .map(|previous| now_ms.saturating_sub(previous))
        .unwrap_or(u64::MAX)
}

fn auto_fish_steps() -> Vec<AutoFishTickStep> {
    vec![
        AutoFishTickStep::new(
            AutoFishTickPhase::Throttle,
            AutoFishTickCondition::WhenTickElapsedLessOrEqual67Ms,
            AutoFishTickAction::SkipTick,
        ),
        AutoFishTickStep::new(
            AutoFishTickPhase::ExclusiveGate,
            AutoFishTickCondition::WhenNotExclusive,
            AutoFishTickAction::DetectFishingUiAndToggleExclusive,
        ),
        AutoFishTickStep::new(
            AutoFishTickPhase::ExclusiveGate,
            AutoFishTickCondition::WhenExitFishingButtonDetected,
            AutoFishTickAction::TickSemiAutoFishingTree,
        ),
        AutoFishTickStep::new(
            AutoFishTickPhase::BiteDetection,
            AutoFishTickCondition::WhenBiteTextBlockOrLiftButtonOrOcrDetected,
            AutoFishTickAction::RaiseRod,
        ),
        AutoFishTickStep::new(
            AutoFishTickPhase::FishBoxDetection,
            AutoFishTickCondition::WhenFishBoxRectsFound,
            AutoFishTickAction::CaptureFishBoxArea,
        ),
        AutoFishTickStep::new(
            AutoFishTickPhase::FishingBar,
            AutoFishTickCondition::WhenCursorBeforeTarget,
            AutoFishTickAction::HoldLeftButton,
        ),
        AutoFishTickStep::new(
            AutoFishTickPhase::FishingBar,
            AutoFishTickCondition::WhenCursorPastTarget,
            AutoFishTickAction::ReleaseLeftButton,
        ),
        AutoFishTickStep::new(
            AutoFishTickPhase::Completion,
            AutoFishTickCondition::WhenNoBarDetectedForGraceWindow,
            AutoFishTickAction::ClearFishingOverlays,
        ),
    ]
}

fn auto_fish_locators(size: Size) -> AutoFishLocators {
    let right_half_lower_quarter = Rect {
        x: size.width as i32 - size.width as i32 / 2,
        y: size.height as i32 - size.height as i32 / 4,
        width: size.width as i32 / 2,
        height: size.height as i32 / 4,
    };
    AutoFishLocators {
        space_button: template(
            "SpaceButton",
            AUTO_FISH_SPACE_BUTTON,
            Rect {
                x: size.width as i32 - size.width as i32 / 3,
                y: size.height as i32 - size.height as i32 / 5,
                width: size.width as i32 / 3,
                height: size.height as i32 / 5,
            },
            0.8,
        ),
        bait_button: template(
            "BaitButton",
            AUTO_FISH_BAIT_BUTTON,
            right_half_lower_quarter,
            0.7,
        ),
        wait_bite_button: template(
            "WaitBiteButton",
            AUTO_FISH_WAIT_BITE_BUTTON,
            right_half_lower_quarter,
            0.7,
        ),
        lift_rod_button: template(
            "LiftRodButton",
            AUTO_FISH_LIFT_ROD_BUTTON,
            right_half_lower_quarter,
            0.7,
        ),
        exit_fishing_button: template(
            "ExitFishingButton",
            AUTO_FISH_EXIT_FISHING_BUTTON,
            Rect {
                x: size.width as i32 - scaled(140, size),
                y: size.height as i32 - scaled(150, size),
                width: scaled(140, size),
                height: scaled(150, size),
            },
            0.8,
        ),
    }
}

fn template(name: &str, asset: &str, roi: Rect, threshold: f64) -> AutoFishTemplateLocator {
    AutoFishTemplateLocator {
        name: name.to_string(),
        asset: asset.to_string(),
        roi,
        threshold,
        match_mode: TemplateMatchMode::CCoeffNormed,
        use_3_channels: false,
        draw_on_window: false,
    }
}

fn fishing_time_policy(value: &Value) -> AutoFishTimePolicy {
    match value {
        Value::String(value) => match value.as_str() {
            "All" => AutoFishTimePolicy::All,
            "Daytime" => AutoFishTimePolicy::Daytime,
            "Nighttime" => AutoFishTimePolicy::Nighttime,
            "DontChange" => AutoFishTimePolicy::DontChange,
            other => AutoFishTimePolicy::Unknown(other.to_string()),
        },
        Value::Number(value) => match value.as_u64() {
            Some(0) => AutoFishTimePolicy::All,
            Some(1) => AutoFishTimePolicy::Daytime,
            Some(2) => AutoFishTimePolicy::Nighttime,
            Some(3) => AutoFishTimePolicy::DontChange,
            _ => AutoFishTimePolicy::Unknown(value.to_string()),
        },
        _ => AutoFishTimePolicy::Unknown(value.to_string()),
    }
}

fn scaled(value_1080p: i32, size: Size) -> i32 {
    ((value_1080p as i64 * size.width as i64) / AUTO_FISH_DEFAULT_CAPTURE_WIDTH as i64) as i32
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

fn u32_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<u32> {
    keys.into_iter()
        .filter_map(|key| value.get(key))
        .find_map(|value| value.as_u64().and_then(|value| u32::try_from(value).ok()))
}

#[cfg(test)]
mod auto_fish_tests {
    use super::*;
    use crate::TaskError;
    use std::collections::VecDeque;

    #[derive(Debug, Default)]
    struct FakeAutoFishRuntime {
        now_ms: u64,
        exit_fishing_button_detected: bool,
        lift_rod_button_detected: bool,
        bite_text_block_detected: bool,
        bite_ocr_text: Option<String>,
        fish_box_rects: VecDeque<Vec<Rect>>,
        fishing_bar_rects: VecDeque<Vec<Rect>>,
        inputs: Vec<AutoFishInputAction>,
        overlays: Vec<AutoFishOverlayAction>,
    }

    impl AutoFishRuntime for FakeAutoFishRuntime {
        fn auto_fish_now_ms(&mut self, _plan: &AutoFishExecutionPlan) -> Result<u64> {
            Ok(self.now_ms)
        }

        fn detect_auto_fish_template(&mut self, locator: &AutoFishTemplateLocator) -> Result<bool> {
            match locator.asset.as_str() {
                AUTO_FISH_EXIT_FISHING_BUTTON => Ok(self.exit_fishing_button_detected),
                AUTO_FISH_LIFT_ROD_BUTTON => Ok(self.lift_rod_button_detected),
                asset => Err(TaskError::VisionPlan(format!(
                    "unexpected template {asset}"
                ))),
            }
        }

        fn detect_auto_fish_bite_text_block(&mut self, _rule: &AutoFishBiteRule) -> Result<bool> {
            Ok(self.bite_text_block_detected)
        }

        fn ocr_auto_fish_bite_text(&mut self, _rule: &AutoFishBiteRule) -> Result<Option<String>> {
            Ok(self.bite_ocr_text.clone())
        }

        fn detect_auto_fish_fish_box_rects(
            &mut self,
            _plan: &AutoFishExecutionPlan,
        ) -> Result<Vec<Rect>> {
            Ok(self.fish_box_rects.pop_front().unwrap_or_default())
        }

        fn detect_auto_fish_fishing_bar_rects(
            &mut self,
            _plan: &AutoFishExecutionPlan,
            _fish_box_rect: Rect,
        ) -> Result<Vec<Rect>> {
            Ok(self.fishing_bar_rects.pop_front().unwrap_or_default())
        }

        fn dispatch_auto_fish_input(&mut self, action: AutoFishInputAction) -> Result<()> {
            self.inputs.push(action);
            Ok(())
        }

        fn update_auto_fish_overlay(&mut self, action: &AutoFishOverlayAction) -> Result<()> {
            self.overlays.push(action.clone());
            Ok(())
        }
    }

    fn enabled_plan() -> AutoFishExecutionPlan {
        let mut config = AutoFishExecutionConfig::default();
        config.auto_fishing_config.enabled = true;
        plan_auto_fish(config)
    }

    fn rect(x: i32, y: i32, width: i32, height: i32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    fn detected_fish_box_rects() -> Vec<Rect> {
        vec![rect(700, 100, 20, 40), rect(850, 100, 200, 38)]
    }

    #[test]
    fn auto_fish_tick_disabled_skips_without_mutating_state() {
        let plan = plan_auto_fish(AutoFishExecutionConfig::default());
        assert!(plan.executor_ready);
        assert!(plan
            .pending_native
            .iter()
            .any(|item| item.contains("desktop live adapter now covers BitBlt capture")));
        assert!(plan
            .pending_native
            .iter()
            .any(|item| item.contains("RodNet")));

        let mut state = AutoFishTriggerState {
            prev_execute_ms: Some(10),
            exclusive: true,
            stage: AutoFishTickStage::Fishing,
            fish_box_rect: Some(rect(1, 2, 3, 4)),
            left_button_down: true,
            no_bar_since_ms: Some(20),
        };
        let original = state.clone();
        let mut runtime = FakeAutoFishRuntime {
            now_ms: 100,
            exit_fishing_button_detected: true,
            ..FakeAutoFishRuntime::default()
        };

        let report = execute_auto_fish_tick_plan(&plan, &mut state, &mut runtime).unwrap();

        assert_eq!(
            report.decision.skip_reason,
            Some(AutoFishTickSkipReason::Disabled)
        );
        assert!(!report.decision.processed);
        assert_eq!(state, original);
        assert!(runtime.inputs.is_empty());
        assert!(runtime.overlays.is_empty());
    }

    #[test]
    fn auto_fish_tick_throttle_skips_when_elapsed_is_less_or_equal_67ms() {
        let plan = enabled_plan();
        let mut state = AutoFishTriggerState {
            prev_execute_ms: Some(1_000),
            exclusive: true,
            ..AutoFishTriggerState::default()
        };
        let mut runtime = FakeAutoFishRuntime {
            now_ms: 1_067,
            exit_fishing_button_detected: true,
            bite_text_block_detected: true,
            ..FakeAutoFishRuntime::default()
        };

        let report = execute_auto_fish_tick_plan(&plan, &mut state, &mut runtime).unwrap();

        assert_eq!(
            report.decision.skip_reason,
            Some(AutoFishTickSkipReason::TickThrottle)
        );
        assert_eq!(state.prev_execute_ms, Some(1_000));
        assert_eq!(state.stage, AutoFishTickStage::WaitingForBite);
        assert!(runtime.inputs.is_empty());
    }

    #[test]
    fn auto_fish_tick_bite_detected_clicks_raise_rod_and_waits_for_box() {
        let plan = enabled_plan();
        let mut state = AutoFishTriggerState::default();
        let mut runtime = FakeAutoFishRuntime {
            now_ms: 1_000,
            exit_fishing_button_detected: true,
            bite_text_block_detected: true,
            ..FakeAutoFishRuntime::default()
        };

        let report = execute_auto_fish_tick_plan(&plan, &mut state, &mut runtime).unwrap();

        assert!(report.decision.processed);
        assert_eq!(
            report.decision.bite_method,
            Some(AutoFishBiteDetectionMethod::WordBlock)
        );
        assert!(state.exclusive);
        assert_eq!(state.stage, AutoFishTickStage::WaitingForFishBox);
        assert_eq!(runtime.inputs, vec![AutoFishInputAction::LeftButtonClick]);
        assert_eq!(
            runtime.overlays,
            vec![AutoFishOverlayAction::Clear {
                key: AUTO_FISH_BITE_TIPS_OVERLAY.to_string()
            }]
        );
    }

    #[test]
    fn auto_fish_tick_fish_box_then_bar_holds_and_releases_left_button() {
        let plan = enabled_plan();
        let mut state = AutoFishTriggerState {
            prev_execute_ms: Some(900),
            exclusive: true,
            stage: AutoFishTickStage::WaitingForFishBox,
            ..AutoFishTriggerState::default()
        };
        let mut runtime = FakeAutoFishRuntime {
            now_ms: 1_000,
            exit_fishing_button_detected: true,
            fish_box_rects: VecDeque::from([detected_fish_box_rects()]),
            ..FakeAutoFishRuntime::default()
        };

        let fish_box = execute_auto_fish_tick_plan(&plan, &mut state, &mut runtime).unwrap();

        assert_eq!(state.stage, AutoFishTickStage::Fishing);
        assert!(state.fish_box_rect.is_some());
        assert_eq!(
            fish_box
                .decision
                .fish_box
                .as_ref()
                .map(|report| report.decision),
            Some(AutoFishFishBoxDecisionKind::Detected)
        );
        assert!(runtime
            .overlays
            .iter()
            .any(|action| matches!(action, AutoFishOverlayAction::DrawFishBox { .. })));

        runtime.now_ms = 1_100;
        runtime.fishing_bar_rects =
            VecDeque::from([vec![rect(610, 300, 20, 20), rect(700, 300, 260, 20)]]);
        let hold = execute_auto_fish_tick_plan(&plan, &mut state, &mut runtime).unwrap();

        assert_eq!(
            hold.decision.bar.as_ref().map(|report| report.decision),
            Some(AutoFishBarDecisionKind::PressLeftButton)
        );
        assert!(state.left_button_down);
        assert_eq!(
            runtime.inputs.last(),
            Some(&AutoFishInputAction::LeftButtonDown)
        );

        runtime.now_ms = 1_200;
        runtime.fishing_bar_rects =
            VecDeque::from([vec![rect(980, 300, 20, 20), rect(700, 300, 260, 20)]]);
        let release = execute_auto_fish_tick_plan(&plan, &mut state, &mut runtime).unwrap();

        assert_eq!(
            release.decision.bar.as_ref().map(|report| report.decision),
            Some(AutoFishBarDecisionKind::ReleaseLeftButton)
        );
        assert!(!state.left_button_down);
        assert_eq!(
            runtime.inputs,
            vec![
                AutoFishInputAction::LeftButtonDown,
                AutoFishInputAction::LeftButtonUp
            ]
        );
    }

    #[test]
    fn auto_fish_tick_completion_cleanup_after_no_bar_grace() {
        let plan = enabled_plan();
        let mut state = AutoFishTriggerState {
            prev_execute_ms: Some(900),
            exclusive: true,
            stage: AutoFishTickStage::Fishing,
            fish_box_rect: Some(rect(660, 90, 620, 60)),
            left_button_down: true,
            no_bar_since_ms: Some(1_000),
        };
        let mut runtime = FakeAutoFishRuntime {
            now_ms: 2_000,
            exit_fishing_button_detected: true,
            fishing_bar_rects: VecDeque::from([Vec::new()]),
            ..FakeAutoFishRuntime::default()
        };

        let report = execute_auto_fish_tick_plan(&plan, &mut state, &mut runtime).unwrap();

        assert_eq!(
            report.decision.bar.as_ref().map(|bar| bar.decision),
            Some(AutoFishBarDecisionKind::CompleteNoBarGrace)
        );
        assert_eq!(state.stage, AutoFishTickStage::WaitingForBite);
        assert_eq!(state.fish_box_rect, None);
        assert!(!state.left_button_down);
        assert_eq!(state.no_bar_since_ms, None);
        assert_eq!(runtime.inputs, vec![AutoFishInputAction::LeftButtonUp]);
        for key in [
            AUTO_FISH_BITE_TIPS_OVERLAY,
            AUTO_FISH_BOX_OVERLAY,
            AUTO_FISH_BAR_OVERLAY,
        ] {
            assert!(runtime.overlays.contains(&AutoFishOverlayAction::Clear {
                key: key.to_string()
            }));
        }
    }
}
