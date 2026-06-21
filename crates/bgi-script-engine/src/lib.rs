use bgi_script::{
    genshin_command_to_task_input, DispatcherCommand, GenshinCommand, GlobalInputDispatchMode,
    HtmlMaskMessage, HttpDispatchMode, InputCancellationToken, KeyMouseScriptDispatchMode,
    KeyMouseScriptExecution, KeyMouseScriptHost, LoadedScriptModule, MacroPlaybackContext,
    NotificationDispatchMode, PathingScriptExecution, PathingScriptHost, PreparedScriptExecution,
    ScriptCodeExecutionMode, ScriptExecutionStep, ScriptGroup, ScriptGroupProject,
    ScriptGroupResumePointer, ScriptHostCall, ScriptHostCallResult, ScriptHostExecutionRoots,
    ScriptHostRuntime, ScriptHostRuntimeConfig, ScriptHostTarget, ScriptLogRecord,
    ScriptModuleLoader, ScriptProjectError, ScriptProjectStatus, ScriptProjectType,
};
use bgi_task::{
    evaluate_task_invocation_plans, execute_shell_task_with_cancel, execute_task_invocation_plans,
    AutoBossParam, AutoDomainParam, AutoFightParam, AutoLeyLineOutcropParam, AutoSkipConfigParam,
    AutoStygianOnslaughtParam, DispatcherRuntime, ShellConfig, ShellExecutionResult,
    ShellExecutionStatus, ShellTaskParam, TaskInvocationExecutionMode,
    TaskInvocationExecutionResult, TaskInvocationPlan,
};
use bgi_vision::{
    BvImage as VisionBvImage, BvLocator as VisionBvLocator, BvPage as VisionBvPage,
    RecognitionObject as VisionRecognitionObject, Rect as VisionRect,
};
use boa_engine::{
    builtins::promise::PromiseState,
    js_string,
    module::ModuleLoader,
    module::Referrer,
    native_function::NativeFunction,
    object::{builtins::JsPromise, FunctionObjectBuilder, JsObject, ObjectInitializer},
    property::Attribute,
    script::Script,
    Context, JsError, JsNativeError, JsResult, JsString, JsValue, Module, Source,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread;

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
    fn evaluate(
        invocations: &JavaScriptTaskInvocations,
        mode: TaskInvocationExecutionMode,
    ) -> Self {
        Self {
            mode,
            dispatcher: evaluate_task_invocation_plans(invocations.dispatcher.clone(), mode),
            genshin: evaluate_task_invocation_plans(invocations.genshin.clone(), mode),
        }
    }

    fn execute_ready(
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
    fn from_host(host: &ScriptHostRuntime) -> Self {
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
        }
    }

    fn host_roots(&self) -> ScriptHostExecutionRoots {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ScriptGroupStepStatus {
    Completed,
    Planned,
    Cancelled,
    Skipped,
    Failed,
}

#[derive(Clone)]
struct EngineState {
    console: Rc<RefCell<Vec<String>>>,
    host: Rc<RefCell<ScriptHostRuntime>>,
    host_calls: Rc<RefCell<Vec<ExecutedHostCall>>>,
    next_callback_id: Rc<RefCell<u64>>,
}

#[derive(Debug)]
struct BetterGiModuleLoader {
    loader: RefCell<ScriptModuleLoader>,
    modules: RefCell<HashMap<PathBuf, Module>>,
}

const JAVASCRIPT_EXECUTION_BUDGET: u32 = 256;

pub fn execute_javascript_project(
    scripts_root: impl AsRef<Path>,
    folder_name: impl Into<String>,
    settings: Option<Value>,
) -> Result<JavaScriptExecutionOutcome> {
    execute_javascript_project_with_host_configurator(scripts_root, folder_name, settings, |_| {})
}

pub fn execute_javascript_project_with_host_configurator(
    scripts_root: impl AsRef<Path>,
    folder_name: impl Into<String>,
    settings: Option<Value>,
    configure_host: impl FnOnce(&mut ScriptHostRuntimeConfig),
) -> Result<JavaScriptExecutionOutcome> {
    let mut prepared = prepare_javascript_project(scripts_root, folder_name, settings)?;
    configure_host(&mut prepared.host_runtime_config);
    execute_prepared_javascript(&prepared)
}

pub fn execute_javascript_project_with_host_and_task_dispatcher(
    scripts_root: impl AsRef<Path>,
    folder_name: impl Into<String>,
    settings: Option<Value>,
    configure_host: impl FnOnce(&mut ScriptHostRuntimeConfig),
    dispatcher: &mut DispatcherRuntime,
) -> Result<JavaScriptExecutionOutcome> {
    execute_javascript_project_with_host_task_dispatcher_and_cancellation(
        scripts_root,
        folder_name,
        settings,
        configure_host,
        dispatcher,
        None,
    )
}

pub fn execute_javascript_project_with_host_task_dispatcher_and_cancellation(
    scripts_root: impl AsRef<Path>,
    folder_name: impl Into<String>,
    settings: Option<Value>,
    configure_host: impl FnOnce(&mut ScriptHostRuntimeConfig),
    dispatcher: &mut DispatcherRuntime,
    cancellation: Option<&InputCancellationToken>,
) -> Result<JavaScriptExecutionOutcome> {
    let mut prepared = prepare_javascript_project(scripts_root, folder_name, settings)?;
    configure_host(&mut prepared.host_runtime_config);
    execute_prepared_javascript_with_task_dispatcher_and_cancellation(
        &prepared,
        dispatcher,
        cancellation,
    )
}

fn prepare_javascript_project(
    scripts_root: impl AsRef<Path>,
    folder_name: impl Into<String>,
    settings: Option<Value>,
) -> Result<PreparedScriptExecution> {
    let folder_name = folder_name.into();
    let manifest = bgi_script::Manifest::read_from(
        scripts_root
            .as_ref()
            .join(&folder_name)
            .join("manifest.json"),
    )
    .map_err(|err| ScriptProjectError::Io {
        path: scripts_root
            .as_ref()
            .join(&folder_name)
            .join("manifest.json"),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()),
    })?;
    let project = ScriptGroupProject {
        name: manifest.name.clone(),
        folder_name,
        project_type: ScriptProjectType::Javascript,
        js_script_settings_object: settings,
        ..ScriptGroupProject::default()
    };
    let step =
        ScriptExecutionStep::from_group_project(&project, Some(&manifest), scripts_root.as_ref())?;
    PreparedScriptExecution::prepare_javascript(&step, scripts_root).map_err(Into::into)
}

pub fn execute_script_group(
    app_root: impl AsRef<Path>,
    group: &ScriptGroup,
) -> ScriptGroupExecutionOutcome {
    let roots = ScriptGroupExecutionRoots::from_app_root(app_root);
    execute_script_group_with_roots(&roots, group)
}

pub fn execute_script_group_with_roots(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
) -> ScriptGroupExecutionOutcome {
    execute_script_group_with_host_configurator(roots, group, |_| {})
}

pub fn execute_script_group_with_host_configurator(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
) -> ScriptGroupExecutionOutcome {
    execute_script_group_with_host_hooks(roots, group, configure_host, |_| {})
}

pub fn execute_script_group_with_host_hooks(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> ScriptGroupExecutionOutcome {
    execute_script_group_with_task_dispatcher_hooks(
        roots,
        group,
        None,
        configure_host,
        after_javascript,
    )
}

pub fn execute_script_group_with_task_dispatcher_hooks(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    mut dispatcher: Option<&mut DispatcherRuntime>,
    mut configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    mut after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> ScriptGroupExecutionOutcome {
    execute_script_group_with_task_dispatcher_hooks_and_cancellation(
        roots,
        group,
        dispatcher.as_deref_mut(),
        None,
        &mut configure_host,
        &mut after_javascript,
    )
}

pub fn execute_script_group_with_task_dispatcher_hooks_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    mut dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    mut configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    mut after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> ScriptGroupExecutionOutcome {
    let mut indexed_projects = group
        .projects
        .iter()
        .cloned()
        .enumerate()
        .collect::<Vec<_>>();
    indexed_projects.sort_by_key(|(_, project)| project.index);

    let mut steps = Vec::new();
    'projects: for (project_index, project) in indexed_projects {
        if project.status == ScriptProjectStatus::Disabled || project.skip_flag.unwrap_or(false) {
            steps.push(skipped_group_step(project_index, &project));
            continue;
        }
        if cancellation_requested(cancellation) {
            steps.push(cancelled_group_step(project_index, &project, 0));
            break;
        }

        let run_count = project.run_num.max(1) as u32;
        for run_iteration in 1..=run_count {
            if cancellation_requested(cancellation) {
                steps.push(cancelled_group_step(project_index, &project, run_iteration));
                break 'projects;
            }
            let step = execute_group_project_once(
                roots,
                group,
                project_index,
                &project,
                run_iteration,
                dispatcher.as_deref_mut(),
                &mut configure_host,
                &mut after_javascript,
                cancellation,
            );
            let cancelled = step.status == ScriptGroupStepStatus::Cancelled;
            steps.push(step);
            if cancelled {
                break 'projects;
            }
        }
    }

    script_group_execution_outcome(group.name.clone(), group.projects.len(), steps)
}

pub fn execute_script_group_from_resume_with_task_dispatcher_hooks_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    resume_pointer: &ScriptGroupResumePointer,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> ScriptGroupExecutionOutcome {
    let (projects, _) =
        bgi_script::select_script_group_projects_from_resume(group, Some(resume_pointer));
    let mut resumed_group = group.clone();
    resumed_group.projects = projects;
    execute_script_group_with_task_dispatcher_hooks_and_cancellation(
        roots,
        &resumed_group,
        dispatcher,
        cancellation,
        configure_host,
        after_javascript,
    )
}

pub fn execute_script_group_project_with_host_hooks(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project_index: usize,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    execute_script_group_project_with_task_dispatcher_hooks(
        roots,
        group,
        project_index,
        None,
        configure_host,
        after_javascript,
    )
}

pub fn execute_script_group_project_with_task_dispatcher_hooks(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project_index: usize,
    dispatcher: Option<&mut DispatcherRuntime>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    execute_script_group_project_with_run_policy(
        roots,
        group,
        project_index,
        false,
        dispatcher,
        None,
        configure_host,
        after_javascript,
    )
}

pub fn execute_script_group_project_with_task_dispatcher_hooks_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project_index: usize,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    execute_script_group_project_with_run_policy(
        roots,
        group,
        project_index,
        false,
        dispatcher,
        cancellation,
        configure_host,
        after_javascript,
    )
}

pub fn execute_script_group_project_repeated_with_task_dispatcher_hooks(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project_index: usize,
    dispatcher: Option<&mut DispatcherRuntime>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    execute_script_group_project_with_run_policy(
        roots,
        group,
        project_index,
        true,
        dispatcher,
        None,
        configure_host,
        after_javascript,
    )
}

pub fn execute_script_group_project_repeated_with_task_dispatcher_hooks_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project_index: usize,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    execute_script_group_project_with_run_policy(
        roots,
        group,
        project_index,
        true,
        dispatcher,
        cancellation,
        configure_host,
        after_javascript,
    )
}

fn execute_script_group_project_with_run_policy(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project_index: usize,
    honor_run_count: bool,
    mut dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    mut configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    mut after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    let mut indexed_projects = group
        .projects
        .iter()
        .cloned()
        .enumerate()
        .collect::<Vec<_>>();
    indexed_projects.sort_by_key(|(_, project)| project.index);
    let (project_index, project) = indexed_projects
        .into_iter()
        .find(|(index, _)| *index == project_index)
        .ok_or_else(|| {
            ScriptEngineError::ValueConversion(format!(
                "script group project index {project_index} was not found"
            ))
        })?;

    let steps =
        if project.status == ScriptProjectStatus::Disabled || project.skip_flag.unwrap_or(false) {
            vec![skipped_group_step(project_index, &project)]
        } else if cancellation_requested(cancellation) {
            vec![cancelled_group_step(project_index, &project, 0)]
        } else {
            let run_count = if honor_run_count {
                project.run_num.max(1) as u32
            } else {
                1
            };
            let mut steps = Vec::with_capacity(run_count as usize);
            for run_iteration in 1..=run_count {
                if cancellation_requested(cancellation) {
                    steps.push(cancelled_group_step(project_index, &project, run_iteration));
                    break;
                }
                let step = execute_group_project_once(
                    roots,
                    group,
                    project_index,
                    &project,
                    run_iteration,
                    dispatcher.as_deref_mut(),
                    &mut configure_host,
                    &mut after_javascript,
                    cancellation,
                );
                let cancelled = step.status == ScriptGroupStepStatus::Cancelled;
                steps.push(step);
                if cancelled {
                    break;
                }
            }
            steps
        };

    Ok(script_group_execution_outcome(group.name.clone(), 1, steps))
}

fn skipped_group_step(
    project_index: usize,
    project: &ScriptGroupProject,
) -> ScriptGroupStepExecutionOutcome {
    group_step(
        project_index,
        project,
        0,
        project.run_num.max(1) as u32,
        ScriptGroupStepStatus::Skipped,
    )
}

fn cancelled_group_step(
    project_index: usize,
    project: &ScriptGroupProject,
    run_iteration: u32,
) -> ScriptGroupStepExecutionOutcome {
    group_step(
        project_index,
        project,
        run_iteration,
        project.run_num.max(1) as u32,
        ScriptGroupStepStatus::Cancelled,
    )
}

fn cancellation_requested(cancellation: Option<&InputCancellationToken>) -> bool {
    cancellation.is_some_and(InputCancellationToken::is_cancelled)
}

fn script_group_execution_outcome(
    group_name: String,
    requested_projects: usize,
    steps: Vec<ScriptGroupStepExecutionOutcome>,
) -> ScriptGroupExecutionOutcome {
    let attempted_steps = steps
        .iter()
        .filter(|step| step.status != ScriptGroupStepStatus::Skipped)
        .count();
    let completed_steps = steps
        .iter()
        .filter(|step| step.status == ScriptGroupStepStatus::Completed)
        .count();
    let planned_steps = steps
        .iter()
        .filter(|step| step.status == ScriptGroupStepStatus::Planned)
        .count();
    let cancelled_steps = steps
        .iter()
        .filter(|step| step.status == ScriptGroupStepStatus::Cancelled)
        .count();
    let failed_steps = steps
        .iter()
        .filter(|step| step.status == ScriptGroupStepStatus::Failed)
        .count();
    let skipped_steps = steps
        .iter()
        .filter(|step| step.status == ScriptGroupStepStatus::Skipped)
        .count();

    ScriptGroupExecutionOutcome {
        group_name,
        requested_projects,
        steps,
        attempted_steps,
        completed_steps,
        planned_steps,
        cancelled_steps,
        failed_steps,
        skipped_steps,
    }
}

