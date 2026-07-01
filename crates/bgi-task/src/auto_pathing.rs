use crate::{
    plan_common_job, CommonJobExecutionPlan, CommonJobLiveExecutionReport, Result, TaskError,
    TeleportExecutionReport, TELEPORT_TASK_KEY,
};
use bgi_core::{
    legacy_track_map_point_for_pathing, read_pathing_task, PathingActionPlan,
    PathingCommonJobActionPlan, PathingCoordinateSpace, PathingExecutionPlan,
    PathingLogOutputActionPlan, PathingMovementDependency, PathingMovementPhaseContract,
    PathingMovementSegmentContract, PathingMovementWaypointContract, PathingPoint,
    PathingPreflightPlan, PathingSetTimeActionPlan, PathingSummary, PathingTask,
    PathingTrackConversionContext, PathingUseGadgetActionPlan, PathingWaypointPhase,
    PathingWaypointPlan,
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
    pub completion_scope: AutoPathingActionBoundaryCompletionScope,
    pub boundary_completed: bool,
    pub movement_attempted: bool,
    pub movement_completion_status: AutoPathingMovementCompletionStatus,
    pub native_pathing_completed: bool,
    pub movement_executor_ready: bool,
    pub movement_contract_version: u8,
    pub movement_pending_dependencies: Vec<PathingMovementDependency>,
    pub movement_segment_count: usize,
    pub movement_waypoint_count: usize,
    pub movement_report: Option<AutoPathingMovementBoundaryReport>,
    pub executed_actions: usize,
    pub skipped_actions: usize,
    pub unsupported_actions: usize,
    pub invalid_actions: usize,
    pub unsupported_phases: usize,
    pub waypoint_reports: Vec<PathingWaypointBoundaryReport>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoPathingActionBoundaryCompletionScope {
    ActionBoundaryOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoPathingMovementCompletionStatus {
    NotAttempted,
    NativePending,
    Completed,
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
pub struct AutoPathingMovementBoundaryReport {
    pub source: &'static str,
    pub route: String,
    pub normalized_path: PathBuf,
    pub movement_contract_consumed: bool,
    pub movement_completed: bool,
    pub movement_completion_status: AutoPathingMovementCompletionStatus,
    pub native_pathing_completed: bool,
    pub movement_executor_ready: bool,
    pub movement_contract_version: u8,
    pub movement_pending_dependencies: Vec<PathingMovementDependency>,
    pub movement_segment_count: usize,
    pub movement_waypoint_count: usize,
    pub executed_phases: usize,
    pub skipped_phases: usize,
    pub unsupported_phases: usize,
    pub failed_phases: usize,
    pub cancelled_phases: usize,
    pub executed_pre_teleport_delays: usize,
    pub skipped_pre_teleport_delays: usize,
    pub unsupported_pre_teleport_delays: usize,
    pub failed_pre_teleport_delays: usize,
    pub cancelled_pre_teleport_delays: usize,
    pub failed_phase: Option<AutoPathingMovementFailedPhase>,
    pub failed_segment_delay: Option<AutoPathingMovementFailedSegmentDelay>,
    pub segment_reports: Vec<PathingMovementSegmentBoundaryReport>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPathingMovementFailedPhase {
    pub global_index: usize,
    pub segment_index: usize,
    pub segment_waypoint_index: usize,
    pub phase: PathingWaypointPhase,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPathingMovementFailedSegmentDelay {
    pub segment_index: usize,
    pub pre_teleport_delay_ms: u32,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingMovementSegmentBoundaryReport {
    pub segment_index: usize,
    pub starts_with_teleport: bool,
    pub pre_teleport_delay_ms: u32,
    pub pre_teleport_delay_status: Option<AutoPathingPhaseExecutionStatus>,
    pub pre_teleport_delay_message: Option<String>,
    pub waypoint_reports: Vec<PathingMovementWaypointBoundaryReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingMovementWaypointBoundaryReport {
    pub global_index: usize,
    pub segment_index: usize,
    pub segment_waypoint_index: usize,
    pub waypoint_type: String,
    pub move_mode: String,
    pub action: Option<String>,
    pub phase_reports: Vec<PathingMovementPhaseBoundaryReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingMovementPhaseBoundaryReport {
    pub phase: PathingWaypointPhase,
    pub status: AutoPathingPhaseExecutionStatus,
    pub message: String,
    pub pending_dependencies: Vec<PathingMovementDependency>,
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

pub struct AutoPathingMovementPhaseExecutionContext<'a> {
    pub plan: &'a AutoPathingExecutionPlan,
    pub segment: &'a PathingMovementSegmentContract,
    pub waypoint: &'a PathingMovementWaypointContract,
    pub phase: &'a PathingMovementPhaseContract,
}

pub struct AutoPathingSegmentDelayExecutionContext<'a> {
    pub plan: &'a AutoPathingExecutionPlan,
    pub segment: &'a PathingMovementSegmentContract,
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

pub trait AutoPathingMovementRuntime {
    fn execute_segment_pre_teleport_delay(
        &mut self,
        context: AutoPathingSegmentDelayExecutionContext<'_>,
    ) -> Result<AutoPathingPhaseExecution> {
        if context.segment.pre_teleport_delay_ms == 0 {
            return Ok(AutoPathingPhaseExecution::skipped(format!(
                "AutoPathing segment {} has no legacy pre-teleport delay",
                context.segment.segment_index
            )));
        }

        Ok(AutoPathingPhaseExecution::unsupported(format!(
            "AutoPathing segment {} has a legacy pre-teleport delay of {}ms for route {}, but the injected movement runtime has no delay adapter yet",
            context.segment.segment_index,
            context.segment.pre_teleport_delay_ms,
            context.plan.route
        )))
    }

    fn execute_movement_phase(
        &mut self,
        context: AutoPathingMovementPhaseExecutionContext<'_>,
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

pub struct UnsupportedAutoPathingMovementRuntime;

impl AutoPathingMovementRuntime for UnsupportedAutoPathingMovementRuntime {
    fn execute_movement_phase(
        &mut self,
        context: AutoPathingMovementPhaseExecutionContext<'_>,
    ) -> Result<AutoPathingPhaseExecution> {
        if context.phase.pending_dependencies.is_empty() {
            return Ok(AutoPathingPhaseExecution::skipped(format!(
                "{:?} has no native side effect for AutoPathing route {}",
                context.phase.phase, context.plan.route
            )));
        }

        Ok(AutoPathingPhaseExecution::unsupported(format!(
            "{:?} is still native-pending for AutoPathing route {} at waypoint {} because it requires {:?}",
            context.phase.phase,
            context.plan.route,
            context.waypoint.global_index,
            context.phase.pending_dependencies
        )))
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
    pub common_job_task_key: Option<String>,
    pub common_job_plan: Option<CommonJobExecutionPlan>,
    pub common_job_live_execution: Option<CommonJobLiveExecutionReport>,
    pub navigation_seed: Option<PathingNavigationSeedReport>,
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingNavigationSeedReport {
    pub source_task_key: String,
    pub previous_position: PathingPoint,
    pub map_name: Option<String>,
    pub coordinate_space: PathingCoordinateSpace,
    pub requires_track_conversion: bool,
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
    plan_auto_pathing_with_legacy_track_converter(working_directory, route)
}

pub fn plan_auto_pathing_with_legacy_track_converter(
    working_directory: impl AsRef<Path>,
    route: &str,
) -> Result<AutoPathingExecutionPlan> {
    plan_auto_pathing_with_track_converter(
        working_directory,
        route,
        legacy_track_map_point_for_pathing,
    )
}

pub fn plan_auto_pathing_with_track_converter<F>(
    working_directory: impl AsRef<Path>,
    route: &str,
    converter: F,
) -> Result<AutoPathingExecutionPlan>
where
    F: FnMut(PathingTrackConversionContext<'_>) -> Option<PathingPoint>,
{
    plan_auto_pathing_with_execution_plan(working_directory, route, |task| {
        task.execution_plan_with_track_converter(converter)
    })
}

fn plan_auto_pathing_with_execution_plan<F>(
    working_directory: impl AsRef<Path>,
    route: &str,
    build_execution_plan: F,
) -> Result<AutoPathingExecutionPlan>
where
    F: FnOnce(&PathingTask) -> PathingExecutionPlan,
{
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
        execution_plan: build_execution_plan(&task),
        dispatched: false,
        completed: false,
        notes:
            "Route JSON is parsed and converted into the migrated PathExecutor preparation plan; Teyvat routes are mapped into legacy TrackMap coordinates before movement-contract reporting, and the desktop action boundary can consume healthy RecoverWhenLowHp probes, run the AutoEat QuickUseGadget low-HP recovery slice with a follow-up HP probe when a desktop dispatcher is injected, model the PathExecutor use_gadget not_wait QuickUseGadget action sequence, execute HandleTeleport through the Teleport common-job live bridge, execute cancellable injected legacy pre-teleport segment delays once earlier movement phases advance, and honor the only-in-teleport recovery gate; sequence-safe action input dispatch, native movement dispatch, and full PathExecutor recovery side effects remain pending."
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

pub fn execute_auto_pathing_movement_contract_with_runtime<R>(
    plan: &AutoPathingExecutionPlan,
    runtime: &mut R,
) -> Result<AutoPathingMovementBoundaryReport>
where
    R: AutoPathingMovementRuntime,
{
    execute_auto_pathing_movement_contract_with_runtime_and_cancellation(plan, runtime, || false)
}

pub fn execute_auto_pathing_movement_contract_with_runtime_and_cancellation<R, F>(
    plan: &AutoPathingExecutionPlan,
    runtime: &mut R,
    mut should_cancel: F,
) -> Result<AutoPathingMovementBoundaryReport>
where
    R: AutoPathingMovementRuntime,
    F: FnMut() -> bool,
{
    let contract = &plan.execution_plan.movement_contract;
    let mut report = AutoPathingMovementBoundaryReport {
        source: plan.source,
        route: plan.route.clone(),
        normalized_path: plan.normalized_path.clone(),
        movement_contract_consumed: contract.waypoint_count > 0,
        movement_completed: false,
        movement_completion_status: AutoPathingMovementCompletionStatus::NotAttempted,
        native_pathing_completed: false,
        movement_executor_ready: contract.movement_executor_ready,
        movement_contract_version: contract.contract_version,
        movement_pending_dependencies: contract.pending_dependencies.clone(),
        movement_segment_count: contract.segment_count,
        movement_waypoint_count: contract.waypoint_count,
        executed_phases: 0,
        skipped_phases: 0,
        unsupported_phases: 0,
        failed_phases: 0,
        cancelled_phases: 0,
        executed_pre_teleport_delays: 0,
        skipped_pre_teleport_delays: 0,
        unsupported_pre_teleport_delays: 0,
        failed_pre_teleport_delays: 0,
        cancelled_pre_teleport_delays: 0,
        failed_phase: None,
        failed_segment_delay: None,
        segment_reports: Vec::new(),
        notes:
            "AutoPathing movement contract consumer reports native PathExecutor phase readiness, executes injected legacy pre-teleport segment delays, and does not claim completion unless every injected movement boundary succeeds."
                .to_string(),
    };

    if contract.waypoint_count == 0 {
        return Ok(report);
    }

    for segment in &contract.segments {
        let mut segment_report = PathingMovementSegmentBoundaryReport {
            segment_index: segment.segment_index,
            starts_with_teleport: segment.starts_with_teleport,
            pre_teleport_delay_ms: segment.pre_teleport_delay_ms,
            pre_teleport_delay_status: None,
            pre_teleport_delay_message: None,
            waypoint_reports: Vec::new(),
        };

        if segment.pre_teleport_delay_ms > 0 {
            let delay_execution = if should_cancel() {
                AutoPathingPhaseExecution {
                    status: AutoPathingPhaseExecutionStatus::Cancelled,
                    message: "AutoPathing movement execution cancelled before pre-teleport delay"
                        .to_string(),
                }
            } else {
                runtime.execute_segment_pre_teleport_delay(
                    AutoPathingSegmentDelayExecutionContext { plan, segment },
                )?
            };

            match delay_execution.status {
                AutoPathingPhaseExecutionStatus::Executed => {
                    report.executed_pre_teleport_delays += 1
                }
                AutoPathingPhaseExecutionStatus::Skipped => report.skipped_pre_teleport_delays += 1,
                AutoPathingPhaseExecutionStatus::Unsupported => {
                    report.unsupported_pre_teleport_delays += 1
                }
                AutoPathingPhaseExecutionStatus::Failed => report.failed_pre_teleport_delays += 1,
                AutoPathingPhaseExecutionStatus::Cancelled => {
                    report.cancelled_pre_teleport_delays += 1
                }
            }

            let should_stop = matches!(
                delay_execution.status,
                AutoPathingPhaseExecutionStatus::Unsupported
                    | AutoPathingPhaseExecutionStatus::Failed
                    | AutoPathingPhaseExecutionStatus::Cancelled
            );
            let delay_message = delay_execution.message.clone();
            segment_report.pre_teleport_delay_status = Some(delay_execution.status);
            segment_report.pre_teleport_delay_message = Some(delay_execution.message);

            if should_stop {
                report.failed_segment_delay = Some(AutoPathingMovementFailedSegmentDelay {
                    segment_index: segment.segment_index,
                    pre_teleport_delay_ms: segment.pre_teleport_delay_ms,
                    message: delay_message,
                });
                report.segment_reports.push(segment_report);
                report.movement_completion_status =
                    AutoPathingMovementCompletionStatus::NativePending;
                return Ok(report);
            }
        }

        for waypoint in &segment.waypoints {
            let mut waypoint_report = PathingMovementWaypointBoundaryReport {
                global_index: waypoint.global_index,
                segment_index: waypoint.segment_index,
                segment_waypoint_index: waypoint.segment_waypoint_index,
                waypoint_type: waypoint.waypoint_type.clone(),
                move_mode: waypoint.move_mode.clone(),
                action: waypoint.action.clone(),
                phase_reports: Vec::new(),
            };

            for phase in &waypoint.phase_contracts {
                let phase_execution = if should_cancel() {
                    AutoPathingPhaseExecution {
                        status: AutoPathingPhaseExecutionStatus::Cancelled,
                        message: "AutoPathing movement execution cancelled before phase dispatch"
                            .to_string(),
                    }
                } else {
                    runtime.execute_movement_phase(AutoPathingMovementPhaseExecutionContext {
                        plan,
                        segment,
                        waypoint,
                        phase,
                    })?
                };

                match phase_execution.status {
                    AutoPathingPhaseExecutionStatus::Executed => report.executed_phases += 1,
                    AutoPathingPhaseExecutionStatus::Skipped => report.skipped_phases += 1,
                    AutoPathingPhaseExecutionStatus::Unsupported => report.unsupported_phases += 1,
                    AutoPathingPhaseExecutionStatus::Failed => report.failed_phases += 1,
                    AutoPathingPhaseExecutionStatus::Cancelled => report.cancelled_phases += 1,
                }

                let should_stop = matches!(
                    phase_execution.status,
                    AutoPathingPhaseExecutionStatus::Unsupported
                        | AutoPathingPhaseExecutionStatus::Failed
                        | AutoPathingPhaseExecutionStatus::Cancelled
                );
                let message = phase_execution.message.clone();
                waypoint_report
                    .phase_reports
                    .push(PathingMovementPhaseBoundaryReport {
                        phase: phase.phase,
                        status: phase_execution.status,
                        message: phase_execution.message,
                        pending_dependencies: phase.pending_dependencies.clone(),
                    });

                if should_stop {
                    report.failed_phase = Some(AutoPathingMovementFailedPhase {
                        global_index: waypoint.global_index,
                        segment_index: waypoint.segment_index,
                        segment_waypoint_index: waypoint.segment_waypoint_index,
                        phase: phase.phase,
                        message,
                    });
                    segment_report.waypoint_reports.push(waypoint_report);
                    report.segment_reports.push(segment_report);
                    report.movement_completion_status =
                        AutoPathingMovementCompletionStatus::NativePending;
                    return Ok(report);
                }
            }

            segment_report.waypoint_reports.push(waypoint_report);
        }

        report.segment_reports.push(segment_report);
    }

    report.movement_completed = true;
    report.native_pathing_completed = true;
    report.movement_completion_status = AutoPathingMovementCompletionStatus::Completed;
    Ok(report)
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
    live_executor: F,
) -> Result<AutoPathingActionBoundaryReport>
where
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
{
    let mut movement_runtime = UnsupportedAutoPathingMovementRuntime;
    execute_auto_pathing_action_boundary_with_movement_runtime(
        plan,
        capture_size,
        &mut movement_runtime,
        live_executor,
    )
}

pub fn execute_auto_pathing_action_boundary_with_movement_runtime<R, F>(
    plan: &AutoPathingExecutionPlan,
    capture_size: Size,
    movement_runtime: &mut R,
    live_executor: F,
) -> Result<AutoPathingActionBoundaryReport>
where
    R: AutoPathingMovementRuntime,
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
{
    execute_auto_pathing_action_boundary_with_movement_runtime_and_cancellation(
        plan,
        capture_size,
        movement_runtime,
        || false,
        live_executor,
    )
}

pub fn execute_auto_pathing_action_boundary_with_movement_runtime_and_cancellation<R, F, C>(
    plan: &AutoPathingExecutionPlan,
    capture_size: Size,
    movement_runtime: &mut R,
    should_cancel: C,
    mut live_executor: F,
) -> Result<AutoPathingActionBoundaryReport>
where
    R: AutoPathingMovementRuntime,
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
    C: FnMut() -> bool,
{
    let movement_contract = &plan.execution_plan.movement_contract;
    let mut teleporting_movement_runtime = TeleportingAutoPathingMovementRuntime::new(
        movement_runtime,
        capture_size,
        &mut live_executor,
    );
    let movement_report = execute_auto_pathing_movement_contract_with_runtime_and_cancellation(
        plan,
        &mut teleporting_movement_runtime,
        should_cancel,
    )?;
    let mut movement_teleport_phase_reports =
        teleporting_movement_runtime.into_teleport_phase_reports();
    let movement_attempted = movement_report.executed_phases > 0
        || movement_report.failed_phases > 0
        || movement_report.cancelled_phases > 0;
    let mut report = AutoPathingActionBoundaryReport {
        source: plan.source,
        route: plan.route.clone(),
        normalized_path: plan.normalized_path.clone(),
        completion_scope: AutoPathingActionBoundaryCompletionScope::ActionBoundaryOnly,
        boundary_completed: false,
        movement_attempted,
        movement_completion_status: movement_report.movement_completion_status,
        native_pathing_completed: false,
        movement_executor_ready: movement_contract.movement_executor_ready,
        movement_contract_version: movement_contract.contract_version,
        movement_pending_dependencies: movement_contract.pending_dependencies.clone(),
        movement_segment_count: movement_contract.segment_count,
        movement_waypoint_count: movement_contract.waypoint_count,
        movement_report: Some(movement_report),
        executed_actions: 0,
        skipped_actions: 0,
        unsupported_actions: 0,
        invalid_actions: 0,
        unsupported_phases: 0,
        waypoint_reports: Vec::new(),
        notes:
            "Pathing runtime boundary reports ready pure actions, can hand mapped run-actions and teleport phases to a caller-provided live executor, consumes the movement contract through an injectable movement runtime, and consumes Teleport navigation seeds into previous-position reports when the live bridge provides them; desktop movement adapters, final navigation state writes, combat, recovery, and camera phases remain native-pending."
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
                    let common_job_task_key = action_report.common_job_task_key.clone();
                    let common_job_plan = action_report.common_job_plan.clone();
                    waypoint_report.action_report = Some(action_report);
                    PathingPhaseBoundaryReport {
                        phase: *phase,
                        status,
                        reason,
                        common_job_task_key,
                        common_job_plan,
                        common_job_live_execution: None,
                        navigation_seed: None,
                    }
                } else if *phase == PathingWaypointPhase::HandleTeleport {
                    if let Some(phase_report) = take_movement_teleport_phase_report(
                        &mut movement_teleport_phase_reports,
                        waypoint,
                        *phase,
                    ) {
                        phase_report
                    } else {
                        execute_teleport_phase_boundary(
                            waypoint,
                            plan,
                            capture_size,
                            &mut live_executor,
                        )?
                    }
                } else {
                    pathing_phase_boundary_report(*phase, waypoint, plan, capture_size)?
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

struct TeleportingAutoPathingMovementRuntime<'a, R, F> {
    inner: &'a mut R,
    capture_size: Size,
    live_executor: &'a mut F,
    teleport_phase_reports: Vec<MovementTeleportPhaseReport>,
}

impl<'a, R, F> TeleportingAutoPathingMovementRuntime<'a, R, F> {
    fn new(inner: &'a mut R, capture_size: Size, live_executor: &'a mut F) -> Self {
        Self {
            inner,
            capture_size,
            live_executor,
            teleport_phase_reports: Vec::new(),
        }
    }

    fn into_teleport_phase_reports(self) -> Vec<MovementTeleportPhaseReport> {
        self.teleport_phase_reports
    }
}

impl<R, F> AutoPathingMovementRuntime for TeleportingAutoPathingMovementRuntime<'_, R, F>
where
    R: AutoPathingMovementRuntime,
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
{
    fn execute_movement_phase(
        &mut self,
        context: AutoPathingMovementPhaseExecutionContext<'_>,
    ) -> Result<AutoPathingPhaseExecution> {
        if context.phase.phase != PathingWaypointPhase::HandleTeleport {
            return self.inner.execute_movement_phase(context);
        }

        let force_teleport = context
            .waypoint
            .action
            .as_deref()
            .is_some_and(|action| action.eq_ignore_ascii_case("force_tp"));
        let phase_report = execute_teleport_phase_boundary_for_target(
            context.waypoint.route_point,
            force_teleport,
            &context.plan.execution_plan.map_name,
            self.capture_size,
            self.live_executor,
        )?;
        let phase_execution = auto_pathing_phase_execution_from_pathing_boundary(&phase_report);
        self.teleport_phase_reports
            .push(MovementTeleportPhaseReport {
                key: MovementPhaseReportKey::from_movement_context(&context),
                report: phase_report,
            });
        Ok(phase_execution)
    }
}

struct MovementTeleportPhaseReport {
    key: MovementPhaseReportKey,
    report: PathingPhaseBoundaryReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MovementPhaseReportKey {
    global_index: usize,
    segment_index: usize,
    segment_waypoint_index: usize,
    phase: PathingWaypointPhase,
}

impl MovementPhaseReportKey {
    fn from_movement_context(context: &AutoPathingMovementPhaseExecutionContext<'_>) -> Self {
        Self {
            global_index: context.waypoint.global_index,
            segment_index: context.waypoint.segment_index,
            segment_waypoint_index: context.waypoint.segment_waypoint_index,
            phase: context.phase.phase,
        }
    }

    fn from_waypoint_phase(waypoint: &PathingWaypointPlan, phase: PathingWaypointPhase) -> Self {
        Self {
            global_index: waypoint.global_index,
            segment_index: waypoint.segment_index,
            segment_waypoint_index: waypoint.segment_waypoint_index,
            phase,
        }
    }
}

fn take_movement_teleport_phase_report(
    reports: &mut Vec<MovementTeleportPhaseReport>,
    waypoint: &PathingWaypointPlan,
    phase: PathingWaypointPhase,
) -> Option<PathingPhaseBoundaryReport> {
    let key = MovementPhaseReportKey::from_waypoint_phase(waypoint, phase);
    if let Some(index) = reports.iter().position(|report| report.key == key) {
        return Some(reports.remove(index).report);
    }
    None
}

fn auto_pathing_phase_execution_from_pathing_boundary(
    report: &PathingPhaseBoundaryReport,
) -> AutoPathingPhaseExecution {
    match report.status {
        PathingBoundaryStatus::Executed | PathingBoundaryStatus::Reported => {
            AutoPathingPhaseExecution::executed(report.reason.clone())
        }
        PathingBoundaryStatus::Skipped => AutoPathingPhaseExecution::skipped(report.reason.clone()),
        PathingBoundaryStatus::Unsupported => {
            AutoPathingPhaseExecution::unsupported(report.reason.clone())
        }
        PathingBoundaryStatus::Invalid => AutoPathingPhaseExecution::failed(report.reason.clone()),
    }
}

fn pathing_phase_boundary_report(
    phase: PathingWaypointPhase,
    _waypoint: &PathingWaypointPlan,
    _plan: &AutoPathingExecutionPlan,
    _capture_size: Size,
) -> Result<PathingPhaseBoundaryReport> {
    let (status, reason, common_job_plan): (
        PathingBoundaryStatus,
        String,
        Option<CommonJobExecutionPlan>,
    ) = match phase {
        PathingWaypointPhase::RecoverWhenLowHp => (
            PathingBoundaryStatus::Unsupported,
            "low-HP recovery detection and food dispatch are still native-pending".to_string(),
            None,
        ),
        PathingWaypointPhase::HandleTeleport => unreachable!(
            "HandleTeleport is executed through execute_teleport_phase_boundary before generic phase reporting"
        ),
        PathingWaypointPhase::FaceTo => (
            PathingBoundaryStatus::Unsupported,
            "camera orientation dispatch is not implemented in the Rust pathing boundary"
                .to_string(),
            None,
        ),
        PathingWaypointPhase::MoveTo | PathingWaypointPhase::MoveCloseTo => (
            PathingBoundaryStatus::Unsupported,
            "path movement and close-range adjustment are not implemented in the Rust pathing boundary"
                .to_string(),
            None,
        ),
        PathingWaypointPhase::BeforeMoveToTarget
        | PathingWaypointPhase::BeforeMoveCloseToTarget => (
            PathingBoundaryStatus::Reported,
            "legacy hook phase recorded; no native side effect is required in the Rust boundary"
                .to_string(),
            None,
        ),
        PathingWaypointPhase::RunAction => (
            PathingBoundaryStatus::Skipped,
            "run-action phase requires action-specific dispatch".to_string(),
            None,
        ),
    };

    Ok(PathingPhaseBoundaryReport {
        phase,
        status,
        reason,
        common_job_task_key: common_job_plan
            .as_ref()
            .map(|common_job_plan| common_job_plan.task_key().to_string()),
        common_job_plan,
        common_job_live_execution: None,
        navigation_seed: None,
    })
}

fn execute_teleport_phase_boundary<F>(
    waypoint: &PathingWaypointPlan,
    plan: &AutoPathingExecutionPlan,
    capture_size: Size,
    live_executor: &mut F,
) -> Result<PathingPhaseBoundaryReport>
where
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
{
    let force_teleport = match waypoint.action_plan.as_ref() {
        Some(PathingActionPlan::ForceTeleport(force_teleport)) => force_teleport.force_teleport,
        _ => false,
    };
    execute_teleport_phase_boundary_for_target(
        waypoint.route_point,
        force_teleport,
        &plan.execution_plan.map_name,
        capture_size,
        live_executor,
    )
}

fn execute_teleport_phase_boundary_for_target<F>(
    route_point: PathingPoint,
    force_teleport: bool,
    map_name: &str,
    capture_size: Size,
    live_executor: &mut F,
) -> Result<PathingPhaseBoundaryReport>
where
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
{
    let common_job_plan =
        plan_teleport_phase_common_job(route_point, force_teleport, map_name, capture_size)?;
    let common_job_task_key = Some(common_job_plan.task_key().to_string());

    let live_execution = match live_executor(&common_job_plan) {
        Ok(live_execution) => live_execution,
        Err(error) => {
            return Ok(PathingPhaseBoundaryReport {
                phase: PathingWaypointPhase::HandleTeleport,
                status: PathingBoundaryStatus::Unsupported,
                reason: format!(
                    "teleport waypoint to ({:.3}, {:.3}) reached the Teleport common-job live boundary, but live execution is still unavailable: {error}",
                    route_point.x, route_point.y
                ),
                common_job_task_key,
                common_job_plan: Some(common_job_plan),
                common_job_live_execution: None,
                navigation_seed: None,
            });
        }
    };

    match live_execution {
        Some(CommonJobLiveExecutionReport::Teleport(teleport_report)) => {
            let navigation_seed = teleport_navigation_seed_report(&teleport_report);
            let (status, reason) = match navigation_seed.as_ref() {
                Some(seed) => (
                    PathingBoundaryStatus::Executed,
                    format!(
                        "teleport waypoint to ({:.3}, {:.3}) executed through the Teleport common-job live boundary and produced a previous-position seed ({:.3}, {:.3}) for AutoPathing track conversion",
                        route_point.x,
                        route_point.y,
                        seed.previous_position.x,
                        seed.previous_position.y
                    ),
                ),
                None => (
                    PathingBoundaryStatus::Unsupported,
                    format!(
                        "teleport waypoint to ({:.3}, {:.3}) reached the Teleport common-job live boundary, but the report did not contain a navigation previous-position seed",
                        route_point.x, route_point.y
                    ),
                ),
            };

            Ok(PathingPhaseBoundaryReport {
                phase: PathingWaypointPhase::HandleTeleport,
                status,
                reason,
                common_job_task_key,
                common_job_plan: Some(common_job_plan),
                common_job_live_execution: Some(CommonJobLiveExecutionReport::Teleport(
                    teleport_report,
                )),
                navigation_seed,
            })
        }
        Some(other_report) => Ok(PathingPhaseBoundaryReport {
            phase: PathingWaypointPhase::HandleTeleport,
            status: PathingBoundaryStatus::Invalid,
            reason: format!(
                "teleport waypoint expected a Teleport live report but received {}",
                other_report.task_name()
            ),
            common_job_task_key,
            common_job_plan: Some(common_job_plan),
            common_job_live_execution: Some(other_report),
            navigation_seed: None,
        }),
        None => {
            let reason = if force_teleport {
                format!(
                    "force_tp teleport intent to ({:.3}, {:.3}) is planned through the Teleport common-job contract with force_teleport=true, but the live executor returned no report; AutoPathing native TpTask dispatch remains pending",
                    route_point.x, route_point.y
                )
            } else {
                format!(
                    "teleport waypoint to ({:.3}, {:.3}) is planned through the Teleport common-job contract, but the live executor returned no report; AutoPathing native TpTask dispatch remains pending",
                    route_point.x, route_point.y
                )
            };
            Ok(PathingPhaseBoundaryReport {
                phase: PathingWaypointPhase::HandleTeleport,
                status: PathingBoundaryStatus::Unsupported,
                reason,
                common_job_task_key,
                common_job_plan: Some(common_job_plan),
                common_job_live_execution: None,
                navigation_seed: None,
            })
        }
    }
}

fn teleport_navigation_seed_report(
    teleport_report: &TeleportExecutionReport,
) -> Option<PathingNavigationSeedReport> {
    let seed = teleport_report
        .state
        .navigation_previous_position_seed
        .as_ref()?;
    Some(PathingNavigationSeedReport {
        source_task_key: teleport_report.task_key.clone(),
        previous_position: PathingPoint {
            x: seed.x,
            y: seed.y,
        },
        map_name: seed.map_name.clone(),
        coordinate_space: PathingCoordinateSpace::RouteJson,
        requires_track_conversion: true,
    })
}

fn plan_teleport_phase_common_job(
    route_point: PathingPoint,
    force_teleport: bool,
    map_name: &str,
    capture_size: Size,
) -> Result<CommonJobExecutionPlan> {
    let config = json!({
        "x": route_point.x,
        "y": route_point.y,
        "mapName": map_name,
        "force": force_teleport,
        "captureSize": capture_size,
    });
    plan_common_job(TELEPORT_TASK_KEY, Some(&config))?.ok_or_else(|| TaskError::InvalidTaskConfig {
        key: TELEPORT_TASK_KEY.to_string(),
        message: "Teleport common-job task is not registered".to_string(),
    })
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
        Some(PathingActionPlan::UseGadget(use_gadget)) => {
            execute_use_gadget_pathing_action(use_gadget)
        }
        Some(PathingActionPlan::ForceTeleport(_)) => Ok(PathingActionBoundaryReport {
            action_code: "force_tp".to_string(),
            status: PathingBoundaryStatus::Unsupported,
            message:
                "force_tp is handled by the HandleTeleport phase; native TpTask dispatch remains pending"
                    .to_string(),
            common_job_task_key: None,
            common_job_plan: None,
            common_job_live_execution: None,
        }),
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

fn execute_use_gadget_pathing_action(
    use_gadget: &PathingUseGadgetActionPlan,
) -> Result<PathingActionBoundaryReport> {
    if !use_gadget.executor_ready {
        return Ok(PathingActionBoundaryReport {
            action_code: use_gadget.action_code.clone(),
            status: PathingBoundaryStatus::Unsupported,
            message: if let Some(parse_error) = use_gadget.max_wait_parse_error.as_ref() {
                format!(
                    "pathing use_gadget default branch needs cooldown OCR before native execution; {parse_error}"
                )
            } else {
                "pathing use_gadget default branch needs cooldown OCR before native execution"
                    .to_string()
            },
            common_job_task_key: None,
            common_job_plan: None,
            common_job_live_execution: None,
        });
    }

    Ok(PathingActionBoundaryReport {
        action_code: use_gadget.action_code.clone(),
        status: PathingBoundaryStatus::Reported,
        message: format!(
            "pathing use_gadget not_wait action is modeled as {} {:?} press(es), handler delay {}ms, and PathExecutor after-action delay {}ms; sequence-safe desktop input dispatch remains pending",
            use_gadget.quick_use_gadget_press_count,
            use_gadget.genshin_action,
            use_gadget.handler_delay_ms,
            use_gadget.path_executor_after_action_delay_ms
        ),
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
