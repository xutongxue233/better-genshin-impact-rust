use super::*;

use super::group_execution_project::execute_group_project_once;
use super::group_execution_steps::{
    cancelled_group_step, script_group_execution_outcome, skipped_group_step,
    skipped_group_step_with_reason,
};
use bgi_script::{record_farming_session, FarmingRouteRef};
use std::collections::HashMap;

pub(super) fn execute_group_with_task_dispatcher_hooks_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    mut dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    mut configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    mut after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> ScriptGroupExecutionOutcome {
    let mut steps = Vec::new();
    'projects: for (project_index, project) in indexed_projects(group) {
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

pub(super) fn execute_group_with_execution_records_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    storage: &ExecutionRecordStorage,
    clock: &ExecutionRecordClock,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    mut configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    mut after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    execute_indexed_projects_with_execution_records(
        roots,
        group,
        group.projects.len(),
        indexed_projects(group),
        storage,
        clock,
        None,
        true,
        dispatcher,
        cancellation,
        &mut configure_host,
        &mut after_javascript,
    )
}

pub(super) fn execute_group_with_pre_execution_records_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    all_groups: &[ScriptGroup],
    storage: &ExecutionRecordStorage,
    clock: &ExecutionRecordClock,
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
        clock,
        None,
        dispatcher,
        cancellation,
        configure_host,
        after_javascript,
    )
}

pub(super) fn execute_group_with_pre_execution_records_and_farming_plan_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    all_groups: &[ScriptGroup],
    storage: &ExecutionRecordStorage,
    clock: &ExecutionRecordClock,
    farming_context: Option<&FarmingPlanExecutionContext>,
    mut dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    mut configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    mut after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    let mut steps = Vec::new();
    let mut record_contexts = HashMap::new();
    let mut execution_counts = HashMap::new();

    'projects: for (project_index, project) in indexed_projects(group) {
        let pre_execution_plan = try_plan_pre_execution_priority_projects(
            group,
            all_groups,
            &execution_counts,
            |candidate_group, candidate_project| {
                pre_execution_project_skipped_by_records(
                    candidate_group,
                    candidate_project,
                    storage,
                    clock,
                    &mut record_contexts,
                )
            },
        )?;

        for candidate in pre_execution_plan.candidates {
            let Some(candidate_group) = all_groups
                .iter()
                .find(|candidate_group| candidate_group.name == candidate.group_name)
            else {
                continue;
            };
            let record_context = execution_record_context_from_cache(
                candidate_group,
                storage,
                clock,
                &mut record_contexts,
            )?;
            let project_key = candidate.project_key.clone();
            let flow = execute_project_with_execution_records(
                roots,
                candidate_group,
                candidate.project_index,
                &candidate.project,
                storage,
                clock,
                record_context,
                farming_context,
                true,
                &mut dispatcher,
                cancellation,
                &mut configure_host,
                &mut after_javascript,
                &mut steps,
                || {
                    *execution_counts.entry(project_key.clone()).or_insert(0) += 1;
                },
            )?;
            if flow == ProjectExecutionFlow::Stop {
                break 'projects;
            }
        }

        let record_context =
            execution_record_context_from_cache(group, storage, clock, &mut record_contexts)?;
        let flow = execute_project_with_execution_records(
            roots,
            group,
            project_index,
            &project,
            storage,
            clock,
            record_context,
            farming_context,
            true,
            &mut dispatcher,
            cancellation,
            &mut configure_host,
            &mut after_javascript,
            &mut steps,
            || {},
        )?;
        if flow == ProjectExecutionFlow::Stop {
            break;
        }
    }

    Ok(script_group_execution_outcome(
        group.name.clone(),
        group.projects.len(),
        steps,
    ))
}

pub(super) fn execute_group_from_resume_with_execution_records_and_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    resume_pointer: &ScriptGroupResumePointer,
    storage: &ExecutionRecordStorage,
    clock: &ExecutionRecordClock,
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
        clock,
        None,
        dispatcher,
        cancellation,
        configure_host,
        after_javascript,
    )
}

