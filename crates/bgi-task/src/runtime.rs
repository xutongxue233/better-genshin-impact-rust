use crate::{Result, TaskError};
use bgi_core::{initial_triggers, GameUiCategory, TriggerDescriptor};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[path = "runtime_independent.rs"]
mod runtime_independent;
#[path = "runtime_invocation.rs"]
mod runtime_invocation;

pub use runtime_independent::*;
pub use runtime_invocation::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskRuntimeState {
    Stopped,
    Starting,
    Running,
    Suspended,
    Stopping,
    Faulted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DispatcherCaptureMode {
    NormalTrigger,
    OnlyCacheCapture,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskCommandKind {
    Start,
    Stop,
    Pause,
    Resume,
    Cancel,
    ReloadAssets,
    TakeScreenshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskCommand {
    pub kind: TaskCommandKind,
    pub target: Option<String>,
}

impl TaskCommand {
    pub fn new(kind: TaskCommandKind) -> Self {
        Self { kind, target: None }
    }

    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerRuntimeConfig {
    pub key: String,
    pub enabled: bool,
}

impl TriggerRuntimeConfig {
    pub fn from_descriptor(trigger: &TriggerDescriptor, all_enabled: bool) -> Self {
        Self {
            key: trigger.key.to_string(),
            enabled: all_enabled || trigger.default_enabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunnableTrigger {
    pub descriptor: TriggerDescriptor,
    pub enabled: bool,
}

impl RunnableTrigger {
    pub fn is_exclusive(&self) -> bool {
        self.enabled && self.descriptor.exclusive
    }

    pub fn can_run_in_background(&self) -> bool {
        self.enabled && self.descriptor.background
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredRealtimeTrigger {
    pub task_key: String,
    pub interval_ms: u64,
    pub config: Option<Value>,
    pub registered_at_frame: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatcherRuntime {
    pub state: TaskRuntimeState,
    pub interval_ms: u64,
    pub frame_index: u64,
    pub capture_mode: DispatcherCaptureMode,
    pub previous_ui: GameUiCategory,
    pub current_ui: GameUiCategory,
    pub ui_grace_period_ms: u64,
    pub game_active: bool,
    pub game_minimized: bool,
    pub picture_in_picture: bool,
    pub registered_realtime_triggers: Vec<RegisteredRealtimeTrigger>,
}

impl Default for DispatcherRuntime {
    fn default() -> Self {
        Self {
            state: TaskRuntimeState::Stopped,
            interval_ms: 50,
            frame_index: 0,
            capture_mode: DispatcherCaptureMode::NormalTrigger,
            previous_ui: GameUiCategory::Unknown,
            current_ui: GameUiCategory::Unknown,
            ui_grace_period_ms: 30_000,
            game_active: true,
            game_minimized: false,
            picture_in_picture: false,
            registered_realtime_triggers: Vec::new(),
        }
    }
}

impl DispatcherRuntime {
    pub fn advance_frame(&mut self, max_frame_index_second: u64) {
        let max_frame_index = (max_frame_index_second * 1000 / self.interval_ms.max(1)).max(1);
        self.frame_index = (self.frame_index + 1) % max_frame_index;
    }

    pub fn update_ui(&mut self, ui: GameUiCategory) -> bool {
        let changed = self.current_ui != ui;
        self.previous_ui = self.current_ui;
        self.current_ui = ui;
        changed
    }

    pub fn clear_registered_realtime_triggers(&mut self) -> usize {
        let count = self.registered_realtime_triggers.len();
        self.registered_realtime_triggers.clear();
        count
    }

    pub fn add_registered_realtime_trigger(&mut self, plan: &TaskInvocationPlan) -> Result<()> {
        if plan.kind != TaskInvocationKind::AddRealtimeTrigger {
            return Err(TaskError::InvalidInvocationKind {
                expected: TaskInvocationKind::AddRealtimeTrigger,
                actual: plan.kind,
            });
        }
        if plan.clears_existing_triggers {
            self.clear_registered_realtime_triggers();
        }
        let task_key = plan.task_key.clone().ok_or(TaskError::MissingTaskName)?;
        self.registered_realtime_triggers
            .retain(|trigger| !trigger.task_key.eq_ignore_ascii_case(&task_key));
        self.registered_realtime_triggers
            .push(RegisteredRealtimeTrigger {
                task_key,
                interval_ms: plan.interval_ms.unwrap_or(self.interval_ms),
                config: plan.config.clone(),
                registered_at_frame: self.frame_index,
            });
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerRuntime {
    pub state: TaskRuntimeState,
    pub current_task: Option<String>,
    pub continuous_run_group: bool,
    pub pre_execution: bool,
    pub suspended: bool,
    pub auto_pick_pause_count: u32,
    pub party_name: Option<String>,
}

impl Default for RunnerRuntime {
    fn default() -> Self {
        Self {
            state: TaskRuntimeState::Stopped,
            current_task: None,
            continuous_run_group: false,
            pre_execution: false,
            suspended: false,
            auto_pick_pause_count: 0,
            party_name: None,
        }
    }
}

impl RunnerRuntime {
    pub fn start_task(&mut self, task: impl Into<String>) -> Result<()> {
        if matches!(
            self.state,
            TaskRuntimeState::Running | TaskRuntimeState::Suspended
        ) {
            return Err(TaskError::TaskAlreadyRunning(
                self.current_task.clone().unwrap_or_default(),
            ));
        }
        self.current_task = Some(task.into());
        self.state = TaskRuntimeState::Running;
        Ok(())
    }

    pub fn stop_task(&mut self) {
        self.current_task = None;
        self.state = TaskRuntimeState::Stopped;
        self.suspended = false;
        if !self.continuous_run_group {
            self.party_name = None;
        }
    }

    pub fn suspend(&mut self) {
        if self.state == TaskRuntimeState::Running {
            self.state = TaskRuntimeState::Suspended;
        }
        self.suspended = true;
    }

    pub fn resume(&mut self) {
        if self.state == TaskRuntimeState::Suspended {
            self.state = TaskRuntimeState::Running;
        }
        self.suspended = false;
    }

    pub fn stop_auto_pick(&mut self) {
        self.auto_pick_pause_count += 1;
    }

    pub fn resume_auto_pick(&mut self) {
        self.auto_pick_pause_count = self.auto_pick_pause_count.saturating_sub(1);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskRuntimeSnapshot {
    pub dispatcher: DispatcherRuntime,
    pub runner: RunnerRuntime,
    pub triggers: Vec<RunnableTrigger>,
    pub independent_tasks: Vec<IndependentTaskDescriptor>,
}

impl TaskRuntimeSnapshot {
    pub fn default_with_legacy_tasks() -> Self {
        Self {
            dispatcher: DispatcherRuntime::default(),
            runner: RunnerRuntime::default(),
            triggers: runtime_triggers(false),
            independent_tasks: independent_tasks(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskSelection {
    pub triggers: Vec<RunnableTrigger>,
    pub reason: TaskSelectionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TaskSelectionReason {
    NoEnabledTriggers,
    GameMinimized,
    GameInactiveNoBackgroundTrigger,
    ExclusiveTrigger,
    UiGracePeriod,
    CurrentUiMatch,
    BackgroundTriggers,
}

pub fn runtime_triggers(all_enabled: bool) -> Vec<RunnableTrigger> {
    initial_triggers()
        .into_iter()
        .map(|descriptor| RunnableTrigger {
            enabled: all_enabled || descriptor.default_enabled,
            descriptor,
        })
        .collect()
}

pub fn set_trigger_enabled(
    triggers: &mut [RunnableTrigger],
    key: &str,
    enabled: bool,
) -> Result<()> {
    let trigger = triggers
        .iter_mut()
        .find(|trigger| trigger.descriptor.key.eq_ignore_ascii_case(key))
        .ok_or_else(|| TaskError::UnknownTrigger(key.to_string()))?;
    trigger.enabled = enabled;
    Ok(())
}

pub fn select_triggers_for_tick(
    triggers: &[RunnableTrigger],
    runtime: &DispatcherRuntime,
    elapsed_since_ui_change: Duration,
) -> TaskSelection {
    if runtime.game_minimized {
        return TaskSelection {
            triggers: Vec::new(),
            reason: TaskSelectionReason::GameMinimized,
        };
    }

    let enabled: Vec<_> = triggers
        .iter()
        .filter(|trigger| trigger.enabled)
        .cloned()
        .collect();
    if enabled.is_empty() {
        return TaskSelection {
            triggers: Vec::new(),
            reason: TaskSelectionReason::NoEnabledTriggers,
        };
    }

    if let Some(exclusive) = enabled.iter().find(|trigger| trigger.descriptor.exclusive) {
        if runtime.game_active || exclusive.descriptor.background {
            return TaskSelection {
                triggers: vec![exclusive.clone()],
                reason: TaskSelectionReason::ExclusiveTrigger,
            };
        }
    }

    let mut candidates = enabled;
    if !runtime.game_active {
        candidates.retain(|trigger| trigger.descriptor.background);
        if candidates.is_empty() && !runtime.picture_in_picture {
            return TaskSelection {
                triggers: Vec::new(),
                reason: TaskSelectionReason::GameInactiveNoBackgroundTrigger,
            };
        }
    }

    let in_ui_grace_period =
        elapsed_since_ui_change.as_millis() as u64 <= runtime.ui_grace_period_ms;
    if in_ui_grace_period {
        return TaskSelection {
            triggers: candidates,
            reason: TaskSelectionReason::UiGracePeriod,
        };
    }

    candidates.retain(|trigger| {
        trigger.descriptor.supported_game_ui_category == runtime.current_ui
            || trigger.descriptor.supported_game_ui_category == GameUiCategory::Unknown
    });

    let reason = if runtime.game_active {
        TaskSelectionReason::CurrentUiMatch
    } else {
        TaskSelectionReason::BackgroundTriggers
    };

    TaskSelection {
        triggers: candidates,
        reason,
    }
}