fn execute_group_project_once(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project_index: usize,
    project: &ScriptGroupProject,
    run_iteration: u32,
    dispatcher: Option<&mut DispatcherRuntime>,
    configure_host: &mut impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: &mut impl FnMut(&mut JavaScriptExecutionOutcome),
    cancellation: Option<&InputCancellationToken>,
) -> ScriptGroupStepExecutionOutcome {
    let run_count = project.run_num.max(1) as u32;
    let mut outcome = group_step(
        project_index,
        project,
        run_iteration,
        run_count,
        ScriptGroupStepStatus::Completed,
    );

    match project.project_type.clone() {
        ScriptProjectType::Javascript => {
            match execute_group_javascript_project(
                roots,
                group,
                project,
                dispatcher,
                configure_host,
                cancellation,
            ) {
                Ok(mut result) => {
                    after_javascript(&mut result);
                    outcome.javascript = Some(result);
                }
                Err(error) => {
                    outcome.status = if matches!(error, ScriptEngineError::Cancelled) {
                        ScriptGroupStepStatus::Cancelled
                    } else {
                        ScriptGroupStepStatus::Failed
                    };
                    outcome.error = Some(error.to_string());
                }
            }
        }
        ScriptProjectType::KeyMouse => {
            match KeyMouseScriptHost::new(
                &roots.key_mouse_script_root,
                MacroPlaybackContext::default(),
            )
            .execute_file_with_cancellation(
                &project.name,
                roots.key_mouse_dispatch_mode,
                roots.input_window_handle,
                cancellation,
            ) {
                Ok(execution) => {
                    if execution.cancelled {
                        outcome.status = ScriptGroupStepStatus::Cancelled;
                    } else if execution.dispatched {
                        outcome.status = ScriptGroupStepStatus::Completed;
                    } else {
                        outcome.status = ScriptGroupStepStatus::Planned;
                    }
                    outcome.key_mouse_execution = Some(execution);
                }
                Err(error) => {
                    outcome.status = ScriptGroupStepStatus::Failed;
                    outcome.error = Some(error.to_string());
                }
            }
        }
        ScriptProjectType::Pathing => {
            let relative_path = PathBuf::from(&project.folder_name)
                .join(&project.name)
                .to_string_lossy()
                .replace('\\', "/");
            match PathingScriptHost::new(
                &roots.js_script_root,
                &roots.pathing_script_root,
                Some(group.config.pathing_config.clone()),
            )
            .execute_file_from_user(&relative_path)
            {
                Ok(execution) => {
                    outcome.status = ScriptGroupStepStatus::Planned;
                    outcome.pathing_execution = Some(execution);
                }
                Err(error) => {
                    outcome.status = ScriptGroupStepStatus::Failed;
                    outcome.error = Some(error.to_string());
                }
            }
        }
        ScriptProjectType::Shell => {
            let config = if group.config.enable_shell_config {
                ShellConfig::from_value(Some(&group.config.shell_config))
            } else {
                ShellConfig::default()
            };
            let param = ShellTaskParam::build_from_config(
                project.name.clone(),
                config,
                roots.shell_working_directory.clone(),
            );
            match execute_shell_task_with_cancel(&param, || cancellation_requested(cancellation)) {
                Ok(result) => {
                    if result.status == ShellExecutionStatus::Cancelled {
                        outcome.status = ScriptGroupStepStatus::Cancelled;
                    }
                    outcome.shell_result = Some(result);
                }
                Err(error) => {
                    outcome.status = ScriptGroupStepStatus::Failed;
                    outcome.error = Some(error.to_string());
                }
            }
        }
    }

    outcome
}

fn execute_group_javascript_project(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project: &ScriptGroupProject,
    dispatcher: Option<&mut DispatcherRuntime>,
    configure_host: &mut impl FnMut(&mut ScriptHostRuntimeConfig),
    cancellation: Option<&InputCancellationToken>,
) -> Result<JavaScriptExecutionOutcome> {
    let manifest_path = roots
        .js_script_root
        .join(&project.folder_name)
        .join("manifest.json");
    let manifest =
        bgi_script::Manifest::read_from(&manifest_path).map_err(|err| ScriptProjectError::Io {
            path: manifest_path,
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()),
        })?;
    let step =
        ScriptExecutionStep::from_group_project(project, Some(&manifest), &roots.js_script_root)?;
    let host_roots = roots.host_roots();
    let mut prepared = PreparedScriptExecution::prepare_javascript_with_host_roots(
        &step,
        &roots.js_script_root,
        Some(&host_roots),
        Some(group.config.pathing_config.clone()),
    )?;
    prepared.host_runtime_config.global_input_dispatch_mode = roots.global_input_dispatch_mode;
    prepared.host_runtime_config.key_mouse_dispatch_mode = roots.key_mouse_dispatch_mode;
    prepared.host_runtime_config.input_window_handle = roots.input_window_handle;
    prepared.host_runtime_config.http_dispatch_mode = roots.http_dispatch_mode;
    prepared.host_runtime_config.notification_dispatch_mode = roots.notification_dispatch_mode;
    prepared.host_runtime_config.cancellation = cancellation.cloned().map(Arc::new);
    configure_host(&mut prepared.host_runtime_config);
    match dispatcher {
        Some(dispatcher) => execute_prepared_javascript_with_task_dispatcher_and_cancellation(
            &prepared,
            dispatcher,
            cancellation,
        ),
        None => execute_prepared_javascript_with_task_mode_and_cancellation(
            &prepared,
            roots.task_invocation_mode,
            cancellation,
        ),
    }
}

fn group_step(
    project_index: usize,
    project: &ScriptGroupProject,
    run_iteration: u32,
    run_count: u32,
    status: ScriptGroupStepStatus,
) -> ScriptGroupStepExecutionOutcome {
    ScriptGroupStepExecutionOutcome {
        project_index,
        project_order: project.index,
        run_iteration,
        run_count,
        name: project.name.clone(),
        folder_name: project.folder_name.clone(),
        project_type: project.project_type.clone(),
        status,
        javascript: None,
        key_mouse_execution: None,
        pathing_execution: None,
        shell_result: None,
        error: None,
    }
}

pub fn execute_prepared_javascript(
    prepared: &PreparedScriptExecution,
) -> Result<JavaScriptExecutionOutcome> {
    execute_prepared_javascript_with_task_mode(prepared, TaskInvocationExecutionMode::PlanOnly)
}

fn execute_prepared_javascript_with_task_mode(
    prepared: &PreparedScriptExecution,
    task_invocation_mode: TaskInvocationExecutionMode,
) -> Result<JavaScriptExecutionOutcome> {
    execute_prepared_javascript_with_task_mode_and_cancellation(
        prepared,
        task_invocation_mode,
        None,
    )
}

fn execute_prepared_javascript_with_task_mode_and_cancellation(
    prepared: &PreparedScriptExecution,
    task_invocation_mode: TaskInvocationExecutionMode,
    cancellation: Option<&InputCancellationToken>,
) -> Result<JavaScriptExecutionOutcome> {
    let host = script_host_runtime(prepared, cancellation)?;
    execute_prepared_javascript_with_host(prepared, task_invocation_mode, None, cancellation, host)
}

pub fn execute_prepared_javascript_with_task_dispatcher(
    prepared: &PreparedScriptExecution,
    dispatcher: &mut DispatcherRuntime,
) -> Result<JavaScriptExecutionOutcome> {
    execute_prepared_javascript_with_task_dispatcher_and_cancellation(prepared, dispatcher, None)
}

pub fn execute_prepared_javascript_with_task_dispatcher_and_cancellation(
    prepared: &PreparedScriptExecution,
    dispatcher: &mut DispatcherRuntime,
    cancellation: Option<&InputCancellationToken>,
) -> Result<JavaScriptExecutionOutcome> {
    let host = script_host_runtime(prepared, cancellation)?;
    execute_prepared_javascript_with_host(
        prepared,
        TaskInvocationExecutionMode::ExecuteReady,
        Some(dispatcher),
        cancellation,
        host,
    )
}

fn script_host_runtime(
    prepared: &PreparedScriptExecution,
    cancellation: Option<&InputCancellationToken>,
) -> Result<ScriptHostRuntime> {
    let mut config = prepared.host_runtime_config.clone();
    if config.cancellation.is_none() {
        config.cancellation = cancellation.cloned().map(Arc::new);
    }
    ScriptHostRuntime::new(config).map_err(Into::into)
}

fn execute_prepared_javascript_with_host(
    prepared: &PreparedScriptExecution,
    task_invocation_mode: TaskInvocationExecutionMode,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    host: ScriptHostRuntime,
) -> Result<JavaScriptExecutionOutcome> {
    let module_loader = Rc::new(BetterGiModuleLoader::new(
        prepared.project_layout.project_path.clone(),
        prepared.project_layout.search_paths.clone(),
    )?);
    let mut context = Context::builder()
        .module_loader(module_loader.clone())
        .build()
        .map_err(|err| ScriptEngineError::JavaScript(err.to_string()))?;
    let state = EngineState {
        console: Rc::new(RefCell::new(Vec::new())),
        host: Rc::new(RefCell::new(host)),
        host_calls: Rc::new(RefCell::new(Vec::new())),
        next_callback_id: Rc::new(RefCell::new(1)),
    };

    register_console(&mut context, state.clone())?;
    register_log_host(&mut context, state.clone())?;
    register_global_host_functions(&mut context, state.clone())?;
    register_host_objects(&mut context, state.clone())?;
    if let Some(settings) = &prepared.settings {
        let settings = JsValue::from_json(settings, &mut context).map_err(|err| {
            ScriptEngineError::ValueConversion(js_error_to_string(err, &mut context))
        })?;
        context
            .register_global_property(js_string!("settings"), settings, Attribute::all())
            .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, &mut context)))?;
    }

    let result = execute_source(prepared, &module_loader, &mut context, cancellation)?;
    let result_display = result_to_display(&result, &mut context)?;
    let result = value_to_json_option(&result, &mut context)?;
    let logs = state.host.borrow().log_records().to_vec();
    let console = state.console.borrow().clone();
    let host_calls = state.host_calls.borrow().clone();
    let task_invocations = JavaScriptTaskInvocations::from_host(&state.host.borrow());
    let task_execution = dispatcher.map_or_else(
        || JavaScriptTaskExecution::evaluate(&task_invocations, task_invocation_mode),
        |dispatcher| JavaScriptTaskExecution::execute_ready(&task_invocations, dispatcher),
    );
    let html_mask_from_html = state.host.borrow().html_mask_remaining_from_html_messages();

    Ok(JavaScriptExecutionOutcome {
        runtime: ScriptEngineRuntimeKind::Boa,
        project: prepared.step.name.clone(),
        folder_name: prepared.step.folder_name.clone(),
        execution_mode: prepared.execution_mode,
        main_script_path: prepared.main_module.resolution.resolved_path.clone(),
        result,
        result_display,
        console,
        logs,
        host_calls,
        task_invocations,
        task_execution,
        html_mask_from_html,
    })
}

impl BetterGiModuleLoader {
    fn new(script_root: PathBuf, search_paths: Vec<PathBuf>) -> Result<Self> {
        Ok(Self {
            loader: RefCell::new(ScriptModuleLoader::new(script_root, search_paths)?),
            modules: RefCell::new(HashMap::new()),
        })
    }

    fn parse_loaded_module(
        &self,
        module: &LoadedScriptModule,
        context: &mut Context,
    ) -> JsResult<Module> {
        let path = module.resolution.resolved_path.clone();
        if let Some(module) = self.modules.borrow().get(&path).cloned() {
            return Ok(module);
        }

        let parsed = Module::parse(
            Source::from_reader(module.code.as_bytes(), Some(path.as_path())),
            None,
            context,
        )
        .map_err(|err| {
            JsError::from(
                JsNativeError::syntax()
                    .with_message(format!("could not parse module `{}`", path.display()))
                    .with_cause(err),
            )
        })?;
        self.modules.borrow_mut().insert(path, parsed.clone());
        Ok(parsed)
    }
}

impl ModuleLoader for BetterGiModuleLoader {
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        let specifier_text = specifier.to_std_string_escaped();
        let referrer_path = referrer.path().map(Path::to_path_buf);
        let result: JsResult<Module> = (|| {
            let module = self
                .loader
                .borrow_mut()
                .load_js_module(&specifier_text, referrer_path.as_deref())
                .map_err(|err| {
                    JsError::from(
                        JsNativeError::typ()
                            .with_message(err.to_string())
                            .with_cause(JsError::from_opaque(
                                js_string!(specifier_text.clone()).into(),
                            )),
                    )
                })?;
            self.parse_loaded_module(&module, context)
        })();

        finish_load(result, context);
    }

    fn init_import_meta(&self, import_meta: &JsObject, module: &Module, context: &mut Context) {
        let Some(path) = module.path() else {
            return;
        };
        let loader = self.loader.borrow();
        let (url, dirname) = import_meta_paths(loader.script_root(), path);
        let _ = import_meta.set(
            js_string!("url"),
            JsString::from(url.as_str()),
            true,
            context,
        );
        let _ = import_meta.set(
            js_string!("dirname"),
            JsString::from(dirname.as_str()),
            true,
            context,
        );
    }
}

fn execute_source(
    prepared: &PreparedScriptExecution,
    module_loader: &BetterGiModuleLoader,
    context: &mut Context,
    cancellation: Option<&InputCancellationToken>,
) -> Result<JsValue> {
    if cancellation_requested(cancellation) {
        return Err(ScriptEngineError::Cancelled);
    }

    match prepared.execution_mode {
        ScriptCodeExecutionMode::ClassicScript => settle_promise_value(
            execute_classic_source(prepared, context, cancellation)?,
            context,
        ),
        ScriptCodeExecutionMode::StandardModule => {
            let module = module_loader
                .parse_loaded_module(&prepared.main_module, context)
                .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))?;
            let promise = module.load_link_evaluate(context);
            settle_js_promise(promise, context)
        }
    }
}

fn execute_classic_source(
    prepared: &PreparedScriptExecution,
    context: &mut Context,
    cancellation: Option<&InputCancellationToken>,
) -> Result<JsValue> {
    let script = Script::parse(
        Source::from_reader(
            prepared.main_module.code.as_bytes(),
            Some(prepared.main_module.resolution.resolved_path.as_path()),
        ),
        None,
        context,
    )
    .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))?;
    let js_result = poll_future_with_cancellation(
        script.evaluate_async_with_budget(context, JAVASCRIPT_EXECUTION_BUDGET),
        cancellation,
    )?;
    js_result.map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))
}

fn poll_future_with_cancellation<F, T>(
    future: F,
    cancellation: Option<&InputCancellationToken>,
) -> Result<T>
where
    F: Future<Output = T>,
{
    let waker = noop_waker();
    let mut context = TaskContext::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        if cancellation_requested(cancellation) {
            return Err(ScriptEngineError::Cancelled);
        }
        match Future::poll(future.as_mut(), &mut context) {
            Poll::Ready(value) => return Ok(value),
            Poll::Pending => {
                if cancellation_requested(cancellation) {
                    return Err(ScriptEngineError::Cancelled);
                }
                thread::yield_now();
            }
        }
    }
}

fn noop_waker() -> Waker {
    // SAFETY: all operations use the same no-op vtable and do not dereference the data pointer.
    unsafe { Waker::from_raw(noop_raw_waker()) }
}

fn noop_raw_waker() -> RawWaker {
    RawWaker::new(std::ptr::null(), &NOOP_WAKER_VTABLE)
}

unsafe fn noop_waker_clone(_: *const ()) -> RawWaker {
    noop_raw_waker()
}

unsafe fn noop_waker_noop(_: *const ()) {}

static NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    noop_waker_clone,
    noop_waker_noop,
    noop_waker_noop,
    noop_waker_noop,
);

fn import_meta_paths(script_root: &Path, module_path: &Path) -> (String, String) {
    let module_path = path_relative_to_script_root(script_root, module_path)
        .unwrap_or_else(|| path_to_slash_string(module_path));
    let dirname = module_path
        .rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default();
    (module_path, dirname)
}

fn path_relative_to_script_root(script_root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(script_root)
        .ok()
        .map(path_to_slash_string)
}

fn path_to_slash_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn settle_promise_value(value: JsValue, context: &mut Context) -> Result<JsValue> {
    let Some(promise) = value
        .as_promise()
        .cloned()
        .and_then(|object| JsPromise::from_object(object).ok())
    else {
        return Ok(value);
    };
    settle_js_promise(promise, context)
}

