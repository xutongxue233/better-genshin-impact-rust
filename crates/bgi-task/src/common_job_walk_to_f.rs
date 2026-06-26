use super::{image_locator, task_vision_result};
use crate::{Result, TaskPortState};
use bgi_core::GenshinAction;
use bgi_input::{InputEvent, InputSequence};
use bgi_vision::{BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const WALK_TO_F_TASK_KEY: &str = "WalkToF";
pub const WALK_TO_F_PICK_KEY: &str = "AutoPick:F.png";
pub const WALK_TO_F_DEFAULT_TIMEOUT_MS: u32 = 30_000;
pub const WALK_TO_F_RETRY_INTERVAL_MS: u32 = 100;
pub const WALK_TO_F_MOVE_START_DELAY_MS: u32 = 30;
pub const WALK_TO_F_RELEASE_GAP_MS: u32 = 50;
pub const WALK_TO_F_VK_F: u16 = 0x46;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalkToFExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub need_press: bool,
    pub run_to_f: bool,
    pub timeout_ms: u32,
    pub retry_rule: WalkToFRetryRule,
    pub pick_locator: BvLocatorPlan,
    pub steps: Vec<WalkToFStep>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkToFRetryRule {
    pub max_attempts: u32,
    pub interval_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalkToFStep {
    pub phase: WalkToFStepPhase,
    pub condition: WalkToFStepCondition,
    pub label: String,
    pub action: WalkToFStepAction,
}

impl WalkToFStep {
    fn new(
        phase: WalkToFStepPhase,
        condition: WalkToFStepCondition,
        label: impl Into<String>,
        action: WalkToFStepAction,
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
pub enum WalkToFStepPhase {
    Setup,
    Search,
    Press,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalkToFStepCondition {
    Always,
    WhenRunToF,
    WhenPickDetected,
    WhenNeedPressAndPickDetected,
    WhenPickMissing,
    Finally,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalkToFActionPress {
    KeyDown,
    KeyUp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum WalkToFStepAction {
    GenshinAction {
        action: GenshinAction,
        press: WalkToFActionPress,
    },
    Page {
        command: BvPageCommand,
    },
    Locator {
        locator: BvLocatorPlan,
    },
    Input {
        events: Vec<InputEvent>,
    },
    ReturnResult {
        result: WalkToFStepResult,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalkToFStepResult {
    PickDetectedAndPressed,
    PickDetectedWithoutPress,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WalkToFExecutionConfig {
    pub capture_size: Size,
    pub need_press: bool,
    pub run_to_f: bool,
    pub timeout_ms: u32,
    pub pick_vk: u16,
}

impl Default for WalkToFExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
            need_press: true,
            run_to_f: false,
            timeout_ms: WALK_TO_F_DEFAULT_TIMEOUT_MS,
            pick_vk: WALK_TO_F_VK_F,
        }
    }
}

impl WalkToFExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        if config.timeout_ms == 0 {
            config.timeout_ms = WALK_TO_F_DEFAULT_TIMEOUT_MS;
        }
        if config.pick_vk == 0 {
            config.pick_vk = WALK_TO_F_VK_F;
        }
        config
    }
}

pub fn plan_walk_to_f(config: WalkToFExecutionConfig) -> Result<WalkToFExecutionPlan> {
    let page = BvPage {
        capture_size: config.capture_size,
        ..BvPage::default()
    };
    let retry_rule = WalkToFRetryRule {
        max_attempts: config.timeout_ms / WALK_TO_F_RETRY_INTERVAL_MS + 1,
        interval_ms: WALK_TO_F_RETRY_INTERVAL_MS,
    };
    let pick_locator = image_locator(
        &page,
        WALK_TO_F_PICK_KEY,
        None,
        0.8,
        BvLocatorOperation::WaitFor,
        Some(config.timeout_ms),
    )?;
    let pick_events = InputSequence::new()
        .key_press(config.pick_vk)
        .events()
        .to_vec();
    let steps = vec![
        WalkToFStep::new(
            WalkToFStepPhase::Setup,
            WalkToFStepCondition::Always,
            "log walk to F start",
            WalkToFStepAction::Log {
                message: "start WalkToF common job plan".to_string(),
            },
        ),
        WalkToFStep::new(
            WalkToFStepPhase::Setup,
            WalkToFStepCondition::Always,
            "hold move forward",
            WalkToFStepAction::GenshinAction {
                action: GenshinAction::MoveForward,
                press: WalkToFActionPress::KeyDown,
            },
        ),
        WalkToFStep::new(
            WalkToFStepPhase::Setup,
            WalkToFStepCondition::Always,
            "wait after move-forward key down",
            WalkToFStepAction::Page {
                command: task_vision_result(page.wait(WALK_TO_F_MOVE_START_DELAY_MS))?,
            },
        ),
        WalkToFStep::new(
            WalkToFStepPhase::Setup,
            WalkToFStepCondition::WhenRunToF,
            "hold sprint while walking to F",
            WalkToFStepAction::GenshinAction {
                action: GenshinAction::SprintKeyboard,
                press: WalkToFActionPress::KeyDown,
            },
        ),
        WalkToFStep::new(
            WalkToFStepPhase::Search,
            WalkToFStepCondition::Always,
            "wait for AutoPick interaction key",
            WalkToFStepAction::Locator {
                locator: pick_locator.clone(),
            },
        ),
        WalkToFStep::new(
            WalkToFStepPhase::Press,
            WalkToFStepCondition::WhenNeedPressAndPickDetected,
            "press pick key",
            WalkToFStepAction::Input {
                events: pick_events,
            },
        ),
        WalkToFStep::new(
            WalkToFStepPhase::Press,
            WalkToFStepCondition::WhenNeedPressAndPickDetected,
            "return pressed result",
            WalkToFStepAction::ReturnResult {
                result: WalkToFStepResult::PickDetectedAndPressed,
            },
        ),
        WalkToFStep::new(
            WalkToFStepPhase::Press,
            WalkToFStepCondition::WhenPickDetected,
            "return detected-without-press result",
            WalkToFStepAction::ReturnResult {
                result: WalkToFStepResult::PickDetectedWithoutPress,
            },
        ),
        WalkToFStep::new(
            WalkToFStepPhase::Search,
            WalkToFStepCondition::WhenPickMissing,
            "return timeout result",
            WalkToFStepAction::ReturnResult {
                result: WalkToFStepResult::Timeout,
            },
        ),
        WalkToFStep::new(
            WalkToFStepPhase::Cleanup,
            WalkToFStepCondition::Finally,
            "release move forward",
            WalkToFStepAction::GenshinAction {
                action: GenshinAction::MoveForward,
                press: WalkToFActionPress::KeyUp,
            },
        ),
        WalkToFStep::new(
            WalkToFStepPhase::Cleanup,
            WalkToFStepCondition::Finally,
            "wait between key releases",
            WalkToFStepAction::Page {
                command: task_vision_result(page.wait(WALK_TO_F_RELEASE_GAP_MS))?,
            },
        ),
        WalkToFStep::new(
            WalkToFStepPhase::Cleanup,
            WalkToFStepCondition::WhenRunToF,
            "release sprint",
            WalkToFStepAction::GenshinAction {
                action: GenshinAction::SprintKeyboard,
                press: WalkToFActionPress::KeyUp,
            },
        ),
    ];

    Ok(WalkToFExecutionPlan {
        task_key: WALK_TO_F_TASK_KEY.to_string(),
        display_name: "Walk To F".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size: config.capture_size,
        need_press: config.need_press,
        run_to_f: config.run_to_f,
        timeout_ms: config.timeout_ms,
        retry_rule,
        pick_locator,
        steps,
        notes: "Legacy WalkToF hold-forward, optional sprint, F-key detection, optional press, and key-release flow is represented as a Rust plan with desktop live template/input execution.".to_string(),
    })
}
