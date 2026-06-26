use bgi_script::{
    genshin_command_to_task_input, DispatcherCommand, GenshinCommand, GlobalInputDispatchMode,
    HtmlMaskMessage, HttpDispatchMode, KeyMouseScriptDispatchMode, KeyMouseScriptExecution,
    NotificationDispatchMode, PathingScriptExecution, ScriptCodeExecutionMode,
    ScriptHostExecutionRoots, ScriptHostRuntime, ScriptHostTarget, ScriptLogRecord,
    ScriptProjectError, ScriptProjectType,
};
use bgi_task::{
    evaluate_task_invocation_plans, execute_task_invocation_plans, DispatcherRuntime,
    ShellExecutionResult, TaskInvocationExecutionMode, TaskInvocationExecutionResult,
    TaskInvocationPlan,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ScriptEngineError {
    #[error("script runtime preparation failed: {0}")]
    Runtime(#[from] bgi_script::ScriptRuntimeError),
    #[error("script project load failed: {0}")]
    Project(#[from] ScriptProjectError),
    #[error("JavaScript module evaluation is still pending after job drain")]
    ModuleEvaluationPending,
    #[error("JavaScript promise evaluation is still pending after job drain")]
    PromiseEvaluationPending,
    #[error("JavaScript evaluation failed: {0}")]
    JavaScript(String),
    #[error("JavaScript execution cancelled")]
    Cancelled,
    #[error("JavaScript value conversion failed: {0}")]
    ValueConversion(String),
    #[error("host runtime initialization failed: {0}")]
    HostRuntime(#[from] bgi_script::ScriptHostRuntimeError),
    #[error("execution record storage failed: {0}")]
    ExecutionRecord(#[from] bgi_script::ExecutionRecordStorageError),
}

pub type Result<T> = std::result::Result<T, ScriptEngineError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptEngineRuntimeKind {
    Boa,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JavaScriptExecutionOutcome {
    pub runtime: ScriptEngineRuntimeKind,
    pub project: String,
    pub folder_name: String,
    pub execution_mode: ScriptCodeExecutionMode,
    pub main_script_path: PathBuf,
    pub result: Option<Value>,
    pub result_display: String,
    pub console: Vec<String>,
    pub logs: Vec<ScriptLogRecord>,
    pub host_calls: Vec<ExecutedHostCall>,
    pub task_invocations: JavaScriptTaskInvocations,
    pub task_execution: JavaScriptTaskExecution,
    pub html_mask_from_html: Vec<(String, HtmlMaskMessage)>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExecutedHostCall {
    pub target: ScriptHostTarget,
    pub method: String,
    pub args: Vec<Value>,
    pub result: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JavaScriptTaskInvocations {
    pub dispatcher: Vec<TaskInvocationPlan>,
    pub genshin: Vec<TaskInvocationPlan>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JavaScriptTaskExecution {
    pub mode: TaskInvocationExecutionMode,
    pub dispatcher: Vec<TaskInvocationExecutionResult>,
    pub genshin: Vec<TaskInvocationExecutionResult>,
}

impl JavaScriptTaskExecution {
    pub(crate) fn evaluate(
        invocations: &JavaScriptTaskInvocations,
        mode: TaskInvocationExecutionMode,
    ) -> Self {
        Self {
            mode,
            dispatcher: evaluate_task_invocation_plans(invocations.dispatcher.clone(), mode),
            genshin: evaluate_task_invocation_plans(invocations.genshin.clone(), mode),
        }
    }

    pub(crate) fn execute_ready(
        invocations: &JavaScriptTaskInvocations,
        dispatcher: &mut DispatcherRuntime,
    ) -> Self {
        Self {
            mode: TaskInvocationExecutionMode::ExecuteReady,
            dispatcher: execute_task_invocation_plans(dispatcher, invocations.dispatcher.clone()),
            genshin: execute_task_invocation_plans(dispatcher, invocations.genshin.clone()),
        }
    }

    pub fn total(&self) -> usize {
        self.dispatcher.len() + self.genshin.len()
    }
}

impl JavaScriptTaskInvocations {
    pub(crate) fn from_host(host: &ScriptHostRuntime) -> Self {
        let (dispatcher, mut errors) =
            dispatcher_invocation_plans_lossy(host.dispatcher_commands());
        let (genshin, genshin_errors) = genshin_invocation_plans_lossy(host.genshin_commands());
        errors.extend(genshin_errors);

        Self {
            dispatcher,
            genshin,
            errors,
        }
    }

    pub fn total(&self) -> usize {
        self.dispatcher.len() + self.genshin.len()
    }
}

fn dispatcher_invocation_plans_lossy(
    commands: &[DispatcherCommand],
) -> (Vec<TaskInvocationPlan>, Vec<String>) {
    commands.iter().cloned().enumerate().fold(
        (Vec::new(), Vec::new()),
        |mut acc, (index, command)| {
            match TaskInvocationPlan::from_script_dispatcher_command(command.into()) {
                Ok(plan) => acc.0.push(plan),
                Err(error) => acc.1.push(format!("dispatcher[{index}]: {error}")),
            }
            acc
        },
    )
}

fn genshin_invocation_plans_lossy(
    commands: &[GenshinCommand],
) -> (Vec<TaskInvocationPlan>, Vec<String>) {
    commands
        .iter()
        .filter_map(genshin_command_to_task_input)
        .enumerate()
        .fold((Vec::new(), Vec::new()), |mut acc, (index, command)| {
            match TaskInvocationPlan::from_script_dispatcher_command(command) {
                Ok(plan) => acc.0.push(plan),
                Err(error) => acc.1.push(format!("genshin[{index}]: {error}")),
            }
            acc
        })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptGroupExecutionRoots {
    pub js_script_root: PathBuf,
    pub key_mouse_script_root: PathBuf,
    pub pathing_script_root: PathBuf,
    pub strategy_root: PathBuf,
    pub shell_working_directory: PathBuf,
    pub global_input_dispatch_mode: GlobalInputDispatchMode,
    pub key_mouse_dispatch_mode: KeyMouseScriptDispatchMode,
    pub input_window_handle: Option<isize>,
    pub http_dispatch_mode: HttpDispatchMode,
    pub notification_dispatch_mode: NotificationDispatchMode,
    pub task_invocation_mode: TaskInvocationExecutionMode,
    pub app_version: Option<String>,
}

impl ScriptGroupExecutionRoots {
    pub fn from_app_root(app_root: impl AsRef<Path>) -> Self {
        let app_root = app_root.as_ref();
        Self {
            js_script_root: app_root.join("User").join("JsScript"),
            key_mouse_script_root: app_root.join("User").join("KeyMouseScript"),
            pathing_script_root: app_root.join("User").join("AutoPathing"),
            strategy_root: app_root.join("User").join("AutoFight"),
            shell_working_directory: app_root.to_path_buf(),
            global_input_dispatch_mode: GlobalInputDispatchMode::SendInput,
            key_mouse_dispatch_mode: KeyMouseScriptDispatchMode::SendInput,
            input_window_handle: None,
            http_dispatch_mode: HttpDispatchMode::Reqwest,
            notification_dispatch_mode: NotificationDispatchMode::Sink,
            task_invocation_mode: TaskInvocationExecutionMode::PlanOnly,
            app_version: None,
        }
    }

    pub(crate) fn host_roots(&self) -> ScriptHostExecutionRoots {
        ScriptHostExecutionRoots::new(self.strategy_root.clone(), self.pathing_script_root.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScriptGroupExecutionOutcome {
    pub group_name: String,
    pub requested_projects: usize,
    pub steps: Vec<ScriptGroupStepExecutionOutcome>,
    pub attempted_steps: usize,
    pub completed_steps: usize,
    pub planned_steps: usize,
    pub cancelled_steps: usize,
    pub failed_steps: usize,
    pub skipped_steps: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScriptGroupStepExecutionOutcome {
    pub project_index: usize,
    pub project_order: i32,
    pub run_iteration: u32,
    pub run_count: u32,
    pub name: String,
    pub folder_name: String,
    pub project_type: ScriptProjectType,
    pub status: ScriptGroupStepStatus,
    pub javascript: Option<JavaScriptExecutionOutcome>,
    pub key_mouse_execution: Option<KeyMouseScriptExecution>,
    pub pathing_execution: Option<PathingScriptExecution>,
    pub shell_result: Option<ShellExecutionResult>,
    pub error: Option<String>,
    pub skip_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ScriptGroupStepStatus {
    Completed,
    Planned,
    Cancelled,
    Skipped,
    Failed,
}
