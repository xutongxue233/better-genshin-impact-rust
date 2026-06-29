use bgi_core::MacroConfig;
use bgi_vision::Size;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Result, TaskPortState};

pub const TURN_AROUND_MACRO_TASK_KEY: &str = "TurnAroundMacro";
pub const QUICK_ENHANCE_ARTIFACT_MACRO_TASK_KEY: &str = "QuickEnhanceArtifactMacro";
pub const MACRO_HOTKEY_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const MACRO_HOTKEY_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MacroHotkeyExecutionConfig {
    pub capture_size: Size,
    pub macro_config: MacroConfig,
}

impl Default for MacroHotkeyExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                MACRO_HOTKEY_DEFAULT_CAPTURE_WIDTH,
                MACRO_HOTKEY_DEFAULT_CAPTURE_HEIGHT,
            ),
            macro_config: MacroConfig::default(),
        }
    }
}

impl MacroHotkeyExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };

        if let Some(capture_size) = macro_hotkey_capture_size_from_value(value) {
            config.capture_size = capture_size;
        }

        let macro_value = value
            .get("macroConfig")
            .or_else(|| value.get("MacroConfig"))
            .or_else(|| value.get("macro_config"))
            .unwrap_or(value);
        config.macro_config = serde_json::from_value(macro_value.clone()).unwrap_or_default();
        config
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MacroHotkeyExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub kind: MacroHotkeyKind,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub config_rule: MacroHotkeyConfigRule,
    pub preflight_rule: Option<MacroHotkeyPreflightRule>,
    pub steps: Vec<MacroHotkeyStep>,
    pub pending_native: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroHotkeyKind {
    TurnAround,
    QuickEnhanceArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroHotkeyConfigRule {
    pub enhance_wait_delay_ms: u64,
    pub runaround_interval_ms: u64,
    pub original_runaround_mouse_x_interval: i64,
    pub effective_runaround_mouse_x_interval: i64,
    pub zero_runaround_mouse_x_is_rewritten_to_one: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroHotkeyPreflightRule {
    pub requires_initialized_task_context: bool,
    pub uninitialized_toast_message: String,
    pub uninitialized_returns_without_actions: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MacroHotkeyStep {
    pub phase: MacroHotkeyStepPhase,
    pub condition: MacroHotkeyStepCondition,
    pub label: String,
    pub action: MacroHotkeyStepAction,
}

impl MacroHotkeyStep {
    fn new(
        phase: MacroHotkeyStepPhase,
        condition: MacroHotkeyStepCondition,
        label: impl Into<String>,
        action: MacroHotkeyStepAction,
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
pub enum MacroHotkeyStepPhase {
    Preflight,
    TurnAround,
    EnhanceArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroHotkeyStepCondition {
    Always,
    WhenInitialized,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum MacroHotkeyStepAction {
    Preflight { rule: MacroHotkeyPreflightRule },
    MoveMouseBy { dx: i64, dy: i64 },
    ClickCapturePoint { point: MacroHotkeyScreenPoint },
    MoveCapturePoint { point: MacroHotkeyScreenPoint },
    Delay { delay_ms: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MacroHotkeyScreenPoint {
    pub x_1080p: f64,
    pub y_1080p: f64,
    pub screen_x: f64,
    pub screen_y: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroHotkeyExecutorState {
    pub preflight_passed: bool,
    pub input_actions_dispatched: usize,
    pub waits_dispatched: usize,
    pub result: Option<MacroHotkeyExecutionResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroHotkeyExecutionResult {
    Completed,
    PreflightSkipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroHotkeyRuntimeActionKind {
    Preflight,
    MoveMouseBy,
    ClickCapturePoint,
    MoveCapturePoint,
    Delay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroHotkeySkipReason {
    PreflightSkipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroHotkeyRuntimeStepReport {
    pub phase: MacroHotkeyStepPhase,
    pub condition: MacroHotkeyStepCondition,
    pub label: String,
    pub action_kind: MacroHotkeyRuntimeActionKind,
}

impl MacroHotkeyRuntimeStepReport {
    fn executed(step: &MacroHotkeyStep) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: macro_hotkey_action_kind(&step.action),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroHotkeySkippedStep {
    pub phase: MacroHotkeyStepPhase,
    pub condition: MacroHotkeyStepCondition,
    pub label: String,
    pub reason: MacroHotkeySkipReason,
}

impl MacroHotkeySkippedStep {
    fn new(step: &MacroHotkeyStep, reason: MacroHotkeySkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroHotkeyExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: MacroHotkeyExecutorState,
    pub executed_steps: Vec<MacroHotkeyRuntimeStepReport>,
    pub skipped_steps: Vec<MacroHotkeySkippedStep>,
}

pub trait MacroHotkeyRuntime {
    fn macro_hotkey_preflight(&mut self, rule: &MacroHotkeyPreflightRule) -> Result<bool>;

    fn move_macro_hotkey_mouse_by(&mut self, dx: i64, dy: i64) -> Result<()>;

    fn click_macro_hotkey_capture_point(&mut self, point: &MacroHotkeyScreenPoint) -> Result<()>;

    fn move_macro_hotkey_capture_point(&mut self, point: &MacroHotkeyScreenPoint) -> Result<()>;

    fn wait_macro_hotkey(&mut self, delay_ms: u64) -> Result<()>;
}

pub fn plan_turn_around_macro(config: MacroHotkeyExecutionConfig) -> MacroHotkeyExecutionPlan {
    let config_rule = macro_hotkey_config_rule(&config.macro_config);
    let steps = vec![
        MacroHotkeyStep::new(
            MacroHotkeyStepPhase::TurnAround,
            MacroHotkeyStepCondition::Always,
            "move mouse by configured horizontal interval",
            MacroHotkeyStepAction::MoveMouseBy {
                dx: config_rule.effective_runaround_mouse_x_interval,
                dy: 0,
            },
        ),
        MacroHotkeyStep::new(
            MacroHotkeyStepPhase::TurnAround,
            MacroHotkeyStepCondition::Always,
            "wait after turning around",
            MacroHotkeyStepAction::Delay {
                delay_ms: config_rule.runaround_interval_ms,
            },
        ),
    ];

    MacroHotkeyExecutionPlan {
        task_key: TURN_AROUND_MACRO_TASK_KEY.to_string(),
        display_name: "Turn Around Macro".to_string(),
        kind: MacroHotkeyKind::TurnAround,
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size: config.capture_size,
        config_rule,
        preflight_rule: None,
        steps,
        pending_native: vec![
            "desktop hotkey route still calls the legacy macro path until the desktop hotkey bridge consumes this Rust plan".to_string(),
        ],
        notes: "Rust preserves the legacy TurnAroundMacro behavior: rewrite a zero runaroundMouseXInterval to one, move mouse by the effective horizontal interval, then wait runaroundInterval milliseconds.".to_string(),
    }
}

pub fn plan_quick_enhance_artifact_macro(
    config: MacroHotkeyExecutionConfig,
) -> MacroHotkeyExecutionPlan {
    let config_rule = macro_hotkey_config_rule(&config.macro_config);
    let preflight_rule = MacroHotkeyPreflightRule {
        requires_initialized_task_context: true,
        uninitialized_toast_message: "请先启动".to_string(),
        uninitialized_returns_without_actions: true,
    };
    let capture_size = config.capture_size;
    let steps = vec![
        MacroHotkeyStep::new(
            MacroHotkeyStepPhase::Preflight,
            MacroHotkeyStepCondition::Always,
            "require initialized task context",
            MacroHotkeyStepAction::Preflight {
                rule: preflight_rule.clone(),
            },
        ),
        MacroHotkeyStep::new(
            MacroHotkeyStepPhase::EnhanceArtifact,
            MacroHotkeyStepCondition::WhenInitialized,
            "click quick add artifacts",
            MacroHotkeyStepAction::ClickCapturePoint {
                point: macro_hotkey_point(1760.0, 770.0, capture_size),
            },
        ),
        MacroHotkeyStep::new(
            MacroHotkeyStepPhase::EnhanceArtifact,
            MacroHotkeyStepCondition::WhenInitialized,
            "wait after quick add",
            MacroHotkeyStepAction::Delay { delay_ms: 100 },
        ),
        MacroHotkeyStep::new(
            MacroHotkeyStepPhase::EnhanceArtifact,
            MacroHotkeyStepCondition::WhenInitialized,
            "click enhance",
            MacroHotkeyStepAction::ClickCapturePoint {
                point: macro_hotkey_point(1760.0, 1020.0, capture_size),
            },
        ),
        MacroHotkeyStep::new(
            MacroHotkeyStepPhase::EnhanceArtifact,
            MacroHotkeyStepCondition::WhenInitialized,
            "wait for enhance animation",
            MacroHotkeyStepAction::Delay {
                delay_ms: 100 + config_rule.enhance_wait_delay_ms,
            },
        ),
        MacroHotkeyStep::new(
            MacroHotkeyStepPhase::EnhanceArtifact,
            MacroHotkeyStepCondition::WhenInitialized,
            "click artifact details menu",
            MacroHotkeyStepAction::ClickCapturePoint {
                point: macro_hotkey_point(150.0, 150.0, capture_size),
            },
        ),
        MacroHotkeyStep::new(
            MacroHotkeyStepPhase::EnhanceArtifact,
            MacroHotkeyStepCondition::WhenInitialized,
            "wait after details menu",
            MacroHotkeyStepAction::Delay { delay_ms: 100 },
        ),
        MacroHotkeyStep::new(
            MacroHotkeyStepPhase::EnhanceArtifact,
            MacroHotkeyStepCondition::WhenInitialized,
            "click enhance menu",
            MacroHotkeyStepAction::ClickCapturePoint {
                point: macro_hotkey_point(150.0, 220.0, capture_size),
            },
        ),
        MacroHotkeyStep::new(
            MacroHotkeyStepPhase::EnhanceArtifact,
            MacroHotkeyStepCondition::WhenInitialized,
            "wait after enhance menu",
            MacroHotkeyStepAction::Delay { delay_ms: 100 },
        ),
        MacroHotkeyStep::new(
            MacroHotkeyStepPhase::EnhanceArtifact,
            MacroHotkeyStepCondition::WhenInitialized,
            "move back to quick add artifacts",
            MacroHotkeyStepAction::MoveCapturePoint {
                point: macro_hotkey_point(1760.0, 770.0, capture_size),
            },
        ),
    ];

    MacroHotkeyExecutionPlan {
        task_key: QUICK_ENHANCE_ARTIFACT_MACRO_TASK_KEY.to_string(),
        display_name: "Quick Enhance Artifact Macro".to_string(),
        kind: MacroHotkeyKind::QuickEnhanceArtifact,
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        config_rule,
        preflight_rule: Some(preflight_rule),
        steps,
        pending_native: vec![
            "desktop hotkey route still calls the legacy macro path until the desktop hotkey bridge consumes this Rust plan".to_string(),
            "real TaskContext initialization and toast adapters remain desktop-side work".to_string(),
        ],
        notes: "Rust preserves QuickEnhanceArtifactMacro's initialized-context guard, fixed 1080p click chain, enhanceWaitDelay addition, and final cursor move back to the quick-add button.".to_string(),
    }
}

pub fn execute_macro_hotkey_plan<R>(
    plan: &MacroHotkeyExecutionPlan,
    runtime: &mut R,
) -> Result<MacroHotkeyExecutionReport>
where
    R: MacroHotkeyRuntime,
{
    let mut state = MacroHotkeyExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        if step.condition == MacroHotkeyStepCondition::WhenInitialized && !state.preflight_passed {
            skipped_steps.push(MacroHotkeySkippedStep::new(
                step,
                MacroHotkeySkipReason::PreflightSkipped,
            ));
            continue;
        }

        match &step.action {
            MacroHotkeyStepAction::Preflight { rule } => {
                let passed = runtime.macro_hotkey_preflight(rule)?;
                state.preflight_passed = passed;
                executed_steps.push(MacroHotkeyRuntimeStepReport::executed(step));
                if !passed {
                    state.result = Some(MacroHotkeyExecutionResult::PreflightSkipped);
                }
            }
            MacroHotkeyStepAction::MoveMouseBy { dx, dy } => {
                runtime.move_macro_hotkey_mouse_by(*dx, *dy)?;
                state.input_actions_dispatched += 1;
                state.preflight_passed = true;
                executed_steps.push(MacroHotkeyRuntimeStepReport::executed(step));
            }
            MacroHotkeyStepAction::ClickCapturePoint { point } => {
                runtime.click_macro_hotkey_capture_point(point)?;
                state.input_actions_dispatched += 1;
                executed_steps.push(MacroHotkeyRuntimeStepReport::executed(step));
            }
            MacroHotkeyStepAction::MoveCapturePoint { point } => {
                runtime.move_macro_hotkey_capture_point(point)?;
                state.input_actions_dispatched += 1;
                executed_steps.push(MacroHotkeyRuntimeStepReport::executed(step));
            }
            MacroHotkeyStepAction::Delay { delay_ms } => {
                runtime.wait_macro_hotkey(*delay_ms)?;
                state.waits_dispatched += 1;
                executed_steps.push(MacroHotkeyRuntimeStepReport::executed(step));
            }
        }
    }

    let completed = state.result != Some(MacroHotkeyExecutionResult::PreflightSkipped);
    if completed {
        state.result = Some(MacroHotkeyExecutionResult::Completed);
    }

    Ok(MacroHotkeyExecutionReport {
        task_key: plan.task_key.clone(),
        completed,
        state,
        executed_steps,
        skipped_steps,
    })
}

fn macro_hotkey_config_rule(config: &MacroConfig) -> MacroHotkeyConfigRule {
    let effective_runaround_mouse_x_interval = if config.runaround_mouse_x_interval == 0 {
        1
    } else {
        config.runaround_mouse_x_interval
    };
    MacroHotkeyConfigRule {
        enhance_wait_delay_ms: config.enhance_wait_delay,
        runaround_interval_ms: config.runaround_interval,
        original_runaround_mouse_x_interval: config.runaround_mouse_x_interval,
        effective_runaround_mouse_x_interval,
        zero_runaround_mouse_x_is_rewritten_to_one: config.runaround_mouse_x_interval == 0,
    }
}

fn macro_hotkey_point(x_1080p: f64, y_1080p: f64, capture_size: Size) -> MacroHotkeyScreenPoint {
    MacroHotkeyScreenPoint {
        x_1080p,
        y_1080p,
        screen_x: x_1080p * capture_size.width as f64 / MACRO_HOTKEY_DEFAULT_CAPTURE_WIDTH as f64,
        screen_y: y_1080p * capture_size.height as f64 / MACRO_HOTKEY_DEFAULT_CAPTURE_HEIGHT as f64,
    }
}

fn macro_hotkey_action_kind(action: &MacroHotkeyStepAction) -> MacroHotkeyRuntimeActionKind {
    match action {
        MacroHotkeyStepAction::Preflight { .. } => MacroHotkeyRuntimeActionKind::Preflight,
        MacroHotkeyStepAction::MoveMouseBy { .. } => MacroHotkeyRuntimeActionKind::MoveMouseBy,
        MacroHotkeyStepAction::ClickCapturePoint { .. } => {
            MacroHotkeyRuntimeActionKind::ClickCapturePoint
        }
        MacroHotkeyStepAction::MoveCapturePoint { .. } => {
            MacroHotkeyRuntimeActionKind::MoveCapturePoint
        }
        MacroHotkeyStepAction::Delay { .. } => MacroHotkeyRuntimeActionKind::Delay,
    }
}

fn macro_hotkey_capture_size_from_value(value: &Value) -> Option<Size> {
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
