#![allow(clippy::result_large_err, clippy::too_many_arguments)]

use bgi_script::{
    farming_plan_skip_decision_from_pathing_file, is_skip_task, pathing_pre_run_skip_decision,
    try_plan_pre_execution_priority_projects, DailyExecutionRecord, ExecutionRecord,
    ExecutionRecordClock, ExecutionRecordProjectRef, ExecutionRecordStorage,
    FarmingPlanExecutionContext, InputCancellationToken, KeyMouseScriptHost, MacroPlaybackContext,
    PreparedScriptExecution, ScriptCodeExecutionMode, ScriptExecutionStep, ScriptGroup,
    ScriptGroupProject, ScriptGroupResumePointer, ScriptHostRuntime, ScriptHostRuntimeConfig,
    ScriptHostTarget, ScriptProjectError, ScriptProjectStatus, ScriptProjectType,
    TaskCompletionSkipRuleConfig,
};
#[cfg(test)]
use bgi_script::{
    GlobalInputDispatchMode, HttpDispatchMode, KeyMouseScriptDispatchMode, NotificationDispatchMode,
};
use bgi_task::{
    execute_shell_task_with_cancel, AutoBossParam, AutoDomainParam, AutoFightParam,
    AutoLeyLineOutcropParam, AutoSkipConfigParam, AutoStygianOnslaughtParam, DispatcherRuntime,
    ShellConfig, ShellExecutionStatus, ShellTaskParam, TaskInvocationExecutionMode,
    TaskInvocationPlanningContext,
};
use bgi_vision::{
    BvImage as VisionBvImage, BvLocator as VisionBvLocator, BvPage as VisionBvPage,
    RecognitionObject as VisionRecognitionObject, Rect as VisionRect,
};
use boa_engine::{
    builtins::promise::PromiseState,
    js_string,
    native_function::NativeFunction,
    object::{builtins::JsPromise, FunctionObjectBuilder, ObjectInitializer},
    property::Attribute,
    script::Script,
    Context, JsNativeError, JsResult, JsString, JsValue, Source,
};
use serde::Serialize;
use serde_json::Value;
use std::cell::RefCell;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread;

mod group_execution;
mod model;
mod module_loader;
mod runtime_bindings;

use group_execution::cancellation_requested;
pub use group_execution::{
    execute_script_group,
    execute_script_group_from_resume_with_execution_records_and_farming_plan_hooks_and_cancellation,
    execute_script_group_from_resume_with_execution_records_hooks_and_cancellation,
    execute_script_group_from_resume_with_task_dispatcher_hooks_and_cancellation,
    execute_script_group_project_repeated_with_task_dispatcher_hooks,
    execute_script_group_project_repeated_with_task_dispatcher_hooks_and_cancellation,
    execute_script_group_project_with_execution_records_and_farming_plan_hooks_and_cancellation,
    execute_script_group_project_with_execution_records_hooks_and_cancellation,
    execute_script_group_project_with_host_hooks,
    execute_script_group_project_with_task_dispatcher_hooks,
    execute_script_group_project_with_task_dispatcher_hooks_and_cancellation,
    execute_script_group_with_execution_records,
    execute_script_group_with_execution_records_and_clock,
    execute_script_group_with_execution_records_hooks_and_cancellation,
    execute_script_group_with_host_configurator, execute_script_group_with_host_hooks,
    execute_script_group_with_pre_execution_records_and_farming_plan_hooks_and_cancellation,
    execute_script_group_with_pre_execution_records_hooks_and_cancellation,
    execute_script_group_with_roots, execute_script_group_with_task_dispatcher_hooks,
    execute_script_group_with_task_dispatcher_hooks_and_cancellation,
};
pub use model::*;
use module_loader::BetterGiModuleLoader;
use runtime_bindings::*;

#[derive(Clone)]
struct EngineState {
    console: Rc<RefCell<Vec<String>>>,
    host: Rc<RefCell<ScriptHostRuntime>>,
    host_calls: Rc<RefCell<Vec<ExecutedHostCall>>>,
    next_callback_id: Rc<RefCell<u64>>,
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
    execute_prepared_javascript_with_task_mode_context_and_cancellation(
        prepared,
        task_invocation_mode,
        &TaskInvocationPlanningContext::default(),
        cancellation,
    )
}

fn execute_prepared_javascript_with_task_mode_context_and_cancellation(
    prepared: &PreparedScriptExecution,
    task_invocation_mode: TaskInvocationExecutionMode,
    task_invocation_context: &TaskInvocationPlanningContext,
    cancellation: Option<&InputCancellationToken>,
) -> Result<JavaScriptExecutionOutcome> {
    let host = script_host_runtime(prepared, cancellation)?;
    execute_prepared_javascript_with_host(
        prepared,
        task_invocation_mode,
        task_invocation_context,
        None,
        cancellation,
        host,
    )
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
    execute_prepared_javascript_with_task_dispatcher_context_and_cancellation(
        prepared,
        dispatcher,
        &TaskInvocationPlanningContext::default(),
        cancellation,
    )
}

fn execute_prepared_javascript_with_task_dispatcher_context_and_cancellation(
    prepared: &PreparedScriptExecution,
    dispatcher: &mut DispatcherRuntime,
    task_invocation_context: &TaskInvocationPlanningContext,
    cancellation: Option<&InputCancellationToken>,
) -> Result<JavaScriptExecutionOutcome> {
    let host = script_host_runtime(prepared, cancellation)?;
    execute_prepared_javascript_with_host(
        prepared,
        TaskInvocationExecutionMode::ExecuteReady,
        task_invocation_context,
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
    task_invocation_context: &TaskInvocationPlanningContext,
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
        || {
            JavaScriptTaskExecution::evaluate(
                &task_invocations,
                task_invocation_mode,
                task_invocation_context,
            )
        },
        |dispatcher| {
            JavaScriptTaskExecution::execute_ready(
                &task_invocations,
                dispatcher,
                task_invocation_context,
            )
        },
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

#[cfg(test)]
mod tests;
