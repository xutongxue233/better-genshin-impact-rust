use super::{image_locator, task_vision_result};
use crate::{Result, TaskError, TaskPortState};
use bgi_core::{GenshinAction, KeyBindingsConfig};
use bgi_input::{input_events_for_action, InputEvent, KeyActionType};
use bgi_vision::{BvLocatorOperation, BvLocatorPlan, BvPage, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const LOWER_HEAD_THEN_WALK_TO_TASK_KEY: &str = "LowerHeadThenWalkTo";
pub const LOWER_HEAD_THEN_WALK_TO_DEFAULT_TARGET: &str = "chest_tip.png";
pub const LOWER_HEAD_THEN_WALK_TO_PICK_KEY: &str = "AutoPick:F.png";
pub const LOWER_HEAD_THEN_WALK_TO_DEFAULT_TIMEOUT_MS: u32 = 30_000;
pub const LOWER_HEAD_THEN_WALK_TO_LOOP_DELAY_MS: u32 = 100;
pub const LOWER_HEAD_THEN_WALK_TO_ACTIVATION_TEXT: &str = "激活";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LowerHeadThenWalkToExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub target_mat_name: String,
    pub timeout_ms: u32,
    pub locators: LowerHeadThenWalkToLocators,
    pub movement_rule: LowerHeadThenWalkToMovementRule,
    pub f_key_rule: LowerHeadThenWalkToFKeyRule,
    pub steps: Vec<LowerHeadThenWalkToStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LowerHeadThenWalkToLocators {
    pub track_point: BvLocatorPlan,
    pub pick_key: BvLocatorPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LowerHeadThenWalkToMovementRule {
    pub center_y_threshold_ratio: f64,
    pub target_below_center_mouse_dx: i32,
    pub target_below_center_release_forward: bool,
    pub direction_divisor: f64,
    pub small_turn_threshold: i32,
    pub medium_turn_min_abs: i32,
    pub medium_turn_max_abs: i32,
    pub small_turn_boost: i32,
    pub medium_turn_boost: i32,
    pub press_forward_when_move_zero_or_direction_reversed: bool,
    pub look_down_mouse_dx: i32,
    pub look_down_mouse_dy: i32,
    pub loop_delay_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LowerHeadThenWalkToFKeyRule {
    pub pick_key_locator: BvLocatorPlan,
    pub text_x_offset_1080p: i32,
    pub text_width_1080p: i32,
    pub min_white_bounding_width: i32,
    pub min_white_bounding_height: i32,
    pub activation_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LowerHeadThenWalkToStep {
    pub phase: LowerHeadThenWalkToStepPhase,
    pub condition: LowerHeadThenWalkToStepCondition,
    pub label: String,
    pub action: LowerHeadThenWalkToStepAction,
}

impl LowerHeadThenWalkToStep {
    fn new(
        phase: LowerHeadThenWalkToStepPhase,
        condition: LowerHeadThenWalkToStepCondition,
        label: impl Into<String>,
        action: LowerHeadThenWalkToStepAction,
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
pub enum LowerHeadThenWalkToStepPhase {
    Setup,
    TrackingLoop,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LowerHeadThenWalkToStepCondition {
    Always,
    WhenInitialTargetFound,
    WhenInitialTargetMissing,
    WhenActivationTextDetected,
    WhenTimeout,
    Finally,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LowerHeadThenWalkToActionPress {
    KeyDown,
    KeyUp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum LowerHeadThenWalkToStepAction {
    Locator {
        locator: BvLocatorPlan,
    },
    TrackingLoop {
        target_locator: BvLocatorPlan,
        movement_rule: LowerHeadThenWalkToMovementRule,
        f_key_rule: LowerHeadThenWalkToFKeyRule,
    },
    GenshinAction {
        action: GenshinAction,
        press: LowerHeadThenWalkToActionPress,
    },
    ClearVisionDrawings,
    ReturnResult {
        result: LowerHeadThenWalkToStepResult,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LowerHeadThenWalkToStepResult {
    Activated,
    InitialTargetMissing,
    Timeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LowerHeadThenWalkToTrackingObservation {
    pub capture_size: Size,
    pub target_rect: Option<bgi_vision::Rect>,
    pub activation_text_detected: bool,
    pub elapsed_ms: u32,
    pub previous_move_x: i32,
    pub dpi_scale: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LowerHeadThenWalkToTrackingDecisionKind {
    TargetMissing,
    TargetBelowCenter,
    TurnAndWalk,
    Activated,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LowerHeadThenWalkToTrackingDecision {
    pub kind: LowerHeadThenWalkToTrackingDecisionKind,
    pub result: Option<LowerHeadThenWalkToStepResult>,
    pub next_previous_move_x: i32,
    pub input_events: Vec<InputEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LowerHeadThenWalkToExecutionConfig {
    pub capture_size: Size,
    pub target_mat_name: String,
    pub timeout_ms: u32,
    pub activation_text: String,
}

impl Default for LowerHeadThenWalkToExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
            target_mat_name: LOWER_HEAD_THEN_WALK_TO_DEFAULT_TARGET.to_string(),
            timeout_ms: LOWER_HEAD_THEN_WALK_TO_DEFAULT_TIMEOUT_MS,
            activation_text: LOWER_HEAD_THEN_WALK_TO_ACTIVATION_TEXT.to_string(),
        }
    }
}

impl LowerHeadThenWalkToExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        if config.target_mat_name.trim().is_empty() {
            config.target_mat_name = LOWER_HEAD_THEN_WALK_TO_DEFAULT_TARGET.to_string();
        }
        if config.timeout_ms == 0 {
            config.timeout_ms = LOWER_HEAD_THEN_WALK_TO_DEFAULT_TIMEOUT_MS;
        }
        if config.activation_text.trim().is_empty() {
            config.activation_text = LOWER_HEAD_THEN_WALK_TO_ACTIVATION_TEXT.to_string();
        }
        config
    }
}

pub fn plan_lower_head_then_walk_to(
    config: LowerHeadThenWalkToExecutionConfig,
) -> Result<LowerHeadThenWalkToExecutionPlan> {
    let page = BvPage {
        capture_size: config.capture_size,
        ..BvPage::default()
    };
    let target_asset = format!("Common/Element:{}", config.target_mat_name);
    let track_roi = task_vision_result(bgi_vision::Rect::new(
        scaled_width(config.capture_size, 300) as i32,
        0,
        (config
            .capture_size
            .width
            .saturating_sub(scaled_width(config.capture_size, 600))) as i32,
        config.capture_size.height as i32,
    ))?;
    let track_point = image_locator(
        &page,
        &target_asset,
        Some(track_roi),
        0.6,
        BvLocatorOperation::IsExist,
        Some(1_000),
    )?;
    let pick_key = image_locator(
        &page,
        LOWER_HEAD_THEN_WALK_TO_PICK_KEY,
        None,
        0.8,
        BvLocatorOperation::IsExist,
        Some(1_000),
    )?;
    let movement_rule = LowerHeadThenWalkToMovementRule {
        center_y_threshold_ratio: 0.5,
        target_below_center_mouse_dx: -50,
        target_below_center_release_forward: true,
        direction_divisor: 8.0,
        small_turn_threshold: 10,
        medium_turn_min_abs: 10,
        medium_turn_max_abs: 50,
        small_turn_boost: 10,
        medium_turn_boost: 80,
        press_forward_when_move_zero_or_direction_reversed: true,
        look_down_mouse_dx: 0,
        look_down_mouse_dy: 800,
        loop_delay_ms: LOWER_HEAD_THEN_WALK_TO_LOOP_DELAY_MS,
    };
    let f_key_rule = LowerHeadThenWalkToFKeyRule {
        pick_key_locator: pick_key.clone(),
        text_x_offset_1080p: 115,
        text_width_1080p: 285,
        min_white_bounding_width: 5,
        min_white_bounding_height: 5,
        activation_text: config.activation_text,
    };
    let locators = LowerHeadThenWalkToLocators {
        track_point,
        pick_key,
    };
    let steps = vec![
        LowerHeadThenWalkToStep::new(
            LowerHeadThenWalkToStepPhase::Setup,
            LowerHeadThenWalkToStepCondition::Always,
            "log lower-head tracking start",
            LowerHeadThenWalkToStepAction::Log {
                message: "start LowerHeadThenWalkTo common job plan".to_string(),
            },
        ),
        LowerHeadThenWalkToStep::new(
            LowerHeadThenWalkToStepPhase::Setup,
            LowerHeadThenWalkToStepCondition::Always,
            "detect initial tracking target",
            LowerHeadThenWalkToStepAction::Locator {
                locator: locators.track_point.clone(),
            },
        ),
        LowerHeadThenWalkToStep::new(
            LowerHeadThenWalkToStepPhase::TrackingLoop,
            LowerHeadThenWalkToStepCondition::WhenInitialTargetFound,
            "loop camera and movement until F activation text appears",
            LowerHeadThenWalkToStepAction::TrackingLoop {
                target_locator: locators.track_point.clone(),
                movement_rule,
                f_key_rule: f_key_rule.clone(),
            },
        ),
        LowerHeadThenWalkToStep::new(
            LowerHeadThenWalkToStepPhase::TrackingLoop,
            LowerHeadThenWalkToStepCondition::WhenActivationTextDetected,
            "return activated result",
            LowerHeadThenWalkToStepAction::ReturnResult {
                result: LowerHeadThenWalkToStepResult::Activated,
            },
        ),
        LowerHeadThenWalkToStep::new(
            LowerHeadThenWalkToStepPhase::TrackingLoop,
            LowerHeadThenWalkToStepCondition::WhenInitialTargetMissing,
            "return missing-target result",
            LowerHeadThenWalkToStepAction::ReturnResult {
                result: LowerHeadThenWalkToStepResult::InitialTargetMissing,
            },
        ),
        LowerHeadThenWalkToStep::new(
            LowerHeadThenWalkToStepPhase::TrackingLoop,
            LowerHeadThenWalkToStepCondition::WhenTimeout,
            "return timeout result",
            LowerHeadThenWalkToStepAction::ReturnResult {
                result: LowerHeadThenWalkToStepResult::Timeout,
            },
        ),
        LowerHeadThenWalkToStep::new(
            LowerHeadThenWalkToStepPhase::Cleanup,
            LowerHeadThenWalkToStepCondition::Finally,
            "release move forward",
            LowerHeadThenWalkToStepAction::GenshinAction {
                action: GenshinAction::MoveForward,
                press: LowerHeadThenWalkToActionPress::KeyUp,
            },
        ),
        LowerHeadThenWalkToStep::new(
            LowerHeadThenWalkToStepPhase::Cleanup,
            LowerHeadThenWalkToStepCondition::Finally,
            "clear vision overlays",
            LowerHeadThenWalkToStepAction::ClearVisionDrawings,
        ),
    ];

    Ok(LowerHeadThenWalkToExecutionPlan {
        task_key: LOWER_HEAD_THEN_WALK_TO_TASK_KEY.to_string(),
        display_name: "Lower Head Then Walk To".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size: config.capture_size,
        target_mat_name: config.target_mat_name,
        timeout_ms: config.timeout_ms,
        locators,
        movement_rule,
        f_key_rule,
        steps,
        notes: "Legacy lower-head tracking loop is represented and executable through an injectable target tracking/F-key state machine with cleanup; desktop live phase 1 wires BitBlt capture, PureRust target/F-key template matching, WinRT F-key text OCR, SendInput camera/W movement, and no-op overlay cleanup while Paddle OCR white-text cropping and real overlay drawing remain pending.".to_string(),
    })
}

pub fn reduce_lower_head_then_walk_to_tracking_frame(
    observation: LowerHeadThenWalkToTrackingObservation,
    movement_rule: &LowerHeadThenWalkToMovementRule,
    key_bindings: &KeyBindingsConfig,
    timeout_ms: u32,
) -> Result<LowerHeadThenWalkToTrackingDecision> {
    if observation.elapsed_ms > timeout_ms {
        return Ok(LowerHeadThenWalkToTrackingDecision {
            kind: LowerHeadThenWalkToTrackingDecisionKind::Timeout,
            result: Some(LowerHeadThenWalkToStepResult::Timeout),
            next_previous_move_x: observation.previous_move_x,
            input_events: Vec::new(),
        });
    }

    let Some(target_rect) = observation.target_rect else {
        return Ok(LowerHeadThenWalkToTrackingDecision {
            kind: LowerHeadThenWalkToTrackingDecisionKind::TargetMissing,
            result: None,
            next_previous_move_x: observation.previous_move_x,
            input_events: lower_head_look_down_events(movement_rule),
        });
    };

    let mut input_events = Vec::new();
    let target_center = target_rect.center();
    let center_y_threshold =
        observation.capture_size.height as f64 * movement_rule.center_y_threshold_ratio;
    if f64::from(target_center.y) > center_y_threshold {
        input_events.push(InputEvent::MouseMoveRelative {
            dx: movement_rule.target_below_center_mouse_dx,
            dy: 0,
        });
        if movement_rule.target_below_center_release_forward {
            input_events.extend(lower_head_move_forward_events(
                key_bindings,
                KeyActionType::KeyUp,
            )?);
        }
        return Ok(LowerHeadThenWalkToTrackingDecision {
            kind: LowerHeadThenWalkToTrackingDecisionKind::TargetBelowCenter,
            result: None,
            next_previous_move_x: observation.previous_move_x,
            input_events,
        });
    }

    let dpi_scale = if observation.dpi_scale.is_finite() && observation.dpi_scale > 0.0 {
        observation.dpi_scale
    } else {
        1.0
    };
    let raw_move_x = ((f64::from(target_center.x) - observation.capture_size.width as f64 / 2.0)
        / movement_rule.direction_divisor
        / dpi_scale) as i32;
    let move_x = adjusted_lower_head_move_x(raw_move_x, movement_rule);
    if move_x != 0 {
        input_events.push(InputEvent::MouseMoveRelative { dx: move_x, dy: 0 });
    }

    let direction_reversed = observation.previous_move_x != 0
        && move_x != 0
        && observation.previous_move_x.signum() != move_x.signum();
    let should_press_forward = movement_rule.press_forward_when_move_zero_or_direction_reversed
        && (move_x == 0 || direction_reversed);
    input_events.extend(lower_head_move_forward_events(
        key_bindings,
        if should_press_forward {
            KeyActionType::KeyDown
        } else {
            KeyActionType::KeyUp
        },
    )?);

    if observation.activation_text_detected {
        input_events.extend(lower_head_move_forward_events(
            key_bindings,
            KeyActionType::KeyUp,
        )?);
        return Ok(LowerHeadThenWalkToTrackingDecision {
            kind: LowerHeadThenWalkToTrackingDecisionKind::Activated,
            result: Some(LowerHeadThenWalkToStepResult::Activated),
            next_previous_move_x: move_x,
            input_events,
        });
    }

    input_events.extend(lower_head_look_down_events(movement_rule));
    Ok(LowerHeadThenWalkToTrackingDecision {
        kind: LowerHeadThenWalkToTrackingDecisionKind::TurnAndWalk,
        result: None,
        next_previous_move_x: move_x,
        input_events,
    })
}

fn adjusted_lower_head_move_x(move_x: i32, movement_rule: &LowerHeadThenWalkToMovementRule) -> i32 {
    match move_x {
        value
            if value >= movement_rule.medium_turn_min_abs
                && value < movement_rule.medium_turn_max_abs =>
        {
            movement_rule.medium_turn_boost + value
        }
        value
            if value > -movement_rule.medium_turn_max_abs
                && value <= -movement_rule.medium_turn_min_abs =>
        {
            -movement_rule.medium_turn_boost + value
        }
        value if value > 0 && value < movement_rule.small_turn_threshold => {
            movement_rule.small_turn_boost + value
        }
        value if value > -movement_rule.small_turn_threshold && value < 0 => {
            -movement_rule.small_turn_boost + value
        }
        value => value,
    }
}

fn lower_head_look_down_events(movement_rule: &LowerHeadThenWalkToMovementRule) -> Vec<InputEvent> {
    vec![
        InputEvent::MouseMoveRelative {
            dx: movement_rule.look_down_mouse_dx,
            dy: movement_rule.look_down_mouse_dy,
        },
        InputEvent::Delay {
            milliseconds: u64::from(movement_rule.loop_delay_ms),
        },
    ]
}

fn lower_head_move_forward_events(
    key_bindings: &KeyBindingsConfig,
    action_type: KeyActionType,
) -> Result<Vec<InputEvent>> {
    input_events_for_action(key_bindings, GenshinAction::MoveForward, action_type)
        .map_err(|error| TaskError::CommonJobExecution(error.to_string()))
}

fn scaled_width(size: Size, value_1080p: u32) -> u32 {
    ((value_1080p as u64 * size.width as u64) / 1920) as u32
}
