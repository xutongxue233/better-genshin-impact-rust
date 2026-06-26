use super::{task_vision_result, RETURN_MAIN_UI_PAIMON_MENU};
use crate::{Result, TaskPortState};
use bgi_input::{InputEvent, InputSequence};
use bgi_vision::{BvImage, BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const WONDERLAND_CYCLE_TASK_KEY: &str = "WonderlandCycle";
pub const WONDERLAND_CYCLE_CLOSE: &str = "Common/Element:wonderland_close.png";
pub const WONDERLAND_CYCLE_BLACK_CONFIRM: &str = "Common/Element:btn_black_confirm.png";
pub const WONDERLAND_CYCLE_BACK_TEYVAT: &str = "Common/Element:btn_back_teyvat.png";
pub const WONDERLAND_CYCLE_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const WONDERLAND_CYCLE_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;

const VK_F6: u16 = 0x75;
const VK_ESCAPE: u16 = 0x1B;
const OPEN_WONDERLAND_ATTEMPTS: u16 = 10;
const OPEN_WONDERLAND_INTERVAL_MS: u32 = 1_000;
const SELECT_WONDERLAND_ATTEMPTS: u16 = 5;
const SELECT_WONDERLAND_INTERVAL_MS: u32 = 800;
const CONFIRM_DISAPPEAR_ATTEMPTS: u16 = 5;
const CONFIRM_DISAPPEAR_INTERVAL_MS: u32 = 1_000;
const MAIN_UI_ATTEMPTS: u16 = 120;
const MAIN_UI_INTERVAL_MS: u32 = 1_000;
const BACK_TEYVAT_MENU_ATTEMPTS: u16 = 20;
const BACK_TEYVAT_MENU_INTERVAL_MS: u32 = 800;
const AFTER_CONFIRM_DELAY_MS: u32 = 1_000;
const AFTER_MAIN_UI_DELAY_MS: u32 = 500;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WonderlandCycleExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub locators: WonderlandCycleLocators,
    pub steps: Vec<WonderlandCycleStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WonderlandCycleLocators {
    pub wonderland_close: BvLocatorPlan,
    pub black_confirm: BvLocatorPlan,
    pub paimon_menu: BvLocatorPlan,
    pub back_teyvat: BvLocatorPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WonderlandCycleRetryRule {
    pub max_attempts: u16,
    pub interval_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WonderlandCycleStep {
    pub phase: WonderlandCycleStepPhase,
    pub condition: WonderlandCycleStepCondition,
    pub label: String,
    pub action: WonderlandCycleStepAction,
}

impl WonderlandCycleStep {
    fn new(
        phase: WonderlandCycleStepPhase,
        condition: WonderlandCycleStepCondition,
        label: impl Into<String>,
        action: WonderlandCycleStepAction,
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
pub enum WonderlandCycleStepPhase {
    EnterWonderland,
    ReturnTeyvat,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WonderlandCycleStepCondition {
    Always,
    WhenWonderlandMenuDetected,
    WhenConfirmDialogDetected,
    WhenInWonderlandMainUi,
    WhenBackTeyvatMenuDetected,
    WhenReturnedToTeyvat,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum WonderlandCycleStepAction {
    RetryUntilAppear {
        locator: BvLocatorPlan,
        rule: WonderlandCycleRetryRule,
        retry_action: WonderlandCycleRetryAction,
    },
    RetryUntilDisappear {
        locator: BvLocatorPlan,
        rule: WonderlandCycleRetryRule,
        retry_action: WonderlandCycleRetryAction,
    },
    Page {
        command: BvPageCommand,
    },
    ReturnResult {
        result: WonderlandCycleStepResult,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum WonderlandCycleRetryAction {
    None,
    Input { events: Vec<InputEvent> },
    Page { command: BvPageCommand },
    Locator { locator: BvLocatorPlan },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WonderlandCycleStepResult {
    EnteredWonderland,
    ReturnedToTeyvat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WonderlandCycleExecutionConfig {
    pub capture_size: Size,
}

impl Default for WonderlandCycleExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                WONDERLAND_CYCLE_DEFAULT_CAPTURE_WIDTH,
                WONDERLAND_CYCLE_DEFAULT_CAPTURE_HEIGHT,
            ),
        }
    }
}

impl WonderlandCycleExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

pub fn plan_wonderland_cycle(capture_size: Size) -> Result<WonderlandCycleExecutionPlan> {
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let locators = WonderlandCycleLocators {
        wonderland_close: image_locator(
            &page,
            WONDERLAND_CYCLE_CLOSE,
            None,
            0.8,
            false,
            BvLocatorOperation::WaitFor,
            Some(OPEN_WONDERLAND_INTERVAL_MS),
        )?,
        black_confirm: image_locator(
            &page,
            WONDERLAND_CYCLE_BLACK_CONFIRM,
            None,
            0.8,
            true,
            BvLocatorOperation::WaitFor,
            Some(SELECT_WONDERLAND_INTERVAL_MS),
        )?,
        paimon_menu: image_locator(
            &page,
            RETURN_MAIN_UI_PAIMON_MENU,
            Some(top_left_quarter_rect(capture_size)?),
            0.8,
            false,
            BvLocatorOperation::WaitFor,
            Some(MAIN_UI_INTERVAL_MS),
        )?,
        back_teyvat: image_locator(
            &page,
            WONDERLAND_CYCLE_BACK_TEYVAT,
            None,
            0.8,
            false,
            BvLocatorOperation::WaitFor,
            Some(BACK_TEYVAT_MENU_INTERVAL_MS),
        )?,
    };
    let confirm_click_locator = with_operation(
        &locators.black_confirm,
        BvLocatorOperation::Click,
        Some(CONFIRM_DISAPPEAR_INTERVAL_MS),
    );
    let back_teyvat_click_locator = with_operation(
        &locators.back_teyvat,
        BvLocatorOperation::Click,
        Some(BACK_TEYVAT_MENU_INTERVAL_MS),
    );
    let steps = vec![
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::EnterWonderland,
            WonderlandCycleStepCondition::Always,
            "log wonderland cycle start",
            WonderlandCycleStepAction::Log {
                message: "start WonderlandCycle common job plan".to_string(),
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::EnterWonderland,
            WonderlandCycleStepCondition::Always,
            "open wonderland menu until close button appears",
            WonderlandCycleStepAction::RetryUntilAppear {
                locator: locators.wonderland_close.clone(),
                rule: retry_rule(OPEN_WONDERLAND_ATTEMPTS, OPEN_WONDERLAND_INTERVAL_MS),
                retry_action: WonderlandCycleRetryAction::Input {
                    events: InputSequence::new().key_press(VK_F6).events().to_vec(),
                },
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::EnterWonderland,
            WonderlandCycleStepCondition::WhenWonderlandMenuDetected,
            "click first wonderland entry until confirm dialog appears",
            WonderlandCycleStepAction::RetryUntilAppear {
                locator: locators.black_confirm.clone(),
                rule: retry_rule(SELECT_WONDERLAND_ATTEMPTS, SELECT_WONDERLAND_INTERVAL_MS),
                retry_action: WonderlandCycleRetryAction::Page {
                    command: page.click_1080p(680.0, 310.0),
                },
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::EnterWonderland,
            WonderlandCycleStepCondition::WhenConfirmDialogDetected,
            "click confirm until dialog disappears",
            WonderlandCycleStepAction::RetryUntilDisappear {
                locator: with_operation(
                    &locators.black_confirm,
                    BvLocatorOperation::WaitForDisappear,
                    Some(CONFIRM_DISAPPEAR_INTERVAL_MS),
                ),
                rule: retry_rule(CONFIRM_DISAPPEAR_ATTEMPTS, CONFIRM_DISAPPEAR_INTERVAL_MS),
                retry_action: WonderlandCycleRetryAction::Locator {
                    locator: confirm_click_locator.clone(),
                },
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::EnterWonderland,
            WonderlandCycleStepCondition::WhenConfirmDialogDetected,
            "wait after entering wonderland",
            WonderlandCycleStepAction::Page {
                command: task_vision_result(page.wait(AFTER_CONFIRM_DELAY_MS))?,
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::EnterWonderland,
            WonderlandCycleStepCondition::WhenConfirmDialogDetected,
            "wait for main UI after entering wonderland",
            WonderlandCycleStepAction::RetryUntilAppear {
                locator: locators.paimon_menu.clone(),
                rule: retry_rule(MAIN_UI_ATTEMPTS, MAIN_UI_INTERVAL_MS),
                retry_action: WonderlandCycleRetryAction::None,
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::EnterWonderland,
            WonderlandCycleStepCondition::WhenInWonderlandMainUi,
            "return entered wonderland result",
            WonderlandCycleStepAction::ReturnResult {
                result: WonderlandCycleStepResult::EnteredWonderland,
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::EnterWonderland,
            WonderlandCycleStepCondition::WhenInWonderlandMainUi,
            "wait before opening back-to-teyvat menu",
            WonderlandCycleStepAction::Page {
                command: task_vision_result(page.wait(AFTER_MAIN_UI_DELAY_MS))?,
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::ReturnTeyvat,
            WonderlandCycleStepCondition::WhenInWonderlandMainUi,
            "press Escape until back-to-teyvat button appears",
            WonderlandCycleStepAction::RetryUntilAppear {
                locator: locators.back_teyvat.clone(),
                rule: retry_rule(BACK_TEYVAT_MENU_ATTEMPTS, BACK_TEYVAT_MENU_INTERVAL_MS),
                retry_action: WonderlandCycleRetryAction::Input {
                    events: InputSequence::new().key_press(VK_ESCAPE).events().to_vec(),
                },
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::ReturnTeyvat,
            WonderlandCycleStepCondition::WhenBackTeyvatMenuDetected,
            "click back-to-teyvat until confirm dialog appears",
            WonderlandCycleStepAction::RetryUntilAppear {
                locator: locators.black_confirm.clone(),
                rule: retry_rule(SELECT_WONDERLAND_ATTEMPTS, SELECT_WONDERLAND_INTERVAL_MS),
                retry_action: WonderlandCycleRetryAction::Locator {
                    locator: back_teyvat_click_locator,
                },
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::ReturnTeyvat,
            WonderlandCycleStepCondition::WhenConfirmDialogDetected,
            "confirm back-to-teyvat until dialog disappears",
            WonderlandCycleStepAction::RetryUntilDisappear {
                locator: with_operation(
                    &locators.black_confirm,
                    BvLocatorOperation::WaitForDisappear,
                    Some(CONFIRM_DISAPPEAR_INTERVAL_MS),
                ),
                rule: retry_rule(CONFIRM_DISAPPEAR_ATTEMPTS, CONFIRM_DISAPPEAR_INTERVAL_MS),
                retry_action: WonderlandCycleRetryAction::Locator {
                    locator: confirm_click_locator,
                },
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::ReturnTeyvat,
            WonderlandCycleStepCondition::WhenConfirmDialogDetected,
            "wait after returning to teyvat",
            WonderlandCycleStepAction::Page {
                command: task_vision_result(page.wait(AFTER_CONFIRM_DELAY_MS))?,
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::ReturnTeyvat,
            WonderlandCycleStepCondition::WhenConfirmDialogDetected,
            "wait for main UI after returning to teyvat",
            WonderlandCycleStepAction::RetryUntilAppear {
                locator: locators.paimon_menu.clone(),
                rule: retry_rule(MAIN_UI_ATTEMPTS, MAIN_UI_INTERVAL_MS),
                retry_action: WonderlandCycleRetryAction::None,
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::ReturnTeyvat,
            WonderlandCycleStepCondition::WhenReturnedToTeyvat,
            "return teyvat result",
            WonderlandCycleStepAction::ReturnResult {
                result: WonderlandCycleStepResult::ReturnedToTeyvat,
            },
        ),
        WonderlandCycleStep::new(
            WonderlandCycleStepPhase::Cleanup,
            WonderlandCycleStepCondition::WhenReturnedToTeyvat,
            "final settle delay",
            WonderlandCycleStepAction::Page {
                command: task_vision_result(page.wait(AFTER_MAIN_UI_DELAY_MS))?,
            },
        ),
    ];

    Ok(WonderlandCycleExecutionPlan {
        task_key: WONDERLAND_CYCLE_TASK_KEY.to_string(),
        display_name: "Wonderland Cycle".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        locators,
        steps,
        notes: "Legacy EnterAndExitWonderland flow is represented as a Rust retry/input/locator plan with a template/input executor boundary.".to_string(),
    })
}

fn retry_rule(max_attempts: u16, interval_ms: u32) -> WonderlandCycleRetryRule {
    WonderlandCycleRetryRule {
        max_attempts,
        interval_ms,
    }
}

fn image_locator(
    page: &BvPage,
    asset: &str,
    roi: Option<Rect>,
    threshold: f64,
    use_3_channels: bool,
    operation: BvLocatorOperation,
    timeout_ms: Option<u32>,
) -> Result<BvLocatorPlan> {
    let image = task_vision_result(BvImage::new(asset))?;
    let mut locator = task_vision_result(page.locator_for_image(&image, roi, threshold))?;
    locator.recognition_object.template.use_3_channels = use_3_channels;
    Ok(locator.plan(operation, timeout_ms))
}

fn with_operation(
    locator: &BvLocatorPlan,
    operation: BvLocatorOperation,
    timeout_ms: Option<u32>,
) -> BvLocatorPlan {
    let mut locator = locator.clone();
    locator.operation = operation;
    if let Some(timeout_ms) = timeout_ms {
        locator.timeout_ms = timeout_ms;
    }
    locator
}

fn top_left_quarter_rect(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        0,
        (size.width / 4) as i32,
        (size.height / 4) as i32,
    ))
}
