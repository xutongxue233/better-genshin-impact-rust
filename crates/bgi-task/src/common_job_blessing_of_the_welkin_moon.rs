use super::{image_locator, task_vision_result};
use crate::{Result, TaskPortState};
use bgi_input::{InputEvent, InputSequence, MouseButton};
use bgi_vision::{BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const BLESSING_WELKIN_TASK_KEY: &str = "BlessingOfTheWelkinMoon";
pub const BLESSING_WELKIN_GIRL_MOON: &str = "GameLoading:girl_moon.png";
pub const BLESSING_WELKIN_WELKIN_MOON: &str = "GameLoading:welkin_moon_logo.png";
pub const BLESSING_WELKIN_PRIMOGEM: &str = "Common/Element:primogem.png";
pub const BLESSING_WELKIN_MAX_ITERATIONS: u8 = 20;
pub const BLESSING_WELKIN_STABLE_CLEAR_COUNT: u8 = 3;
pub const BLESSING_WELKIN_RETRY_DELAY_MS: u32 = 500;

const CLAIM_CLICK_X_1080P: f64 = 100.0;
const CLAIM_CLICK_Y_1080P: f64 = 100.0;
const CLAIM_CLICK_GAP_MS: u64 = 100;
const SERVER_TIME_OFFSET_MINUTES: i32 = 5;
const SERVER_RESET_HOUR: u8 = 4;
const SERVER_RESET_GRACE_MINUTES: u8 = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlessingOfTheWelkinMoonExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub server_time_gate: BlessingOfTheWelkinMoonServerTimeGate,
    pub detection_locators: BlessingOfTheWelkinMoonDetectionLocators,
    pub loop_rule: BlessingOfTheWelkinMoonLoopRule,
    pub claim_click_events: Vec<InputEvent>,
    pub steps: Vec<BlessingOfTheWelkinMoonStep>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlessingOfTheWelkinMoonServerTimeGate {
    pub offset_minutes: i32,
    pub reset_hour: u8,
    pub grace_minutes: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlessingOfTheWelkinMoonDetectionLocators {
    pub girl_moon: BvLocatorPlan,
    pub welkin_moon: BvLocatorPlan,
    pub primogem: BvLocatorPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlessingOfTheWelkinMoonLoopRule {
    pub max_iterations: u8,
    pub stable_clear_count: u8,
    pub retry_delay_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlessingOfTheWelkinMoonStep {
    pub condition: BlessingOfTheWelkinMoonStepCondition,
    pub label: String,
    pub action: BlessingOfTheWelkinMoonStepAction,
}

impl BlessingOfTheWelkinMoonStep {
    fn new(
        condition: BlessingOfTheWelkinMoonStepCondition,
        label: impl Into<String>,
        action: BlessingOfTheWelkinMoonStepAction,
    ) -> Self {
        Self {
            condition,
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlessingOfTheWelkinMoonStepCondition {
    Always,
    WhenServerTimeInsideClaimWindow,
    WhenBlessingOrPrimogemDetected,
    UntilStableClear,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum BlessingOfTheWelkinMoonStepAction {
    ServerTimeGate {
        gate: BlessingOfTheWelkinMoonServerTimeGate,
    },
    DetectClaimUi {
        locators: BlessingOfTheWelkinMoonDetectionLocators,
    },
    Input {
        events: Vec<InputEvent>,
    },
    Page {
        command: BvPageCommand,
    },
    LoopUntilClear {
        rule: BlessingOfTheWelkinMoonLoopRule,
        locators: BlessingOfTheWelkinMoonDetectionLocators,
        claim_click_events: Vec<InputEvent>,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BlessingOfTheWelkinMoonExecutionConfig {
    pub capture_size: Size,
    pub max_iterations: u8,
    pub stable_clear_count: u8,
}

impl Default for BlessingOfTheWelkinMoonExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
            max_iterations: BLESSING_WELKIN_MAX_ITERATIONS,
            stable_clear_count: BLESSING_WELKIN_STABLE_CLEAR_COUNT,
        }
    }
}

impl BlessingOfTheWelkinMoonExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        if config.max_iterations == 0 {
            config.max_iterations = BLESSING_WELKIN_MAX_ITERATIONS;
        }
        if config.stable_clear_count == 0 {
            config.stable_clear_count = BLESSING_WELKIN_STABLE_CLEAR_COUNT;
        }
        config
    }
}

pub fn plan_blessing_of_the_welkin_moon(
    capture_size: Size,
    max_iterations: u8,
    stable_clear_count: u8,
) -> Result<BlessingOfTheWelkinMoonExecutionPlan> {
    let max_iterations = if max_iterations == 0 {
        BLESSING_WELKIN_MAX_ITERATIONS
    } else {
        max_iterations
    };
    let stable_clear_count = if stable_clear_count == 0 {
        BLESSING_WELKIN_STABLE_CLEAR_COUNT
    } else {
        stable_clear_count
    };
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let server_time_gate = BlessingOfTheWelkinMoonServerTimeGate {
        offset_minutes: SERVER_TIME_OFFSET_MINUTES,
        reset_hour: SERVER_RESET_HOUR,
        grace_minutes: SERVER_RESET_GRACE_MINUTES,
    };
    let detection_locators = blessing_detection_locators(&page)?;
    let loop_rule = BlessingOfTheWelkinMoonLoopRule {
        max_iterations,
        stable_clear_count,
        retry_delay_ms: BLESSING_WELKIN_RETRY_DELAY_MS,
    };
    let claim_click_events = claim_click_events(capture_size);
    let steps = vec![
        BlessingOfTheWelkinMoonStep::new(
            BlessingOfTheWelkinMoonStepCondition::Always,
            "log blessing start",
            BlessingOfTheWelkinMoonStepAction::Log {
                message: "start BlessingOfTheWelkinMoon common job plan".to_string(),
            },
        ),
        BlessingOfTheWelkinMoonStep::new(
            BlessingOfTheWelkinMoonStepCondition::Always,
            "check server reset claim window",
            BlessingOfTheWelkinMoonStepAction::ServerTimeGate {
                gate: server_time_gate,
            },
        ),
        BlessingOfTheWelkinMoonStep::new(
            BlessingOfTheWelkinMoonStepCondition::WhenServerTimeInsideClaimWindow,
            "detect blessing or primogem screen",
            BlessingOfTheWelkinMoonStepAction::DetectClaimUi {
                locators: detection_locators.clone(),
            },
        ),
        BlessingOfTheWelkinMoonStep::new(
            BlessingOfTheWelkinMoonStepCondition::WhenBlessingOrPrimogemDetected,
            "double click safe point to claim blessing",
            BlessingOfTheWelkinMoonStepAction::Input {
                events: claim_click_events.clone(),
            },
        ),
        BlessingOfTheWelkinMoonStep::new(
            BlessingOfTheWelkinMoonStepCondition::WhenBlessingOrPrimogemDetected,
            "wait after claim click",
            BlessingOfTheWelkinMoonStepAction::Page {
                command: task_vision_result(page.wait(BLESSING_WELKIN_RETRY_DELAY_MS))?,
            },
        ),
        BlessingOfTheWelkinMoonStep::new(
            BlessingOfTheWelkinMoonStepCondition::UntilStableClear,
            "repeat until blessing and primogem screens disappear",
            BlessingOfTheWelkinMoonStepAction::LoopUntilClear {
                rule: loop_rule,
                locators: detection_locators.clone(),
                claim_click_events: claim_click_events.clone(),
            },
        ),
    ];

    Ok(BlessingOfTheWelkinMoonExecutionPlan {
        task_key: BLESSING_WELKIN_TASK_KEY.to_string(),
        display_name: "Blessing of the Welkin Moon".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        server_time_gate,
        detection_locators,
        loop_rule,
        claim_click_events,
        steps,
        notes: "Legacy Welkin Moon popup handling now has a Rust server-time-gated template/input executor with the stable-clear loop preserved.".to_string(),
    })
}

fn blessing_detection_locators(page: &BvPage) -> Result<BlessingOfTheWelkinMoonDetectionLocators> {
    Ok(BlessingOfTheWelkinMoonDetectionLocators {
        girl_moon: image_locator(
            page,
            BLESSING_WELKIN_GIRL_MOON,
            Some(bottom_half_roi(page.capture_size)?),
            0.8,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
        welkin_moon: image_locator(
            page,
            BLESSING_WELKIN_WELKIN_MOON,
            Some(bottom_half_roi(page.capture_size)?),
            0.8,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
        primogem: image_locator(
            page,
            BLESSING_WELKIN_PRIMOGEM,
            None,
            0.8,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
    })
}

fn claim_click_events(capture_size: Size) -> Vec<InputEvent> {
    let (x, y) = screen_point_1080p(capture_size, CLAIM_CLICK_X_1080P, CLAIM_CLICK_Y_1080P);
    InputSequence::new()
        .move_mouse_to(x, y)
        .mouse_click(MouseButton::Left)
        .delay(CLAIM_CLICK_GAP_MS)
        .mouse_click(MouseButton::Left)
        .events()
        .to_vec()
}

fn bottom_half_roi(size: Size) -> Result<bgi_vision::Rect> {
    task_vision_result(bgi_vision::Rect::new(
        0,
        (size.height / 2) as i32,
        size.width as i32,
        (size.height / 2) as i32,
    ))
}

fn screen_point_1080p(capture_size: Size, x_1080p: f64, y_1080p: f64) -> (i32, i32) {
    let scale = capture_size.width as f64 / 1920.0;
    (
        (x_1080p * scale).round() as i32,
        (y_1080p * scale).round() as i32,
    )
}