pub(super) fn execute_group_from_resume_with_execution_records_and_farming_plan_cancellation(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    resume_pointer: &ScriptGroupResumePointer,
    storage: &ExecutionRecordStorage,
    clock: &ExecutionRecordClock,
    farming_context: Option<&FarmingPlanExecutionContext>,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    let (projects, _) =
        bgi_script::select_script_group_projects_from_resume(group, Some(resume_pointer));
    let mut resumed_group = group.clone();
    resumed_group.projects = projects;
    execute_indexed_projects_with_execution_records(
        roots,
        &resumed_group,
        resumed_group.projects.len(),
        indexed_projects(&resumed_group),
        storage,
        clock,
        farming_context,
        true,
        dispatcher,
        cancellation,
        configure_host,
        after_javascript,
    )
}

pub(super) fn execute_group_project_with_execution_records_run_policy(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project_index: usize,
    storage: &ExecutionRecordStorage,
    clock: &ExecutionRecordClock,
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
        clock,
        None,
        honor_run_count,
        dispatcher,
        cancellation,
        configure_host,
        after_javascript,
    )
}

pub(super) fn execute_group_project_with_execution_records_and_farming_plan_run_policy(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project_index: usize,
    storage: &ExecutionRecordStorage,
    clock: &ExecutionRecordClock,
    farming_context: Option<&FarmingPlanExecutionContext>,
    honor_run_count: bool,
    dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    mut configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    mut after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    let (project_index, project) = indexed_projects(group)
        .into_iter()
        .find(|(index, _)| *index == project_index)
        .ok_or_else(|| {
            ScriptEngineError::ValueConversion(format!(
                "script group project index {project_index} was not found"
            ))
        })?;
    execute_indexed_projects_with_execution_records(
        roots,
        group,
        1,
        vec![(project_index, project)],
        storage,
        clock,
        farming_context,
        honor_run_count,
        dispatcher,
        cancellation,
        &mut configure_host,
        &mut after_javascript,
    )
}

