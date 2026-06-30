use super::*;

use super::group_execution_steps::group_step;
use bgi_core::{is_new_version, read_pathing_task};
use bgi_script::{
    LimitedFileHost, PathingScriptExecution, PathingScriptRunPlan, PathingScriptSource,
};

pub(super) fn execute_group_project_once(
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
        ScriptProjectType::Pathing => match execute_group_pathing_project(roots, group, project) {
            Ok(execution) => {
                outcome.status = ScriptGroupStepStatus::Planned;
                outcome.pathing_execution = Some(execution);
            }
            Err(error) => {
                outcome.status = ScriptGroupStepStatus::Failed;
                outcome.error = Some(error.to_string());
            }
        },
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

fn execute_group_pathing_project(
    roots: &ScriptGroupExecutionRoots,
    group: &ScriptGroup,
    project: &ScriptGroupProject,
) -> std::result::Result<PathingScriptExecution, String> {
    let relative_path = PathBuf::from(&project.folder_name)
        .join(&project.name)
        .to_string_lossy()
        .replace('\\', "/");
    let file_host = LimitedFileHost::new(&roots.pathing_script_root);
    let normalized_path = file_host
        .normalize_path(&relative_path)
        .map_err(|error| error.to_string())?;
    let task = read_pathing_task(&normalized_path).map_err(|error| error.to_string())?;
    if let (Some(required), Some(app_version)) = (&task.info.bgi_version, &roots.app_version) {
        if !required.trim().is_empty() && is_new_version(app_version, required) {
            return Err(format!(
                "地图追踪任务 {} 版本号要求 {} 大于当前 BetterGI 版本号 {}，禁止运行，请更新 BetterGI 版本！",
                project.name, required, app_version
            ));
        }
    }
    let summary = task.summary();
    let execution_plan = task.execution_plan_with_legacy_track_converter();
    Ok(PathingScriptExecution {
        plan: PathingScriptRunPlan {
            source: PathingScriptSource::UserAutoPathingFile,
            normalized_path: Some(normalized_path),
            summary,
            task,
            party_config: Some(group.config.pathing_config.clone()),
        },
        execution_plan,
        dispatched: false,
        completed: false,
    })
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
        Some(dispatcher) => {
            execute_prepared_javascript_with_task_dispatcher_context_and_cancellation(
                &prepared,
                dispatcher,
                roots.task_invocation_planning_context(),
                cancellation,
            )
        }
        None => execute_prepared_javascript_with_task_mode_context_and_cancellation(
            &prepared,
            roots.task_invocation_mode,
            roots.task_invocation_planning_context(),
            cancellation,
        ),
    }
}