fn settle_js_promise(promise: JsPromise, context: &mut Context) -> Result<JsValue> {
    context.run_jobs();
    match promise.state() {
        PromiseState::Fulfilled(value) => Ok(value),
        PromiseState::Rejected(reason) => Err(ScriptEngineError::JavaScript(js_value_to_string(
            &reason, context,
        ))),
        PromiseState::Pending => Err(ScriptEngineError::PromiseEvaluationPending),
    }
}

fn settle_promise_value_for_callback(value: JsValue, context: &mut Context) -> JsResult<JsValue> {
    settle_promise_value(value, context)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()).into())
}

fn register_console(context: &mut Context, state: EngineState) -> Result<()> {
    let console = ObjectInitializer::new(context)
        .function(
            native_closure({
                let state = state.clone();
                move |_, args, context| {
                    let message = join_js_args(args, context)?;
                    state.console.borrow_mut().push(message);
                    Ok(JsValue::Undefined)
                }
            }),
            js_string!("log"),
            1,
        )
        .build();
    context
        .register_global_property(js_string!("console"), console, Attribute::all())
        .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))
}

fn register_log_host(context: &mut Context, state: EngineState) -> Result<()> {
    let mut initializer = ObjectInitializer::new(context);
    for (method, length) in [("debug", 1), ("info", 1), ("warn", 1), ("error", 1)] {
        initializer.function(
            host_function(state.clone(), ScriptHostTarget::Log, method),
            JsString::from(method),
            length,
        );
        if let Some(alias) = pascal_case_alias(method) {
            initializer.function(
                host_function(state.clone(), ScriptHostTarget::Log, method),
                JsString::from(alias.as_str()),
                length,
            );
        }
    }
    let log = initializer.build();
    context
        .register_global_property(js_string!("log"), log, Attribute::all())
        .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))
}

fn register_global_host_functions(context: &mut Context, state: EngineState) -> Result<()> {
    for (name, length) in [
        ("sleep", 1),
        ("getVersion", 0),
        ("keyDown", 1),
        ("keyUp", 1),
        ("keyPress", 1),
        ("setGameMetrics", 2),
        ("getGameMetrics", 0),
        ("moveMouseBy", 2),
        ("moveMouseTo", 2),
        ("click", 0),
        ("leftButtonClick", 0),
        ("leftButtonDown", 0),
        ("leftButtonUp", 0),
        ("rightButtonClick", 0),
        ("rightButtonDown", 0),
        ("rightButtonUp", 0),
        ("middleButtonClick", 0),
        ("middleButtonDown", 0),
        ("middleButtonUp", 0),
        ("verticalScroll", 1),
        ("captureGameRegion", 0),
        ("getAvatars", 0),
        ("inputText", 1),
    ] {
        context
            .register_global_builtin_callable(
                JsString::from(name),
                length,
                host_function(state.clone(), ScriptHostTarget::Global, name),
            )
            .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))?;
        if let Some(alias) = pascal_case_alias(name) {
            context
                .register_global_builtin_callable(
                    JsString::from(alias.as_str()),
                    length,
                    host_function(state.clone(), ScriptHostTarget::Global, name),
                )
                .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))?;
        }
    }
    Ok(())
}

fn register_host_objects(context: &mut Context, state: EngineState) -> Result<()> {
    register_file_object(context, state.clone())?;
    register_script_type_constructors(context)?;

    register_simple_object(
        context,
        state.clone(),
        js_string!("vision"),
        ScriptHostTarget::Vision,
        &[
            ("findTemplate", 3),
            ("findColor", 2),
            ("crop", 2),
            ("to1080p", 1),
        ],
    )?;
    register_simple_object(
        context,
        state.clone(),
        js_string!("keyMouseScript"),
        ScriptHostTarget::KeyMouseScript,
        &[("run", 1), ("runFile", 1), ("plan", 1), ("planFile", 1)],
    )?;
    register_simple_object(
        context,
        state.clone(),
        js_string!("pathingScript"),
        ScriptHostTarget::PathingScript,
        &[
            ("run", 1),
            ("runFile", 1),
            ("runFileFromUser", 1),
            ("plan", 1),
            ("planFile", 1),
            ("planFileFromUser", 1),
            ("isExists", 1),
            ("isFile", 1),
            ("isFolder", 1),
            ("readPathSync", 1),
        ],
    )?;
    register_simple_object(
        context,
        state.clone(),
        js_string!("http"),
        ScriptHostTarget::Http,
        &[("request", 2)],
    )?;
    register_simple_object(
        context,
        state.clone(),
        js_string!("notification"),
        ScriptHostTarget::Notification,
        &[("send", 1), ("success", 1), ("error", 1), ("records", 0)],
    )?;
    register_simple_object(
        context,
        state.clone(),
        js_string!("dispatcher"),
        ScriptHostTarget::Dispatcher,
        &[
            ("addTimer", 1),
            ("addTrigger", 1),
            ("clearAllTriggers", 0),
            ("runTask", 1),
            ("getLinkedCancellationTokenSource", 0),
            ("getLinkedCancellationToken", 0),
            ("runAutoDomainTask", 1),
            ("runAutoBossTask", 1),
            ("runAutoFightTask", 1),
            ("runAutoLeyLineOutcropTask", 1),
            ("runAutoStygianOnslaughtTask", 1),
            ("commands", 0),
        ],
    )?;
    register_simple_object(
        context,
        state.clone(),
        js_string!("PostMessage"),
        ScriptHostTarget::PostMessage,
        &[("keyDown", 1), ("keyUp", 1), ("keyPress", 1), ("click", 2)],
    )?;
    register_simple_object(
        context,
        state.clone(),
        js_string!("strategyFile"),
        ScriptHostTarget::StrategyFile,
        &[
            ("isFolder", 1),
            ("isFile", 1),
            ("isExists", 1),
            ("readPathSync", 1),
        ],
    )?;
    register_simple_object(
        context,
        state.clone(),
        js_string!("ServerTime"),
        ScriptHostTarget::ServerTime,
        &[
            ("getServerTimeZoneOffset", 0),
            ("serverTimeZoneOffsetMilliseconds", 0),
        ],
    )?;
    register_simple_object(
        context,
        state.clone(),
        js_string!("htmlMask"),
        ScriptHostTarget::HtmlMask,
        &[
            ("show", 2),
            ("close", 1),
            ("closeAll", 0),
            ("getWindowIds", 0),
            ("exists", 1),
            ("setClickThrough", 2),
            ("getClickThrough", 1),
            ("toggleClickThrough", 1),
            ("send", 3),
            ("request", 4),
            ("receive", 2),
            ("poll", 1),
            ("pollAll", 1),
            ("flushPendingMessages", 1),
            ("sendFromHtml", 4),
            ("snapshot", 0),
        ],
    )?;
    register_key_mouse_hook_object(context, state.clone())?;
    register_simple_object(
        context,
        state.clone(),
        js_string!("host"),
        ScriptHostTarget::CustomHostFunctions,
        &[
            ("newObj", 1),
            ("delObj", 1),
            ("type", 1),
            ("toIterator", 1),
            ("newVarOfArr", 2),
        ],
    )?;
    register_simple_object(
        context,
        state,
        js_string!("genshin"),
        ScriptHostTarget::Genshin,
        &[
            ("uid", 0),
            ("tp", 1),
            ("moveMapTo", 2),
            ("moveIndependentMapTo", 3),
            ("getBigMapZoomLevel", 0),
            ("setBigMapZoomLevel", 1),
            ("tpToStatueOfTheSeven", 0),
            ("getPositionFromBigMap", 0),
            ("getPositionFromMap", 0),
            ("getPositionFromMapWithMatchingMethod", 1),
            ("getCameraOrientation", 0),
            ("switchParty", 1),
            ("clearPartyCache", 0),
            ("blessingOfTheWelkinMoon", 0),
            ("chooseTalkOption", 1),
            ("claimBattlePassRewards", 0),
            ("claimEncounterPointsRewards", 0),
            ("goToAdventurersGuild", 1),
            ("goToCraftingBench", 1),
            ("returnMainUi", 0),
            ("autoFishing", 0),
            ("relogin", 0),
            ("wonderlandCycle", 0),
            ("setTime", 2),
            ("commands", 0),
        ],
    )
}

fn register_file_object(context: &mut Context, state: EngineState) -> Result<()> {
    let methods = [
        ("readPathSync", "readPathSync", 1),
        ("ReadPathSync", "ReadPathSync", 1),
        ("createDirectory", "createDirectory", 1),
        ("CreateDirectory", "CreateDirectory", 1),
        ("isFolder", "isFolder", 1),
        ("IsFolder", "IsFolder", 1),
        ("isFile", "isFile", 1),
        ("IsFile", "IsFile", 1),
        ("isExists", "isExists", 1),
        ("IsExists", "IsExists", 1),
        ("readTextSync", "readTextSync", 1),
        ("ReadTextSync", "ReadTextSync", 1),
        ("readText", "readText", 1),
        ("ReadText", "ReadText", 1),
        ("writeTextSync", "writeTextSync", 2),
        ("WriteTextSync", "WriteTextSync", 2),
        ("writeText", "writeText", 2),
        ("WriteText", "WriteText", 2),
        ("readImageMatSync", "readImageMatSync", 1),
        ("ReadImageMatSync", "ReadImageMatSync", 1),
        (
            "readImageMatWithResizeSync",
            "readImageMatWithResizeSync",
            3,
        ),
        (
            "ReadImageMatWithResizeSync",
            "ReadImageMatWithResizeSync",
            3,
        ),
        ("writeImageSync", "writeImageSync", 2),
        ("WriteImageSync", "WriteImageSync", 2),
        ("renamePathSync", "renamePathSync", 2),
        ("RenamePathSync", "RenamePathSync", 2),
    ];
    let mut initializer = ObjectInitializer::new(context);
    for (property_name, host_method, length) in methods {
        initializer.function(
            host_function(state.clone(), ScriptHostTarget::File, host_method),
            JsString::from(property_name),
            length,
        );
    }
    let file = initializer.build();
    context
        .register_global_property(js_string!("file"), file, Attribute::all())
        .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))
}

fn register_script_type_constructors(context: &mut Context) -> Result<()> {
    for (name, length, function) in [
        (
            js_string!("RealtimeTimer"),
            2,
            NativeFunction::from_fn_ptr(realtime_timer_constructor),
        ),
        (
            js_string!("SoloTask"),
            2,
            NativeFunction::from_fn_ptr(solo_task_constructor),
        ),
        (
            js_string!("AutoSkipConfig"),
            0,
            NativeFunction::from_fn_ptr(auto_skip_config_constructor),
        ),
        (
            js_string!("AutoDomainParam"),
            2,
            NativeFunction::from_fn_ptr(auto_domain_param_constructor),
        ),
        (
            js_string!("AutoBossParam"),
            1,
            NativeFunction::from_fn_ptr(auto_boss_param_constructor),
        ),
        (
            js_string!("AutoFightParam"),
            1,
            NativeFunction::from_fn_ptr(auto_fight_param_constructor),
        ),
        (
            js_string!("AutoLeyLineOutcropParam"),
            3,
            NativeFunction::from_fn_ptr(auto_ley_line_outcrop_param_constructor),
        ),
        (
            js_string!("AutoStygianOnslaughtParam"),
            1,
            NativeFunction::from_fn_ptr(auto_stygian_onslaught_param_constructor),
        ),
        (
            js_string!("Rect"),
            4,
            NativeFunction::from_fn_ptr(rect_constructor),
        ),
        (
            js_string!("RecognitionObject"),
            0,
            NativeFunction::from_fn_ptr(recognition_object_constructor),
        ),
        (
            js_string!("BvImage"),
            3,
            NativeFunction::from_fn_ptr(bv_image_constructor),
        ),
        (
            js_string!("BvLocator"),
            1,
            NativeFunction::from_fn_ptr(bv_locator_constructor),
        ),
        (
            js_string!("BvPage"),
            0,
            NativeFunction::from_fn_ptr(bv_page_constructor),
        ),
    ] {
        let constructor = FunctionObjectBuilder::new(context.realm(), function)
            .name(name.clone())
            .length(length)
            .constructor(true)
            .build();
        context
            .register_global_property(name, constructor, Attribute::all())
            .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))?;
    }
    Ok(())
}

fn register_simple_object(
    context: &mut Context,
    state: EngineState,
    global_name: JsString,
    target: ScriptHostTarget,
    methods: &[(&'static str, usize)],
) -> Result<()> {
    let mut initializer = ObjectInitializer::new(context);
    for (method, length) in methods {
        initializer.function(
            host_function(state.clone(), target, method),
            JsString::from(*method),
            *length,
        );
        if let Some(alias) = pascal_case_alias(method) {
            initializer.function(
                host_function(state.clone(), target, method),
                JsString::from(alias.as_str()),
                *length,
            );
        }
    }
    let object = initializer.build();
    context
        .register_global_property(global_name, object, Attribute::all())
        .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))
}

fn register_key_mouse_hook_object(context: &mut Context, state: EngineState) -> Result<()> {
    ensure_callback_registry(context)
        .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))?;
    let mut initializer = ObjectInitializer::new(context);
    for (method, length) in [
        ("onKeyDown", 2),
        ("onKeyUp", 2),
        ("onMouseDown", 1),
        ("onMouseUp", 1),
        ("onMouseMove", 2),
        ("onMouseWheel", 1),
        ("removeAllListeners", 0),
        ("dispose", 0),
        ("dispatchEvent", 1),
        ("snapshot", 0),
    ] {
        initializer.function(
            key_mouse_hook_function(state.clone(), method),
            JsString::from(method),
            length,
        );
        if let Some(alias) = pascal_case_alias(method) {
            initializer.function(
                key_mouse_hook_function(state.clone(), method),
                JsString::from(alias.as_str()),
                length,
            );
        }
    }
    let object = initializer.build();
    context
        .register_global_property(js_string!("KeyMouseHook"), object, Attribute::all())
        .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))
}

fn pascal_case_alias(name: &str) -> Option<String> {
    let mut chars = name.chars();
    let first = chars.next()?;
    if !first.is_ascii_lowercase() {
        return None;
    }

    let mut alias = String::with_capacity(name.len());
    alias.push(first.to_ascii_uppercase());
    alias.push_str(chars.as_str());
    Some(alias)
}

fn host_function(
    state: EngineState,
    target: ScriptHostTarget,
    method: &'static str,
) -> NativeFunction {
    native_closure(move |_, args, context| call_host(&state, target, method, args, context))
}

fn key_mouse_hook_function(state: EngineState, method: &'static str) -> NativeFunction {
    native_closure(move |_, args, context| call_key_mouse_hook_host(&state, method, args, context))
}

fn native_closure<F>(closure: F) -> NativeFunction
where
    F: Fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue> + 'static,
{
    // The closures capture only Rust-side state behind Rc/RefCell and never store JsValue/JsObject.
    unsafe { NativeFunction::from_closure(closure) }
}

fn realtime_timer_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let name = constructor_string_arg(args, 0, context)?;
    let config = constructor_config_arg(args, 1, context)?;
    let object = ObjectInitializer::new(context)
        .property(js_string!("name"), name.clone(), Attribute::all())
        .property(js_string!("Name"), name, Attribute::all())
        .property(js_string!("interval"), 50, Attribute::all())
        .property(js_string!("Interval"), 50, Attribute::all())
        .property(js_string!("config"), config.clone(), Attribute::all())
        .property(js_string!("Config"), config, Attribute::all())
        .build();
    Ok(object.into())
}

