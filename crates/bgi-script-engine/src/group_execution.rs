use super::*;

#[path = "group_execution_project.rs"]
mod group_execution_project;

#[path = "group_execution_runner.rs"]
mod group_execution_runner;

#[path = "group_execution_steps.rs"]
mod group_execution_steps;

use self::group_execution_runner::{
    execute_group_from_resume_with_execution_records_and_cancellation,
    execute_group_from_resume_with_execution_records_and_farming_plan_cancellation,
    execute_group_project_with_execution_records_and_farming_plan_run_policy,
    execute_group_project_with_execution_records_run_policy,
    execute_group_project_with_run_policy as execute_script_group_project_with_run_policy,
    execute_group_with_execution_records_and_cancellation,
    execute_group_with_pre_execution_records_and_cancellation,
    execute_group_with_pre_execution_records_and_farming_plan_cancellation,
    execute_group_with_task_dispatcher_hooks_and_cancellation,
};

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
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> ScriptGroupExecutionOutcome {
    execute_group_with_task_dispatcher_hooks_and_cancellation(
        roots,
        group,
        dispatcher,
        cancellation,
        configure_host,
        after_javascript,
    )
}

pub fn execute_script_group_with_execution_records(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    storage: &ExecutionRecordStorage,
) -> Result<ScriptGroupExecutionOutcome> {
    execute_script_group_with_execution_records_and_clock(
        roots,
        group,
        storage,
        ExecutionRecordClock::now(),
    )
}

pub fn execute_script_group_with_execution_records_and_clock(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    storage: &ExecutionRecordStorage,
    clock: ExecutionRecordClock,
) -> Result<ScriptGroupExecutionOutcome> {
    execute_script_group_with_execution_records_hooks_and_cancellation(
        roots,
        group,
        storage,
        clock,
        None,
        None,
        |_| {},
        |_| {},
    )
}

pub fn execute_script_group_with_execution_records_hooks_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    storage: &ExecutionRecordStorage,
    clock: ExecutionRecordClock,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    execute_group_with_execution_records_and_cancellation(
        roots,
        group,
        storage,
        &clock,
        dispatcher,
        cancellation,
        configure_host,
        after_javascript,
    )
}

pub fn execute_script_group_with_pre_execution_records_hooks_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    all_groups: &[ScriptGroup],
    storage: &ExecutionRecordStorage,
    clock: ExecutionRecordClock,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    execute_group_with_pre_execution_records_and_cancellation(
        roots,
        group,
        all_groups,
        storage,
        &clock,
        dispatcher,
        cancellation,
        configure_host,
        after_javascript,
    )
}

pub fn execute_script_group_with_pre_execution_records_and_farming_plan_hooks_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    all_groups: &[ScriptGroup],
    storage: &ExecutionRecordStorage,
    clock: ExecutionRecordClock,
    farming_context: &FarmingPlanExecutionContext,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    execute_group_with_pre_execution_records_and_farming_plan_cancellation(
        roots,
        group,
        all_groups,
        storage,
        &clock,
        Some(farming_context),
        dispatcher,
        cancellation,
        configure_host,
        after_javascript,
    )
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

pub fn execute_script_group_from_resume_with_execution_records_hooks_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    resume_pointer: &ScriptGroupResumePointer,
    storage: &ExecutionRecordStorage,
    clock: ExecutionRecordClock,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    execute_group_from_resume_with_execution_records_and_cancellation(
        roots,
        group,
        resume_pointer,
        storage,
        &clock,
        dispatcher,
        cancellation,
        configure_host,
        after_javascript,
    )
}

pub fn execute_script_group_from_resume_with_execution_records_and_farming_plan_hooks_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    resume_pointer: &ScriptGroupResumePointer,
    storage: &ExecutionRecordStorage,
    clock: ExecutionRecordClock,
    farming_context: &FarmingPlanExecutionContext,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    execute_group_from_resume_with_execution_records_and_farming_plan_cancellation(
        roots,
        group,
        resume_pointer,
        storage,
        &clock,
        Some(farming_context),
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

pub fn execute_script_group_project_with_execution_records_hooks_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project_index: usize,
    storage: &ExecutionRecordStorage,
    clock: ExecutionRecordClock,
    honor_run_count: bool,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    execute_group_project_with_execution_records_run_policy(
        roots,
        group,
        project_index,
        storage,
        &clock,
        honor_run_count,
        dispatcher,
        cancellation,
        configure_host,
        after_javascript,
    )
}

pub fn execute_script_group_project_with_execution_records_and_farming_plan_hooks_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project_index: usize,
    storage: &ExecutionRecordStorage,
    clock: ExecutionRecordClock,
    farming_context: &FarmingPlanExecutionContext,
    honor_run_count: bool,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    execute_group_project_with_execution_records_and_farming_plan_run_policy(
        roots,
        group,
        project_index,
        storage,
        &clock,
        Some(farming_context),
        honor_run_count,
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

pub(super) fn cancellation_requested(cancellation: Option<&InputCancellationToken>) -> bool {
    cancellation.is_some_and(InputCancellationToken::is_cancelled)
}
