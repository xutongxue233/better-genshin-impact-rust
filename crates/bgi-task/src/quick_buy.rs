use crate::{Result, TaskError, TaskPortState};
use bgi_input::{InputEvent, MouseButton};
use bgi_vision::{BvImage, BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const QUICK_BUY_TASK_KEY: &str = "QuickBuy";
pub const QUICK_BUY_DISPLAY_NAME: &str = "Quick Buy";
pub const QUICK_BUY_SERENITEA_POT_COIN: &str = "QuickBuy:SereniteaPotCoin.png";

const TEMPLATE_THRESHOLD: f64 = 0.8;
const SERENITEA_POT_COIN_ROI_X_1080P: f64 = 1610.0;
const SERENITEA_POT_COIN_ROI_Y_1080P: f64 = 28.0;
const SERENITEA_POT_COIN_ROI_W_1080P: f64 = 160.0;
const SERENITEA_POT_COIN_ROI_H_1080P: f64 = 45.0;
const BUY_BUTTON_RIGHT_OFFSET_1080P: f64 = 225.0;
const BUY_BUTTON_BOTTOM_OFFSET_1080P: f64 = 60.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QuickBuyExecutionConfig {
    pub capture_size: Size,
}

impl Default for QuickBuyExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
        }
    }
}