fn solo_task_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let name = constructor_string_arg(args, 0, context)?;
    let config = constructor_config_arg(args, 1, context)?;
    let object = ObjectInitializer::new(context)
        .property(js_string!("name"), name.clone(), Attribute::all())
        .property(js_string!("Name"), name, Attribute::all())
        .property(js_string!("config"), config.clone(), Attribute::all())
        .property(js_string!("Config"), config, Attribute::all())
        .build();
    Ok(object.into())
}

fn auto_skip_config_constructor(
    _: &JsValue,
    _: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    task_param_object(AutoSkipConfigParam::default(), context)
}

fn auto_domain_param_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let round_num = constructor_i32_arg(args, 0, context)?.unwrap_or(0);
    let strategy_name = constructor_optional_string_arg(args, 1, context)?;
    task_param_object(
        AutoDomainParam::new(round_num, strategy_name.as_deref()),
        context,
    )
}

fn auto_boss_param_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let strategy_name = constructor_optional_string_arg(args, 0, context)?;
    task_param_object(AutoBossParam::new(strategy_name.as_deref()), context)
}

fn auto_fight_param_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let strategy_name = constructor_optional_string_arg(args, 0, context)?;
    task_param_object(AutoFightParam::new(strategy_name.as_deref()), context)
}

fn auto_ley_line_outcrop_param_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    if args.is_empty()
        || args
            .first()
            .is_some_and(|value| value.is_undefined() || value.is_null())
    {
        return task_param_object(AutoLeyLineOutcropParam::default(), context);
    }
    let count = constructor_i32_arg(args, 0, context)?.unwrap_or(0);
    let country = constructor_optional_string_arg(args, 1, context)?.unwrap_or_default();
    let outcrop_type = constructor_optional_string_arg(args, 2, context)?.unwrap_or_default();
    task_param_object(
        AutoLeyLineOutcropParam::new(count, country, outcrop_type),
        context,
    )
}

fn auto_stygian_onslaught_param_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let strategy_name = constructor_optional_string_arg(args, 0, context)?;
    task_param_object(
        AutoStygianOnslaughtParam::new(strategy_name.as_deref()),
        context,
    )
}

fn rect_constructor(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let x = constructor_i32_arg(args, 0, context)?.unwrap_or(0);
    let y = constructor_i32_arg(args, 1, context)?.unwrap_or(0);
    let width = constructor_i32_arg(args, 2, context)?.unwrap_or(0);
    let height = constructor_i32_arg(args, 3, context)?.unwrap_or(0);
    let rect = VisionRect::new(x, y, width, height)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
    task_param_object(rect, context)
}

fn recognition_object_constructor(
    _: &JsValue,
    _: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    task_param_object(VisionRecognitionObject::default(), context)
}

fn bv_image_constructor(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let template_assert = constructor_optional_string_arg(args, 0, context)?
        .ok_or_else(|| JsNativeError::typ().with_message("BvImage template asset is required"))?;
    let roi = constructor_rect_arg(args, 1, context)?;
    let threshold = constructor_f64_arg(args, 2, context)?.unwrap_or(0.8);
    let image = VisionBvImage::new(template_assert)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
    let recognition_object = image
        .to_recognition_object(roi, threshold)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
    let mut value = to_json_value(&image);
    if let Value::Object(map) = &mut value {
        map.insert(
            "recognition_object".to_string(),
            to_json_value(&recognition_object),
        );
    }
    add_pascal_case_aliases(&mut value);
    JsValue::from_json(&value, context)
}

fn bv_locator_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let object = constructor_recognition_object_arg(args, 0, context)?;
    task_param_object(VisionBvLocator::new(object), context)
}

fn bv_page_constructor(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    task_param_object(VisionBvPage::default(), context)
}

fn task_param_object<T: Serialize>(value: T, context: &mut Context) -> JsResult<JsValue> {
    let mut value = to_json_value(&value);
    add_pascal_case_aliases(&mut value);
    JsValue::from_json(&value, context)
}

fn add_pascal_case_aliases(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for value in map.values_mut() {
                add_pascal_case_aliases(value);
            }
            let aliases = map
                .iter()
                .filter_map(|(key, value)| {
                    snake_to_pascal_case_alias(key).map(|alias| (alias, value.clone()))
                })
                .collect::<Vec<_>>();
            for (alias, value) in aliases {
                map.entry(alias).or_insert(value);
            }
        }
        Value::Array(items) => {
            for value in items {
                add_pascal_case_aliases(value);
            }
        }
        _ => {}
    }
}

fn snake_to_pascal_case_alias(name: &str) -> Option<String> {
    if name.is_empty() || name.chars().any(|ch| ch.is_ascii_uppercase()) {
        return None;
    }
    let mut alias = String::with_capacity(name.len());
    for part in name.split('_') {
        if part.is_empty() {
            continue;
        }
        let mut chars = part.chars();
        let first = chars.next()?;
        alias.push(first.to_ascii_uppercase());
        alias.push_str(chars.as_str());
    }
    (!alias.is_empty() && alias != name).then_some(alias)
}

fn constructor_string_arg(
    args: &[JsValue],
    index: usize,
    context: &mut Context,
) -> JsResult<JsValue> {
    let Some(value) = args.get(index) else {
        return Ok(JsValue::Null);
    };
    if value.is_undefined() || value.is_null() {
        return Ok(JsValue::Null);
    }
    Ok(JsValue::from(value.to_string(context)?))
}

fn constructor_config_arg(
    args: &[JsValue],
    index: usize,
    context: &mut Context,
) -> JsResult<JsValue> {
    let Some(value) = args.get(index) else {
        return Ok(JsValue::Null);
    };
    if value.is_undefined() {
        return Ok(JsValue::Null);
    }
    JsValue::from_json(&value.to_json(context)?, context)
}

fn constructor_optional_string_arg(
    args: &[JsValue],
    index: usize,
    context: &mut Context,
) -> JsResult<Option<String>> {
    let Some(value) = args.get(index) else {
        return Ok(None);
    };
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    Ok(Some(value.to_string(context)?.to_std_string_escaped()))
}

fn constructor_i32_arg(
    args: &[JsValue],
    index: usize,
    _context: &mut Context,
) -> JsResult<Option<i32>> {
    let Some(value) = args.get(index) else {
        return Ok(None);
    };
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    let Some(number) = value.as_number() else {
        return Err(JsNativeError::typ()
            .with_message(format!("constructor argument {index} must be number"))
            .into());
    };
    if !number.is_finite() || number < i32::MIN as f64 || number > i32::MAX as f64 {
        return Err(JsNativeError::typ()
            .with_message(format!("constructor argument {index} is outside i32 range"))
            .into());
    }
    let truncated = number.trunc();
    if (number - truncated).abs() > f64::EPSILON {
        return Err(JsNativeError::typ()
            .with_message(format!("constructor argument {index} must be an integer"))
            .into());
    }
    Ok(Some(truncated as i32))
}

fn constructor_f64_arg(
    args: &[JsValue],
    index: usize,
    _context: &mut Context,
) -> JsResult<Option<f64>> {
    let Some(value) = args.get(index) else {
        return Ok(None);
    };
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    let Some(number) = value.as_number() else {
        return Err(JsNativeError::typ()
            .with_message(format!("constructor argument {index} must be number"))
            .into());
    };
    if !number.is_finite() {
        return Err(JsNativeError::typ()
            .with_message(format!("constructor argument {index} must be finite"))
            .into());
    }
    Ok(Some(number))
}

fn constructor_rect_arg(
    args: &[JsValue],
    index: usize,
    context: &mut Context,
) -> JsResult<Option<VisionRect>> {
    let Some(value) = args.get(index) else {
        return Ok(None);
    };
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    let json = value.to_json(context)?;
    rect_from_json(&json, index).map(Some)
}

fn constructor_recognition_object_arg(
    args: &[JsValue],
    index: usize,
    context: &mut Context,
) -> JsResult<VisionRecognitionObject> {
    let Some(value) = args.get(index) else {
        return Ok(VisionRecognitionObject::default());
    };
    if value.is_undefined() || value.is_null() {
        return Ok(VisionRecognitionObject::default());
    }
    let json = value.to_json(context)?;
    let json = json
        .as_object()
        .and_then(|map| {
            map.get("recognition_object")
                .or_else(|| map.get("RecognitionObject"))
                .cloned()
        })
        .unwrap_or(json);
    serde_json::from_value(json).map_err(|err| {
        JsNativeError::typ()
            .with_message(format!(
                "constructor argument {index} must be RecognitionObject: {err}"
            ))
            .into()
    })
}

fn rect_from_json(value: &Value, index: usize) -> JsResult<VisionRect> {
    let Value::Object(map) = value else {
        return Err(JsNativeError::typ()
            .with_message(format!("constructor argument {index} must be Rect object"))
            .into());
    };
    let x = json_i32_field(map, &["x", "X"]).unwrap_or(0);
    let y = json_i32_field(map, &["y", "Y"]).unwrap_or(0);
    let width = json_i32_field(map, &["width", "Width"]).unwrap_or(0);
    let height = json_i32_field(map, &["height", "Height"]).unwrap_or(0);
    VisionRect::new(x, y, width, height)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()).into())
}

fn json_i32_field(map: &serde_json::Map<String, Value>, names: &[&str]) -> Option<i32> {
    names
        .iter()
        .filter_map(|name| map.get(*name))
        .find_map(|value| value.as_i64().and_then(|number| i32::try_from(number).ok()))
}

fn call_host(
    state: &EngineState,
    target: ScriptHostTarget,
    method: &str,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let args = args_to_json(args, context)?;
    let call = ScriptHostCall::new(target, method, args.clone());
    let result = state
        .host
        .borrow_mut()
        .call(call)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
    let json_result = host_result_to_json(&result);
    state.host_calls.borrow_mut().push(ExecutedHostCall {
        target,
        method: method.to_string(),
        args,
        result: json_result.clone(),
    });
    JsValue::from_json(&json_result, context)
}

fn call_key_mouse_hook_host(
    state: &EngineState,
    method: &str,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let args = key_mouse_hook_args_to_json(state, method, args, context)?;
    let call = ScriptHostCall::new(ScriptHostTarget::KeyMouseHook, method, args.clone());
    let result = state
        .host
        .borrow_mut()
        .call(call)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
    let json_result = host_result_to_json(&result);
    state.host_calls.borrow_mut().push(ExecutedHostCall {
        target: ScriptHostTarget::KeyMouseHook,
        method: method.to_string(),
        args,
        result: json_result.clone(),
    });

    if method.eq_ignore_ascii_case("dispatchEvent") {
        if let ScriptHostCallResult::KeyMouseHookDispatches(dispatches) = &result {
            dispatch_key_mouse_hook_callbacks(dispatches, context)?;
        }
    }

    JsValue::from_json(&json_result, context)
}

fn key_mouse_hook_args_to_json(
    state: &EngineState,
    method: &str,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<Vec<Value>> {
    if is_key_mouse_hook_registration_method(method) {
        let mut converted = Vec::with_capacity(args.len());
        if let Some(first) = args.first() {
            if let Some(callback) = first.as_callable() {
                let callback_id = register_callback(state, callback.clone(), context)?;
                converted.push(Value::String(callback_id));
            } else if first.is_undefined() {
                converted.push(Value::Null);
            } else {
                converted.push(first.to_json(context)?);
            }
        }
        for arg in args.iter().skip(1) {
            converted.push(if arg.is_undefined() {
                Value::Null
            } else {
                arg.to_json(context)?
            });
        }
        return Ok(converted);
    }

    args_to_json(args, context)
}

fn is_key_mouse_hook_registration_method(method: &str) -> bool {
    matches!(
        method,
        "onKeyDown"
            | "OnKeyDown"
            | "onKeyUp"
            | "OnKeyUp"
            | "onMouseDown"
            | "OnMouseDown"
            | "onMouseUp"
            | "OnMouseUp"
            | "onMouseMove"
            | "OnMouseMove"
            | "onMouseWheel"
            | "OnMouseWheel"
    )
}

fn register_callback(
    state: &EngineState,
    callback: JsObject,
    context: &mut Context,
) -> JsResult<String> {
    ensure_callback_registry(context)?;
    let id = {
        let mut next = state.next_callback_id.borrow_mut();
        let id = format!("callback-{}", *next);
        *next = next.saturating_add(1);
        id
    };
    let registry = callback_registry(context)?;
    registry.set(JsString::from(id.as_str()), callback, true, context)?;
    Ok(id)
}

fn dispatch_key_mouse_hook_callbacks(
    dispatches: &[bgi_script::KeyMouseHookDispatch],
    context: &mut Context,
) -> JsResult<()> {
    let registry = callback_registry(context)?;
    for dispatch in dispatches {
        let value = registry.get(JsString::from(dispatch.listener_id.as_str()), context)?;
        let Some(callback) = value.as_callable() else {
            continue;
        };
        let args = dispatch
            .args
            .iter()
            .map(|value| JsValue::from_json(value, context))
            .collect::<JsResult<Vec<_>>>()?;
        let result = callback.call(&JsValue::Undefined, &args, context)?;
        settle_promise_value_for_callback(result, context)?;
    }
    Ok(())
}

fn ensure_callback_registry(context: &mut Context) -> JsResult<()> {
    let global = context.global_object().clone();
    if global
        .get(js_string!("__bettergiCallbacks"), context)?
        .as_object()
        .is_some()
    {
        return Ok(());
    }
    let registry = ObjectInitializer::new(context).build();
    global.set(js_string!("__bettergiCallbacks"), registry, true, context)?;
    Ok(())
}

fn callback_registry(context: &mut Context) -> JsResult<JsObject> {
    ensure_callback_registry(context)?;
    context
        .global_object()
        .clone()
        .get(js_string!("__bettergiCallbacks"), context)?
        .as_object()
        .cloned()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("callback registry is not an object")
                .into()
        })
}

fn join_js_args(args: &[JsValue], context: &mut Context) -> JsResult<String> {
    let mut parts = Vec::with_capacity(args.len());
    for arg in args {
        parts.push(arg.to_string(context)?.to_std_string_escaped());
    }
    Ok(parts.join(" "))
}

fn args_to_json(args: &[JsValue], context: &mut Context) -> JsResult<Vec<Value>> {
    args.iter()
        .map(|value| {
            if value.is_undefined() {
                Ok(Value::Null)
            } else {
                value.to_json(context)
            }
        })
        .collect()
}

