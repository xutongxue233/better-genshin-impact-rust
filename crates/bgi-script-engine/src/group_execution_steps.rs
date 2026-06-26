use super::*;

pub(super) fn skipped_group_step(
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

pub(super) fn skipped_group_step_with_reason(
    project_index: usize,
    project: &ScriptGroupProject,
    reason: impl Into<String>,
) -> ScriptGroupStepExecutionOutcome {
    let mut step = skipped_group_step(project_index, project);
    step.skip_reason = Some(reason.into());
    step
}

pub(super) fn cancelled_group_step(
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

pub(super) fn script_group_execution_outcome(
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

pub(super) fn group_step(
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
        skip_reason: None,
    }
}
