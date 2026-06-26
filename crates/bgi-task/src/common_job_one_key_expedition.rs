use super::{image_locator, task_vision_result};
use crate::{Result, TaskPortState};
use bgi_input::{InputEvent, InputSequence};
use bgi_vision::{BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const ONE_KEY_EXPEDITION_TASK_KEY: &str = "OneKeyExpedition";
pub const ONE_KEY_EXPEDITION_COLLECT: &str = "AutoSkip:collect.png";
pub const ONE_KEY_EXPEDITION_RE_DISPATCH: &str = "AutoSkip:re.png";
pub const ONE_KEY_EXPEDITION_VK_ESCAPE: u16 = 0x1B;

const TEMPLATE_THRESHOLD: f64 = 0.8;
const COLLECT_ATTEMPTS: u8 = 2;
const WAIT_BEFORE_COLLECT_RETRY_MS: u32 = 1_000;
const AFTER_COLLECT_DELAY_MS: u32 = 1_100;
const RE_DISPATCH_RETRY_ATTEMPTS: u8 = 3;
const RE_DISPATCH_RETRY_WINDOW_MS: u32 = 1_000;
const RE_DISPATCH_PRE_CAPTURE_DELAY_MS: u32 = 1;
const AFTER_RE_DISPATCH_DELAY_MS: u32 = 500;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OneKeyExpeditionExecutionPlan {
    pub task_key: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub activate_window_before_run: bool,
    pub collect_locator: BvLocatorPlan,
    pub re_dispatch_locator: BvLocatorPlan,
    pub collect_attempts: u8,
    pub wait_before_retry_ms: u32,
    pub after_collect_delay_ms: u32,
    pub re_dispatch_retry_attempts: u8,
    pub re_dispatch_retry_window_ms: u32,
    pub re_dispatch_pre_capture_delay_ms: u32,
    pub after_re_dispatch_delay_ms: u32,
    pub exit_events: Vec<InputEvent>,
    pub catches_and_logs_exceptions: bool,
    pub clears_vision_drawings_finally: bool,
    pub steps: Vec<OneKeyExpeditionStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OneKeyExpeditionExecutionConfig {
    pub capture_size: Size,
}

impl Default for OneKeyExpeditionExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
        }
    }
}

impl OneKeyExpeditionExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OneKeyExpeditionStep {
    pub phase: OneKeyExpeditionStepPhase,
    pub condition: OneKeyExpeditionStepCondition,
    pub attempt: Option<u8>,
    pub label: String,
    pub action: OneKeyExpeditionStepAction,
}