fn host_result_to_json(result: &ScriptHostCallResult) -> Value {
    match result {
        ScriptHostCallResult::None => Value::Null,
        ScriptHostCallResult::Bool(value) => Value::Bool(*value),
        ScriptHostCallResult::Integer(value) => json!(value),
        ScriptHostCallResult::String(value) => json!(value),
        ScriptHostCallResult::StringList(value) => json!(value),
        ScriptHostCallResult::GameMetrics(value) => to_json_value(value),
        ScriptHostCallResult::CaptureGameRegionPlan(value) => to_json_value(value),
        ScriptHostCallResult::CaptureGameRegionExecution(value) => to_json_value(value),
        ScriptHostCallResult::AvatarRecognitionPlan(value) => to_json_value(value),
        ScriptHostCallResult::ImageMatReadPlan(value) => to_json_value(value),
        ScriptHostCallResult::ImageMatWritePlan(value) => to_json_value(value),
        ScriptHostCallResult::ImageMatReadExecution(value) => to_json_value(value),
        ScriptHostCallResult::ImageMatWriteExecution(value) => to_json_value(value),
        ScriptHostCallResult::VisionRecognitionExecution(value) => to_json_value(value),
        ScriptHostCallResult::VisionImageMatExecution(value) => to_json_value(value),
        ScriptHostCallResult::CustomHostFunctionCommand(value) => to_json_value(value),
        ScriptHostCallResult::InputEvents(value) => to_json_value(value),
        ScriptHostCallResult::InputExecution(value) => to_json_value(value),
        ScriptHostCallResult::PostMessageEvents(value) => to_json_value(value),
        ScriptHostCallResult::HttpRequestPlan(value) => to_json_value(value),
        ScriptHostCallResult::HttpExecution(value) => to_json_value(value),
        ScriptHostCallResult::DispatcherCommand(value) => to_json_value(value),
        ScriptHostCallResult::DispatcherCommands(value) => to_json_value(value),
        ScriptHostCallResult::GenshinCommand(value) => to_json_value(value),
        ScriptHostCallResult::GenshinCommands(value) => to_json_value(value),
        ScriptHostCallResult::PathingPlan(value) => to_json_value(value),
        ScriptHostCallResult::PathingExecution(value) => to_json_value(value),
        ScriptHostCallResult::KeyMousePlan(value) => to_json_value(value),
        ScriptHostCallResult::KeyMouseExecution(value) => to_json_value(value),
        ScriptHostCallResult::HtmlMaskCommand(value) => to_json_value(value),
        ScriptHostCallResult::HtmlMaskSnapshot(value) => to_json_value(value),
        ScriptHostCallResult::KeyMouseHookCommand(value) => to_json_value(value),
        ScriptHostCallResult::KeyMouseHookDispatches(value) => to_json_value(value),
        ScriptHostCallResult::KeyMouseHookSnapshot(value) => to_json_value(value),
        ScriptHostCallResult::LogRecords(value) => to_json_value(value),
        ScriptHostCallResult::NotificationExecution(value) => to_json_value(value),
        ScriptHostCallResult::NotificationRecords(value) => to_json_value(value),
    }
}

fn to_json_value<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

fn value_to_json_option(value: &JsValue, context: &mut Context) -> Result<Option<Value>> {
    if value.is_undefined() {
        return Ok(None);
    }
    value
        .to_json(context)
        .map(Some)
        .map_err(|err| ScriptEngineError::ValueConversion(js_error_to_string(err, context)))
}

fn result_to_display(value: &JsValue, context: &mut Context) -> Result<String> {
    if value.is_undefined() {
        return Ok("undefined".to_string());
    }
    value
        .to_string(context)
        .map(|value| value.to_std_string_escaped())
        .map_err(|err| ScriptEngineError::ValueConversion(js_error_to_string(err, context)))
}

fn js_value_to_string(value: &JsValue, context: &mut Context) -> String {
    value
        .to_string(context)
        .map(|message| message.to_std_string_escaped())
        .unwrap_or_else(|_| value.display().to_string())
}

