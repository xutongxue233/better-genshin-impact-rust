use super::task_vision_result;
use crate::{Result, TaskPortState};
use bgi_core::GenshinAction;
use bgi_input::{InputEvent, MouseButton};
use bgi_vision::{BvPage, BvPageCommand, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SCAN_PICK_DROPS_TASK_KEY: &str = "ScanPickDrops";
pub const SCAN_PICK_DROPS_DEFAULT_SCAN_SECONDS: u32 = 15;
pub const SCAN_PICK_DROPS_WORLD_MODEL_NAME: &str = "BgiWorld";
pub const SCAN_PICK_DROPS_WORLD_MODEL_PATH: &str = "Assets/Model/World/bgi_world.onnx";
pub const SCAN_PICK_DROPS_DROP_LABEL: &str = "drops";
pub const SCAN_PICK_DROPS_ORE_LABEL: &str = "ore";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScanPickDropsExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub scan_seconds: u32,
    pub dpi_scale: f64,
    pub yolo_rule: ScanPickYoloRule,
    pub target_ordering_rule: ScanPickTargetOrderingRule,
    pub movement_rule: ScanPickMovementRule,
    pub search_rule: ScanPickSearchRule,
    pub camera_reset_rule: ScanPickCameraResetRule,
    pub steps: Vec<ScanPickDropsStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScanPickYoloRule {
    pub model_name: String,
    pub model_relative_path: String,
    pub accepted_labels: Vec<String>,
    pub confidence_threshold: Option<f32>,
    pub source: ScanPickYoloSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanPickYoloSource {
    FullCapture,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScanPickTargetOrderingRule {
    pub center_x_1080p: f64,
    pub reference_bottom_y_1080p: f64,
    pub vertical_weight: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScanPickMovementRule {
    pub horizontal_bottom_min_1080p: f64,
    pub move_left_when_x_below_1080p: f64,
    pub move_right_when_x_above_1080p: f64,
    pub move_forward_when_bottom_below_1080p: f64,
    pub move_backward_when_bottom_above_1080p: f64,
    pub approach_delay_ms: u32,
    pub left_action: GenshinAction,
    pub right_action: GenshinAction,
    pub forward_action: GenshinAction,
    pub backward_action: GenshinAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanPickSearchRule {
    pub iterations: u8,
    pub mouse_move_dx: i32,
    pub mouse_move_dy: i32,
    pub walk_forward_after_index: u8,
    pub walk_forward_ms: u32,
    pub wait_after_drop_ms: u32,
    pub walk_action: GenshinAction,
    pub drop_action: GenshinAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScanPickCameraResetRule {
    pub dpi_scale: f64,
    pub middle_click_events: Vec<InputEvent>,
    pub wait_after_middle_click_ms: u32,
    pub look_down_mouse_dx: i32,
    pub look_down_mouse_dy: i32,
    pub wait_after_look_down_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScanPickDropsStep {
    pub phase: ScanPickDropsStepPhase,
    pub condition: ScanPickDropsStepCondition,
    pub label: String,
    pub action: ScanPickDropsStepAction,
}

impl ScanPickDropsStep {
    fn new(
        phase: ScanPickDropsStepPhase,
        condition: ScanPickDropsStepCondition,
        label: impl Into<String>,
        action: ScanPickDropsStepAction,
    ) -> Self {
        Self {
            phase,
            condition,
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanPickDropsStepPhase {
    Setup,
    DetectionLoop,
    SearchSweep,
    Approach,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanPickDropsStepCondition {
    Always,
    WhileBeforeTimeout,
    WhenItemsDetected,
    WhenNoItemsDetected,
    Finally,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanPickDropsActionPress {
    KeyDown,
    KeyUp,
    KeyPress,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ScanPickDropsStepAction {
    Log {
        message: String,
    },
    CameraReset {
        rule: ScanPickCameraResetRule,
    },
    YoloDetect {
        rule: ScanPickYoloRule,
    },
    SearchSweep {
        rule: ScanPickSearchRule,
        yolo_rule: ScanPickYoloRule,
    },
    SelectTarget {
        rule: ScanPickTargetOrderingRule,
    },
    ApproachTarget {
        rule: ScanPickMovementRule,
    },
    GenshinAction {
        action: GenshinAction,
        press: ScanPickDropsActionPress,
    },
    Page {
        command: BvPageCommand,
    },
    ReleaseAllKeys,
    ClearVisionDrawings,
    ReturnResult {
        result: ScanPickDropsStepResult,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanPickDropsStepResult {
    ScanComplete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScanPickDropsExecutionConfig {
    pub capture_size: Size,
    pub scan_seconds: u32,
    pub dpi_scale: f64,
}

impl Default for ScanPickDropsExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
            scan_seconds: SCAN_PICK_DROPS_DEFAULT_SCAN_SECONDS,
            dpi_scale: 1.0,
        }
    }
}

impl ScanPickDropsExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        if config.dpi_scale <= 0.0 {
            config.dpi_scale = 1.0;
        }
        config
    }
}

pub fn plan_scan_pick_drops(
    config: ScanPickDropsExecutionConfig,
) -> Result<ScanPickDropsExecutionPlan> {
    let page = BvPage {
        capture_size: config.capture_size,
        ..BvPage::default()
    };
    let yolo_rule = ScanPickYoloRule {
        model_name: SCAN_PICK_DROPS_WORLD_MODEL_NAME.to_string(),
        model_relative_path: SCAN_PICK_DROPS_WORLD_MODEL_PATH.to_string(),
        accepted_labels: vec![
            SCAN_PICK_DROPS_DROP_LABEL.to_string(),
            SCAN_PICK_DROPS_ORE_LABEL.to_string(),
        ],
        confidence_threshold: None,
        source: ScanPickYoloSource::FullCapture,
    };
    let target_ordering_rule = ScanPickTargetOrderingRule {
        center_x_1080p: 960.0,
        reference_bottom_y_1080p: 888.88,
        vertical_weight: 14.0,
    };
    let movement_rule = ScanPickMovementRule {
        horizontal_bottom_min_1080p: 560.0,
        move_left_when_x_below_1080p: 760.0,
        move_right_when_x_above_1080p: 1040.0,
        move_forward_when_bottom_below_1080p: 770.0,
        move_backward_when_bottom_above_1080p: 900.0,
        approach_delay_ms: 200,
        left_action: GenshinAction::MoveLeft,
        right_action: GenshinAction::MoveRight,
        forward_action: GenshinAction::MoveForward,
        backward_action: GenshinAction::MoveBackward,
    };
    let search_rule = ScanPickSearchRule {
        iterations: 10,
        mouse_move_dx: 400,
        mouse_move_dy: 0,
        walk_forward_after_index: 5,
        walk_forward_ms: 100,
        wait_after_drop_ms: 300,
        walk_action: GenshinAction::MoveForward,
        drop_action: GenshinAction::Drop,
    };
    let camera_reset_rule = ScanPickCameraResetRule {
        dpi_scale: config.dpi_scale,
        middle_click_events: vec![
            InputEvent::MouseButtonDown {
                button: MouseButton::Middle,
            },
            InputEvent::MouseButtonUp {
                button: MouseButton::Middle,
            },
        ],
        wait_after_middle_click_ms: 500,
        look_down_mouse_dx: 0,
        look_down_mouse_dy: (500.0 * config.dpi_scale).round() as i32,
        wait_after_look_down_ms: 100,
    };
    let steps = vec![
        ScanPickDropsStep::new(
            ScanPickDropsStepPhase::Setup,
            ScanPickDropsStepCondition::Always,
            "log scan-pick start",
            ScanPickDropsStepAction::Log {
                message: "start ScanPickDrops common job plan".to_string(),
            },
        ),
        ScanPickDropsStep::new(
            ScanPickDropsStepPhase::Setup,
            ScanPickDropsStepCondition::Always,
            "reset camera before scanning loot",
            ScanPickDropsStepAction::CameraReset {
                rule: camera_reset_rule.clone(),
            },
        ),
        ScanPickDropsStep::new(
            ScanPickDropsStepPhase::Setup,
            ScanPickDropsStepCondition::Always,
            "drop to normalize airborne state",
            ScanPickDropsStepAction::GenshinAction {
                action: GenshinAction::Drop,
                press: ScanPickDropsActionPress::KeyPress,
            },
        ),
        ScanPickDropsStep::new(
            ScanPickDropsStepPhase::DetectionLoop,
            ScanPickDropsStepCondition::WhileBeforeTimeout,
            "detect pickable drops or ore with the world YOLO model",
            ScanPickDropsStepAction::YoloDetect {
                rule: yolo_rule.clone(),
            },
        ),
        ScanPickDropsStep::new(
            ScanPickDropsStepPhase::SearchSweep,
            ScanPickDropsStepCondition::WhenNoItemsDetected,
            "release keys, sweep camera, walk forward after the sixth sweep, and detect again",
            ScanPickDropsStepAction::SearchSweep {
                rule: search_rule,
                yolo_rule: yolo_rule.clone(),
            },
        ),
        ScanPickDropsStep::new(
            ScanPickDropsStepPhase::Approach,
            ScanPickDropsStepCondition::WhenItemsDetected,
            "select closest pickable target by legacy weighted bottom-distance score",
            ScanPickDropsStepAction::SelectTarget {
                rule: target_ordering_rule,
            },
        ),
        ScanPickDropsStep::new(
            ScanPickDropsStepPhase::Approach,
            ScanPickDropsStepCondition::WhenItemsDetected,
            "hold movement keys toward the selected target",
            ScanPickDropsStepAction::ApproachTarget {
                rule: movement_rule,
            },
        ),
        ScanPickDropsStep::new(
            ScanPickDropsStepPhase::Approach,
            ScanPickDropsStepCondition::WhenItemsDetected,
            "wait after movement adjustment",
            ScanPickDropsStepAction::Page {
                command: task_vision_result(page.wait(movement_rule.approach_delay_ms))?,
            },
        ),
        ScanPickDropsStep::new(
            ScanPickDropsStepPhase::Approach,
            ScanPickDropsStepCondition::WhenItemsDetected,
            "drop after approaching the selected target",
            ScanPickDropsStepAction::GenshinAction {
                action: GenshinAction::Drop,
                press: ScanPickDropsActionPress::KeyPress,
            },
        ),
        ScanPickDropsStep::new(
            ScanPickDropsStepPhase::Cleanup,
            ScanPickDropsStepCondition::Finally,
            "release all movement keys",
            ScanPickDropsStepAction::ReleaseAllKeys,
        ),
        ScanPickDropsStep::new(
            ScanPickDropsStepPhase::Cleanup,
            ScanPickDropsStepCondition::Finally,
            "drop once after cleanup",
            ScanPickDropsStepAction::GenshinAction {
                action: GenshinAction::Drop,
                press: ScanPickDropsActionPress::KeyPress,
            },
        ),
        ScanPickDropsStep::new(
            ScanPickDropsStepPhase::Cleanup,
            ScanPickDropsStepCondition::Finally,
            "clear vision overlay drawings",
            ScanPickDropsStepAction::ClearVisionDrawings,
        ),
        ScanPickDropsStep::new(
            ScanPickDropsStepPhase::Cleanup,
            ScanPickDropsStepCondition::Finally,
            "return scan completion",
            ScanPickDropsStepAction::ReturnResult {
                result: ScanPickDropsStepResult::ScanComplete,
            },
        ),
    ];

    Ok(ScanPickDropsExecutionPlan {
        task_key: SCAN_PICK_DROPS_TASK_KEY.to_string(),
        display_name: "Scan Pick Drops".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size: config.capture_size,
        scan_seconds: config.scan_seconds,
        dpi_scale: config.dpi_scale,
        yolo_rule,
        target_ordering_rule,
        movement_rule,
        search_rule,
        camera_reset_rule,
        steps,
        notes: "YOLO drop/ore detection, target ordering, search sweep, movement thresholds, camera reset, and cleanup are migrated and executable as a Rust state machine through injectable detection hooks; direct capture, ONNX inference, live input dispatch, and overlay cleanup remain pending."
            .to_string(),
    })
}