impl OneKeyExpeditionStep {
    fn new(
        phase: OneKeyExpeditionStepPhase,
        condition: OneKeyExpeditionStepCondition,
        attempt: Option<u8>,
        label: impl Into<String>,
        action: OneKeyExpeditionStepAction,
    ) -> Self {
        Self {
            phase,
            condition,
            attempt,
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OneKeyExpeditionStepPhase {
    Preflight,
    Collect,
    ReDispatch,
    Exit,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OneKeyExpeditionStepCondition {
    Always,
    ForCollectAttempt,
    WhenCollectMissingAndCanRetry,
    WhenCollectMissingAfterRetries,
    WhenCollectDetected,
    ForReDispatchAttempt,
    WhenReDispatchMissingAndCanRetry,
    WhenReDispatchMissingAfterRetries,
    WhenReDispatchDetected,
    Finally,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OneKeyExpeditionStepResult {
    Completed,
    CollectAllMissing,
    ReDispatchMissing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum OneKeyExpeditionStepAction {
    ActivateWindow,
    Locator { locator: BvLocatorPlan },
    Page { command: BvPageCommand },
    Input { events: Vec<InputEvent> },
    Log { message: String },
    ClearVisionDrawings,
    ReturnResult { result: OneKeyExpeditionStepResult },
}

pub fn plan_one_key_expedition(capture_size: Size) -> Result<OneKeyExpeditionExecutionPlan> {
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let collect_locator = image_locator(
        &page,
        ONE_KEY_EXPEDITION_COLLECT,
        Some(collect_roi(capture_size)?),
        TEMPLATE_THRESHOLD,
        BvLocatorOperation::Click,
        Some(1_000),
    )?;
    let re_dispatch_locator = image_locator(
        &page,
        ONE_KEY_EXPEDITION_RE_DISPATCH,
        Some(re_dispatch_roi(capture_size)?),
        TEMPLATE_THRESHOLD,
        BvLocatorOperation::Click,
        Some(RE_DISPATCH_RETRY_WINDOW_MS),
    )?;
    plan_one_key_expedition_with_locators(capture_size, collect_locator, re_dispatch_locator)
}

pub fn plan_one_key_expedition_with_locators(
    capture_size: Size,
    collect_locator: BvLocatorPlan,
    re_dispatch_locator: BvLocatorPlan,
) -> Result<OneKeyExpeditionExecutionPlan> {
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let exit_events = InputSequence::new()
        .key_press(ONE_KEY_EXPEDITION_VK_ESCAPE)
        .events()
        .to_vec();
    let mut steps = vec![OneKeyExpeditionStep::new(
        OneKeyExpeditionStepPhase::Preflight,
        OneKeyExpeditionStepCondition::Always,
        None,
        "activate game window before one-key expedition",
        OneKeyExpeditionStepAction::ActivateWindow,
    )];

    for attempt in 1..=COLLECT_ATTEMPTS {
        steps.push(OneKeyExpeditionStep::new(
            OneKeyExpeditionStepPhase::Collect,
            OneKeyExpeditionStepCondition::ForCollectAttempt,
            Some(attempt),
            format!("probe and click collect-all button attempt {attempt}"),
            OneKeyExpeditionStepAction::Locator {
                locator: collect_locator.clone(),
            },
        ));
        if attempt < COLLECT_ATTEMPTS {
            steps.push(OneKeyExpeditionStep::new(
                OneKeyExpeditionStepPhase::Collect,
                OneKeyExpeditionStepCondition::WhenCollectMissingAndCanRetry,
                Some(attempt),
                "log collect-all missing before retry",
                OneKeyExpeditionStepAction::Log {
                    message: "探索派遣：未找到领取按钮".to_string(),
                },
            ));
            steps.push(OneKeyExpeditionStep::new(
                OneKeyExpeditionStepPhase::Collect,
                OneKeyExpeditionStepCondition::WhenCollectMissingAndCanRetry,
                Some(attempt),
                "log collect-all retry wait",
                OneKeyExpeditionStepAction::Log {
                    message: "探索派遣：等待1s后重试".to_string(),
                },
            ));
            steps.push(OneKeyExpeditionStep::new(
                OneKeyExpeditionStepPhase::Collect,
                OneKeyExpeditionStepCondition::WhenCollectMissingAndCanRetry,
                Some(attempt),
                "wait before retrying collect-all detection",
                OneKeyExpeditionStepAction::Page {
                    command: task_vision_result(page.wait(WAIT_BEFORE_COLLECT_RETRY_MS))?,
                },
            ));
        }
    }

    steps.push(OneKeyExpeditionStep::new(
        OneKeyExpeditionStepPhase::Collect,
        OneKeyExpeditionStepCondition::WhenCollectMissingAfterRetries,
        Some(COLLECT_ATTEMPTS),
        "log collect-all missing after retries",
        OneKeyExpeditionStepAction::Log {
            message: "探索派遣：未找到领取按钮".to_string(),
        },
    ));
    steps.push(OneKeyExpeditionStep::new(
        OneKeyExpeditionStepPhase::Collect,
        OneKeyExpeditionStepCondition::WhenCollectMissingAfterRetries,
        Some(COLLECT_ATTEMPTS),
        "return collect-all missing",
        OneKeyExpeditionStepAction::ReturnResult {
            result: OneKeyExpeditionStepResult::CollectAllMissing,
        },
    ));

    steps.push(OneKeyExpeditionStep::new(
        OneKeyExpeditionStepPhase::Collect,
        OneKeyExpeditionStepCondition::WhenCollectDetected,
        None,
        "log collect-all success",
        OneKeyExpeditionStepAction::Log {
            message: "探索派遣：全部领取".to_string(),
        },
    ));
    steps.push(OneKeyExpeditionStep::new(
        OneKeyExpeditionStepPhase::Collect,
        OneKeyExpeditionStepCondition::WhenCollectDetected,
        None,
        "wait after collect-all click",
        OneKeyExpeditionStepAction::Page {
            command: task_vision_result(page.wait(AFTER_COLLECT_DELAY_MS))?,
        },
    ));

    for attempt in 1..=RE_DISPATCH_RETRY_ATTEMPTS {
        steps.push(OneKeyExpeditionStep::new(
            OneKeyExpeditionStepPhase::ReDispatch,
            OneKeyExpeditionStepCondition::ForReDispatchAttempt,
            Some(attempt),
            "wait before re-dispatch probe capture",
            OneKeyExpeditionStepAction::Page {
                command: task_vision_result(page.wait(RE_DISPATCH_PRE_CAPTURE_DELAY_MS))?,
            },
        ));
        steps.push(OneKeyExpeditionStep::new(
            OneKeyExpeditionStepPhase::ReDispatch,
            OneKeyExpeditionStepCondition::ForReDispatchAttempt,
            Some(attempt),
            format!("probe and click re-dispatch button attempt {attempt}"),
            OneKeyExpeditionStepAction::Locator {
                locator: re_dispatch_locator.clone(),
            },
        ));
        if attempt < RE_DISPATCH_RETRY_ATTEMPTS {
            steps.push(OneKeyExpeditionStep::new(
                OneKeyExpeditionStepPhase::ReDispatch,
                OneKeyExpeditionStepCondition::WhenReDispatchMissingAndCanRetry,
                Some(attempt),
                "wait before retrying re-dispatch detection",
                OneKeyExpeditionStepAction::Page {
                    command: task_vision_result(page.wait(RE_DISPATCH_RETRY_WINDOW_MS))?,
                },
            ));
        }
    }

    steps.extend([
        OneKeyExpeditionStep::new(
            OneKeyExpeditionStepPhase::ReDispatch,
            OneKeyExpeditionStepCondition::WhenReDispatchMissingAfterRetries,
            Some(RE_DISPATCH_RETRY_ATTEMPTS),
            "log re-dispatch popup missing after retries",
            OneKeyExpeditionStepAction::Log {
                message: "未检测到弹出菜单".to_string(),
            },
        ),
        OneKeyExpeditionStep::new(
            OneKeyExpeditionStepPhase::ReDispatch,
            OneKeyExpeditionStepCondition::WhenReDispatchMissingAfterRetries,
            Some(RE_DISPATCH_RETRY_ATTEMPTS),
            "return re-dispatch missing",
            OneKeyExpeditionStepAction::ReturnResult {
                result: OneKeyExpeditionStepResult::ReDispatchMissing,
            },
        ),
        OneKeyExpeditionStep::new(
            OneKeyExpeditionStepPhase::ReDispatch,
            OneKeyExpeditionStepCondition::WhenReDispatchDetected,
            None,
            "log re-dispatch success",
            OneKeyExpeditionStepAction::Log {
                message: "探索派遣：再次派遣".to_string(),
            },
        ),
        OneKeyExpeditionStep::new(
            OneKeyExpeditionStepPhase::Exit,
            OneKeyExpeditionStepCondition::WhenReDispatchDetected,
            None,
            "wait after re-dispatch click",
            OneKeyExpeditionStepAction::Page {
                command: task_vision_result(page.wait(AFTER_RE_DISPATCH_DELAY_MS))?,
            },
        ),
        OneKeyExpeditionStep::new(
            OneKeyExpeditionStepPhase::Exit,
            OneKeyExpeditionStepCondition::WhenReDispatchDetected,
            None,
            "press Escape to exit expedition page",
            OneKeyExpeditionStepAction::Input {
                events: exit_events.clone(),
            },
        ),
        OneKeyExpeditionStep::new(
            OneKeyExpeditionStepPhase::Exit,
            OneKeyExpeditionStepCondition::WhenReDispatchDetected,
            None,
            "log one-key expedition completed",
            OneKeyExpeditionStepAction::Log {
                message: "探索派遣：完成".to_string(),
            },
        ),
        OneKeyExpeditionStep::new(
            OneKeyExpeditionStepPhase::Exit,
            OneKeyExpeditionStepCondition::WhenReDispatchDetected,
            None,
            "return one-key expedition completed",
            OneKeyExpeditionStepAction::ReturnResult {
                result: OneKeyExpeditionStepResult::Completed,
            },
        ),
        OneKeyExpeditionStep::new(
            OneKeyExpeditionStepPhase::Cleanup,
            OneKeyExpeditionStepCondition::Finally,
            None,
            "clear vision drawings",
            OneKeyExpeditionStepAction::ClearVisionDrawings,
        ),
    ]);

    Ok(OneKeyExpeditionExecutionPlan {
        task_key: ONE_KEY_EXPEDITION_TASK_KEY.to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        activate_window_before_run: true,
        collect_locator,
        re_dispatch_locator,
        collect_attempts: COLLECT_ATTEMPTS,
        wait_before_retry_ms: WAIT_BEFORE_COLLECT_RETRY_MS,
        after_collect_delay_ms: AFTER_COLLECT_DELAY_MS,
        re_dispatch_retry_attempts: RE_DISPATCH_RETRY_ATTEMPTS,
        re_dispatch_retry_window_ms: RE_DISPATCH_RETRY_WINDOW_MS,
        re_dispatch_pre_capture_delay_ms: RE_DISPATCH_PRE_CAPTURE_DELAY_MS,
        after_re_dispatch_delay_ms: AFTER_RE_DISPATCH_DELAY_MS,
        exit_events,
        catches_and_logs_exceptions: true,
        clears_vision_drawings_finally: true,
        steps,
    })
}

fn collect_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        (size.height - size.height / 3) as i32,
        (size.width / 4) as i32,
        (size.height / 3) as i32,
    ))
}

fn re_dispatch_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        (size.width / 2) as i32,
        (size.height - size.height / 4) as i32,
        (size.width / 4) as i32,
        (size.height / 4) as i32,
    ))
}
