use std::path::{Component, Path, PathBuf};

use bgi_core::{
    read_pathing_task, PathingMovementContractPlan, PathingMovementDependency, PathingSummary,
};
use serde::Serialize;

use crate::auto_pathing::AutoPathingExecutionPlan;
use crate::{task_asset_root, Result, TaskError};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CommonJobPathingPreflightReport {
    pub pathing_json: String,
    pub resolved_path: String,
    pub summary: PathingSummary,
    pub map_name: String,
    pub map_match_method: Option<String>,
    pub has_positions: bool,
    pub segment_count: usize,
    pub waypoint_count: usize,
    pub movement_executor_ready: bool,
    pub native_pathing_completed: bool,
    pub pending_dependencies: Vec<PathingMovementDependency>,
    pub movement_contract: PathingMovementContractPlan,
    pub notes: String,
}

pub fn preflight_common_job_pathing_rule(
    pathing_json: &str,
) -> Result<CommonJobPathingPreflightReport> {
    let execution_plan = plan_common_job_pathing_action_boundary(pathing_json)?;
    let movement_contract = execution_plan.execution_plan.movement_contract.clone();

    Ok(CommonJobPathingPreflightReport {
        pathing_json: pathing_json.to_string(),
        resolved_path: task_asset_root()
            .join(&execution_plan.normalized_path)
            .to_string_lossy()
            .to_string(),
        summary: execution_plan.summary,
        map_name: execution_plan.execution_plan.map_name,
        map_match_method: execution_plan.execution_plan.map_match_method,
        has_positions: execution_plan.execution_plan.has_positions,
        segment_count: execution_plan.execution_plan.segment_count,
        waypoint_count: execution_plan.execution_plan.waypoint_count,
        movement_executor_ready: movement_contract.movement_executor_ready,
        native_pathing_completed: movement_contract.native_pathing_completed,
        pending_dependencies: movement_contract.pending_dependencies.clone(),
        movement_contract,
        notes: "Common-job pathing asset is readable and converted into the shared PathExecutor movement contract; desktop movement execution remains native-pending until a movement runtime consumes the contract.".to_string(),
    })
}

pub fn plan_common_job_pathing_action_boundary(
    pathing_json: &str,
) -> Result<AutoPathingExecutionPlan> {
    validate_common_job_pathing_json(pathing_json)?;
    let root = task_asset_root();
    let path = root.join(pathing_json);
    let task = read_pathing_task(&path).map_err(|error| {
        TaskError::PathingPlan(format!(
            "failed to read common-job pathing asset {pathing_json}: {error}"
        ))
    })?;
    let execution_plan = task.execution_plan_with_legacy_track_converter();

    Ok(AutoPathingExecutionPlan {
        source: "CommonJobPathingAsset",
        route: pathing_json.to_string(),
        normalized_path: PathBuf::from(pathing_json),
        summary: task.summary(),
        execution_plan,
        dispatched: false,
        completed: false,
        notes: "Common-job PathExecutor JSON is parsed into the shared AutoPathing action boundary plan with legacy TrackMap coordinate conversion; native desktop movement remains pending until a movement runtime completes every phase.".to_string(),
    })
}

fn validate_common_job_pathing_json(pathing_json: &str) -> Result<()> {
    let path = Path::new(pathing_json);
    if pathing_json.trim().is_empty()
        || path.components().any(|component| {
            matches!(
                component,
                Component::Prefix(_) | Component::RootDir | Component::ParentDir
            )
        })
    {
        return Err(TaskError::PathingPlan(format!(
            "common-job pathing asset must be a relative path inside task assets: {pathing_json}"
        )));
    }
    Ok(())
}