impl QuickBuyExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickBuyExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub preflight_rule: QuickBuyPreflightRule,
    pub serenitea_pot_coin_rule: QuickBuySereniteaPotCoinRule,
    pub normal_purchase_rule: QuickBuyPurchaseRule,
    pub serenitea_pot_purchase_rule: QuickBuyPurchaseRule,
    pub steps: Vec<QuickBuyStep>,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickBuyPreflightRule {
    pub requires_initialized_task_context: bool,
    pub shows_toast_when_uninitialized: bool,
    pub toast_message: String,
    pub requires_active_genshin_process: bool,
    pub inactive_process_returns_without_warning: bool,
    pub catches_and_logs_exceptions: bool,
    pub clears_vision_drawings_finally: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickBuySereniteaPotCoinRule {
    pub locator: BvLocatorPlan,
    pub use_3_channels: bool,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickBuyPurchaseRule {
    pub branch: QuickBuyBranch,
    pub drag_start: QuickBuyScreenPoint,
    pub drag_delta_x: i32,
    pub drag_delta_y: i32,
    pub wait_after_move_ms: u32,
    pub hold_before_drag_ms: u32,
    pub wait_after_drag_ms: u32,
    pub wait_after_release_ms: u32,
    pub final_clicks: Vec<QuickBuyClickTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickBuyBranch {
    NormalPurchase,
    SereniteaPotCoinPurchase,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct QuickBuyScreenPoint {
    pub x_1080p: f64,
    pub y_1080p: f64,
    pub screen_x: f64,
    pub screen_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum QuickBuyClickTarget {
    Fixed1080p(QuickBuyScreenPoint),
    BottomRightOffset {
        x_from_right_1080p: f64,
        y_from_bottom_1080p: f64,
        screen_x: f64,
        screen_y: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickBuyStep {
    pub phase: QuickBuyStepPhase,
    pub condition: QuickBuyStepCondition,
    pub label: String,
    pub action: QuickBuyStepAction,
}

impl QuickBuyStep {
    fn new(
        phase: QuickBuyStepPhase,
        condition: QuickBuyStepCondition,
        label: impl Into<String>,
        action: QuickBuyStepAction,
    ) -> Self {
        Self {
            phase,
            condition,
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickBuyStepPhase {
    Preflight,
    DetectBranch,
    NormalPurchase,
    SereniteaPotCoinPurchase,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickBuyStepCondition {
    Always,
    WhenSereniteaPotCoinDetected,
    WhenSereniteaPotCoinMissing,
    Finally,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum QuickBuyStepAction {
    Preflight { rule: QuickBuyPreflightRule },
    Locator { locator: BvLocatorPlan },
    MoveTo1080p { point: QuickBuyScreenPoint },
    Page { command: BvPageCommand },
    Input { events: Vec<InputEvent> },
    Click { target: QuickBuyClickTarget },
    ClearVisionDrawings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickBuyExecutionResult {
    Completed,
    PreflightSkipped,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct QuickBuyExecutorState {
    pub preflight_passed: bool,
    pub serenitea_pot_coin_detected: Option<bool>,
    pub selected_branch: Option<QuickBuyBranch>,
    pub moved_to_slider: bool,
    pub input_batches_dispatched: usize,
    pub clicks_dispatched: usize,
    pub vision_drawings_cleared: bool,
    pub result: Option<QuickBuyExecutionResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickBuyRuntimeActionKind {
    Preflight,
    Locator,
    MoveTo1080p,
    Page,
    Input,
    Click,
    ClearVisionDrawings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickBuySkipReason {
    PreflightSkipped,
    SereniteaPotCoinDetected,
    SereniteaPotCoinMissing,
    BranchNotDetected,
    ResultAlreadySet,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickBuyRuntimeStepReport {
    pub phase: QuickBuyStepPhase,
    pub condition: QuickBuyStepCondition,
    pub label: String,
    pub action_kind: QuickBuyRuntimeActionKind,
}

impl QuickBuyRuntimeStepReport {
    fn executed(step: &QuickBuyStep) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: quick_buy_action_kind(&step.action),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickBuySkippedStep {
    pub phase: QuickBuyStepPhase,
    pub condition: QuickBuyStepCondition,
    pub label: String,
    pub reason: QuickBuySkipReason,
}

impl QuickBuySkippedStep {
    fn new(step: &QuickBuyStep, reason: QuickBuySkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickBuyExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: QuickBuyExecutorState,
    pub executed_steps: Vec<QuickBuyRuntimeStepReport>,
    pub skipped_steps: Vec<QuickBuySkippedStep>,
}

pub trait QuickBuyRuntime {
    fn quick_buy_preflight(&mut self, rule: &QuickBuyPreflightRule) -> Result<bool>;

    fn locate_quick_buy_template(&mut self, locator: &BvLocatorPlan) -> Result<bool>;

    fn move_quick_buy_cursor(&mut self, point: &QuickBuyScreenPoint) -> Result<()>;

    fn execute_quick_buy_page_command(&mut self, command: &BvPageCommand) -> Result<()>;

    fn dispatch_quick_buy_input(&mut self, events: &[InputEvent]) -> Result<()>;

    fn click_quick_buy_target(&mut self, target: &QuickBuyClickTarget) -> Result<()>;

    fn clear_quick_buy_vision_drawings(&mut self) -> Result<()>;
}

pub fn execute_quick_buy_plan<R>(
    plan: &QuickBuyExecutionPlan,
    runtime: &mut R,
) -> Result<QuickBuyExecutionReport>
where
    R: QuickBuyRuntime,
{
    let mut state = QuickBuyExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_quick_buy_step(step, &state) {
            Ok(()) => {
                execute_quick_buy_step(step, runtime, &mut state)?;
                executed_steps.push(QuickBuyRuntimeStepReport::executed(step));
            }
            Err(reason) => skipped_steps.push(QuickBuySkippedStep::new(step, reason)),
        }
    }

    if state.result.is_none() && state.preflight_passed {
        state.result = Some(QuickBuyExecutionResult::Completed);
    }

    Ok(QuickBuyExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result == Some(QuickBuyExecutionResult::Completed),
        state,
        executed_steps,
        skipped_steps,
    })
}

pub fn plan_quick_buy(config: QuickBuyExecutionConfig) -> Result<QuickBuyExecutionPlan> {
    let page = BvPage {
        capture_size: config.capture_size,
        ..BvPage::default()
    };
    let coin_roi = scaled_rect(
        config.capture_size,
        SERENITEA_POT_COIN_ROI_X_1080P,
        SERENITEA_POT_COIN_ROI_Y_1080P,
        SERENITEA_POT_COIN_ROI_W_1080P,
        SERENITEA_POT_COIN_ROI_H_1080P,
    )?;
    let mut coin_locator = image_locator(
        &page,
        QUICK_BUY_SERENITEA_POT_COIN,
        Some(coin_roi),
        TEMPLATE_THRESHOLD,
        BvLocatorOperation::IsExist,
        Some(1),
    )?;
    coin_locator.recognition_object.template.use_3_channels = true;
    coin_locator.recognition_object.template.draw_on_window = true;
    let preflight_rule = QuickBuyPreflightRule {
        requires_initialized_task_context: true,
        shows_toast_when_uninitialized: true,
        toast_message: "请先启动".to_string(),
        requires_active_genshin_process: true,
        inactive_process_returns_without_warning: true,
        catches_and_logs_exceptions: true,
        clears_vision_drawings_finally: true,
    };
    let serenitea_pot_coin_rule = QuickBuySereniteaPotCoinRule {
        locator: coin_locator.clone(),
        use_3_channels: true,
        draw_on_window: true,
    };
    let normal_purchase_rule = QuickBuyPurchaseRule {
        branch: QuickBuyBranch::NormalPurchase,
        drag_start: screen_point(config.capture_size, 742.0, 601.0),
        drag_delta_x: 1000,
        drag_delta_y: 0,
        wait_after_move_ms: 100,
        hold_before_drag_ms: 50,
        wait_after_drag_ms: 200,
        wait_after_release_ms: 100,
        final_clicks: vec![
            fixed_bottom_right_click(config.capture_size),
            QuickBuyClickTarget::Fixed1080p(screen_point(config.capture_size, 1100.0, 780.0)),
            fixed_bottom_right_click(config.capture_size),
        ],
    };
    let serenitea_pot_purchase_rule = QuickBuyPurchaseRule {
        branch: QuickBuyBranch::SereniteaPotCoinPurchase,
        drag_start: screen_point(config.capture_size, 1450.0, 690.0),
        drag_delta_x: 1000,
        drag_delta_y: 0,
        wait_after_move_ms: 100,
        hold_before_drag_ms: 50,
        wait_after_drag_ms: 200,
        wait_after_release_ms: 200,
        final_clicks: vec![
            QuickBuyClickTarget::Fixed1080p(screen_point(config.capture_size, 1600.0, 1020.0)),
            QuickBuyClickTarget::Fixed1080p(screen_point(config.capture_size, 960.0, 850.0)),
        ],
    };
    let mut steps = vec![
        QuickBuyStep::new(
            QuickBuyStepPhase::Preflight,
            QuickBuyStepCondition::Always,
            "check task context and active Genshin process",
            QuickBuyStepAction::Preflight {
                rule: preflight_rule.clone(),
            },
        ),
        QuickBuyStep::new(
            QuickBuyStepPhase::DetectBranch,
            QuickBuyStepCondition::Always,
            "detect Serenitea Pot coin branch",
            QuickBuyStepAction::Locator {
                locator: coin_locator.clone(),
            },
        ),
    ];
    steps.extend(purchase_steps(
        QuickBuyStepPhase::SereniteaPotCoinPurchase,
        QuickBuyStepCondition::WhenSereniteaPotCoinDetected,
        &serenitea_pot_purchase_rule,
        &page,
    )?);
    steps.extend(purchase_steps(
        QuickBuyStepPhase::NormalPurchase,
        QuickBuyStepCondition::WhenSereniteaPotCoinMissing,
        &normal_purchase_rule,
        &page,
    )?);
    steps.push(QuickBuyStep::new(
        QuickBuyStepPhase::Cleanup,
        QuickBuyStepCondition::Finally,
        "clear vision drawings",
        QuickBuyStepAction::ClearVisionDrawings,
    ));

    Ok(QuickBuyExecutionPlan {
        task_key: QUICK_BUY_TASK_KEY.to_string(),
        display_name: QUICK_BUY_DISPLAY_NAME.to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size: config.capture_size,
        preflight_rule,
        serenitea_pot_coin_rule,
        normal_purchase_rule,
        serenitea_pot_purchase_rule,
        steps,
        pending_native: vec![
            "live TaskContext/SystemControl preflight and Toast warning dispatch".to_string(),
            "live capture and Serenitea Pot coin template matching adapter".to_string(),
            "live GameCaptureRegion desktop coordinate mapping and SendInput mouse dispatch adapter"
                .to_string(),
            "live Vision overlay cleanup adapter".to_string(),
        ],
    })
}

fn should_execute_quick_buy_step(
    step: &QuickBuyStep,
    state: &QuickBuyExecutorState,
) -> std::result::Result<(), QuickBuySkipReason> {
    if step.condition == QuickBuyStepCondition::Finally {
        return if state.preflight_passed {
            Ok(())
        } else {
            Err(QuickBuySkipReason::PreflightSkipped)
        };
    }
    if state.result.is_some() {
        return Err(QuickBuySkipReason::ResultAlreadySet);
    }

    match step.condition {
        QuickBuyStepCondition::Always => {
            if step.phase == QuickBuyStepPhase::Preflight || state.preflight_passed {
                Ok(())
            } else {
                Err(QuickBuySkipReason::PreflightSkipped)
            }
        }
        QuickBuyStepCondition::WhenSereniteaPotCoinDetected => {
            match state.serenitea_pot_coin_detected {
                Some(true) => Ok(()),
                Some(false) => Err(QuickBuySkipReason::SereniteaPotCoinMissing),
                None => Err(QuickBuySkipReason::BranchNotDetected),
            }
        }
        QuickBuyStepCondition::WhenSereniteaPotCoinMissing => {
            match state.serenitea_pot_coin_detected {
                Some(false) => Ok(()),
                Some(true) => Err(QuickBuySkipReason::SereniteaPotCoinDetected),
                None => Err(QuickBuySkipReason::BranchNotDetected),
            }
        }
        QuickBuyStepCondition::Finally => Ok(()),
    }
}

fn execute_quick_buy_step<R>(
    step: &QuickBuyStep,
    runtime: &mut R,
    state: &mut QuickBuyExecutorState,
) -> Result<()>
where
    R: QuickBuyRuntime,
{
    match &step.action {
        QuickBuyStepAction::Preflight { rule } => {
            let passed = runtime.quick_buy_preflight(rule)?;
            state.preflight_passed = passed;
            if !passed {
                state.result = Some(QuickBuyExecutionResult::PreflightSkipped);
            }
        }
        QuickBuyStepAction::Locator { locator } => {
            let detected = runtime.locate_quick_buy_template(locator)?;
            state.serenitea_pot_coin_detected = Some(detected);
            state.selected_branch = Some(if detected {
                QuickBuyBranch::SereniteaPotCoinPurchase
            } else {
                QuickBuyBranch::NormalPurchase
            });
        }
        QuickBuyStepAction::MoveTo1080p { point } => {
            runtime.move_quick_buy_cursor(point)?;
            state.moved_to_slider = true;
        }
        QuickBuyStepAction::Page { command } => {
            runtime.execute_quick_buy_page_command(command)?;
        }
        QuickBuyStepAction::Input { events } => {
            runtime.dispatch_quick_buy_input(events)?;
            state.input_batches_dispatched += 1;
        }
        QuickBuyStepAction::Click { target } => {
            runtime.click_quick_buy_target(target)?;
            state.clicks_dispatched += 1;
        }
        QuickBuyStepAction::ClearVisionDrawings => {
            runtime.clear_quick_buy_vision_drawings()?;
            state.vision_drawings_cleared = true;
        }
    }
    Ok(())
}

fn quick_buy_action_kind(action: &QuickBuyStepAction) -> QuickBuyRuntimeActionKind {
    match action {
        QuickBuyStepAction::Preflight { .. } => QuickBuyRuntimeActionKind::Preflight,
        QuickBuyStepAction::Locator { .. } => QuickBuyRuntimeActionKind::Locator,
        QuickBuyStepAction::MoveTo1080p { .. } => QuickBuyRuntimeActionKind::MoveTo1080p,
        QuickBuyStepAction::Page { .. } => QuickBuyRuntimeActionKind::Page,
        QuickBuyStepAction::Input { .. } => QuickBuyRuntimeActionKind::Input,
        QuickBuyStepAction::Click { .. } => QuickBuyRuntimeActionKind::Click,
        QuickBuyStepAction::ClearVisionDrawings => QuickBuyRuntimeActionKind::ClearVisionDrawings,
    }
}

fn purchase_steps(
    phase: QuickBuyStepPhase,
    condition: QuickBuyStepCondition,
    rule: &QuickBuyPurchaseRule,
    page: &BvPage,
) -> Result<Vec<QuickBuyStep>> {
    let mut steps = Vec::new();
    if rule.branch == QuickBuyBranch::NormalPurchase {
        steps.push(QuickBuyStep::new(
            phase,
            condition,
            "click purchase or exchange button",
            QuickBuyStepAction::Click {
                target: rule.final_clicks[0],
            },
        ));
        steps.push(QuickBuyStep::new(
            phase,
            condition,
            "wait for purchase dialog",
            QuickBuyStepAction::Page {
                command: page_wait(page, 100)?,
            },
        ));
    }
    steps.push(QuickBuyStep::new(
        phase,
        condition,
        "move to quantity slider handle",
        QuickBuyStepAction::MoveTo1080p {
            point: rule.drag_start,
        },
    ));
    steps.push(QuickBuyStep::new(
        phase,
        condition,
        "wait after mouse move",
        QuickBuyStepAction::Page {
            command: page_wait(page, rule.wait_after_move_ms)?,
        },
    ));
    steps.push(QuickBuyStep::new(
        phase,
        condition,
        "hold left mouse button",
        QuickBuyStepAction::Input {
            events: vec![InputEvent::MouseButtonDown {
                button: MouseButton::Left,
            }],
        },
    ));
    steps.push(QuickBuyStep::new(
        phase,
        condition,
        "wait before slider drag",
        QuickBuyStepAction::Page {
            command: page_wait(page, rule.hold_before_drag_ms)?,
        },
    ));
    steps.push(QuickBuyStep::new(
        phase,
        condition,
        "drag quantity slider to maximum",
        QuickBuyStepAction::Input {
            events: vec![InputEvent::MouseMoveRelative {
                dx: rule.drag_delta_x,
                dy: rule.drag_delta_y,
            }],
        },
    ));
    steps.push(QuickBuyStep::new(
        phase,
        condition,
        "wait after slider drag",
        QuickBuyStepAction::Page {
            command: page_wait(page, rule.wait_after_drag_ms)?,
        },
    ));
    steps.push(QuickBuyStep::new(
        phase,
        condition,
        "release left mouse button",
        QuickBuyStepAction::Input {
            events: vec![InputEvent::MouseButtonUp {
                button: MouseButton::Left,
            }],
        },
    ));
    steps.push(QuickBuyStep::new(
        phase,
        condition,
        "wait after slider release",
        QuickBuyStepAction::Page {
            command: page_wait(page, rule.wait_after_release_ms)?,
        },
    ));
    let final_clicks = if rule.branch == QuickBuyBranch::NormalPurchase {
        &rule.final_clicks[1..]
    } else {
        &rule.final_clicks[..]
    };
    for (index, target) in final_clicks.iter().enumerate() {
        steps.push(QuickBuyStep::new(
            phase,
            condition,
            format!("click purchase confirmation {}", index + 1),
            QuickBuyStepAction::Click { target: *target },
        ));
        if rule.branch == QuickBuyBranch::NormalPurchase || index == 0 {
            steps.push(QuickBuyStep::new(
                phase,
                condition,
                "wait after confirmation click",
                QuickBuyStepAction::Page {
                    command: page_wait(page, 200)?,
                },
            ));
        }
    }
    Ok(steps)
}

fn fixed_bottom_right_click(capture_size: Size) -> QuickBuyClickTarget {
    let scale = scale(capture_size);
    QuickBuyClickTarget::BottomRightOffset {
        x_from_right_1080p: BUY_BUTTON_RIGHT_OFFSET_1080P,
        y_from_bottom_1080p: BUY_BUTTON_BOTTOM_OFFSET_1080P,
        screen_x: capture_size.width as f64 - BUY_BUTTON_RIGHT_OFFSET_1080P * scale,
        screen_y: capture_size.height as f64 - BUY_BUTTON_BOTTOM_OFFSET_1080P * scale,
    }
}

fn page_wait(page: &BvPage, milliseconds: u32) -> Result<BvPageCommand> {
    page.wait(milliseconds)
        .map_err(|error| TaskError::VisionPlan(error.to_string()))
}

fn image_locator(
    page: &BvPage,
    asset: &str,
    roi: Option<Rect>,
    threshold: f64,
    operation: BvLocatorOperation,
    timeout_ms: Option<u32>,
) -> Result<BvLocatorPlan> {
    let image = BvImage::new(asset).map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let locator = page
        .locator_for_image(&image, roi, threshold)
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    Ok(locator.plan(operation, timeout_ms))
}

fn scaled_rect(size: Size, x: f64, y: f64, width: f64, height: f64) -> Result<Rect> {
    let scale = scale(size);
    Rect::new(
        (x * scale).round() as i32,
        (y * scale).round() as i32,
        (width * scale).round() as i32,
        (height * scale).round() as i32,
    )
    .map_err(|error| TaskError::VisionPlan(error.to_string()))
}

fn screen_point(size: Size, x_1080p: f64, y_1080p: f64) -> QuickBuyScreenPoint {
    let scale = scale(size);
    QuickBuyScreenPoint {
        x_1080p,
        y_1080p,
        screen_x: x_1080p * scale,
        screen_y: y_1080p * scale,
    }
}

fn scale(size: Size) -> f64 {
    size.width as f64 / 1920.0
}