fn js_error_to_string(error: JsError, context: &mut Context) -> String {
    error
        .to_opaque(context)
        .to_string(context)
        .map(|message| message.to_std_string_escaped())
        .unwrap_or_else(|_| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bgi_capture::CaptureFrame;
    use bgi_script::{GameCaptureFrameSource, ScriptGroupConfig};
    use bgi_vision::{BgrImage, Size as VisionSize};
    use std::fs;
    use std::sync::Arc;

    struct StaticFrameSource {
        frame: CaptureFrame,
        area: bgi_script::GameCaptureArea,
    }

    impl GameCaptureFrameSource for StaticFrameSource {
        fn capture_frame(
            &self,
        ) -> std::result::Result<CaptureFrame, bgi_script::ScriptHostRuntimeError> {
            Ok(self.frame.clone())
        }

        fn capture_frame_area(&self, _frame: &CaptureFrame) -> bgi_script::GameCaptureArea {
            self.area
        }
    }

    #[test]
    fn executes_classic_javascript_with_settings_and_log_host() {
        let root = test_root("classic");
        let project = root.join("demo");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                console.log("hello", settings.name);
                log.info("level " + settings.level);
                getGameMetrics().width + settings.level;
            "#,
        )
        .unwrap();

        let outcome = execute_javascript_project(
            &root,
            "demo",
            Some(json!({"name": "BetterGI", "level": 2})),
        )
        .unwrap();

        assert_eq!(outcome.runtime, ScriptEngineRuntimeKind::Boa);
        assert_eq!(outcome.result, Some(json!(1922)));
        assert_eq!(outcome.console, vec!["hello BetterGI"]);
        assert_eq!(outcome.logs.len(), 1);
        assert_eq!(outcome.logs[0].message, "level 2");
        assert!(
            outcome
                .host_calls
                .iter()
                .any(|call| call.target == ScriptHostTarget::Global
                    && call.method == "getGameMetrics")
        );
    }

    #[test]
    fn classic_javascript_settles_top_level_promise_result() {
        let root = test_root("classic-promise");
        let project = root.join("demo");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                Promise.resolve(40).then((value) => {
                    console.log("promise", value);
                    return value + 2;
                });
            "#,
        )
        .unwrap();

        let outcome = execute_javascript_project(&root, "demo", None).unwrap();

        assert_eq!(outcome.result, Some(json!(42)));
        assert_eq!(outcome.result_display, "42");
        assert_eq!(outcome.console, vec!["promise 40"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn classic_javascript_receives_injected_capture_game_region_execution() {
        let root = test_root("capture-game-region-js");
        let project = root.join("demo");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Capture","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                const capture = CaptureGameRegion();
                `${capture.width}x${capture.height}:${capture.pixels.join(",")}:${capture.image_region.source}`;
            "#,
        )
        .unwrap();

        let step = ScriptExecutionStep {
            index: 1,
            name: "Capture".to_string(),
            folder_name: "demo".to_string(),
            project_type: ScriptProjectType::Javascript,
            engine: bgi_script::ScriptEngineKind::RustJavaScript,
            schedule: bgi_script::ScriptSchedule::parse(""),
            run_count: 1,
            settings: None,
            allow_notification: true,
            allow_http_hash: None,
            target_path: None,
            manifest_main: Some("main.js".to_string()),
            skipped: false,
        };
        let mut prepared = PreparedScriptExecution::prepare_javascript(&step, &root).unwrap();
        prepared.host_runtime_config.capture_area = bgi_script::GameCaptureArea {
            x: 1,
            y: 0,
            width: 2,
            height: 2,
        };
        prepared.host_runtime_config.capture_frame_source = Some(Arc::new(StaticFrameSource {
            frame: CaptureFrame::packed_bgr(
                4,
                2,
                vec![
                    1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, //
                    5, 5, 5, 6, 6, 6, 7, 7, 7, 8, 8, 8,
                ],
            )
            .unwrap(),
            area: bgi_script::GameCaptureArea {
                x: 1,
                y: 0,
                width: 2,
                height: 2,
            },
        }));
        let host = ScriptHostRuntime::new(prepared.host_runtime_config.clone()).unwrap();
        let outcome = execute_prepared_javascript_with_host(
            &prepared,
            TaskInvocationExecutionMode::PlanOnly,
            None,
            None,
            host,
        )
        .unwrap();

        assert_eq!(
            outcome.result,
            Some(json!("2x2:2,2,2,3,3,3,6,6,6,7,7,7:DerivedCrop"))
        );
        let capture_call = outcome
            .host_calls
            .iter()
            .find(|call| call.target == ScriptHostTarget::Global)
            .unwrap();
        assert_eq!(capture_call.result["pixel_format"], json!("BGR24"));
        assert_eq!(capture_call.result["source_width"], json!(4));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn executes_javascript_image_mat_io_with_real_bgr_payload() {
        let root = test_root("image-mat-io");
        let project = root.join("demo");
        fs::create_dir_all(project.join("assets")).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        let source = BgrImage::new(VisionSize::new(2, 1), vec![11, 22, 33, 44, 55, 66]).unwrap();
        source.write_png(project.join("assets/source.png")).unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                const mat = file.readImageMatSync("assets/source.png");
                const write = file.writeImageSync("assets/copy", mat);
                mat.width + mat.height + mat.pixels[0] + write.width;
            "#,
        )
        .unwrap();

        let outcome = execute_javascript_project(&root, "demo", None).unwrap();
        let copied = BgrImage::read(project.join("assets/copy.png")).unwrap();
        let read_call = outcome
            .host_calls
            .iter()
            .find(|call| call.method == "readImageMatSync")
            .unwrap();
        let write_call = outcome
            .host_calls
            .iter()
            .find(|call| call.method == "writeImageSync")
            .unwrap();

        assert_eq!(outcome.result, Some(json!(16)));
        assert_eq!(read_call.result["pixel_format"], json!("BGR24"));
        assert_eq!(read_call.result["width"], json!(2));
        assert_eq!(read_call.result["pixels"], json!([11, 22, 33, 44, 55, 66]));
        assert_eq!(
            write_call.result["normalized_path"],
            serde_json::to_value(project.join("assets").join("copy.png")).unwrap()
        );
        assert_eq!(copied, source);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn executes_javascript_vision_matching_with_mat_payloads() {
        let root = test_root("vision-matching");
        let project = root.join("demo");
        fs::create_dir_all(project.join("assets")).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        let source = BgrImage::new(
            VisionSize::new(3, 3),
            vec![
                1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 40, 40, 40, 50, 50, 50, 6, 6, 6, 70, 70, 70,
                80, 80, 80,
            ],
        )
        .unwrap();
        let template = BgrImage::new(
            VisionSize::new(2, 2),
            vec![40, 40, 40, 50, 50, 50, 70, 70, 70, 80, 80, 80],
        )
        .unwrap();
        source.write_png(project.join("assets/source.png")).unwrap();
        template
            .write_png(project.join("assets/template.png"))
            .unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                const source = file.readImageMatSync("assets/source.png");
                const template = file.readImageMatSync("assets/template.png");
                const cropped = vision.crop(source, { x: 1, y: 1, width: 2, height: 2 });
                const hit = vision.findTemplate(cropped, template, {
                    threshold: 0.99,
                    use3Channels: true,
                    mode: "CCorrNormed",
                    maxMatchCount: 1,
                    name: "patch"
                });
                const color = vision.findColor(cropped, {
                    conversion: "BgrToRgb",
                    lowerColor: [40, 40, 40],
                    upperColor: [40, 40, 40],
                    matchCount: 1,
                    name: "gray"
                });
                cropped.width + hit.first.rect.x + hit.first.rect.y + hit.matches.length + color.first.rect.width;
            "#,
        )
        .unwrap();

        let outcome = execute_javascript_project(&root, "demo", None).unwrap();
        let crop_call = outcome
            .host_calls
            .iter()
            .find(|call| call.target == ScriptHostTarget::Vision && call.method == "crop")
            .unwrap();
        let template_call = outcome
            .host_calls
            .iter()
            .find(|call| call.target == ScriptHostTarget::Vision && call.method == "findTemplate")
            .unwrap();
        let color_call = outcome
            .host_calls
            .iter()
            .find(|call| call.target == ScriptHostTarget::Vision && call.method == "findColor")
            .unwrap();

        assert_eq!(outcome.result, Some(json!(4)));
        assert_eq!(crop_call.result["width"], json!(2));
        assert_eq!(crop_call.result["height"], json!(2));
        assert_eq!(crop_call.result["pixel_format"], json!("BGR24"));
        assert_eq!(
            template_call.result["recognition_type"],
            json!("TemplateMatch")
        );
        assert_eq!(template_call.result["first"]["rect"]["x"], json!(0));
        assert_eq!(template_call.result["first"]["rect"]["y"], json!(0));
        assert_eq!(template_call.result["matched_count"], json!(1));
        assert_eq!(color_call.result["recognition_type"], json!("ColorMatch"));
        assert_eq!(color_call.result["first"]["text"], json!("gray"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_javascript_errors() {
        let root = test_root("error");
        let project = root.join("demo");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(project.join("main.js"), "throw new Error('boom');").unwrap();

        let error = execute_javascript_project(&root, "demo", None).unwrap_err();
        assert!(error.to_string().contains("boom"));
    }

    #[test]
    fn executes_standard_modules_with_relative_imports() {
        let root = test_root("module");
        let project = root.join("demo");
        let lib = project.join("lib");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&lib).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(lib.join("math.js"), "export const bonus = 40;").unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                import { bonus } from './lib/math.js';
                log.info("module " + settings.name);
                globalThis.__bgiModuleResult = bonus + settings.level;
            "#,
        )
        .unwrap();

        let outcome =
            execute_javascript_project(&root, "demo", Some(json!({"name": "ok", "level": 2})))
                .unwrap();

        assert_eq!(
            outcome.execution_mode,
            ScriptCodeExecutionMode::StandardModule
        );
        assert_eq!(outcome.result, None);
        assert_eq!(outcome.logs.len(), 1);
        assert_eq!(outcome.logs[0].message, "module ok");
    }

    #[test]
    fn executes_standard_modules_with_shared_module_record_cache() {
        let root = test_root("module-cache");
        let project = root.join("demo");
        let lib = project.join("lib");
        fs::create_dir_all(&lib).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(
            lib.join("shared.js"),
            r#"
                globalThis.__sharedRuns = (globalThis.__sharedRuns ?? 0) + 1;
                export const value = 9;
            "#,
        )
        .unwrap();
        fs::write(
            lib.join("a.js"),
            "import { value } from './shared.js'; export const a = value + 1;",
        )
        .unwrap();
        fs::write(
            lib.join("b.js"),
            "import { value } from './shared.js'; export const b = value + 2;",
        )
        .unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                import { a } from './lib/a.js';
                import { b } from './lib/b.js';
                log.info(`${a}:${b}:${globalThis.__sharedRuns}`);
            "#,
        )
        .unwrap();

        let outcome = execute_javascript_project(&root, "demo", None).unwrap();

        assert_eq!(outcome.logs[0].message, "10:11:1");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn executes_standard_modules_with_circular_imports() {
        let root = test_root("module-cycle");
        let project = root.join("demo");
        let lib = project.join("lib");
        fs::create_dir_all(&lib).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(
            lib.join("a.js"),
            r#"
                import { fromB } from './b.js';
                export const valueA = 'A';
                export function fromA() {
                    return valueA + fromB();
                }
            "#,
        )
        .unwrap();
        fs::write(
            lib.join("b.js"),
            r#"
                import { valueA } from './a.js';
                export function fromB() {
                    return valueA + 'B';
                }
            "#,
        )
        .unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                import { fromA } from './lib/a.js';
                log.info(fromA());
            "#,
        )
        .unwrap();

        let outcome = execute_javascript_project(&root, "demo", None).unwrap();

        assert_eq!(outcome.logs[0].message, "AAB");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn executes_standard_modules_with_import_meta_path_rewrite() {
        let root = test_root("module-import-meta");
        let project = root.join("demo");
        let lib = project.join("lib");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&lib).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(
            lib.join("where.js"),
            "export const where = import.meta.dirname + ':' + import.meta.url;",
        )
        .unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                import { where } from './lib/where.js';
                log.info(import.meta.url + "|" + where);
            "#,
        )
        .unwrap();

        let outcome = execute_javascript_project(&root, "demo", None).unwrap();

        assert_eq!(outcome.logs[0].message, "main.js|lib:lib/where.js");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn executes_standard_modules_with_import_meta_object_properties() {
        let root = test_root("module-import-meta-object");
        let project = root.join("demo");
        let lib = project.join("lib");
        fs::create_dir_all(&lib).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(
            lib.join("meta.js"),
            r#"
                const meta = import.meta;
                const { url, dirname } = meta;
                export const where = `${dirname}:${url}`;
            "#,
        )
        .unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                import { where } from './lib/meta.js';
                const meta = import.meta;
                log.info(`${meta.dirname}:${meta.url}|${where}`);
            "#,
        )
        .unwrap();

        let outcome = execute_javascript_project(&root, "demo", None).unwrap();

        assert_eq!(outcome.logs[0].message, ":main.js|lib:lib/meta.js");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn executes_standard_modules_with_nested_referrer_paths_and_resources() {
        let root = test_root("module-nested-referrer");
        let project = root.join("demo");
        let nested = project.join("lib").join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(nested.join("note.txt"), "nested note").unwrap();
        fs::write(nested.join("value.js"), "export const value = 7;").unwrap();
        fs::write(
            nested.join("entry.js"),
            r#"
                import { value } from './value.js';
                import note from './note.txt';
                export const combined = `${import.meta.dirname}:${import.meta.url}:${note}:${value}`;
            "#,
        )
        .unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                import { combined } from './lib/nested/entry.js';
                log.info(combined);
            "#,
        )
        .unwrap();

        let outcome = execute_javascript_project(&root, "demo", None).unwrap();

        assert_eq!(
            outcome.logs[0].message,
            "lib/nested:lib/nested/entry.js:nested note:7"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn executes_standard_modules_with_legacy_shaped_resource_imports() {
        let root = test_root("module-resource-imports");
        let project = root.join("demo");
        fs::create_dir_all(project.join("assets")).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(project.join("assets/config.json"), r#"{"enabled":true}"#).unwrap();
        fs::write(project.join("assets/template.txt"), "template body").unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                import config, { unused } from './assets/config.json';
                import * as template from './assets/template.txt';
                log.info(`${config}:${template}`);
            "#,
        )
        .unwrap();

        let outcome = execute_javascript_project(&root, "demo", None).unwrap();

        assert_eq!(outcome.logs[0].message, r#"{"enabled":true}:template body"#);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn executes_script_group_in_order_and_honors_run_count() {
        let app_root = test_root("group");
        let scripts_root = app_root.join("User").join("JsScript");
        write_js_project(
            &scripts_root,
            "first",
            "First",
            r#"
                log.info("first " + settings.level);
                "first";
            "#,
        );
        write_js_project(
            &scripts_root,
            "second",
            "Second",
            r#"
                console.log("second");
                "second";
            "#,
        );

        let group = ScriptGroup {
            name: "daily".to_string(),
            config: ScriptGroupConfig {
                pathing_config: json!({"partyName": "team-a"}),
                ..ScriptGroupConfig::default()
            },
            projects: vec![
                ScriptGroupProject {
                    index: 20,
                    name: "First".to_string(),
                    folder_name: "first".to_string(),
                    project_type: ScriptProjectType::Javascript,
                    js_script_settings_object: Some(json!({"level": 2})),
                    ..ScriptGroupProject::default()
                },
                ScriptGroupProject {
                    index: 10,
                    name: "Second".to_string(),
                    folder_name: "second".to_string(),
                    project_type: ScriptProjectType::Javascript,
                    run_num: 2,
                    ..ScriptGroupProject::default()
                },
            ],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_for_test(&app_root, &group);

        assert_eq!(outcome.requested_projects, 2);
        assert_eq!(outcome.attempted_steps, 3);
        assert_eq!(outcome.completed_steps, 3);
        assert_eq!(outcome.failed_steps, 0);
        assert_eq!(outcome.steps.len(), 3);
        assert_eq!(outcome.steps[0].name, "Second");
        assert_eq!(outcome.steps[0].run_iteration, 1);
        assert_eq!(outcome.steps[1].name, "Second");
        assert_eq!(outcome.steps[1].run_iteration, 2);
        assert_eq!(outcome.steps[2].name, "First");
        assert_eq!(
            outcome.steps[2].javascript.as_ref().unwrap().logs[0].message,
            "first 2"
        );
    }

    #[test]
    fn script_group_records_failures_and_continues() {
        let app_root = test_root("group-failure");
        let scripts_root = app_root.join("User").join("JsScript");
        write_js_project(&scripts_root, "bad", "Bad", "throw new Error('boom');");
        write_js_project(&scripts_root, "good", "Good", r#""good";"#);

        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![
                ScriptGroupProject {
                    index: 1,
                    name: "Bad".to_string(),
                    folder_name: "bad".to_string(),
                    project_type: ScriptProjectType::Javascript,
                    ..ScriptGroupProject::default()
                },
                ScriptGroupProject {
                    index: 2,
                    name: "Good".to_string(),
                    folder_name: "good".to_string(),
                    project_type: ScriptProjectType::Javascript,
                    ..ScriptGroupProject::default()
                },
            ],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_for_test(&app_root, &group);

        assert_eq!(outcome.attempted_steps, 2);
        assert_eq!(outcome.failed_steps, 1);
        assert_eq!(outcome.completed_steps, 1);
        assert_eq!(outcome.steps[0].status, ScriptGroupStepStatus::Failed);
        assert!(outcome.steps[0].error.as_deref().unwrap().contains("boom"));
        assert_eq!(outcome.steps[1].status, ScriptGroupStepStatus::Completed);
        assert_eq!(
            outcome.steps[1].javascript.as_ref().unwrap().result,
            Some(json!("good"))
        );
    }

    #[test]
    fn script_group_executes_shell_steps() {
        let app_root = test_root("group-shell");
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![ScriptGroupProject {
                index: 1,
                name: shell_echo_command("group-shell-ok"),
                project_type: ScriptProjectType::Shell,
                ..ScriptGroupProject::default()
            }],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_for_test(&app_root, &group);

        assert_eq!(outcome.attempted_steps, 1);
        assert_eq!(outcome.completed_steps, 1);
        assert_eq!(outcome.failed_steps, 0);
        let shell = outcome.steps[0].shell_result.as_ref().unwrap();
        assert_eq!(shell.status, bgi_task::ShellExecutionStatus::Completed);
        assert!(
            format!("{}\n{}", shell.output_shell, shell.output).contains("group-shell-ok"),
            "unexpected shell output: {:?}",
            shell
        );
    }

    #[test]
    fn script_group_resume_execution_skips_projects_before_pointer() {
        let app_root = test_root("group-resume-shell");
        let resume_command = shell_echo_command("resume-here");
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![
                ScriptGroupProject {
                    index: 1,
                    name: shell_echo_command("before-resume"),
                    project_type: ScriptProjectType::Shell,
                    ..ScriptGroupProject::default()
                },
                ScriptGroupProject {
                    index: 2,
                    name: resume_command.clone(),
                    project_type: ScriptProjectType::Shell,
                    ..ScriptGroupProject::default()
                },
            ],
            ..ScriptGroup::default()
        };
        let pointer = ScriptGroupResumePointer {
            group_name: "daily".to_string(),
            project_index: 2,
            folder_name: String::new(),
            project_name: resume_command,
        };
        let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

        let outcome = execute_script_group_from_resume_with_task_dispatcher_hooks_and_cancellation(
            &roots,
            &group,
            &pointer,
            None,
            None,
            |_| {},
            |_| {},
        );

        assert_eq!(outcome.requested_projects, 2);
        assert_eq!(outcome.skipped_steps, 1);
        assert_eq!(outcome.completed_steps, 1);
        assert_eq!(outcome.steps[0].status, ScriptGroupStepStatus::Skipped);
        assert_eq!(outcome.steps[1].status, ScriptGroupStepStatus::Completed);
        assert!(outcome.steps[1]
            .shell_result
            .as_ref()
            .unwrap()
            .output
            .contains("resume-here"));
        fs::remove_dir_all(app_root).unwrap();
    }

    #[test]
    fn script_group_shell_cancellation_stops_group_execution() {
        let app_root = test_root("group-shell-cancel");
        let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
        roots.key_mouse_dispatch_mode = KeyMouseScriptDispatchMode::PlanOnly;
        roots.http_dispatch_mode = HttpDispatchMode::PlanOnly;
        roots.notification_dispatch_mode = NotificationDispatchMode::RecordOnly;
        let cancellation = InputCancellationToken::new();
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![
                ScriptGroupProject {
                    index: 1,
                    name: shell_sleep_command(2),
                    project_type: ScriptProjectType::Shell,
                    ..ScriptGroupProject::default()
                },
                ScriptGroupProject {
                    index: 2,
                    name: shell_echo_command("after-shell-cancel"),
                    project_type: ScriptProjectType::Shell,
                    ..ScriptGroupProject::default()
                },
            ],
            ..ScriptGroup::default()
        };

        let outcome = std::thread::scope(|scope| {
            scope.spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(50));
                cancellation.cancel();
            });
            execute_script_group_with_task_dispatcher_hooks_and_cancellation(
                &roots,
                &group,
                None,
                Some(&cancellation),
                |_| {},
                |_| {},
            )
        });

        assert_eq!(outcome.attempted_steps, 1);
        assert_eq!(outcome.cancelled_steps, 1);
        assert_eq!(outcome.steps.len(), 1);
        assert_eq!(outcome.steps[0].status, ScriptGroupStepStatus::Cancelled);
        let shell = outcome.steps[0].shell_result.as_ref().unwrap();
        assert_eq!(shell.status, ShellExecutionStatus::Cancelled);
        assert!(shell.waited_for_exit);
    }

    #[test]
    fn script_group_classic_javascript_cancellation_stops_group_execution() {
        let app_root = test_root("group-js-cancel");
        let scripts_root = app_root.join("User").join("JsScript");
        write_js_project(
            &scripts_root,
            "infinite",
            "Infinite",
            r#"
                while (true) {
                  Math.random();
                }
            "#,
        );
        let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
        roots.http_dispatch_mode = HttpDispatchMode::PlanOnly;
        roots.notification_dispatch_mode = NotificationDispatchMode::RecordOnly;
        let cancellation = InputCancellationToken::new();
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![
                ScriptGroupProject {
                    index: 1,
                    name: "Infinite".to_string(),
                    folder_name: "infinite".to_string(),
                    project_type: ScriptProjectType::Javascript,
                    ..ScriptGroupProject::default()
                },
                ScriptGroupProject {
                    index: 2,
                    name: shell_echo_command("after-js-cancel"),
                    project_type: ScriptProjectType::Shell,
                    ..ScriptGroupProject::default()
                },
            ],
            ..ScriptGroup::default()
        };

        let outcome = std::thread::scope(|scope| {
            scope.spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(50));
                cancellation.cancel();
            });
            execute_script_group_with_task_dispatcher_hooks_and_cancellation(
                &roots,
                &group,
                None,
                Some(&cancellation),
                |_| {},
                |_| {},
            )
        });

        assert_eq!(outcome.attempted_steps, 1);
        assert_eq!(outcome.cancelled_steps, 1);
        assert_eq!(outcome.failed_steps, 0);
        assert_eq!(outcome.steps.len(), 1);
        assert_eq!(outcome.steps[0].status, ScriptGroupStepStatus::Cancelled);
        assert!(outcome.steps[0]
            .error
            .as_deref()
            .unwrap()
            .contains("cancelled"));
        assert!(outcome.steps[0].javascript.is_none());
        assert!(outcome.steps[0].shell_result.is_none());
        fs::remove_dir_all(app_root).unwrap();
    }

    #[test]
    fn script_group_project_execution_runs_only_selected_project_once() {
        let app_root = test_root("group-project-shell");
        let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
        roots.key_mouse_dispatch_mode = KeyMouseScriptDispatchMode::PlanOnly;
        roots.http_dispatch_mode = HttpDispatchMode::PlanOnly;
        roots.notification_dispatch_mode = NotificationDispatchMode::RecordOnly;
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![
                ScriptGroupProject {
                    index: 2,
                    name: shell_echo_command("selected-project"),
                    project_type: ScriptProjectType::Shell,
                    run_num: 3,
                    ..ScriptGroupProject::default()
                },
                ScriptGroupProject {
                    index: 1,
                    name: shell_echo_command("not-selected"),
                    project_type: ScriptProjectType::Shell,
                    ..ScriptGroupProject::default()
                },
            ],
            ..ScriptGroup::default()
        };

        let outcome =
            execute_script_group_project_with_host_hooks(&roots, &group, 0, |_| {}, |_| {})
                .unwrap();

        assert_eq!(outcome.requested_projects, 1);
        assert_eq!(outcome.attempted_steps, 1);
        assert_eq!(outcome.completed_steps, 1);
        assert_eq!(outcome.steps.len(), 1);
        assert_eq!(outcome.steps[0].project_index, 0);
        assert_eq!(outcome.steps[0].project_order, 2);
        assert_eq!(outcome.steps[0].run_iteration, 1);
        assert_eq!(outcome.steps[0].run_count, 3);
        let shell = outcome.steps[0].shell_result.as_ref().unwrap();
        assert!(!format!("{}\n{}", shell.output_shell, shell.output).contains("not-selected"));
        assert!(format!("{}\n{}", shell.output_shell, shell.output).contains("selected-project"));
    }

    #[test]
    fn script_group_project_repeated_execution_honors_run_count() {
        let app_root = test_root("group-project-repeated-shell");
        let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
        roots.key_mouse_dispatch_mode = KeyMouseScriptDispatchMode::PlanOnly;
        roots.http_dispatch_mode = HttpDispatchMode::PlanOnly;
        roots.notification_dispatch_mode = NotificationDispatchMode::RecordOnly;
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![ScriptGroupProject {
                index: 1,
                name: shell_echo_command("repeat-project"),
                project_type: ScriptProjectType::Shell,
                run_num: 3,
                ..ScriptGroupProject::default()
            }],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_project_repeated_with_task_dispatcher_hooks(
            &roots,
            &group,
            0,
            None,
            |_| {},
            |_| {},
        )
        .unwrap();

        assert_eq!(outcome.requested_projects, 1);
        assert_eq!(outcome.attempted_steps, 3);
        assert_eq!(outcome.completed_steps, 3);
        assert_eq!(outcome.steps.len(), 3);
        assert_eq!(
            outcome
                .steps
                .iter()
                .map(|step| step.run_iteration)
                .collect::<Vec<_>>(),
            [1, 2, 3]
        );
        assert!(outcome
            .steps
            .iter()
            .all(|step| step.run_count == 3 && step.shell_result.is_some()));
    }

    #[test]
    fn script_group_key_mouse_steps_can_run_in_plan_only_mode() {
        let app_root = test_root("group-keymouse");
        let macro_root = app_root.join("User").join("KeyMouseScript");
        fs::create_dir_all(&macro_root).unwrap();
        fs::write(
            macro_root.join("macro.json"),
            r#"{
              "macroEvents": [
                { "type": 0, "keyCode": 87, "time": 10 },
                { "type": 1, "keyCode": 87, "time": 30 }
              ]
            }"#,
        )
        .unwrap();
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![ScriptGroupProject {
                index: 1,
                name: "macro.json".to_string(),
                folder_name: "macro.json".to_string(),
                project_type: ScriptProjectType::KeyMouse,
                ..ScriptGroupProject::default()
            }],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_for_test(&app_root, &group);

        assert_eq!(outcome.attempted_steps, 1);
        assert_eq!(outcome.planned_steps, 1);
        assert_eq!(outcome.failed_steps, 0);
        let execution = outcome.steps[0].key_mouse_execution.as_ref().unwrap();
        assert_eq!(execution.mode, KeyMouseScriptDispatchMode::PlanOnly);
        assert!(!execution.dispatched);
        assert_eq!(execution.plan.summary.event_count, 2);
        assert_eq!(execution.plan.input_events.len(), 4);
    }

    #[test]
    fn script_group_key_mouse_cancellation_stops_group_execution() {
        let app_root = test_root("group-keymouse-cancel");
        let macro_root = app_root.join("User").join("KeyMouseScript");
        fs::create_dir_all(&macro_root).unwrap();
        fs::write(
            macro_root.join("macro.json"),
            r#"{
              "macroEvents": [
                { "type": 0, "keyCode": 87, "time": 10 },
                { "type": 1, "keyCode": 87, "time": 30 }
              ]
            }"#,
        )
        .unwrap();
        let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
        roots.key_mouse_dispatch_mode = KeyMouseScriptDispatchMode::SendInput;
        roots.http_dispatch_mode = HttpDispatchMode::PlanOnly;
        roots.notification_dispatch_mode = NotificationDispatchMode::RecordOnly;
        let cancellation = InputCancellationToken::new();
        cancellation.cancel();
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![
                ScriptGroupProject {
                    index: 1,
                    name: "macro.json".to_string(),
                    folder_name: "macro.json".to_string(),
                    project_type: ScriptProjectType::KeyMouse,
                    run_num: 3,
                    ..ScriptGroupProject::default()
                },
                ScriptGroupProject {
                    index: 2,
                    name: shell_echo_command("after-cancel"),
                    project_type: ScriptProjectType::Shell,
                    ..ScriptGroupProject::default()
                },
            ],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_with_task_dispatcher_hooks_and_cancellation(
            &roots,
            &group,
            None,
            Some(&cancellation),
            |_| {},
            |_| {},
        );

        assert_eq!(outcome.attempted_steps, 1);
        assert_eq!(outcome.cancelled_steps, 1);
        assert_eq!(outcome.completed_steps, 0);
        assert_eq!(outcome.steps.len(), 1);
        assert_eq!(outcome.steps[0].status, ScriptGroupStepStatus::Cancelled);
        assert_eq!(outcome.steps[0].run_iteration, 0);
        assert!(outcome.steps[0].key_mouse_execution.is_none());
        assert!(outcome.steps[0].shell_result.is_none());
    }

    #[test]
    fn script_group_javascript_key_mouse_host_uses_configured_dispatch_mode() {
        let app_root = test_root("group-js-keymouse-host");
        let scripts_root = app_root.join("User").join("JsScript");
        write_js_project(
            &scripts_root,
            "macro-host",
            "MacroHost",
            r#"
                const result = keyMouseScript.run(JSON.stringify({
                    macroEvents: [
                        { type: 0, keyCode: 87, time: 10 },
                        { type: 1, keyCode: 87, time: 30 }
                    ]
                }));
                log.info(result.mode + ":" + result.dispatched);
                result.dispatched_events;
            "#,
        );
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![ScriptGroupProject {
                index: 1,
                name: "MacroHost".to_string(),
                folder_name: "macro-host".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            }],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_for_test(&app_root, &group);

        assert_eq!(outcome.completed_steps, 1);
        let javascript = outcome.steps[0].javascript.as_ref().unwrap();
        assert_eq!(javascript.result, Some(json!(0)));
        assert_eq!(javascript.logs[0].message, "PlanOnly:false");
        assert!(matches!(
            javascript.host_calls[0].result["mode"],
            serde_json::Value::String(ref mode) if mode == "PlanOnly"
        ));
        assert_eq!(javascript.host_calls[0].result["dispatched"], false);
    }

    #[test]
    fn script_group_javascript_pathing_host_returns_execution_plan() {
        let app_root = test_root("group-js-pathing-host");
        let scripts_root = app_root.join("User").join("JsScript");
        write_js_project(
            &scripts_root,
            "pathing-host",
            "PathingHost",
            r#"
                const route = JSON.stringify({
                    info: { name: "route", type: "collect", map_name: "Teyvat" },
                    positions: [
                        { x: 1, y: 2, type: "path" },
                        { x: 3, y: 4, type: "teleport" },
                        { x: 5, y: 6, type: "target", action: "fight" }
                    ]
                });
                const execution = pathingScript.run(route);
                const plan = pathingScript.plan(route);
                log.info(
                    execution.dispatched + ":" +
                    execution.execution_plan.segment_count + ":" +
                    execution.execution_plan.waypoint_count + ":" +
                    plan.summary.waypoint_count
                );
                execution.execution_plan.expected_fight_count;
            "#,
        );
        let group = ScriptGroup {
            name: "daily".to_string(),
            config: ScriptGroupConfig {
                pathing_config: json!({"partyName": "daily"}),
                ..ScriptGroupConfig::default()
            },
            projects: vec![ScriptGroupProject {
                index: 1,
                name: "PathingHost".to_string(),
                folder_name: "pathing-host".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            }],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_for_test(&app_root, &group);

        assert_eq!(outcome.completed_steps, 1);
        let javascript = outcome.steps[0].javascript.as_ref().unwrap();
        assert_eq!(javascript.result, Some(json!(1)));
        assert_eq!(javascript.logs[0].message, "false:2:3:3");
        assert_eq!(javascript.host_calls.len(), 3);
        assert_eq!(
            javascript.host_calls[0].target,
            ScriptHostTarget::PathingScript
        );
        assert_eq!(javascript.host_calls[0].method, "run");
        assert_eq!(javascript.host_calls[0].result["dispatched"], false);
        assert_eq!(
            javascript.host_calls[0].result["execution_plan"]["segment_count"],
            2
        );
        assert_eq!(javascript.host_calls[1].method, "plan");
        assert_eq!(
            javascript.host_calls[1].result["party_config"],
            json!({"partyName": "daily"})
        );
        assert_eq!(javascript.host_calls[2].target, ScriptHostTarget::Log);
    }

    #[test]
    fn script_group_javascript_http_host_uses_plan_only_in_tests() {
        let app_root = test_root("group-js-http-host");
        let scripts_root = app_root.join("User").join("JsScript");
        write_js_project_with_manifest(
            &scripts_root,
            "http-host",
            r#"{
              "name": "HttpHost",
              "version": "1.0",
              "main": "main.js",
              "httpAllowedUrls": ["https://example.com/*"]
            }"#,
            r#"
                const request = http.request(
                    "POST",
                    "https://example.com/status",
                    JSON.stringify({ ok: true }),
                    JSON.stringify({ "Content-Type": "application/json", "X-Test": "1" })
                );
                log.info(request.method + ":" + request.content_type);
                request.headers.length;
            "#,
        );
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![ScriptGroupProject {
                index: 1,
                name: "HttpHost".to_string(),
                folder_name: "http-host".to_string(),
                project_type: ScriptProjectType::Javascript,
                allow_js_http_hash: Some("https://example.com/*".to_string()),
                ..ScriptGroupProject::default()
            }],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_for_test(&app_root, &group);

        assert_eq!(outcome.completed_steps, 1);
        let javascript = outcome.steps[0].javascript.as_ref().unwrap();
        assert_eq!(javascript.result, Some(json!(1)));
        assert_eq!(javascript.logs[0].message, "POST:application/json");
        assert_eq!(javascript.host_calls[0].target, ScriptHostTarget::Http);
        assert_eq!(javascript.host_calls[0].result["method"], "POST");
        assert_eq!(
            javascript.host_calls[0].result["url"],
            "https://example.com/status"
        );
    }

    #[test]
    fn javascript_can_poll_html_mask_messages_from_initial_state() {
        let root = test_root("html-mask-initial-state");
        write_js_project(
            &root,
            "demo",
            "HtmlMaskBridge",
            r#"
                const message = htmlMask.poll("mask");
                message && message.includes("from-html");
            "#,
        );
        let step = ScriptExecutionStep {
            index: 1,
            name: "HtmlMaskBridge".to_string(),
            folder_name: "demo".to_string(),
            project_type: ScriptProjectType::Javascript,
            engine: bgi_script::ScriptEngineKind::RustJavaScript,
            schedule: bgi_script::ScriptSchedule::parse(""),
            run_count: 1,
            settings: None,
            allow_notification: true,
            allow_http_hash: None,
            target_path: None,
            manifest_main: Some("main.js".to_string()),
            skipped: false,
        };
        let mut prepared = PreparedScriptExecution::prepare_javascript(&step, &root).unwrap();
        prepared.host_runtime_config.html_mask_initial_state = bgi_script::HtmlMaskInitialState {
            windows: vec![bgi_script::HtmlMaskWindowPlan {
                window_id: "mask".to_string(),
                final_url: "https://example.com/mask".to_string(),
                requested_url: "https://example.com/mask".to_string(),
                normalized_path: None,
                click_through: true,
            }],
            from_html: vec![(
                "mask".to_string(),
                bgi_script::HtmlMaskMessage {
                    url: "/from-html".to_string(),
                    data: Some(json!({ "from-html": true })),
                    request_id: Some("req-1".to_string()),
                },
            )],
        };

        let outcome = execute_prepared_javascript(&prepared).unwrap();

        assert_eq!(outcome.result, Some(json!(true)));
        assert!(outcome.html_mask_from_html.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn script_group_javascript_exposes_migrated_host_objects() {
        let app_root = test_root("group-js-host-objects");
        let scripts_root = app_root.join("User").join("JsScript");
        write_js_project(
            &scripts_root,
            "host-objects",
            "HostObjects",
            r#"
                file.writeTextSync("overlay.html", "<html></html>");
                file.writeText("notes.txt", "hello");
                const note = file.readText("notes.txt");
                const macroPlan = keyMouseScript.plan(JSON.stringify({
                    macroEvents: [
                        { type: 0, keyCode: 87, time: 0 },
                        { type: 1, keyCode: 87, time: 50 }
                    ]
                }));
                const timer = dispatcher.addTimer({ name: "tick", config: { enabled: true } });
                const task = dispatcher.runAutoFightTask({ strategyName: "default" });
                const notify = notification.success("ready");
                const post = PostMessage.keyPress("F");
                const offset = ServerTime.getServerTimeZoneOffset();
                const legacyOffset = ServerTime.serverTimeZoneOffsetMilliseconds();
                const mask = htmlMask.show("overlay.html", "mask");
                htmlMask.sendFromHtml("mask", "/event", JSON.stringify({ ok: true }), "req-1");
                const polled = htmlMask.poll("mask");
                const hook = KeyMouseHook.onKeyDown("key", true);
                const hookDispatch = KeyMouseHook.dispatchEvent({ type: "keyDown", keyCode: "F", keyData: "F" });
                const jagged = host.newVarOfArr("System.String", 2);
                genshin.tp(1, 2, true);
                genshin.chooseTalkOption("hello", 1, false);
                genshin.claimEncounterPointsRewards();
                log.info([
                    note,
                    macroPlan.summary.event_count,
                    timer.AddRealtimeTimer.name,
                    task.RunBuiltinTask.name,
                    notify.record.kind,
                    post.length,
                    offset === legacyOffset,
                    mask.Show.final_url.length > 0,
                    polled.includes("ok"),
                    hook.AddListener.id,
                    hookDispatch.length,
                    jagged.NewArrayVariable.element_type
                ].join(":"));
                dispatcher.commands().length + genshin.commands().length;
            "#,
        );
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![ScriptGroupProject {
                index: 1,
                name: "HostObjects".to_string(),
                folder_name: "host-objects".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            }],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_for_test(&app_root, &group);

        assert_eq!(outcome.completed_steps, 1);
        let javascript = outcome.steps[0].javascript.as_ref().unwrap();
        assert_eq!(javascript.result, Some(json!(6)));
        assert_eq!(
            javascript.logs[0].message,
            "hello:2:tick:AutoFight:Success:4:true:true:true:key:1:System.String"
        );
        assert!(javascript
            .host_calls
            .iter()
            .any(|call| call.target == ScriptHostTarget::HtmlMask && call.method == "show"));
        assert!(javascript
            .host_calls
            .iter()
            .any(|call| call.target == ScriptHostTarget::CustomHostFunctions
                && call.method == "newVarOfArr"));
        assert_eq!(javascript.task_invocations.dispatcher.len(), 2);
        assert_eq!(
            javascript.task_invocations.dispatcher[0].kind,
            bgi_task::TaskInvocationKind::ClearRealtimeTriggers
        );
        assert_eq!(
            javascript.task_invocations.dispatcher[1]
                .task_key
                .as_deref(),
            Some("AutoFight")
        );
        assert_eq!(javascript.task_invocations.genshin.len(), 3);
        assert!(javascript
            .task_invocations
            .genshin
            .iter()
            .any(|plan| plan.task_key.as_deref() == Some("Teleport")));
        assert!(javascript
            .task_invocations
            .genshin
            .iter()
            .any(|plan| plan.task_key.as_deref() == Some("ChooseTalkOption")));
        assert!(javascript
            .task_invocations
            .genshin
            .iter()
            .any(|plan| plan.task_key.as_deref() == Some("ClaimEncounterPointsRewards")));
        assert!(javascript
            .task_invocations
            .errors
            .iter()
            .any(|error| error.contains("dispatcher[1]") && error.contains("tick")));
        assert_eq!(
            javascript.task_execution.mode,
            TaskInvocationExecutionMode::PlanOnly
        );
        assert_eq!(javascript.task_execution.total(), 5);
        assert_eq!(
            javascript.task_execution.dispatcher[0].status,
            bgi_task::TaskInvocationExecutionStatus::Planned
        );
        assert!(javascript
            .task_execution
            .dispatcher
            .iter()
            .chain(javascript.task_execution.genshin.iter())
            .any(|result| result.status == bgi_task::TaskInvocationExecutionStatus::NativePending));
    }

    #[test]
    fn prepared_javascript_can_apply_dispatcher_timer_invocations_to_runtime() {
        let root = test_root("dispatcher-runtime");
        let project = root.join("demo");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                dispatcher.clearAllTriggers();
                dispatcher.addTimer({ name: "AutoPick", interval: 250, config: { source: "script" } });
                dispatcher.commands().length;
            "#,
        )
        .unwrap();
        let prepared = prepare_javascript_project(&root, "demo", None).unwrap();
        let mut dispatcher = DispatcherRuntime {
            frame_index: 7,
            ..DispatcherRuntime::default()
        };

        let outcome =
            execute_prepared_javascript_with_task_dispatcher(&prepared, &mut dispatcher).unwrap();

        assert_eq!(outcome.result, Some(json!(3)));
        assert_eq!(
            outcome.task_execution.mode,
            TaskInvocationExecutionMode::ExecuteReady
        );
        assert_eq!(outcome.task_execution.dispatcher.len(), 3);
        assert!(outcome
            .task_execution
            .dispatcher
            .iter()
            .all(|result| result.executed));
        assert_eq!(dispatcher.registered_realtime_triggers.len(), 1);
        assert_eq!(
            dispatcher.registered_realtime_triggers[0].task_key,
            "AutoPick"
        );
        assert_eq!(dispatcher.registered_realtime_triggers[0].interval_ms, 250);
        assert_eq!(
            dispatcher.registered_realtime_triggers[0].config,
            Some(json!({"ForceInteraction": false, "TextList": []}))
        );
        assert_eq!(
            dispatcher.registered_realtime_triggers[0].registered_at_frame,
            7
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn script_group_javascript_accepts_legacy_pascal_case_host_member_aliases() {
        let app_root = test_root("group-js-host-aliases");
        let scripts_root = app_root.join("User").join("JsScript");
        write_js_project(
            &scripts_root,
            "host-aliases",
            "HostAliases",
            r#"
                file.WriteTextSync("overlay.html", "<html></html>");
                file.WriteText("notes.txt", "hello");
                const note = file.ReadText("notes.txt");
                const macroPlan = keyMouseScript.Plan(JSON.stringify({
                    macroEvents: [
                        { type: 0, keyCode: 87, time: 0 },
                        { type: 1, keyCode: 87, time: 50 }
                    ]
                }));
                const timer = dispatcher.AddTimer({ name: "tick", config: { enabled: true } });
                const task = dispatcher.RunAutoFightTask({ strategyName: "default" });
                const notify = notification.Success("ready");
                const post = PostMessage.KeyPress("F");
                const offset = ServerTime.GetServerTimeZoneOffset();
                const legacyOffset = ServerTime.ServerTimeZoneOffsetMilliseconds();
                const mask = htmlMask.Show("overlay.html", "mask");
                htmlMask.SendFromHtml("mask", "/event", JSON.stringify({ ok: true }), "req-1");
                const polled = htmlMask.Poll("mask");
                const hook = KeyMouseHook.OnKeyDown("key", true);
                const hookDispatch = KeyMouseHook.DispatchEvent({ type: "KeyDown", keyCode: "F", keyData: "F" });
                const jagged = host.NewVarOfArr("System.String", 2);
                const obj = host.NewObj("System.Text.StringBuilder", "hello");
                const del = host.DelObj(obj);
                const type = host.Type("System.String");
                const iterator = host.ToIterator([1, 2, 3]);
                genshin.Tp(1, 2, true);
                genshin.ChooseTalkOption("hello", 1, false);
                genshin.ClaimEncounterPointsRewards();
                const metrics = GetGameMetrics();
                log.Info([
                    metrics.width,
                    note,
                    macroPlan.summary.event_count,
                    timer.AddRealtimeTimer.name,
                    task.RunBuiltinTask.name,
                    notify.record.kind,
                    post.length,
                    offset === legacyOffset,
                    mask.Show.final_url.length > 0,
                    polled.includes("ok"),
                    hook.AddListener.id,
                    hookDispatch.length,
                    jagged.NewArrayVariable.element_type,
                    obj.NewObject.type_name,
                    del.DeleteObject.target.NewObject.type_name,
                    type.TypeLookup.type_name,
                    iterator.ToIterator.source.length
                ].join(":"));
                dispatcher.Commands().length + genshin.Commands().length;
            "#,
        );
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![ScriptGroupProject {
                index: 1,
                name: "HostAliases".to_string(),
                folder_name: "host-aliases".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            }],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_for_test(&app_root, &group);

        assert_eq!(outcome.completed_steps, 1);
        let javascript = outcome.steps[0].javascript.as_ref().unwrap();
        assert_eq!(javascript.result, Some(json!(6)));
        assert_eq!(
            javascript.logs[0].message,
            "1920:hello:2:tick:AutoFight:Success:4:true:true:true:key:1:System.String:System.Text.StringBuilder:System.Text.StringBuilder:System.String:3"
        );
        assert!(
            javascript
                .host_calls
                .iter()
                .any(|call| call.target == ScriptHostTarget::Global
                    && call.method == "getGameMetrics")
        );
        assert!(javascript
            .host_calls
            .iter()
            .any(|call| call.target == ScriptHostTarget::Log && call.method == "info"));
        assert!(javascript.host_calls.iter().any(|call| call.target
            == ScriptHostTarget::KeyMouseHook
            && call.method == "onKeyDown"));
        assert!(javascript
            .host_calls
            .iter()
            .any(|call| call.target == ScriptHostTarget::CustomHostFunctions
                && call.method == "newObj"));
        assert!(javascript
            .host_calls
            .iter()
            .any(|call| call.target == ScriptHostTarget::CustomHostFunctions
                && call.method == "toIterator"));
        assert_eq!(javascript.task_invocations.dispatcher.len(), 2);
        assert_eq!(javascript.task_invocations.genshin.len(), 3);
    }

    #[test]
    fn script_group_javascript_exposes_legacy_scheduler_type_constructors() {
        let app_root = test_root("group-js-scheduler-types");
        let scripts_root = app_root.join("User").join("JsScript");
        write_js_project(
            &scripts_root,
            "scheduler-types",
            "SchedulerTypes",
            r#"
                const timer = new RealtimeTimer("AutoPick", {
                    textList: ["Crystal Core"],
                    forceInteraction: true
                });
                timer.Interval = 125;
                const added = dispatcher.AddTimer(timer);

                const solo = SoloTask("AutoFight", { strategyName: "daily" });
                const task = dispatcher.RunTask(solo);

                log.Info([
                    added.AddRealtimeTimer.name,
                    added.AddRealtimeTimer.interval_ms,
                    added.AddRealtimeTimer.config.TextList[0],
                    added.AddRealtimeTimer.config.ForceInteraction,
                    task.RunSoloTask.name,
                    task.RunSoloTask.config.strategyName
                ].join(":"));
                dispatcher.Commands().length;
            "#,
        );
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![ScriptGroupProject {
                index: 1,
                name: "SchedulerTypes".to_string(),
                folder_name: "scheduler-types".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            }],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_for_test(&app_root, &group);

        assert_eq!(outcome.completed_steps, 1);
        let javascript = outcome.steps[0].javascript.as_ref().unwrap();
        assert_eq!(javascript.result, Some(json!(3)));
        assert_eq!(
            javascript.logs[0].message,
            "AutoPick:125:Crystal Core:true:AutoFight:daily"
        );
        assert_eq!(javascript.task_invocations.dispatcher.len(), 3);
        assert!(javascript.task_invocations.errors.is_empty());
    }

    #[test]
    fn script_group_javascript_exposes_legacy_task_param_constructors() {
        let app_root = test_root("group-js-task-param-types");
        let scripts_root = app_root.join("User").join("JsScript");
        write_js_project(
            &scripts_root,
            "task-param-types",
            "TaskParamTypes",
            r#"
                const skip = new AutoSkipConfig();
                skip.ClickChatOption = "随机选择选项";

                const domain = new AutoDomainParam(0, "domain");
                const boss = AutoBossParam("boss");
                const fight = new AutoFightParam("fight");
                const ley = new AutoLeyLineOutcropParam(3, "蒙德", "启示之花");
                const stygian = AutoStygianOnslaughtParam("stygian");

                const domainRun = dispatcher.RunAutoDomainTask(domain);
                const bossRun = dispatcher.RunAutoBossTask(boss);
                const fightRun = dispatcher.RunAutoFightTask(fight);
                const leyRun = dispatcher.RunAutoLeyLineOutcropTask(ley);
                const stygianRun = dispatcher.RunAutoStygianOnslaughtTask(stygian);

                log.Info([
                    skip.ClickChatOption,
                    domain.DomainRoundNum,
                    domain.CombatStrategyPath,
                    boss.CombatStrategyPath,
                    fight.CombatStrategyPath,
                    ley.Country,
                    ley.LeyLineOutcropType,
                    stygian.CombatScriptBagPath,
                    domainRun.RunBuiltinTask.name,
                    bossRun.RunBuiltinTask.name,
                    fightRun.RunBuiltinTask.name,
                    leyRun.RunBuiltinTask.name,
                    stygianRun.RunBuiltinTask.name
                ].join("|"));
                dispatcher.Commands().length;
            "#,
        );
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![ScriptGroupProject {
                index: 1,
                name: "TaskParamTypes".to_string(),
                folder_name: "task-param-types".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            }],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_for_test(&app_root, &group);

        assert_eq!(outcome.completed_steps, 1);
        let javascript = outcome.steps[0].javascript.as_ref().unwrap();
        assert_eq!(javascript.result, Some(json!(5)));
        assert_eq!(
            javascript.logs[0].message,
            "随机选择选项|9999|User/AutoFight/domain.txt|User/AutoFight/boss.txt|User/AutoFight/fight.txt|蒙德|启示之花|stygian|AutoDomain|AutoBoss|AutoFight|AutoLeyLineOutcrop|AutoStygianOnslaught"
        );
        assert_eq!(javascript.task_invocations.dispatcher.len(), 5);
        assert!(javascript.task_invocations.errors.is_empty());
    }

    #[test]
    fn script_group_javascript_exposes_legacy_vision_model_constructors() {
        let app_root = test_root("group-js-vision-types");
        let scripts_root = app_root.join("User").join("JsScript");
        write_js_project(
            &scripts_root,
            "vision-types",
            "VisionTypes",
            r#"
                const roi = new Rect(1, 2, 30, 40);
                const image = new BvImage("AutoPick:F.png", roi, 0.91);
                const locator = new BvLocator(image.RecognitionObject);
                const emptyObject = new RecognitionObject();
                const emptyLocator = new BvLocator(emptyObject);
                const page = new BvPage();

                log.Info([
                    image.FeatureName,
                    image.AssetName,
                    image.RecognitionObject.Name,
                    image.RecognitionObject.RegionOfInterest.Width,
                    image.RecognitionObject.Template.Threshold,
                    locator.RecognitionObject.Template.TemplateAsset.includes("GameTask"),
                    emptyLocator.RecognitionObject.RecognitionType,
                    page.DefaultTimeoutMs,
                    page.CaptureSize.Width
                ].join("|"));
                page.DefaultRetryIntervalMs;
            "#,
        );
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![ScriptGroupProject {
                index: 1,
                name: "VisionTypes".to_string(),
                folder_name: "vision-types".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            }],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_for_test(&app_root, &group);

        assert_eq!(outcome.completed_steps, 1);
        let javascript = outcome.steps[0].javascript.as_ref().unwrap();
        assert_eq!(javascript.result, Some(json!(1000)));
        assert_eq!(
            javascript.logs[0].message,
            "AutoPick|F.png|AutoPick:F.png|30|0.91|true|None|10000|1920"
        );
    }

    #[test]
    fn key_mouse_hook_dispatch_invokes_registered_javascript_callbacks() {
        let root = test_root("key-mouse-hook-callback");
        let project = root.join("demo");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                const seen = [];
                const key = KeyMouseHook.onKeyDown((value) => seen.push(`key:${value}`), true);
                const move = KeyMouseHook.onMouseMove((x, y) => seen.push(`move:${x},${y}`), 25);
                const first = KeyMouseHook.dispatchEvent({ type: "keyDown", keyCode: "F", keyData: "Control, F" });
                const second = KeyMouseHook.dispatchEvent({ type: "mouseMove", x: 12, y: 34, timestampMs: 100 });
                `${key.AddListener.id}:${move.AddListener.interval_ms}:${first.length}:${second.length}:${seen.join("|")}`;
            "#,
        )
        .unwrap();

        let outcome = execute_javascript_project(&root, "demo", None).unwrap();
        let key_registration = outcome
            .host_calls
            .iter()
            .find(|call| {
                call.target == ScriptHostTarget::KeyMouseHook && call.method == "onKeyDown"
            })
            .unwrap();
        let dispatch_count = outcome
            .host_calls
            .iter()
            .filter(|call| {
                call.target == ScriptHostTarget::KeyMouseHook && call.method == "dispatchEvent"
            })
            .count();

        assert_eq!(
            outcome.result,
            Some(json!("callback-1:25:1:1:key:F|move:12,34"))
        );
        assert_eq!(
            key_registration.args,
            vec![json!("callback-1"), json!(true)]
        );
        assert_eq!(dispatch_count, 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn key_mouse_hook_dispatch_settles_async_javascript_callback() {
        let root = test_root("key-mouse-hook-async-callback");
        let project = root.join("demo");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("manifest.json"),
            r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
        )
        .unwrap();
        fs::write(
            project.join("main.js"),
            r#"
                const seen = [];
                KeyMouseHook.onKeyDown(async (value) => {
                    const resolved = await Promise.resolve(value + "!");
                    seen.push(resolved);
                }, true);
                const dispatched = KeyMouseHook.dispatchEvent({ type: "keyDown", keyCode: "F", keyData: "F" });
                `${dispatched.length}:${seen.join("|")}`;
            "#,
        )
        .unwrap();

        let outcome = execute_javascript_project(&root, "demo", None).unwrap();

        assert_eq!(outcome.result, Some(json!("1:F!")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn script_group_pathing_steps_return_execution_plan() {
        let app_root = test_root("group-pathing");
        let pathing_root = app_root.join("User").join("AutoPathing").join("routes");
        fs::create_dir_all(&pathing_root).unwrap();
        fs::write(
            pathing_root.join("route.json"),
            r#"{
              "info": { "name": "route", "type": "collect", "map_name": "Teyvat" },
              "positions": [
                { "x": 1.0, "y": 2.0, "type": "path" },
                { "x": 3.0, "y": 4.0, "type": "teleport" },
                { "x": 5.0, "y": 6.0, "type": "target", "action": "fight" }
              ]
            }"#,
        )
        .unwrap();
        let group = ScriptGroup {
            name: "daily".to_string(),
            config: ScriptGroupConfig {
                pathing_config: json!({"partyName": "daily"}),
                ..ScriptGroupConfig::default()
            },
            projects: vec![ScriptGroupProject {
                index: 1,
                name: "route.json".to_string(),
                folder_name: "routes".to_string(),
                project_type: ScriptProjectType::Pathing,
                ..ScriptGroupProject::default()
            }],
            ..ScriptGroup::default()
        };

        let outcome = execute_script_group_for_test(&app_root, &group);

        assert_eq!(outcome.attempted_steps, 1);
        assert_eq!(outcome.planned_steps, 1);
        assert_eq!(outcome.failed_steps, 0);
        let execution = outcome.steps[0].pathing_execution.as_ref().unwrap();
        assert!(!execution.dispatched);
        assert_eq!(execution.plan.summary.waypoint_count, 3);
        assert_eq!(
            execution.plan.party_config,
            Some(json!({"partyName": "daily"}))
        );
        assert_eq!(execution.execution_plan.segment_count, 2);
        assert_eq!(execution.execution_plan.expected_fight_count, 1);
        assert!(execution.execution_plan.segments[1].starts_with_teleport);
    }

    #[cfg(windows)]
    fn shell_echo_command(message: &str) -> String {
        format!("echo {message} & exit")
    }

    #[cfg(not(windows))]
    fn shell_echo_command(message: &str) -> String {
        format!("echo {message}; exit")
    }

    #[cfg(windows)]
    fn shell_sleep_command(seconds: u64) -> String {
        format!("ping -n {} 127.0.0.1 > nul & exit", seconds + 1)
    }

    #[cfg(not(windows))]
    fn shell_sleep_command(seconds: u64) -> String {
        format!("sleep {seconds}; exit")
    }

    fn write_js_project(root: &Path, folder: &str, name: &str, main: &str) {
        write_js_project_with_manifest(
            root,
            folder,
            &format!(r#"{{"name":"{name}","version":"1.0","main":"main.js"}}"#),
            main,
        );
    }

    fn write_js_project_with_manifest(root: &Path, folder: &str, manifest: &str, main: &str) {
        let project = root.join(folder);
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("manifest.json"), manifest).unwrap();
        fs::write(project.join("main.js"), main).unwrap();
    }

    fn execute_script_group_for_test(
        app_root: &Path,
        group: &ScriptGroup,
    ) -> ScriptGroupExecutionOutcome {
        let mut roots = ScriptGroupExecutionRoots::from_app_root(app_root);
        roots.global_input_dispatch_mode = GlobalInputDispatchMode::PlanOnly;
        roots.key_mouse_dispatch_mode = KeyMouseScriptDispatchMode::PlanOnly;
        roots.http_dispatch_mode = HttpDispatchMode::PlanOnly;
        roots.notification_dispatch_mode = NotificationDispatchMode::RecordOnly;
        execute_script_group_with_roots(&roots, group)
    }

    fn test_root(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("bgi-script-engine-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }
}