fn execute_indexed_projects_with_execution_records(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    requested_projects: usize,
    projects: Vec<(usize, ScriptGroupProject)>,
    storage: &ExecutionRecordStorage,
    clock: &ExecutionRecordClock,
    farming_context: Option<&FarmingPlanExecutionContext>,
    honor_run_count: bool,
    mut dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    mut configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    mut after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    let record_context = execution_record_context(group, storage, clock)?;

    let mut steps = Vec::new();
    for (project_index, project) in projects {
        let flow = execute_project_with_execution_records(
            roots,
            group,
            project_index,
            &project,
            storage,
            clock,
            &record_context,
            farming_context,
            honor_run_count,
            &mut dispatcher,
            cancellation,
            &mut configure_host,
            &mut after_javascript,
            &mut steps,
            || {},
        )?;
        if flow == ProjectExecutionFlow::Stop {
            break;
        }
    }

    Ok(script_group_execution_outcome(
        group.name.clone(),
        requested_projects,
        steps,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GroupExecutionRecordContext {
    skip_config: Option<TaskCompletionSkipRuleConfig>,
    daily_records: Vec<DailyExecutionRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectExecutionFlow {
    Continue,
    Stop,
}

fn execution_record_context(
    group: &ScriptGroup,
    storage: &ExecutionRecordStorage,
    clock: &ExecutionRecordClock,
) -> Result<GroupExecutionRecordContext> {
    let skip_config =
        TaskCompletionSkipRuleConfig::from_pathing_config(&group.config.pathing_config)
            .filter(TaskCompletionSkipRuleConfig::is_effective);
    let daily_records = match skip_config.as_ref() {
        Some(config) => {
            storage.recent_execution_records_by_config_for_today(config, clock.now_local.date())?
        }
        None => Vec::new(),
    };
    Ok(GroupExecutionRecordContext {
        skip_config,
        daily_records,
    })
}

fn execution_record_context_from_cache<'a>(
    group: &ScriptGroup,
    storage: &ExecutionRecordStorage,
    clock: &ExecutionRecordClock,
    cache: &'a mut HashMap<String, GroupExecutionRecordContext>,
) -> Result<&'a GroupExecutionRecordContext> {
    if !cache.contains_key(&group.name) {
        let context = execution_record_context(group, storage, clock)?;
        cache.insert(group.name.clone(), context);
    }
    Ok(cache
        .get(&group.name)
        .expect("execution record context was inserted"))
}

fn pre_execution_project_skipped_by_records(
    group: &ScriptGroup,
    project: &ScriptGroupProject,
    storage: &ExecutionRecordStorage,
    clock: &ExecutionRecordClock,
    record_contexts: &mut HashMap<String, GroupExecutionRecordContext>,
) -> Result<bool> {
    let record_context =
        execution_record_context_from_cache(group, storage, clock, record_contexts)?;
    let project_ref = ExecutionRecordProjectRef::from_group_project(group, project);
    Ok(is_skip_task(
        &project_ref,
        record_context.skip_config.as_ref(),
        &record_context.daily_records,
        clock,
    )?
    .should_skip)
}

fn execute_project_with_execution_records(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project_index: usize,
    project: &ScriptGroupProject,
    storage: &ExecutionRecordStorage,
    clock: &ExecutionRecordClock,
    record_context: &GroupExecutionRecordContext,
    farming_context: Option<&FarmingPlanExecutionContext>,
    honor_run_count: bool,
    dispatcher: &mut Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    configure_host: &mut impl FnMut(&mut ScriptHostRuntimeConfig),
    after_javascript: &mut impl FnMut(&mut JavaScriptExecutionOutcome),
    steps: &mut Vec<ScriptGroupStepExecutionOutcome>,
    mut before_runs: impl FnMut(),
) -> Result<ProjectExecutionFlow> {
    if project.status == ScriptProjectStatus::Disabled || project.skip_flag.unwrap_or(false) {
        steps.push(skipped_group_step(project_index, project));
        return Ok(ProjectExecutionFlow::Continue);
    }
    if cancellation_requested(cancellation) {
        steps.push(cancelled_group_step(project_index, project, 0));
        return Ok(ProjectExecutionFlow::Stop);
    }

    let pre_run_skip =
        pathing_pre_run_skip_decision(&project.name, &group.config.pathing_config, clock);
    if pre_run_skip.should_skip {
        steps.push(skipped_group_step_with_reason(
            project_index,
            project,
            pre_run_skip.message,
        ));
        return Ok(ProjectExecutionFlow::Continue);
    }

    if let Some(skip_decision) =
        farming_plan_skip_decision_for_project(roots, project, farming_context, clock)
    {
        if skip_decision.should_skip {
            steps.push(skipped_group_step_with_reason(
                project_index,
                project,
                farming_plan_skip_reason(&project.name, &skip_decision.message),
            ));
            return Ok(ProjectExecutionFlow::Continue);
        }
    }

    let project_ref = ExecutionRecordProjectRef::from_group_project(group, project);
    let skip_decision = is_skip_task(
        &project_ref,
        record_context.skip_config.as_ref(),
        &record_context.daily_records,
        clock,
    )?;
    if skip_decision.should_skip {
        steps.push(skipped_group_step_with_reason(
            project_index,
            project,
            skip_decision.message,
        ));
        return Ok(ProjectExecutionFlow::Continue);
    }

    before_runs();
    let run_count = if honor_run_count {
        project.run_num.max(1) as u32
    } else {
        1
    };
    for run_iteration in 1..=run_count {
        if cancellation_requested(cancellation) {
            steps.push(cancelled_group_step(project_index, project, run_iteration));
            return Ok(ProjectExecutionFlow::Stop);
        }

        let mut record = ExecutionRecord::started(
            &project_ref,
            clock,
            record_context
                .skip_config
                .as_ref()
                .is_some_and(|config| config.is_boundary_time_based_on_server_time),
        );
        storage.save_execution_record(&record)?;

        let step = execute_group_project_once(
            roots,
            group,
            project_index,
            project,
            run_iteration,
            dispatcher.as_deref_mut(),
            configure_host,
            after_javascript,
            cancellation,
        );
        let cancelled = step.status == ScriptGroupStepStatus::Cancelled;
        record.finish(execution_record_success(&step), clock);
        storage.save_execution_record(&record)?;
        record_completed_farming_session_for_step(group, &step, farming_context, clock);
        steps.push(step);
        if cancelled {
            return Ok(ProjectExecutionFlow::Stop);
        }
    }

    Ok(ProjectExecutionFlow::Continue)
}

fn record_completed_farming_session_for_step(
    group: &ScriptGroup,
    step: &ScriptGroupStepExecutionOutcome,
    farming_context: Option<&FarmingPlanExecutionContext>,
    clock: &ExecutionRecordClock,
) {
    let Some(context) = farming_context else {
        return;
    };
    if step.project_type != ScriptProjectType::Pathing
        || step.status != ScriptGroupStepStatus::Completed
    {
        return;
    }
    let Some(pathing_execution) = step.pathing_execution.as_ref() else {
        return;
    };
    if !pathing_execution.completed || !pathing_execution.execution_plan.farming.allow_farming_count
    {
        return;
    }

    let route = FarmingRouteRef::new(
        group.name.clone(),
        step.name.clone(),
        step.folder_name.clone(),
    );
    let _ = record_farming_session(
        context,
        &route,
        &pathing_execution.execution_plan.farming,
        clock.now_local_with_offset(),
        clock.now_server,
    );
}

fn farming_plan_skip_decision_for_project(
    roots: &ScriptGroupExecutionRoots,
    project: &ScriptGroupProject,
    farming_context: Option<&FarmingPlanExecutionContext>,
    clock: &ExecutionRecordClock,
) -> Option<bgi_script::FarmingPlanSkipDecision> {
    if project.project_type != ScriptProjectType::Pathing {
        return None;
    }

    farming_plan_skip_decision_from_pathing_file(
        &roots.pathing_script_root,
        &project.folder_name,
        &project.name,
        farming_context,
        clock.now_server,
    )
    .ok()
}

fn farming_plan_skip_reason(project_name: &str, message: &str) -> String {
    if message.is_empty() {
        format!("{project_name}:锄地规划统计触发跳过,跳过此任务！")
    } else {
        format!("{project_name}:{message},跳过此任务！")
    }
}

pub(super) fn execute_group_project_with_run_policy(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project_index: usize,
    honor_run_count: bool,
    mut dispatcher: Option<&mut DispatcherRuntime>,
    cancellation: Option<&InputCancellationToken>,
    mut configure_host: impl FnMut(&mut ScriptHostRuntimeConfig),
    mut after_javascript: impl FnMut(&mut JavaScriptExecutionOutcome),
) -> Result<ScriptGroupExecutionOutcome> {
    let (project_index, project) = indexed_projects(group)
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

fn indexed_projects(group: &ScriptGroup) -> Vec<(usize, ScriptGroupProject)> {
    let mut indexed_projects = group
        .projects
        .iter()
        .cloned()
        .enumerate()
        .collect::<Vec<_>>();
    indexed_projects.sort_by_key(|(_, project)| project.index);
    indexed_projects
}

fn execution_record_success(step: &ScriptGroupStepExecutionOutcome) -> bool {
    match step.project_type {
        ScriptProjectType::Pathing => step.status == ScriptGroupStepStatus::Completed,
        _ => step.status == ScriptGroupStepStatus::Completed,
    }
}
