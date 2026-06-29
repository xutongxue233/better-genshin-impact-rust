use crate::{
    plan_common_job, CommonJobExecutionPlan, CommonJobLiveExecutionReport, Result, TaskError,
};
use bgi_core::{
    read_pathing_task, PathingActionPlan, PathingCommonJobActionPlan, PathingExecutionPlan,
    PathingLogOutputActionPlan, PathingPreflightPlan, PathingSetTimeActionPlan, PathingSummary,
    PathingWaypointPhase, PathingWaypointPlan,
};
use bgi_vision::Size;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoPathingExecutionConfig {
    pub route: String,
}

impl AutoPathingExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPathingExecutionPlan {
    pub source: &'static str,
    pub route: String,
    pub normalized_path: PathBuf,
    pub summary: PathingSummary,
    pub execution_plan: PathingExecutionPlan,
    pub dispatched: bool,
    pub completed: bool,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPathingActionBoundaryReport {
    pub source: &'static str,
    pub route: String,
    pub normalized_path: PathBuf,
    pub boundary_completed: bool,
    pub native_pathing_completed: bool,
    pub executed_actions: usize,
    pub skipped_actions: usize,
    pub unsupported_actions: usize,
    pub invalid_actions: usize,
    pub unsupported_phases: usize,
    pub waypoint_reports: Vec<PathingWaypointBoundaryReport>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPathingExecutionReport {
    pub source: &'static str,
    pub route: String,
    pub normalized_path: PathBuf,
    pub completed: bool,
    pub success_end: bool,
    pub expected_fight_count: usize,
    pub success_fight_count: usize,
    pub preflight_completed: bool,
    pub failed_preflight: Option<AutoPathingFailedPreflight>,
    pub preflight_reports: Vec<AutoPathingPreflightExecutionReport>,
    pub executed_waypoints: usize,
    pub failed_waypoint: Option<AutoPathingFailedWaypoint>,
    pub executed_preflight_phases: usize,
    pub skipped_preflight_phases: usize,
    pub unsupported_preflight_phases: usize,
    pub failed_preflight_phases: usize,
    pub executed_phases: usize,
    pub skipped_phases: usize,
    pub unsupported_phases: usize,
    pub failed_phases: usize,
    pub cancelled: bool,
    pub phase_reports: Vec<AutoPathingPhaseExecutionReport>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPathingFailedPreflight {
    pub phase: AutoPathingPreflightPhase,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPathingFailedWaypoint {
    pub global_index: usize,
    pub segment_index: usize,
    pub segment_waypoint_index: usize,
    pub phase: PathingWaypointPhase,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPathingPreflightExecutionReport {
    pub phase: AutoPathingPreflightPhase,
    pub status: AutoPathingPhaseExecutionStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoPathingResolutionPreflightReport {
    pub status: AutoPathingResolutionPreflightStatus,
    pub capture_size: Size,
    pub minimum_width: u32,
    pub minimum_height: u32,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoPathingResolutionPreflightStatus {
    Passed,
    Skipped,
    Not16By9,
    BelowMinimum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoPathingPreflightPhase {
    SwitchPartyBefore,
    ValidateGameWithTask,
    InitializePathing,
    ConvertWaypointsForTrack,
    DelayBeforeWarmUpNavigation,
    WarmUpNavigation,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPathingPhaseExecutionReport {
    pub global_index: usize,
    pub segment_index: usize,
    pub segment_waypoint_index: usize,
    pub waypoint_type: String,
    pub action: Option<String>,
    pub phase: PathingWaypointPhase,
    pub status: AutoPathingPhaseExecutionStatus,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoPathingPhaseExecutionStatus {
    Executed,
    Skipped,
    Unsupported,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoPathingPhaseExecution {
    pub status: AutoPathingPhaseExecutionStatus,
    pub message: String,
}

impl AutoPathingPhaseExecution {
    pub fn executed(message: impl Into<String>) -> Self {
        Self {
            status: AutoPathingPhaseExecutionStatus::Executed,
            message: message.into(),
        }
    }

    pub fn skipped(message: impl Into<String>) -> Self {
        Self {
            status: AutoPathingPhaseExecutionStatus::Skipped,
            message: message.into(),
        }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            status: AutoPathingPhaseExecutionStatus::Unsupported,
            message: message.into(),
        }
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            status: AutoPathingPhaseExecutionStatus::Failed,
            message: message.into(),
        }
    }
}

pub struct AutoPathingPhaseExecutionContext<'a> {
    pub plan: &'a AutoPathingExecutionPlan,
    pub waypoint: &'a PathingWaypointPlan,
    pub phase: PathingWaypointPhase,
}

pub struct AutoPathingPreflightExecutionContext<'a> {
    pub plan: &'a AutoPathingExecutionPlan,
    pub phase: AutoPathingPreflightPhase,
}

pub trait AutoPathingRuntime {
    fn execute_preflight(
        &mut self,
        context: AutoPathingPreflightExecutionContext<'_>,
    ) -> Result<AutoPathingPhaseExecution> {
        Ok(AutoPathingPhaseExecution::unsupported(format!(
            "{:?} is still native-pending in the Rust AutoPathing runtime for route {}",
            context.phase, context.plan.route
        )))
    }

    fn execute_phase(
        &mut self,
        context: AutoPathingPhaseExecutionContext<'_>,
    ) -> Result<AutoPathingPhaseExecution>;
}

pub struct UnsupportedAutoPathingRuntime;

impl AutoPathingRuntime for UnsupportedAutoPathingRuntime {
    fn execute_phase(
        &mut self,
        context: AutoPathingPhaseExecutionContext<'_>,
    ) -> Result<AutoPathingPhaseExecution> {
        let execution = match context.phase {
            PathingWaypointPhase::BeforeMoveToTarget
            | PathingWaypointPhase::BeforeMoveCloseToTarget => AutoPathingPhaseExecution::skipped(
                "legacy hook phase has no standalone native side effect",
            ),
            PathingWaypointPhase::RunAction => AutoPathingPhaseExecution::unsupported(format!(
                "pathing action {} has no native AutoPathing runtime handler yet",
                context.waypoint.action.as_deref().unwrap_or("<none>")
            )),
            _ => AutoPathingPhaseExecution::unsupported(format!(
                "{:?} is still native-pending in the Rust AutoPathing runtime",
                context.phase
            )),
        };
        Ok(execution)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingWaypointBoundaryReport {
    pub global_index: usize,
    pub segment_index: usize,
    pub segment_waypoint_index: usize,
    pub waypoint_type: String,
    pub action: Option<String>,
    pub phase_reports: Vec<PathingPhaseBoundaryReport>,
    pub action_report: Option<PathingActionBoundaryReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingPhaseBoundaryReport {
    pub phase: PathingWaypointPhase,
    pub status: PathingBoundaryStatus,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingActionBoundaryReport {
    pub action_code: String,
    pub status: PathingBoundaryStatus,
    pub message: String,
    pub common_job_task_key: Option<String>,
    pub common_job_plan: Option<CommonJobExecutionPlan>,
    pub common_job_live_execution: Option<CommonJobLiveExecutionReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathingBoundaryStatus {
    Reported,
    Executed,
    Skipped,
    Unsupported,
    Invalid,
}

pub fn plan_auto_pathing(
    working_directory: impl AsRef<Path>,
    route: &str,
) -> Result<AutoPathingExecutionPlan> {
    let normalized_path = normalize_user_auto_pathing_route(route)?;
    let path = working_directory
        .as_ref()
        .join("User")
        .join("AutoPathing")
        .join(&normalized_path);
    let task =
        read_pathing_task(&path).map_err(|error| TaskError::PathingPlan(error.to_string()))?;
    let summary = task.summary();
    Ok(AutoPathingExecutionPlan {
        source: "UserAutoPathingFile",
        route: route.to_string(),
        normalized_path,
        summary,
        execution_plan: task.execution_plan(),
        dispatched: false,
        completed: false,
        notes:
            "Route JSON is parsed and converted into the migrated PathExecutor preparation plan; native movement dispatch remains pending."
                .to_string(),
    })
}

pub fn execute_auto_pathing_with_runtime<R>(
    plan: &AutoPathingExecutionPlan,
    runtime: &mut R,
) -> Result<AutoPathingExecutionReport>
where
    R: AutoPathingRuntime,
{
    execute_auto_pathing_with_runtime_and_cancellation(plan, runtime, || false)
}

pub fn evaluate_auto_pathing_resolution_preflight(
    preflight: &PathingPreflightPlan,
    capture_size: Size,
) -> AutoPathingResolutionPreflightReport {
    if !preflight.require_16_by_9_resolution {
        return AutoPathingResolutionPreflightReport {
            status: AutoPathingResolutionPreflightStatus::Skipped,
            capture_size,
            minimum_width: preflight.minimum_width,
            minimum_height: preflight.minimum_height,
            message: "AutoPathing resolution validation skipped because this route has no pathing positions."
                .to_string(),
        };
    }

    if u64::from(capture_size.width) * 9 != u64::from(capture_size.height) * 16 {
        return AutoPathingResolutionPreflightReport {
            status: AutoPathingResolutionPreflightStatus::Not16By9,
            capture_size,
            minimum_width: preflight.minimum_width,
            minimum_height: preflight.minimum_height,
            message: format!(
                "游戏窗口分辨率不是 16:9 ！当前分辨率为 {}x{} ，无法使用地图追踪功能！",
                capture_size.width, capture_size.height
            ),
        };
    }

    if capture_size.width < preflight.minimum_width
        || capture_size.height < preflight.minimum_height
    {
        return AutoPathingResolutionPreflightReport {
            status: AutoPathingResolutionPreflightStatus::BelowMinimum,
            capture_size,
            minimum_width: preflight.minimum_width,
            minimum_height: preflight.minimum_height,
            message: format!(
                "游戏窗口分辨率小于 {}x{} ！当前分辨率为 {}x{} ，无法使用地图追踪功能！",
                preflight.minimum_width,
                preflight.minimum_height,
                capture_size.width,
                capture_size.height
            ),
        };
    }

    AutoPathingResolutionPreflightReport {
        status: AutoPathingResolutionPreflightStatus::Passed,
        capture_size,
        minimum_width: preflight.minimum_width,
        minimum_height: preflight.minimum_height,
        message: format!(
            "AutoPathing resolution {}x{} satisfies legacy map tracking requirements.",
            capture_size.width, capture_size.height
        ),
    }
}

pub fn execute_auto_pathing_with_runtime_and_cancellation<R, F>(
    plan: &AutoPathingExecutionPlan,
    runtime: &mut R,
    mut should_cancel: F,
) -> Result<AutoPathingExecutionReport>
where
    R: AutoPathingRuntime,
    F: FnMut() -> bool,
{
    let mut report = AutoPathingExecutionReport {
        source: plan.source,
        route: plan.route.clone(),
        normalized_path: plan.normalized_path.clone(),
        completed: false,
        success_end: false,
        expected_fight_count: plan.execution_plan.expected_fight_count,
        success_fight_count: 0,
        preflight_completed: false,
        failed_preflight: None,
        preflight_reports: Vec::new(),
        executed_waypoints: 0,
        failed_waypoint: None,
        executed_preflight_phases: 0,
        skipped_preflight_phases: 0,
        unsupported_preflight_phases: 0,
        failed_preflight_phases: 0,
        executed_phases: 0,
        skipped_phases: 0,
        unsupported_phases: 0,
        failed_phases: 0,
        cancelled: false,
        phase_reports: Vec::new(),
        notes:
            "AutoPathing runtime report executes only phases provided by the injected runtime; unsupported or failed phases stop the route without marking success_end."
                .to_string(),
    };

    if plan.execution_plan.waypoint_count == 0 {
        report.notes =
            "Pathing route contains no waypoints; legacy executor returns without marking success_end."
                .to_string();
        return Ok(report);
    }

    for phase in auto_pathing_preflight_phases(plan) {
        let phase_execution = if should_cancel() {
            report.cancelled = true;
            AutoPathingPhaseExecution {
                status: AutoPathingPhaseExecutionStatus::Cancelled,
                message: "AutoPathing execution cancelled before preflight dispatch".to_string(),
            }
        } else {
            runtime.execute_preflight(AutoPathingPreflightExecutionContext { plan, phase })?
        };

        let preflight_report = AutoPathingPreflightExecutionReport {
            phase,
            status: phase_execution.status,
            message: phase_execution.message.clone(),
        };
        match phase_execution.status {
            AutoPathingPhaseExecutionStatus::Executed => report.executed_preflight_phases += 1,
            AutoPathingPhaseExecutionStatus::Skipped => report.skipped_preflight_phases += 1,
            AutoPathingPhaseExecutionStatus::Unsupported => {
                report.unsupported_preflight_phases += 1
            }
            AutoPathingPhaseExecutionStatus::Failed => report.failed_preflight_phases += 1,
            AutoPathingPhaseExecutionStatus::Cancelled => {}
        }

        let should_stop = matches!(
            phase_execution.status,
            AutoPathingPhaseExecutionStatus::Unsupported
                | AutoPathingPhaseExecutionStatus::Failed
                | AutoPathingPhaseExecutionStatus::Cancelled
        );
        report.preflight_reports.push(preflight_report);
        if should_stop {
            report.failed_preflight = Some(AutoPathingFailedPreflight {
                phase,
                message: phase_execution.message,
            });
            return Ok(report);
        }
    }
    report.preflight_completed = true;

    for segment in &plan.execution_plan.segments {
        for waypoint in &segment.waypoints {
            let mut waypoint_completed = true;
            for phase in &waypoint.phases {
                let phase_execution = if should_cancel() {
                    report.cancelled = true;
                    AutoPathingPhaseExecution {
                        status: AutoPathingPhaseExecutionStatus::Cancelled,
                        message: "AutoPathing execution cancelled before phase dispatch"
                            .to_string(),
                    }
                } else {
                    runtime.execute_phase(AutoPathingPhaseExecutionContext {
                        plan,
                        waypoint,
                        phase: *phase,
                    })?
                };

                let phase_report = AutoPathingPhaseExecutionReport {
                    global_index: waypoint.global_index,
                    segment_index: waypoint.segment_index,
                    segment_waypoint_index: waypoint.segment_waypoint_index,
                    waypoint_type: waypoint.waypoint_type.clone(),
                    action: waypoint.action.clone(),
                    phase: *phase,
                    status: phase_execution.status,
                    message: phase_execution.message.clone(),
                };
                match phase_execution.status {
                    AutoPathingPhaseExecutionStatus::Executed => report.executed_phases += 1,
                    AutoPathingPhaseExecutionStatus::Skipped => report.skipped_phases += 1,
                    AutoPathingPhaseExecutionStatus::Unsupported => report.unsupported_phases += 1,
                    AutoPathingPhaseExecutionStatus::Failed => report.failed_phases += 1,
                    AutoPathingPhaseExecutionStatus::Cancelled => {}
                }

                if phase_execution.status == AutoPathingPhaseExecutionStatus::Executed
                    && *phase == PathingWaypointPhase::RunAction
                    && waypoint
                        .action
                        .as_deref()
                        .is_some_and(|action| action.eq_ignore_ascii_case("fight"))
                {
                    report.success_fight_count += 1;
                }

                let should_stop = matches!(
                    phase_execution.status,
                    AutoPathingPhaseExecutionStatus::Unsupported
                        | AutoPathingPhaseExecutionStatus::Failed
                        | AutoPathingPhaseExecutionStatus::Cancelled
                );
                report.phase_reports.push(phase_report);
                if should_stop {
                    report.failed_waypoint = Some(AutoPathingFailedWaypoint {
                        global_index: waypoint.global_index,
                        segment_index: waypoint.segment_index,
                        segment_waypoint_index: waypoint.segment_waypoint_index,
                        phase: *phase,
                        message: phase_execution.message,
                    });
                    waypoint_completed = false;
                    break;
                }
            }

            if waypoint_completed {
                report.executed_waypoints += 1;
            } else {
                return Ok(report);
            }
        }
    }

    report.completed = true;
    report.success_end = true;
    Ok(report)
}

fn auto_pathing_preflight_phases(
    plan: &AutoPathingExecutionPlan,
) -> Vec<AutoPathingPreflightPhase> {
    let preflight = &plan.execution_plan.preflight;
    let mut phases = Vec::new();
    if preflight.switch_party_before {
        phases.push(AutoPathingPreflightPhase::SwitchPartyBefore);
    }
    if preflight.validate_game_with_task {
        phases.push(AutoPathingPreflightPhase::ValidateGameWithTask);
    }
    if preflight.initialize_pathing {
        phases.push(AutoPathingPreflightPhase::InitializePathing);
    }
    if preflight.convert_waypoints_for_track {
        phases.push(AutoPathingPreflightPhase::ConvertWaypointsForTrack);
    }
    if preflight.delay_before_warm_up_ms > 0 {
        phases.push(AutoPathingPreflightPhase::DelayBeforeWarmUpNavigation);
    }
    if preflight.warm_up_navigation {
        phases.push(AutoPathingPreflightPhase::WarmUpNavigation);
    }
    phases
}

pub fn execute_auto_pathing_action_boundary_with_live_executor<F>(
    plan: &AutoPathingExecutionPlan,
    capture_size: Size,
    mut live_executor: F,
) -> Result<AutoPathingActionBoundaryReport>
where
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
{
    let mut report = AutoPathingActionBoundaryReport {
        source: plan.source,
        route: plan.route.clone(),
        normalized_path: plan.normalized_path.clone(),
        boundary_completed: false,
        native_pathing_completed: false,
        executed_actions: 0,
        skipped_actions: 0,
        unsupported_actions: 0,
        invalid_actions: 0,
        unsupported_phases: 0,
        waypoint_reports: Vec::new(),
        notes:
            "Pathing runtime boundary reports ready pure actions and can hand mapped common-job actions to a caller-provided live executor; movement, teleport, combat, recovery, and camera phases remain native-pending."
                .to_string(),
    };

    for segment in &plan.execution_plan.segments {
        for waypoint in &segment.waypoints {
            let mut waypoint_report = PathingWaypointBoundaryReport {
                global_index: waypoint.global_index,
                segment_index: waypoint.segment_index,
                segment_waypoint_index: waypoint.segment_waypoint_index,
                waypoint_type: waypoint.waypoint_type.clone(),
                action: waypoint.action.clone(),
                phase_reports: Vec::new(),
                action_report: None,
            };

            for phase in &waypoint.phases {
                let phase_report = if *phase == PathingWaypointPhase::RunAction {
                    let action_report = execute_pathing_action_boundary(
                        waypoint.action.as_deref(),
                        waypoint.action_plan.as_ref(),
                        capture_size,
                        &mut live_executor,
                    )?;
                    let status = action_report.status;
                    let reason = action_report.message.clone();
                    waypoint_report.action_report = Some(action_report);
                    PathingPhaseBoundaryReport {
                        phase: *phase,
                        status,
                        reason,
                    }
                } else {
                    pathing_phase_boundary_report(*phase)
                };

                if phase_report.status == PathingBoundaryStatus::Unsupported {
                    report.unsupported_phases += 1;
                }
                waypoint_report.phase_reports.push(phase_report);
            }

            if let Some(action_report) = waypoint_report.action_report.as_ref() {
                match action_report.status {
                    PathingBoundaryStatus::Executed => report.executed_actions += 1,
                    PathingBoundaryStatus::Skipped => report.skipped_actions += 1,
                    PathingBoundaryStatus::Unsupported => report.unsupported_actions += 1,
                    PathingBoundaryStatus::Invalid => report.invalid_actions += 1,
                    PathingBoundaryStatus::Reported => {}
                }
            }
            report.waypoint_reports.push(waypoint_report);
        }
    }

    report.boundary_completed = true;
    report.native_pathing_completed = false;
    Ok(report)
}

fn pathing_phase_boundary_report(phase: PathingWaypointPhase) -> PathingPhaseBoundaryReport {
    let (status, reason) = match phase {
        PathingWaypointPhase::RecoverWhenLowHp => (
            PathingBoundaryStatus::Unsupported,
            "low-HP recovery detection and food dispatch are still native-pending",
        ),
        PathingWaypointPhase::HandleTeleport => (
            PathingBoundaryStatus::Unsupported,
            "teleport handling still depends on the native map/QuickTeleport stack",
        ),
        PathingWaypointPhase::FaceTo => (
            PathingBoundaryStatus::Unsupported,
            "camera orientation dispatch is not implemented in the Rust pathing boundary",
        ),
        PathingWaypointPhase::MoveTo | PathingWaypointPhase::MoveCloseTo => (
            PathingBoundaryStatus::Unsupported,
            "path movement and close-range adjustment are not implemented in the Rust pathing boundary",
        ),
        PathingWaypointPhase::BeforeMoveToTarget
        | PathingWaypointPhase::BeforeMoveCloseToTarget => (
            PathingBoundaryStatus::Reported,
            "legacy hook phase recorded; no native side effect is required in the Rust boundary",
        ),
        PathingWaypointPhase::RunAction => (
            PathingBoundaryStatus::Skipped,
            "run-action phase requires action-specific dispatch",
        ),
    };

    PathingPhaseBoundaryReport {
        phase,
        status,
        reason: reason.to_string(),
    }
}

fn execute_pathing_action_boundary<F>(
    action: Option<&str>,
    action_plan: Option<&PathingActionPlan>,
    capture_size: Size,
    live_executor: &mut F,
) -> Result<PathingActionBoundaryReport>
where
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
{
    match action_plan {
        Some(PathingActionPlan::SetTime(set_time)) => {
            execute_set_time_pathing_action(set_time, capture_size, live_executor)
        }
        Some(PathingActionPlan::LogOutput(log_output)) => {
            execute_log_output_pathing_action(log_output)
        }
        Some(PathingActionPlan::CommonJob(common_job)) => {
            execute_common_job_pathing_action(common_job, capture_size, live_executor)
        }
        Some(PathingActionPlan::LinneaMining(_)) => Ok(PathingActionBoundaryReport {
            action_code: "linnea_mining".to_string(),
            status: PathingBoundaryStatus::Unsupported,
            message:
                "linnea_mining requires avatar switching, capture, ONNX inference, aiming, and overlay execution before it can run natively"
                    .to_string(),
            common_job_task_key: None,
            common_job_plan: None,
            common_job_live_execution: None,
        }),
        None => Ok(PathingActionBoundaryReport {
            action_code: action.unwrap_or("<none>").to_string(),
            status: PathingBoundaryStatus::Unsupported,
            message:
                "pathing action has no Rust action plan yet, so it is reported but not executed"
                    .to_string(),
            common_job_task_key: None,
            common_job_plan: None,
            common_job_live_execution: None,
        }),
    }
}

fn execute_log_output_pathing_action(
    log_output: &PathingLogOutputActionPlan,
) -> Result<PathingActionBoundaryReport> {
    Ok(PathingActionBoundaryReport {
        action_code: log_output.action_code.clone(),
        status: PathingBoundaryStatus::Reported,
        message: format!("pathing log_output action reported: {}", log_output.message),
        common_job_task_key: None,
        common_job_plan: None,
        common_job_live_execution: None,
    })
}

fn execute_common_job_pathing_action<F>(
    common_job: &PathingCommonJobActionPlan,
    capture_size: Size,
    live_executor: &mut F,
) -> Result<PathingActionBoundaryReport>
where
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
{
    if !common_job.executor_ready {
        return Ok(PathingActionBoundaryReport {
            action_code: common_job.action_code.clone(),
            status: PathingBoundaryStatus::Invalid,
            message: format!(
                "pathing action {} is not executor-ready",
                common_job.action_code
            ),
            common_job_task_key: Some(common_job.common_job_task_key.clone()),
            common_job_plan: None,
            common_job_live_execution: None,
        });
    }

    let config = json!({
        "captureSize": capture_size,
    });
    let Some(common_job_plan) = plan_common_job(&common_job.common_job_task_key, Some(&config))?
    else {
        return Ok(PathingActionBoundaryReport {
            action_code: common_job.action_code.clone(),
            status: PathingBoundaryStatus::Invalid,
            message: format!(
                "common-job task {} is not registered",
                common_job.common_job_task_key
            ),
            common_job_task_key: Some(common_job.common_job_task_key.clone()),
            common_job_plan: None,
            common_job_live_execution: None,
        });
    };

    if !common_job_plan.executor_ready() {
        return Ok(PathingActionBoundaryReport {
            action_code: common_job.action_code.clone(),
            status: PathingBoundaryStatus::Unsupported,
            message: format!(
                "common-job task {} is planned but not executor-ready",
                common_job_plan.task_key()
            ),
            common_job_task_key: Some(common_job.common_job_task_key.clone()),
            common_job_plan: Some(common_job_plan),
            common_job_live_execution: None,
        });
    }

    match live_executor(&common_job_plan)? {
        Some(common_job_live_execution) => Ok(PathingActionBoundaryReport {
            action_code: common_job.action_code.clone(),
            status: PathingBoundaryStatus::Executed,
            message: format!(
                "pathing {} action executed through {} common-job live boundary",
                common_job.action_code,
                common_job_plan.task_key()
            ),
            common_job_task_key: Some(common_job.common_job_task_key.clone()),
            common_job_plan: Some(common_job_plan),
            common_job_live_execution: Some(common_job_live_execution),
        }),
        None => Ok(PathingActionBoundaryReport {
            action_code: common_job.action_code.clone(),
            status: PathingBoundaryStatus::Skipped,
            message: format!(
                "pathing {} action reached {} common-job plan, but live executor returned no report",
                common_job.action_code,
                common_job_plan.task_key()
            ),
            common_job_task_key: Some(common_job.common_job_task_key.clone()),
            common_job_plan: Some(common_job_plan),
            common_job_live_execution: None,
        }),
    }
}

fn execute_set_time_pathing_action<F>(
    set_time: &PathingSetTimeActionPlan,
    capture_size: Size,
    live_executor: &mut F,
) -> Result<PathingActionBoundaryReport>
where
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
{
    if !set_time.executor_ready {
        return Ok(PathingActionBoundaryReport {
            action_code: set_time.action_code.clone(),
            status: PathingBoundaryStatus::Invalid,
            message: set_time
                .parse_error
                .clone()
                .unwrap_or_else(|| "set_time action plan is not executor-ready".to_string()),
            common_job_task_key: Some(set_time.common_job_task_key.clone()),
            common_job_plan: None,
            common_job_live_execution: None,
        });
    }

    let Some(hour) = set_time.hour else {
        return invalid_set_time_action_report(set_time, "set_time action is missing hour");
    };
    let Some(minute) = set_time.minute else {
        return invalid_set_time_action_report(set_time, "set_time action is missing minute");
    };
    let Some(skip) = set_time.skip_time_adjustment_animation else {
        return invalid_set_time_action_report(
            set_time,
            "set_time action is missing skip animation flag",
        );
    };

    let config = json!({
        "hour": hour,
        "minute": minute,
        "skip": skip,
        "captureSize": capture_size,
    });
    let Some(common_job_plan) = plan_common_job(&set_time.common_job_task_key, Some(&config))?
    else {
        return Ok(PathingActionBoundaryReport {
            action_code: set_time.action_code.clone(),
            status: PathingBoundaryStatus::Invalid,
            message: format!(
                "common-job task {} is not registered",
                set_time.common_job_task_key
            ),
            common_job_task_key: Some(set_time.common_job_task_key.clone()),
            common_job_plan: None,
            common_job_live_execution: None,
        });
    };

    if !common_job_plan.executor_ready() {
        return Ok(PathingActionBoundaryReport {
            action_code: set_time.action_code.clone(),
            status: PathingBoundaryStatus::Unsupported,
            message: format!(
                "common-job task {} is planned but not executor-ready",
                common_job_plan.task_key()
            ),
            common_job_task_key: Some(set_time.common_job_task_key.clone()),
            common_job_plan: Some(common_job_plan),
            common_job_live_execution: None,
        });
    }

    match live_executor(&common_job_plan)? {
        Some(common_job_live_execution) => Ok(PathingActionBoundaryReport {
            action_code: set_time.action_code.clone(),
            status: PathingBoundaryStatus::Executed,
            message: format!(
                "pathing set_time action executed through {} common-job live boundary",
                common_job_plan.task_key()
            ),
            common_job_task_key: Some(set_time.common_job_task_key.clone()),
            common_job_plan: Some(common_job_plan),
            common_job_live_execution: Some(common_job_live_execution),
        }),
        None => Ok(PathingActionBoundaryReport {
            action_code: set_time.action_code.clone(),
            status: PathingBoundaryStatus::Skipped,
            message: format!(
                "pathing set_time action reached {} common-job plan, but live executor returned no report",
                common_job_plan.task_key()
            ),
            common_job_task_key: Some(set_time.common_job_task_key.clone()),
            common_job_plan: Some(common_job_plan),
            common_job_live_execution: None,
        }),
    }
}

fn invalid_set_time_action_report(
    set_time: &PathingSetTimeActionPlan,
    message: impl Into<String>,
) -> Result<PathingActionBoundaryReport> {
    Ok(PathingActionBoundaryReport {
        action_code: set_time.action_code.clone(),
        status: PathingBoundaryStatus::Invalid,
        message: message.into(),
        common_job_task_key: Some(set_time.common_job_task_key.clone()),
        common_job_plan: None,
        common_job_live_execution: None,
    })
}

fn normalize_user_auto_pathing_route(route: &str) -> Result<PathBuf> {
    let route = route.trim().replace('\\', "/");
    if route.is_empty() {
        return Err(TaskError::EmptyPathingRoute);
    }
    let path = PathBuf::from(&route);
    if path.is_absolute() {
        return Err(TaskError::InvalidPathingRoute(route));
    }
    if path
        .components()
        .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(TaskError::InvalidPathingRoute(route));
    }
    Ok(path)
}
