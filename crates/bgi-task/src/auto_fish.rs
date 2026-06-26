use bgi_core::AutoFishingConfig;
use bgi_vision::{Rect, RgbPixel, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const AUTO_FISH_TASK_KEY: &str = "AutoFish";
pub const AUTO_FISH_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_FISH_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_FISH_SPACE_BUTTON: &str = "AutoFishing:space.png";
pub const AUTO_FISH_BAIT_BUTTON: &str = "AutoFishing:switch_bait.png";
pub const AUTO_FISH_WAIT_BITE_BUTTON: &str = "AutoFishing:wait_bite.png";
pub const AUTO_FISH_LIFT_ROD_BUTTON: &str = "AutoFishing:lift_rod.png";
pub const AUTO_FISH_EXIT_FISHING_BUTTON: &str = "AutoFishing:exit_fishing.png";

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
    pub dynamic_add_trigger_supported_by_csharp: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
            dynamic_add_trigger_supported_by_csharp: false,
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
        executor_ready: false,
        pending_native: vec![
            "live capture template matching for AutoFishing UI buttons".to_string(),
            "BehaviourTree execution with trigger-local exclusive state".to_string(),
            "OpenCV fish-bite word block and fish-bar contour recognition".to_string(),
            "Paddle OCR for localized bite text".to_string(),
            "mouse left-button click/down/up dispatch during fishing".to_string(),
            "DrawContent overlays for FishBiteTips, FishBox, and FishingBarAll".to_string(),
            "BgiFish YOLO predictor construction, RodNet inference, bait selection, and full AutoFishingTask throw-rod branch".to_string(),
        ],
    }
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
