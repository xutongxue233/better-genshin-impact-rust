use crate::{
    common_job_executor::{CommonJobRuntime, CommonJobRuntimeOutcome},
    Result, TaskError, TaskPortState,
};
use bgi_input::{InputEvent, InputSequence};
use bgi_vision::{BvImage, BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[path = "redeem_code_text.rs"]
mod redeem_code_text;

pub use redeem_code_text::{extract_redeem_codes_from_text, REDEEM_CODE_PATTERN};

pub const USE_REDEEM_CODE_TASK_KEY: &str = "UseRedeemCode";
pub const USE_REDEEM_CODE_ESC_RETURN_BUTTON: &str = "UseRedeemCode:esc_return_button.png";
pub const COMMON_BTN_WHITE_CONFIRM: &str = "Common/Element:btn_white_confirm.png";
pub const COMMON_BTN_BLACK_CONFIRM: &str = "Common/Element:btn_black_confirm.png";
pub const VK_ESCAPE: u16 = 0x1B;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedeemCodeEntry {
    pub code: String,
    pub items: Option<String>,
}

impl RedeemCodeEntry {
    pub fn new(code: impl Into<String>, items: Option<String>) -> Option<Self> {
        let code = code.into().trim().to_string();
        if code.is_empty() {
            return None;
        }
        Some(Self { code, items })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UseRedeemCodeStepPhase {
    Setup,
    PerCode,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UseRedeemCodeStepCondition {
    Always,
    WhenSuccessDetected,
    WhenSuccessNotDetected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum UseRedeemCodeStepAction {
    CommonJob { task_key: String },
    Input { events: Vec<InputEvent> },
    Page { command: BvPageCommand },
    Locator { locator: BvLocatorPlan },
    ClipboardSet { text: String },
    ClipboardClear,
    Log { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UseRedeemCodeStep {
    pub phase: UseRedeemCodeStepPhase,
    pub condition: UseRedeemCodeStepCondition,
    pub code: Option<String>,
    pub label: String,
    pub action: UseRedeemCodeStepAction,
}

impl UseRedeemCodeStep {
    fn new(
        phase: UseRedeemCodeStepPhase,
        label: impl Into<String>,
        action: UseRedeemCodeStepAction,
    ) -> Self {
        Self {
            phase,
            condition: UseRedeemCodeStepCondition::Always,
            code: None,
            label: label.into(),
            action,
        }
    }

    fn for_code(code: &str, label: impl Into<String>, action: UseRedeemCodeStepAction) -> Self {
        Self {
            phase: UseRedeemCodeStepPhase::PerCode,
            condition: UseRedeemCodeStepCondition::Always,
            code: Some(code.to_string()),
            label: label.into(),
            action,
        }
    }

    fn conditional_for_code(
        code: &str,
        condition: UseRedeemCodeStepCondition,
        label: impl Into<String>,
        action: UseRedeemCodeStepAction,
    ) -> Self {
        Self {
            phase: UseRedeemCodeStepPhase::PerCode,
            condition,
            code: Some(code.to_string()),
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UseRedeemCodeExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub codes: Vec<RedeemCodeEntry>,
    pub steps: Vec<UseRedeemCodeStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UseRedeemCodeExecutionConfig {
    pub codes: Vec<RedeemCodeEntry>,
    pub capture_size: Size,
}

impl Default for UseRedeemCodeExecutionConfig {
    fn default() -> Self {
        Self {
            codes: Vec::new(),
            capture_size: Size::new(1920, 1080),
        }
    }
}

impl UseRedeemCodeExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let Some(value) = value else {
            return Self::default();
        };
        let mut config = serde_json::from_value::<Self>(value.clone()).unwrap_or_default();
        if let Some(capture_size) = redeem_code_capture_size_from_config_value(value) {
            config.capture_size = capture_size;
        }
        if config.codes.is_empty() {
            config.codes = redeem_code_entries_from_config_value(value);
        }
        config
    }
}

fn redeem_code_capture_size_from_config_value(value: &Value) -> Option<Size> {
    value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn redeem_code_entries_from_config_value(value: &Value) -> Vec<RedeemCodeEntry> {
    let Some(codes) = value
        .get("codes")
        .or_else(|| value.get("Codes"))
        .or_else(|| value.get("redeemCodes"))
        .or_else(|| value.get("redeem_codes"))
    else {
        return Vec::new();
    };

    if let Some(text) = codes.as_str() {
        return redeem_code_entries_from_config_text(text);
    }

    let Some(items) = codes.as_array() else {
        return Vec::new();
    };

    items
        .iter()
        .filter_map(|item| {
            item.as_str()
                .and_then(|code| RedeemCodeEntry::new(code, None))
                .or_else(|| serde_json::from_value::<RedeemCodeEntry>(item.clone()).ok())
        })
        .collect()
}

fn redeem_code_entries_from_config_text(text: &str) -> Vec<RedeemCodeEntry> {
    let extracted = extract_redeem_codes_from_text(text);
    if extracted.is_empty() {
        RedeemCodeEntry::new(text, None).into_iter().collect()
    } else {
        redeem_code_entries_from_strings(extracted.iter().map(String::as_str))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UseRedeemCodeRuntimeActionKind {
    CommonJob,
    Input,
    Page,
    Locator,
    ClipboardSet,
    ClipboardClear,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UseRedeemCodeSkipReason {
    SuccessDetected,
    SuccessMissing,
    SuccessProbeMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseRedeemCodeSuccessDetection {
    pub code: String,
    pub detected: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseRedeemCodeExecutorState {
    pub setup_return_main_ui_completed: Option<bool>,
    pub cleanup_return_main_ui_completed: Option<bool>,
    pub processed_codes: Vec<String>,
    pub successful_codes: Vec<String>,
    pub failed_codes: Vec<String>,
    pub success_detections: Vec<UseRedeemCodeSuccessDetection>,
    pub last_success_detection: Option<UseRedeemCodeSuccessDetection>,
    pub clipboard_sets: Vec<String>,
    pub clipboard_cleared: bool,
    pub failed_required_steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseRedeemCodeRuntimeStepReport {
    pub phase: UseRedeemCodeStepPhase,
    pub condition: UseRedeemCodeStepCondition,
    pub code: Option<String>,
    pub label: String,
    pub action_kind: UseRedeemCodeRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl UseRedeemCodeRuntimeStepReport {
    fn executed(step: &UseRedeemCodeStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            code: step.code.clone(),
            label: step.label.clone(),
            action_kind: redeem_code_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseRedeemCodeSkippedStep {
    pub phase: UseRedeemCodeStepPhase,
    pub condition: UseRedeemCodeStepCondition,
    pub code: Option<String>,
    pub label: String,
    pub reason: UseRedeemCodeSkipReason,
}

impl UseRedeemCodeSkippedStep {
    fn new(step: &UseRedeemCodeStep, reason: UseRedeemCodeSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            code: step.code.clone(),
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseRedeemCodeExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: UseRedeemCodeExecutorState,
    pub executed_steps: Vec<UseRedeemCodeRuntimeStepReport>,
    pub skipped_steps: Vec<UseRedeemCodeSkippedStep>,
}

pub trait UseRedeemCodeRuntime: CommonJobRuntime {
    fn execute_redeem_common_job(&mut self, task_key: &str) -> Result<CommonJobRuntimeOutcome>;

    fn set_redeem_clipboard_text(&mut self, text: &str) -> Result<CommonJobRuntimeOutcome>;

    fn clear_redeem_clipboard(&mut self) -> Result<CommonJobRuntimeOutcome>;
}

pub fn redeem_code_entries_from_strings<'a>(
    codes: impl IntoIterator<Item = &'a str>,
) -> Vec<RedeemCodeEntry> {
    codes
        .into_iter()
        .filter_map(|code| RedeemCodeEntry::new(code, None))
        .collect()
}

pub fn plan_use_redeem_code_strings<'a>(
    codes: impl IntoIterator<Item = &'a str>,
) -> Result<UseRedeemCodeExecutionPlan> {
    plan_use_redeem_codes(
        redeem_code_entries_from_strings(codes),
        Size::new(1920, 1080),
    )
}

pub fn plan_use_redeem_codes(
    codes: impl IntoIterator<Item = RedeemCodeEntry>,
    capture_size: Size,
) -> Result<UseRedeemCodeExecutionPlan> {
    let codes: Vec<RedeemCodeEntry> = codes.into_iter().collect();
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let mut steps = Vec::new();

    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "log redeem codes",
        UseRedeemCodeStepAction::Log {
            message: format!(
                "start use redeem code task with {} non-empty code(s)",
                codes.len()
            ),
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "return to main UI before opening settings",
        UseRedeemCodeStepAction::CommonJob {
            task_key: "ReturnMainUi".to_string(),
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "press Escape to open menu",
        UseRedeemCodeStepAction::Input {
            events: InputSequence::new().key_press(VK_ESCAPE).events().to_vec(),
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "wait for Escape return button",
        UseRedeemCodeStepAction::Locator {
            locator: image_locator(
                &page,
                USE_REDEEM_CODE_ESC_RETURN_BUTTON,
                None,
                0.8,
                false,
                BvLocatorOperation::WaitFor,
                None,
            )?,
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "click settings button",
        UseRedeemCodeStepAction::Page {
            command: page.click_1080p(45.0, 825.0),
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "wait after opening settings",
        UseRedeemCodeStepAction::Page {
            command: task_vision_result(page.wait(1_000))?,
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "click account tab",
        UseRedeemCodeStepAction::Locator {
            locator: text_locator(
                &page,
                "账户",
                Some(left_ratio_rect(capture_size, 0.2)?),
                BvLocatorOperation::Click,
                None,
            ),
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "wait after account tab",
        UseRedeemCodeStepAction::Page {
            command: task_vision_result(page.wait(300))?,
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "click go redeem",
        UseRedeemCodeStepAction::Locator {
            locator: text_locator(
                &page,
                "前往兑换",
                Some(right_ratio_rect(capture_size, 0.3)?),
                BvLocatorOperation::Click,
                None,
            ),
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "wait for redeem dialog",
        UseRedeemCodeStepAction::Locator {
            locator: text_locator(&page, "兑换奖励", None, BvLocatorOperation::WaitFor, None),
        },
    ));

    for entry in &codes {
        let code = &entry.code;
        steps.push(UseRedeemCodeStep::for_code(
            code,
            "log redeem code",
            UseRedeemCodeStepAction::Log {
                message: match entry.items.as_deref() {
                    Some(items) if !items.trim().is_empty() => format!("{code} - {items}"),
                    _ => code.clone(),
                },
            },
        ));
        steps.push(UseRedeemCodeStep::for_code(
            code,
            "set clipboard to redeem code",
            UseRedeemCodeStepAction::ClipboardSet { text: code.clone() },
        ));
        steps.push(UseRedeemCodeStep::for_code(
            code,
            "click paste",
            UseRedeemCodeStepAction::Locator {
                locator: text_locator(
                    &page,
                    "粘贴",
                    Some(right_ratio_rect(capture_size, 0.5)?),
                    BvLocatorOperation::Click,
                    None,
                ),
            },
        ));
        steps.push(UseRedeemCodeStep::for_code(
            code,
            "click redeem confirm",
            UseRedeemCodeStepAction::Locator {
                locator: image_locator(
                    &page,
                    COMMON_BTN_WHITE_CONFIRM,
                    None,
                    0.8,
                    true,
                    BvLocatorOperation::Click,
                    None,
                )?,
            },
        ));
        steps.push(UseRedeemCodeStep::for_code(
            code,
            "wait for success popup",
            UseRedeemCodeStepAction::Locator {
                locator: text_locator(
                    &page,
                    "兑换成功",
                    None,
                    BvLocatorOperation::WaitFor,
                    Some(1_000),
                ),
            },
        ));
        steps.push(UseRedeemCodeStep::conditional_for_code(
            code,
            UseRedeemCodeStepCondition::WhenSuccessDetected,
            "click success confirm",
            UseRedeemCodeStepAction::Locator {
                locator: image_locator(
                    &page,
                    COMMON_BTN_BLACK_CONFIRM,
                    None,
                    0.8,
                    true,
                    BvLocatorOperation::Click,
                    None,
                )?,
            },
        ));
        steps.push(UseRedeemCodeStep::conditional_for_code(
            code,
            UseRedeemCodeStepCondition::WhenSuccessDetected,
            "wait after success",
            UseRedeemCodeStepAction::Page {
                command: task_vision_result(page.wait(5_100))?,
            },
        ));
        steps.push(UseRedeemCodeStep::conditional_for_code(
            code,
            UseRedeemCodeStepCondition::WhenSuccessNotDetected,
            "click clear after failed redeem",
            UseRedeemCodeStepAction::Locator {
                locator: text_locator(
                    &page,
                    "清除",
                    Some(right_ratio_rect(capture_size, 0.5)?),
                    BvLocatorOperation::Click,
                    None,
                ),
            },
        ));
    }

    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Cleanup,
        "clear clipboard",
        UseRedeemCodeStepAction::ClipboardClear,
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Cleanup,
        "return to main UI after redeem",
        UseRedeemCodeStepAction::CommonJob {
            task_key: "ReturnMainUi".to_string(),
        },
    ));

    Ok(UseRedeemCodeExecutionPlan {
        task_key: USE_REDEEM_CODE_TASK_KEY.to_string(),
        display_name: "Use Redeem Code".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        codes,
        steps,
        notes: "Legacy flow is represented as a BV/page/input/clipboard plan with an injectable Rust executor; desktop live wiring now provides WinRT OCR text locators, template clicks, SendInput, clipboard set/clear, and nested ReturnMainUi execution.".to_string(),
    })
}

pub fn execute_use_redeem_code_plan<R>(
    plan: &UseRedeemCodeExecutionPlan,
    runtime: &mut R,
) -> Result<UseRedeemCodeExecutionReport>
where
    R: UseRedeemCodeRuntime,
{
    let mut state = UseRedeemCodeExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        if let Some(reason) = redeem_code_skip_reason(&state, step) {
            skipped_steps.push(UseRedeemCodeSkippedStep::new(step, reason));
            continue;
        }

        let outcome = execute_use_redeem_code_step(step, runtime)?;
        apply_use_redeem_code_outcome(&mut state, step, outcome)?;
        executed_steps.push(UseRedeemCodeRuntimeStepReport::executed(step, outcome));
    }

    let completed = state.setup_return_main_ui_completed == Some(true)
        && state.cleanup_return_main_ui_completed == Some(true)
        && state.clipboard_cleared
        && state.processed_codes.len() == plan.codes.len()
        && state.failed_required_steps.is_empty();

    Ok(UseRedeemCodeExecutionReport {
        task_key: plan.task_key.clone(),
        completed,
        state,
        executed_steps,
        skipped_steps,
    })
}

fn execute_use_redeem_code_step<R>(
    step: &UseRedeemCodeStep,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: UseRedeemCodeRuntime,
{
    match &step.action {
        UseRedeemCodeStepAction::CommonJob { task_key } => {
            runtime.execute_redeem_common_job(task_key)
        }
        UseRedeemCodeStepAction::Input { events } => runtime.dispatch_input(events),
        UseRedeemCodeStepAction::Page { command } => runtime.execute_page_command(command),
        UseRedeemCodeStepAction::Locator { locator } => runtime.execute_locator(locator),
        UseRedeemCodeStepAction::ClipboardSet { text } => runtime.set_redeem_clipboard_text(text),
        UseRedeemCodeStepAction::ClipboardClear => runtime.clear_redeem_clipboard(),
        UseRedeemCodeStepAction::Log { message } => runtime.log(message),
    }
}

fn apply_use_redeem_code_outcome(
    state: &mut UseRedeemCodeExecutorState,
    step: &UseRedeemCodeStep,
    outcome: CommonJobRuntimeOutcome,
) -> Result<()> {
    match &step.action {
        UseRedeemCodeStepAction::CommonJob { .. } => {
            let completed = redeem_code_outcome_as_match(outcome, step)?;
            match step.phase {
                UseRedeemCodeStepPhase::Setup => {
                    state.setup_return_main_ui_completed = Some(completed)
                }
                UseRedeemCodeStepPhase::Cleanup => {
                    state.cleanup_return_main_ui_completed = Some(completed)
                }
                UseRedeemCodeStepPhase::PerCode => {}
            }
        }
        UseRedeemCodeStepAction::ClipboardSet { text } => {
            if redeem_code_side_effect_succeeded(outcome) {
                state.clipboard_sets.push(text.clone());
                state
                    .processed_codes
                    .push(step.code.clone().unwrap_or_else(|| text.clone()));
            } else {
                state.failed_required_steps.push(step.label.clone());
            }
        }
        UseRedeemCodeStepAction::ClipboardClear => {
            state.clipboard_cleared = redeem_code_side_effect_succeeded(outcome);
            if !state.clipboard_cleared {
                state.failed_required_steps.push(step.label.clone());
            }
        }
        UseRedeemCodeStepAction::Locator { .. } if redeem_code_is_success_probe(step) => {
            let detected = redeem_code_outcome_as_match(outcome, step)?;
            let code = step.code.clone().ok_or_else(|| {
                TaskError::CommonJobExecution(
                    "redeem success probe did not carry a redeem code".to_string(),
                )
            })?;
            let detection = UseRedeemCodeSuccessDetection {
                code: code.clone(),
                detected,
            };
            state.success_detections.push(detection.clone());
            state.last_success_detection = Some(detection);
            if detected {
                state.successful_codes.push(code);
            } else {
                state.failed_codes.push(code);
            }
        }
        _ => {
            if matches!(outcome, CommonJobRuntimeOutcome::Matched(false)) {
                state.failed_required_steps.push(step.label.clone());
            }
        }
    }

    Ok(())
}

fn redeem_code_skip_reason(
    state: &UseRedeemCodeExecutorState,
    step: &UseRedeemCodeStep,
) -> Option<UseRedeemCodeSkipReason> {
    match step.condition {
        UseRedeemCodeStepCondition::Always => None,
        UseRedeemCodeStepCondition::WhenSuccessDetected => {
            match redeem_code_last_success_detection(state, step) {
                Some(true) => None,
                Some(false) => Some(UseRedeemCodeSkipReason::SuccessMissing),
                None => Some(UseRedeemCodeSkipReason::SuccessProbeMissing),
            }
        }
        UseRedeemCodeStepCondition::WhenSuccessNotDetected => {
            match redeem_code_last_success_detection(state, step) {
                Some(false) => None,
                Some(true) => Some(UseRedeemCodeSkipReason::SuccessDetected),
                None => Some(UseRedeemCodeSkipReason::SuccessProbeMissing),
            }
        }
    }
}

fn redeem_code_last_success_detection(
    state: &UseRedeemCodeExecutorState,
    step: &UseRedeemCodeStep,
) -> Option<bool> {
    let code = step.code.as_deref()?;
    state
        .last_success_detection
        .as_ref()
        .filter(|detection| detection.code == code)
        .map(|detection| detection.detected)
}

fn redeem_code_is_success_probe(step: &UseRedeemCodeStep) -> bool {
    step.phase == UseRedeemCodeStepPhase::PerCode && step.label == "wait for success popup"
}

fn redeem_code_side_effect_succeeded(outcome: CommonJobRuntimeOutcome) -> bool {
    match outcome {
        CommonJobRuntimeOutcome::None => true,
        CommonJobRuntimeOutcome::Matched(value) => value,
    }
}

fn redeem_code_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &UseRedeemCodeStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "redeem code step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn redeem_code_action_kind(action: &UseRedeemCodeStepAction) -> UseRedeemCodeRuntimeActionKind {
    match action {
        UseRedeemCodeStepAction::CommonJob { .. } => UseRedeemCodeRuntimeActionKind::CommonJob,
        UseRedeemCodeStepAction::Input { .. } => UseRedeemCodeRuntimeActionKind::Input,
        UseRedeemCodeStepAction::Page { .. } => UseRedeemCodeRuntimeActionKind::Page,
        UseRedeemCodeStepAction::Locator { .. } => UseRedeemCodeRuntimeActionKind::Locator,
        UseRedeemCodeStepAction::ClipboardSet { .. } => {
            UseRedeemCodeRuntimeActionKind::ClipboardSet
        }
        UseRedeemCodeStepAction::ClipboardClear => UseRedeemCodeRuntimeActionKind::ClipboardClear,
        UseRedeemCodeStepAction::Log { .. } => UseRedeemCodeRuntimeActionKind::Log,
    }
}

fn task_vision_result<T>(result: bgi_vision::Result<T>) -> Result<T> {
    result.map_err(|error| TaskError::VisionPlan(error.to_string()))
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

fn text_locator(
    page: &BvPage,
    text: &str,
    roi: Option<Rect>,
    operation: BvLocatorOperation,
    timeout_ms: Option<u32>,
) -> BvLocatorPlan {
    page.locator_for_text(text, roi).plan(operation, timeout_ms)
}

fn left_ratio_rect(size: Size, ratio: f64) -> Result<Rect> {
    let width = ratio_width(size, ratio);
    task_vision_result(Rect::new(0, 0, width, size.height as i32))
}

fn right_ratio_rect(size: Size, ratio: f64) -> Result<Rect> {
    let width = ratio_width(size, ratio);
    task_vision_result(Rect::new(
        size.width as i32 - width,
        0,
        width,
        size.height as i32,
    ))
}

fn ratio_width(size: Size, ratio: f64) -> i32 {
    ((size.width as f64) * ratio).round() as i32
}
