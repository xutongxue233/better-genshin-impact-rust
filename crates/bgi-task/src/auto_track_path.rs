use bgi_core::TpConfig;
use bgi_vision::{Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::{Result, TaskError};

pub const AUTO_TRACK_PATH_TASK_KEY: &str = "AutoTrackPath";
pub const AUTO_TRACK_PATH_DISPLAY_NAME: &str = "自动路线";
pub const AUTO_TRACK_PATH_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_TRACK_PATH_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_TRACK_PATH_DEFAULT_PATH_FILE: &str = "log/way/way2.json";
pub const AUTO_TRACK_PATH_CHAR_MOVING_UNIT: i32 = 500;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoTrackPathExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub path_file: String,
    pub config_rule: AutoTrackPathConfigRule,
    pub path_summary: AutoTrackPathSummary,
    pub waypoints: Vec<AutoTrackPathWaypointPlan>,
    pub startup_rule: AutoTrackPathStartupRule,
    pub teleport_rule: AutoTrackPathTeleportRule,
    pub angle_calibration_rule: AutoTrackPathAngleCalibrationRule,
    pub tracking_rule: AutoTrackPathTrackingRule,
    pub status_refresh_rule: AutoTrackPathStatusRefreshRule,
    pub jump_rule: AutoTrackPathJumpRule,
    pub steps: Vec<AutoTrackPathStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTrackPathExecutionConfig {
    pub capture_size: Size,
    pub path_file: String,
    pub tp_config: TpConfig,
}

impl Default for AutoTrackPathExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_TRACK_PATH_DEFAULT_CAPTURE_WIDTH,
                AUTO_TRACK_PATH_DEFAULT_CAPTURE_HEIGHT,
            ),
            path_file: AUTO_TRACK_PATH_DEFAULT_PATH_FILE.to_string(),
            tp_config: TpConfig::default(),
        }
    }
}

impl AutoTrackPathExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };

        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }
        if let Some(path_file) = string_member(value, ["pathFile", "PathFile", "path_file"]) {
            config.path_file = path_file;
        }
        let tp_config_value = value
            .get("tpConfig")
            .or_else(|| value.get("TpConfig"))
            .or_else(|| value.get("tp_config"))
            .unwrap_or(value);
        config.tp_config = serde_json::from_value(tp_config_value.clone()).unwrap_or_default();
        config
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LegacyGiPath {
    #[serde(alias = "wayPointList")]
    pub way_point_list: Vec<LegacyGiPathPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LegacyGiPathPoint {
    #[serde(alias = "pt")]
    pub pt: LegacyPoint2f,
    #[serde(alias = "matchPt")]
    pub match_pt: LegacyPoint2f,
    #[serde(alias = "index")]
    pub index: i32,
    #[serde(default, alias = "type")]
    pub r#type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LegacyPoint2f {
    #[serde(alias = "x")]
    pub x: f64,
    #[serde(alias = "y")]
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoTrackPathConfigRule {
    pub map_zoom_enabled: bool,
    pub map_zoom_out_distance: u64,
    pub map_zoom_in_distance: u64,
    pub step_interval_milliseconds: u64,
    pub max_zoom_level: f64,
    pub min_zoom_level: f64,
    pub tolerance: f64,
    pub max_iterations: u64,
    pub max_mouse_move: u64,
    pub map_scale_factor: f64,
    pub hp_restore_duration: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoTrackPathSummary {
    pub waypoint_count: usize,
    pub first_point: Option<AutoTrackPathPointPlan>,
    pub last_point: Option<AutoTrackPathPointPlan>,
    pub key_point_indices: Vec<i32>,
    pub key_point_types: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct AutoTrackPathPointPlan {
    pub index: i32,
    pub pt_x: f64,
    pub pt_y: f64,
    pub match_x: f64,
    pub match_y: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoTrackPathStartupRule {
    pub uses_task_semaphore_non_blocking: bool,
    pub activates_game_window: bool,
    pub logs_start: bool,
    pub treats_normal_end_as_manual_interrupt: bool,
    pub clears_draw_content_on_finish: bool,
    pub releases_task_semaphore_on_finish: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoTrackPathTeleportRule {
    pub teleports_to_first_waypoint: bool,
    pub post_teleport_sleep_ms: u64,
    pub wait_minimap_retry_attempts: u64,
    pub wait_minimap_retry_interval_ms: u64,
    pub post_minimap_detected_sleep_ms: u64,
    pub paimon_menu_locator_asset: String,
    pub mini_map_crop_from_paimon: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoTrackPathAngleCalibrationRule {
    pub char_moving_unit: i32,
    pub mouse_move_x: i32,
    pub after_mouse_move_sleep_ms: u64,
    pub move_forward_hold_ms: u64,
    pub after_forward_sleep_ms: u64,
    pub fails_when_angle_offset_zero: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoTrackPathTrackingRule {
    pub nearest_lookahead_points: usize,
    pub stop_distance: f64,
    pub key_point_types: Vec<String>,
    pub rotation_unit: i32,
    pub rotation_formula: String,
    pub after_mouse_move_sleep_ms: u64,
    pub after_angle_recheck_sleep_ms: u64,
    pub move_forward_after_rotation: bool,
    pub post_forward_sleep_ms: u64,
    pub release_forward_when_reaching_point: bool,
    pub cancels_track_when_last_point_reached: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoTrackPathStatusRefreshRule {
    pub captures_motion_status: bool,
    pub interval_ms: u64,
    pub main_ui_required_by_minimap_detection: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoTrackPathJumpRule {
    pub only_jumps_when_motion_normal: bool,
    pub first_jump_interval_ms: u64,
    pub second_jump_followup_sleep_ms: u64,
    pub interrupted_motion_sleep_ms: u64,
    pub non_normal_sleep_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoTrackPathStep {
    pub phase: AutoTrackPathPhase,
    pub action: AutoTrackPathAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrackPathPhase {
    Startup,
    Teleport,
    Calibration,
    Tracking,
    StatusRefresh,
    Jump,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrackPathAction {
    AcquireTaskSemaphore,
    ActivateWindow,
    ReadWay2Json,
    TeleportToFirstPoint,
    WaitForMiniMap,
    CalibrateMouseAngleOffset,
    SelectNearestNextPoint,
    RotateTowardTarget,
    HoldForward,
    RefreshMotionStatus,
    JumpWhileNormal,
    CancelTracking,
    ClearOverlayAndReleaseSemaphore,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTrackPathMapPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTrackPathWaypointPlan {
    pub index: i32,
    pub point_type: String,
    pub pt: AutoTrackPathMapPosition,
    pub match_pt: AutoTrackPathMapPosition,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrackPathMotionStatus {
    #[default]
    Normal,
    Fly,
    Climb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrackPathObservationPhase {
    WaitForMiniMap,
    CalibrationBeforeMove,
    CalibrationAfterMove,
    Tracking,
    AngleRecheck,
    StatusRefresh,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTrackPathTrackingObservation {
    pub mini_map_visible: bool,
    pub avatar_map_position: Option<AutoTrackPathMapPosition>,
    pub character_orientation_degrees: Option<f64>,
    #[serde(default)]
    pub motion_status: AutoTrackPathMotionStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTrackPathTargetSelection {
    pub waypoint_position: usize,
    pub waypoint_index: i32,
    pub waypoint_type: String,
    pub target_match_position: AutoTrackPathMapPosition,
    pub distance: f64,
    pub target_angle_degrees: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrackPathActionStatus {
    Succeeded,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrackPathExecutionStatus {
    Completed,
    EmptyPath,
    SemaphoreUnavailable,
    Cancelled,
    IterationLimitReached,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTrackPathActionReport {
    pub phase: AutoTrackPathPhase,
    pub action: AutoTrackPathAction,
    pub status: AutoTrackPathActionStatus,
    pub iteration: u64,
    pub waypoint_index: Option<i32>,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTrackPathExecutorState {
    pub semaphore_acquired: bool,
    pub window_activated: bool,
    pub teleported: bool,
    pub mini_map_ready: bool,
    pub angle_offset_unit: Option<i32>,
    pub current_waypoint_position: usize,
    pub target_waypoint_index: Option<i32>,
    pub target_angle_degrees: Option<f64>,
    pub motion_status: AutoTrackPathMotionStatus,
    pub forward_held: bool,
    pub tracking_cancelled: bool,
    pub cleanup_completed: bool,
    pub tracking_iterations: u64,
    pub status_refresh_count: u64,
    pub jump_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTrackPathExecutionReport {
    pub task_key: String,
    pub status: AutoTrackPathExecutionStatus,
    pub state: AutoTrackPathExecutorState,
    pub actions: Vec<AutoTrackPathActionReport>,
}

pub trait AutoTrackPathRuntime {
    fn acquire_auto_track_path_semaphore(&mut self) -> Result<bool>;

    fn activate_auto_track_path_window(&mut self) -> Result<()>;

    fn teleport_auto_track_path_to(&mut self, waypoint: &AutoTrackPathWaypointPlan) -> Result<()>;

    fn observe_auto_track_path(
        &mut self,
        phase: AutoTrackPathObservationPhase,
    ) -> Result<AutoTrackPathTrackingObservation>;

    fn move_auto_track_path_mouse(&mut self, delta_x: i32, delta_y: i32) -> Result<()>;

    fn set_auto_track_path_forward(&mut self, pressed: bool) -> Result<()>;

    fn tap_auto_track_path_jump(&mut self) -> Result<()>;

    fn cancel_auto_track_path_tracking(&mut self) -> Result<()>;

    fn delay_auto_track_path(&mut self, duration_ms: u64) -> Result<()>;

    fn clear_auto_track_path_overlay(&mut self) -> Result<()>;

    fn release_auto_track_path_semaphore(&mut self) -> Result<()>;

    fn draw_auto_track_path_tracking_overlay(
        &mut self,
        _selection: &AutoTrackPathTargetSelection,
        _observation: &AutoTrackPathTrackingObservation,
    ) -> Result<()> {
        Ok(())
    }

    fn is_auto_track_path_cancelled(&mut self) -> bool {
        false
    }
}

pub fn plan_auto_track_path(
    working_directory: impl AsRef<Path>,
    config: AutoTrackPathExecutionConfig,
) -> Result<AutoTrackPathExecutionPlan> {
    let normalized_path = normalize_auto_track_path_file(&config.path_file)?;
    let path = working_directory.as_ref().join(&normalized_path);
    let text = fs::read_to_string(&path).map_err(|error| TaskError::InvalidTaskConfig {
        key: AUTO_TRACK_PATH_TASK_KEY.to_string(),
        message: format!(
            "failed to read AutoTrackPath file {}: {error}",
            path.display()
        ),
    })?;
    let legacy_path: LegacyGiPath =
        serde_json::from_str(&text).map_err(|error| TaskError::InvalidTaskConfig {
            key: AUTO_TRACK_PATH_TASK_KEY.to_string(),
            message: format!(
                "failed to parse AutoTrackPath file {}: {error}",
                path.display()
            ),
        })?;
    let path_summary = summarize_legacy_path(&legacy_path);
    let waypoints = legacy_path
        .way_point_list
        .iter()
        .map(waypoint_plan)
        .collect();

    Ok(AutoTrackPathExecutionPlan {
        task_key: AUTO_TRACK_PATH_TASK_KEY.to_string(),
        display_name: AUTO_TRACK_PATH_DISPLAY_NAME.to_string(),
        capture_size: config.capture_size,
        path_file: normalized_path.to_string_lossy().replace('\\', "/"),
        config_rule: AutoTrackPathConfigRule {
            map_zoom_enabled: config.tp_config.map_zoom_enabled,
            map_zoom_out_distance: config.tp_config.map_zoom_out_distance,
            map_zoom_in_distance: config.tp_config.map_zoom_in_distance,
            step_interval_milliseconds: config.tp_config.step_interval_milliseconds,
            max_zoom_level: config.tp_config.max_zoom_level,
            min_zoom_level: config.tp_config.min_zoom_level,
            tolerance: config.tp_config.tolerance,
            max_iterations: config.tp_config.max_iterations,
            max_mouse_move: config.tp_config.max_mouse_move,
            map_scale_factor: config.tp_config.map_scale_factor,
            hp_restore_duration: config.tp_config.hp_restore_duration,
        },
        path_summary,
        waypoints,
        startup_rule: AutoTrackPathStartupRule {
            uses_task_semaphore_non_blocking: true,
            activates_game_window: true,
            logs_start: true,
            treats_normal_end_as_manual_interrupt: true,
            clears_draw_content_on_finish: true,
            releases_task_semaphore_on_finish: true,
        },
        teleport_rule: AutoTrackPathTeleportRule {
            teleports_to_first_waypoint: true,
            post_teleport_sleep_ms: 1_000,
            wait_minimap_retry_attempts: 100,
            wait_minimap_retry_interval_ms: 1_000,
            post_minimap_detected_sleep_ms: 1_000,
            paimon_menu_locator_asset: "Common/Element:paimon_menu.png".to_string(),
            mini_map_crop_from_paimon: Rect {
                x: 24,
                y: -15,
                width: 210,
                height: 210,
            },
        },
        angle_calibration_rule: AutoTrackPathAngleCalibrationRule {
            char_moving_unit: AUTO_TRACK_PATH_CHAR_MOVING_UNIT,
            mouse_move_x: AUTO_TRACK_PATH_CHAR_MOVING_UNIT,
            after_mouse_move_sleep_ms: 500,
            move_forward_hold_ms: 100,
            after_forward_sleep_ms: 1_000,
            fails_when_angle_offset_zero: true,
        },
        tracking_rule: AutoTrackPathTrackingRule {
            nearest_lookahead_points: 20,
            stop_distance: 10.0,
            key_point_types: vec![
                "KeyPoint".to_string(),
                "Fighting".to_string(),
                "Collection".to_string(),
            ],
            rotation_unit: AUTO_TRACK_PATH_CHAR_MOVING_UNIT,
            rotation_formula:
                "(target_angle - current_angle) / angle_offset_unit * CharMovingUnit".to_string(),
            after_mouse_move_sleep_ms: 100,
            after_angle_recheck_sleep_ms: 100,
            move_forward_after_rotation: true,
            post_forward_sleep_ms: 50,
            release_forward_when_reaching_point: true,
            cancels_track_when_last_point_reached: true,
        },
        status_refresh_rule: AutoTrackPathStatusRefreshRule {
            captures_motion_status: true,
            interval_ms: 60,
            main_ui_required_by_minimap_detection: true,
        },
        jump_rule: AutoTrackPathJumpRule {
            only_jumps_when_motion_normal: true,
            first_jump_interval_ms: 300,
            second_jump_followup_sleep_ms: 3_500,
            interrupted_motion_sleep_ms: 1_600,
            non_normal_sleep_ms: 1_600,
        },
        steps: auto_track_path_steps(),
        executor_ready: true,
        pending_native: vec![
            "AutoTrackPath now has a Rust injectable execution boundary for semaphore/window startup, teleport, mini-map observations, map matching, orientation, input, status refresh, jump, cancellation, overlay cleanup, and semaphore release".to_string(),
            "desktop live adapters still need to wire TpTask, capture/minimap extraction, map matching, CharacterOrientation/CameraOrientation, SendInput W/Space/mouse dispatch, Bv motion status, overlay, and cancellation into this runtime trait".to_string(),
        ],
    })
}

pub fn execute_auto_track_path_plan<R>(
    plan: &AutoTrackPathExecutionPlan,
    runtime: &mut R,
    max_tracking_iterations: u64,
) -> Result<AutoTrackPathExecutionReport>
where
    R: AutoTrackPathRuntime,
{
    let mut state = AutoTrackPathExecutorState::default();
    let mut actions = Vec::new();

    let result = execute_auto_track_path_plan_inner(
        plan,
        runtime,
        max_tracking_iterations,
        &mut state,
        &mut actions,
    );

    match result {
        Ok(status) => {
            cleanup_auto_track_path(plan, runtime, &mut state, &mut actions, None)?;
            Ok(AutoTrackPathExecutionReport {
                task_key: plan.task_key.clone(),
                status,
                state,
                actions,
            })
        }
        Err(error) => {
            let cleanup_error = cleanup_auto_track_path(
                plan,
                runtime,
                &mut state,
                &mut actions,
                Some(error.to_string()),
            )
            .err();
            if let Some(cleanup_error) = cleanup_error {
                actions.push(AutoTrackPathActionReport {
                    phase: AutoTrackPathPhase::Cleanup,
                    action: AutoTrackPathAction::ClearOverlayAndReleaseSemaphore,
                    status: AutoTrackPathActionStatus::Failed,
                    iteration: state.tracking_iterations,
                    waypoint_index: state.target_waypoint_index,
                    detail: format!("cleanup failed after execution error: {cleanup_error}"),
                });
            }
            Err(error)
        }
    }
}

fn execute_auto_track_path_plan_inner<R>(
    plan: &AutoTrackPathExecutionPlan,
    runtime: &mut R,
    max_tracking_iterations: u64,
    state: &mut AutoTrackPathExecutorState,
    actions: &mut Vec<AutoTrackPathActionReport>,
) -> Result<AutoTrackPathExecutionStatus>
where
    R: AutoTrackPathRuntime,
{
    let acquired = runtime.acquire_auto_track_path_semaphore()?;
    state.semaphore_acquired = acquired;
    push_auto_track_path_action(
        actions,
        AutoTrackPathPhase::Startup,
        AutoTrackPathAction::AcquireTaskSemaphore,
        if acquired {
            AutoTrackPathActionStatus::Succeeded
        } else {
            AutoTrackPathActionStatus::Skipped
        },
        state,
        if acquired {
            "task semaphore acquired"
        } else {
            "task semaphore was already held"
        },
    );
    if !acquired {
        return Ok(AutoTrackPathExecutionStatus::SemaphoreUnavailable);
    }

    runtime.activate_auto_track_path_window()?;
    state.window_activated = true;
    push_auto_track_path_action(
        actions,
        AutoTrackPathPhase::Startup,
        AutoTrackPathAction::ActivateWindow,
        AutoTrackPathActionStatus::Succeeded,
        state,
        "game window activated",
    );

    if plan.waypoints.is_empty() {
        push_auto_track_path_action(
            actions,
            AutoTrackPathPhase::Startup,
            AutoTrackPathAction::ReadWay2Json,
            AutoTrackPathActionStatus::Skipped,
            state,
            "path contains no waypoints",
        );
        return Ok(AutoTrackPathExecutionStatus::EmptyPath);
    }

    push_auto_track_path_action(
        actions,
        AutoTrackPathPhase::Startup,
        AutoTrackPathAction::ReadWay2Json,
        AutoTrackPathActionStatus::Succeeded,
        state,
        format!("loaded {} waypoints", plan.waypoints.len()),
    );

    let first = &plan.waypoints[0];
    runtime.teleport_auto_track_path_to(first)?;
    state.teleported = true;
    state.target_waypoint_index = Some(first.index);
    push_auto_track_path_action(
        actions,
        AutoTrackPathPhase::Teleport,
        AutoTrackPathAction::TeleportToFirstPoint,
        AutoTrackPathActionStatus::Succeeded,
        state,
        format!(
            "teleported to first waypoint {} at ({:.3}, {:.3})",
            first.index, first.pt.x, first.pt.y
        ),
    );
    runtime.delay_auto_track_path(plan.teleport_rule.post_teleport_sleep_ms)?;

    wait_for_auto_track_path_minimap(plan, runtime, state, actions)?;
    runtime.delay_auto_track_path(plan.teleport_rule.post_minimap_detected_sleep_ms)?;

    let angle_offset_unit = calibrate_auto_track_path_angle(plan, runtime, state, actions)?;
    state.angle_offset_unit = Some(angle_offset_unit);

    loop {
        if runtime.is_auto_track_path_cancelled() {
            return Ok(AutoTrackPathExecutionStatus::Cancelled);
        }
        if max_tracking_iterations > 0 && state.tracking_iterations >= max_tracking_iterations {
            return Ok(AutoTrackPathExecutionStatus::IterationLimitReached);
        }

        let iteration = state.tracking_iterations;
        state.tracking_iterations += 1;

        let observation =
            runtime.observe_auto_track_path(AutoTrackPathObservationPhase::Tracking)?;
        ensure_mini_map_visible(&observation, "tracking")?;
        let current_position = observation.avatar_map_position.ok_or_else(|| {
            auto_track_path_runtime_error("tracking observation is missing map position")
        })?;
        let current_angle = observation.character_orientation_degrees.ok_or_else(|| {
            auto_track_path_runtime_error("tracking observation is missing character orientation")
        })?;

        let selection = select_auto_track_path_next_point(
            &plan.waypoints,
            current_position,
            state.current_waypoint_position,
            &plan.tracking_rule,
        )?;
        state.target_waypoint_index = Some(selection.waypoint_index);
        state.target_angle_degrees = Some(selection.target_angle_degrees);
        runtime.draw_auto_track_path_tracking_overlay(&selection, &observation)?;
        actions.push(AutoTrackPathActionReport {
            phase: AutoTrackPathPhase::Tracking,
            action: AutoTrackPathAction::SelectNearestNextPoint,
            status: AutoTrackPathActionStatus::Succeeded,
            iteration,
            waypoint_index: Some(selection.waypoint_index),
            detail: format!(
                "selected waypoint {} distance {:.3} angle {:.3}",
                selection.waypoint_index, selection.distance, selection.target_angle_degrees
            ),
        });

        if selection.distance < plan.tracking_rule.stop_distance {
            state.current_waypoint_position = selection.waypoint_position;
            if state.forward_held && plan.tracking_rule.release_forward_when_reaching_point {
                runtime.set_auto_track_path_forward(false)?;
                state.forward_held = false;
                actions.push(AutoTrackPathActionReport {
                    phase: AutoTrackPathPhase::Tracking,
                    action: AutoTrackPathAction::HoldForward,
                    status: AutoTrackPathActionStatus::Succeeded,
                    iteration,
                    waypoint_index: Some(selection.waypoint_index),
                    detail: "released forward after reaching waypoint".to_string(),
                });
            }
            if state.current_waypoint_position + 1 == plan.waypoints.len() {
                if plan.tracking_rule.cancels_track_when_last_point_reached {
                    runtime.cancel_auto_track_path_tracking()?;
                    state.tracking_cancelled = true;
                    actions.push(AutoTrackPathActionReport {
                        phase: AutoTrackPathPhase::Tracking,
                        action: AutoTrackPathAction::CancelTracking,
                        status: AutoTrackPathActionStatus::Succeeded,
                        iteration,
                        waypoint_index: Some(selection.waypoint_index),
                        detail: "last waypoint reached; tracking cancelled".to_string(),
                    });
                }
                return Ok(AutoTrackPathExecutionStatus::Completed);
            }
        }

        let rotation = compute_auto_track_path_rotation(
            selection.target_angle_degrees,
            current_angle,
            angle_offset_unit,
            plan.tracking_rule.rotation_unit,
        );
        runtime.move_auto_track_path_mouse(rotation, 0)?;
        actions.push(AutoTrackPathActionReport {
            phase: AutoTrackPathPhase::Tracking,
            action: AutoTrackPathAction::RotateTowardTarget,
            status: AutoTrackPathActionStatus::Succeeded,
            iteration,
            waypoint_index: Some(selection.waypoint_index),
            detail: format!(
                "rotated mouse by {rotation}; current angle {:.3}, target {:.3}",
                current_angle, selection.target_angle_degrees
            ),
        });
        runtime.delay_auto_track_path(plan.tracking_rule.after_mouse_move_sleep_ms)?;

        let recheck =
            runtime.observe_auto_track_path(AutoTrackPathObservationPhase::AngleRecheck)?;
        ensure_mini_map_visible(&recheck, "angle recheck")?;
        runtime.delay_auto_track_path(plan.tracking_rule.after_angle_recheck_sleep_ms)?;

        if plan.tracking_rule.move_forward_after_rotation {
            runtime.set_auto_track_path_forward(true)?;
            state.forward_held = true;
            actions.push(AutoTrackPathActionReport {
                phase: AutoTrackPathPhase::Tracking,
                action: AutoTrackPathAction::HoldForward,
                status: AutoTrackPathActionStatus::Succeeded,
                iteration,
                waypoint_index: Some(selection.waypoint_index),
                detail: "forward pressed after rotation".to_string(),
            });
            runtime.delay_auto_track_path(plan.tracking_rule.post_forward_sleep_ms)?;
        }

        refresh_auto_track_path_status(plan, runtime, state, actions, iteration)?;
        run_auto_track_path_jump_cycle(plan, runtime, state, actions, iteration)?;
    }
}

pub fn normalize_auto_track_path_file(path: &str) -> Result<PathBuf> {
    let path = path.trim().replace('\\', "/");
    if path.is_empty() {
        return Err(TaskError::InvalidTaskConfig {
            key: AUTO_TRACK_PATH_TASK_KEY.to_string(),
            message: "AutoTrackPath pathFile is empty".to_string(),
        });
    }
    let path = PathBuf::from(path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(TaskError::InvalidTaskConfig {
            key: AUTO_TRACK_PATH_TASK_KEY.to_string(),
            message: format!("invalid AutoTrackPath pathFile: {}", path.display()),
        });
    }
    Ok(path)
}

fn push_auto_track_path_action(
    actions: &mut Vec<AutoTrackPathActionReport>,
    phase: AutoTrackPathPhase,
    action: AutoTrackPathAction,
    status: AutoTrackPathActionStatus,
    state: &AutoTrackPathExecutorState,
    detail: impl Into<String>,
) {
    actions.push(AutoTrackPathActionReport {
        phase,
        action,
        status,
        iteration: state.tracking_iterations,
        waypoint_index: state.target_waypoint_index,
        detail: detail.into(),
    });
}

fn wait_for_auto_track_path_minimap<R>(
    plan: &AutoTrackPathExecutionPlan,
    runtime: &mut R,
    state: &mut AutoTrackPathExecutorState,
    actions: &mut Vec<AutoTrackPathActionReport>,
) -> Result<()>
where
    R: AutoTrackPathRuntime,
{
    for attempt in 0..plan.teleport_rule.wait_minimap_retry_attempts {
        let observation =
            runtime.observe_auto_track_path(AutoTrackPathObservationPhase::WaitForMiniMap)?;
        if observation.mini_map_visible {
            state.mini_map_ready = true;
            actions.push(AutoTrackPathActionReport {
                phase: AutoTrackPathPhase::Teleport,
                action: AutoTrackPathAction::WaitForMiniMap,
                status: AutoTrackPathActionStatus::Succeeded,
                iteration: state.tracking_iterations,
                waypoint_index: state.target_waypoint_index,
                detail: format!("mini-map detected after {} attempt(s)", attempt + 1),
            });
            return Ok(());
        }
        runtime.delay_auto_track_path(plan.teleport_rule.wait_minimap_retry_interval_ms)?;
    }

    actions.push(AutoTrackPathActionReport {
        phase: AutoTrackPathPhase::Teleport,
        action: AutoTrackPathAction::WaitForMiniMap,
        status: AutoTrackPathActionStatus::Failed,
        iteration: state.tracking_iterations,
        waypoint_index: state.target_waypoint_index,
        detail: "mini-map was not detected before retry limit".to_string(),
    });
    Err(auto_track_path_runtime_error("mini-map wait timed out"))
}

fn calibrate_auto_track_path_angle<R>(
    plan: &AutoTrackPathExecutionPlan,
    runtime: &mut R,
    state: &mut AutoTrackPathExecutorState,
    actions: &mut Vec<AutoTrackPathActionReport>,
) -> Result<i32>
where
    R: AutoTrackPathRuntime,
{
    let before =
        runtime.observe_auto_track_path(AutoTrackPathObservationPhase::CalibrationBeforeMove)?;
    ensure_mini_map_visible(&before, "angle calibration before move")?;
    let angle_before = before.character_orientation_degrees.ok_or_else(|| {
        auto_track_path_runtime_error("angle calibration is missing initial character orientation")
    })?;

    runtime.move_auto_track_path_mouse(plan.angle_calibration_rule.mouse_move_x, 0)?;
    runtime.delay_auto_track_path(plan.angle_calibration_rule.after_mouse_move_sleep_ms)?;
    runtime.set_auto_track_path_forward(true)?;
    state.forward_held = true;
    runtime.delay_auto_track_path(plan.angle_calibration_rule.move_forward_hold_ms)?;
    runtime.set_auto_track_path_forward(false)?;
    state.forward_held = false;
    runtime.delay_auto_track_path(plan.angle_calibration_rule.after_forward_sleep_ms)?;

    let after =
        runtime.observe_auto_track_path(AutoTrackPathObservationPhase::CalibrationAfterMove)?;
    ensure_mini_map_visible(&after, "angle calibration after move")?;
    let angle_after = after.character_orientation_degrees.ok_or_else(|| {
        auto_track_path_runtime_error("angle calibration is missing final character orientation")
    })?;
    let offset = (angle_after - angle_before).round() as i32;
    if plan.angle_calibration_rule.fails_when_angle_offset_zero && offset == 0 {
        actions.push(AutoTrackPathActionReport {
            phase: AutoTrackPathPhase::Calibration,
            action: AutoTrackPathAction::CalibrateMouseAngleOffset,
            status: AutoTrackPathActionStatus::Failed,
            iteration: state.tracking_iterations,
            waypoint_index: state.target_waypoint_index,
            detail: "angle offset calibration returned zero".to_string(),
        });
        return Err(auto_track_path_runtime_error(
            "AutoTrackPath angle offset calibration failed",
        ));
    }

    actions.push(AutoTrackPathActionReport {
        phase: AutoTrackPathPhase::Calibration,
        action: AutoTrackPathAction::CalibrateMouseAngleOffset,
        status: AutoTrackPathActionStatus::Succeeded,
        iteration: state.tracking_iterations,
        waypoint_index: state.target_waypoint_index,
        detail: format!(
            "angle changed from {:.3} to {:.3}; offset unit {offset}",
            angle_before, angle_after
        ),
    });
    Ok(offset)
}

fn refresh_auto_track_path_status<R>(
    plan: &AutoTrackPathExecutionPlan,
    runtime: &mut R,
    state: &mut AutoTrackPathExecutorState,
    actions: &mut Vec<AutoTrackPathActionReport>,
    iteration: u64,
) -> Result<()>
where
    R: AutoTrackPathRuntime,
{
    if !plan.status_refresh_rule.captures_motion_status {
        actions.push(AutoTrackPathActionReport {
            phase: AutoTrackPathPhase::StatusRefresh,
            action: AutoTrackPathAction::RefreshMotionStatus,
            status: AutoTrackPathActionStatus::Skipped,
            iteration,
            waypoint_index: state.target_waypoint_index,
            detail: "motion status refresh disabled".to_string(),
        });
        return Ok(());
    }

    let observation =
        runtime.observe_auto_track_path(AutoTrackPathObservationPhase::StatusRefresh)?;
    if plan
        .status_refresh_rule
        .main_ui_required_by_minimap_detection
    {
        ensure_mini_map_visible(&observation, "status refresh")?;
    }
    state.motion_status = observation.motion_status;
    state.status_refresh_count += 1;
    actions.push(AutoTrackPathActionReport {
        phase: AutoTrackPathPhase::StatusRefresh,
        action: AutoTrackPathAction::RefreshMotionStatus,
        status: AutoTrackPathActionStatus::Succeeded,
        iteration,
        waypoint_index: state.target_waypoint_index,
        detail: format!("motion status refreshed: {:?}", state.motion_status),
    });
    runtime.delay_auto_track_path(plan.status_refresh_rule.interval_ms)?;
    Ok(())
}

fn run_auto_track_path_jump_cycle<R>(
    plan: &AutoTrackPathExecutionPlan,
    runtime: &mut R,
    state: &mut AutoTrackPathExecutorState,
    actions: &mut Vec<AutoTrackPathActionReport>,
    iteration: u64,
) -> Result<()>
where
    R: AutoTrackPathRuntime,
{
    if plan.jump_rule.only_jumps_when_motion_normal
        && state.motion_status != AutoTrackPathMotionStatus::Normal
    {
        actions.push(AutoTrackPathActionReport {
            phase: AutoTrackPathPhase::Jump,
            action: AutoTrackPathAction::JumpWhileNormal,
            status: AutoTrackPathActionStatus::Skipped,
            iteration,
            waypoint_index: state.target_waypoint_index,
            detail: format!("motion status {:?} is not normal", state.motion_status),
        });
        runtime.delay_auto_track_path(plan.jump_rule.non_normal_sleep_ms)?;
        return Ok(());
    }

    runtime.tap_auto_track_path_jump()?;
    state.jump_count += 1;
    actions.push(AutoTrackPathActionReport {
        phase: AutoTrackPathPhase::Jump,
        action: AutoTrackPathAction::JumpWhileNormal,
        status: AutoTrackPathActionStatus::Succeeded,
        iteration,
        waypoint_index: state.target_waypoint_index,
        detail: "first normal-motion jump dispatched".to_string(),
    });
    runtime.delay_auto_track_path(plan.jump_rule.first_jump_interval_ms)?;

    let observation =
        runtime.observe_auto_track_path(AutoTrackPathObservationPhase::StatusRefresh)?;
    state.motion_status = observation.motion_status;
    state.status_refresh_count += 1;
    if !plan.jump_rule.only_jumps_when_motion_normal
        || state.motion_status == AutoTrackPathMotionStatus::Normal
    {
        runtime.tap_auto_track_path_jump()?;
        state.jump_count += 1;
        actions.push(AutoTrackPathActionReport {
            phase: AutoTrackPathPhase::Jump,
            action: AutoTrackPathAction::JumpWhileNormal,
            status: AutoTrackPathActionStatus::Succeeded,
            iteration,
            waypoint_index: state.target_waypoint_index,
            detail: "second normal-motion jump dispatched".to_string(),
        });
        runtime.delay_auto_track_path(plan.jump_rule.second_jump_followup_sleep_ms)?;
    } else {
        actions.push(AutoTrackPathActionReport {
            phase: AutoTrackPathPhase::Jump,
            action: AutoTrackPathAction::JumpWhileNormal,
            status: AutoTrackPathActionStatus::Skipped,
            iteration,
            waypoint_index: state.target_waypoint_index,
            detail: format!(
                "second jump skipped because motion status became {:?}",
                state.motion_status
            ),
        });
        runtime.delay_auto_track_path(plan.jump_rule.interrupted_motion_sleep_ms)?;
    }

    Ok(())
}

fn cleanup_auto_track_path<R>(
    _plan: &AutoTrackPathExecutionPlan,
    runtime: &mut R,
    state: &mut AutoTrackPathExecutorState,
    actions: &mut Vec<AutoTrackPathActionReport>,
    prior_error: Option<String>,
) -> Result<()>
where
    R: AutoTrackPathRuntime,
{
    let mut cleanup_error = None;

    if state.forward_held {
        if let Err(error) = runtime.set_auto_track_path_forward(false) {
            cleanup_error = Some(error);
        } else {
            state.forward_held = false;
        }
    }
    if cleanup_error.is_none() {
        if let Err(error) = runtime.clear_auto_track_path_overlay() {
            cleanup_error = Some(error);
        }
    }
    if state.semaphore_acquired {
        if cleanup_error.is_none() {
            if let Err(error) = runtime.release_auto_track_path_semaphore() {
                cleanup_error = Some(error);
            } else {
                state.semaphore_acquired = false;
            }
        } else if runtime.release_auto_track_path_semaphore().is_ok() {
            state.semaphore_acquired = false;
        }
    }

    state.cleanup_completed = cleanup_error.is_none();
    actions.push(AutoTrackPathActionReport {
        phase: AutoTrackPathPhase::Cleanup,
        action: AutoTrackPathAction::ClearOverlayAndReleaseSemaphore,
        status: if cleanup_error.is_none() {
            AutoTrackPathActionStatus::Succeeded
        } else {
            AutoTrackPathActionStatus::Failed
        },
        iteration: state.tracking_iterations,
        waypoint_index: state.target_waypoint_index,
        detail: match (&prior_error, &cleanup_error) {
            (Some(error), None) => format!("cleanup completed after error: {error}"),
            (Some(error), Some(cleanup_error)) => {
                format!("cleanup failed after error {error}: {cleanup_error}")
            }
            (None, None) => "overlay cleared and semaphore released".to_string(),
            (None, Some(cleanup_error)) => format!("cleanup failed: {cleanup_error}"),
        },
    });

    if let Some(cleanup_error) = cleanup_error {
        return Err(cleanup_error);
    }
    Ok(())
}

pub fn select_auto_track_path_next_point(
    waypoints: &[AutoTrackPathWaypointPlan],
    current_position: AutoTrackPathMapPosition,
    current_waypoint_position: usize,
    tracking_rule: &AutoTrackPathTrackingRule,
) -> Result<AutoTrackPathTargetSelection> {
    if waypoints.is_empty() {
        return Err(auto_track_path_runtime_error(
            "cannot select AutoTrackPath target from an empty waypoint list",
        ));
    }

    let start = current_waypoint_position.min(waypoints.len() - 1);
    let end = (start + tracking_rule.nearest_lookahead_points.max(1)).min(waypoints.len() - 1);
    let mut best_position = start;
    let mut best_distance = f64::MAX;

    for position in start..end {
        let candidate_position = position + 1;
        let candidate = &waypoints[candidate_position];
        let distance = distance_auto_track_path(candidate.match_pt, current_position);
        if distance < best_distance {
            best_distance = distance;
            best_position = candidate_position;
        }
        if tracking_rule
            .key_point_types
            .iter()
            .any(|point_type| point_type == &candidate.point_type)
        {
            break;
        }
    }

    if start == waypoints.len() - 1 {
        best_position = start;
        best_distance = distance_auto_track_path(waypoints[start].match_pt, current_position);
    }

    let waypoint = &waypoints[best_position];
    Ok(AutoTrackPathTargetSelection {
        waypoint_position: best_position,
        waypoint_index: waypoint.index,
        waypoint_type: waypoint.point_type.clone(),
        target_match_position: waypoint.match_pt,
        distance: best_distance,
        target_angle_degrees: angle_auto_track_path(current_position, waypoint.match_pt),
    })
}

pub fn compute_auto_track_path_rotation(
    target_angle_degrees: f64,
    current_angle_degrees: f64,
    angle_offset_unit: i32,
    rotation_unit: i32,
) -> i32 {
    if angle_offset_unit == 0 {
        return 0;
    }
    let angle_delta = normalize_auto_track_path_angle_delta(
        target_angle_degrees.round() - current_angle_degrees.round(),
    );
    (angle_delta / angle_offset_unit as f64 * rotation_unit as f64) as i32
}

fn normalize_auto_track_path_angle_delta(delta: f64) -> f64 {
    let mut normalized = delta;
    while normalized > 180.0 {
        normalized -= 360.0;
    }
    while normalized <= -180.0 {
        normalized += 360.0;
    }
    normalized
}

fn ensure_mini_map_visible(
    observation: &AutoTrackPathTrackingObservation,
    context: &str,
) -> Result<()> {
    if observation.mini_map_visible {
        Ok(())
    } else {
        Err(auto_track_path_runtime_error(format!(
            "{context} requires a visible mini-map"
        )))
    }
}

fn distance_auto_track_path(lhs: AutoTrackPathMapPosition, rhs: AutoTrackPathMapPosition) -> f64 {
    ((lhs.x - rhs.x).powi(2) + (lhs.y - rhs.y).powi(2)).sqrt()
}

fn angle_auto_track_path(from: AutoTrackPathMapPosition, to: AutoTrackPathMapPosition) -> f64 {
    ((to.y - from.y).atan2(to.x - from.x) * 180.0 / std::f64::consts::PI).round()
}

fn auto_track_path_runtime_error(message: impl Into<String>) -> TaskError {
    TaskError::CommonJobExecution(format!(
        "{} execution failed: {}",
        AUTO_TRACK_PATH_TASK_KEY,
        message.into()
    ))
}

fn summarize_legacy_path(path: &LegacyGiPath) -> AutoTrackPathSummary {
    let key_point_types = ["KeyPoint", "Fighting", "Collection"]
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    AutoTrackPathSummary {
        waypoint_count: path.way_point_list.len(),
        first_point: path.way_point_list.first().map(point_plan),
        last_point: path.way_point_list.last().map(point_plan),
        key_point_indices: path
            .way_point_list
            .iter()
            .filter(|point| key_point_types.contains(&point.r#type))
            .map(|point| point.index)
            .collect(),
        key_point_types,
    }
}

fn point_plan(point: &LegacyGiPathPoint) -> AutoTrackPathPointPlan {
    AutoTrackPathPointPlan {
        index: point.index,
        pt_x: point.pt.x,
        pt_y: point.pt.y,
        match_x: point.match_pt.x,
        match_y: point.match_pt.y,
    }
}

fn waypoint_plan(point: &LegacyGiPathPoint) -> AutoTrackPathWaypointPlan {
    AutoTrackPathWaypointPlan {
        index: point.index,
        point_type: point.r#type.clone(),
        pt: AutoTrackPathMapPosition {
            x: point.pt.x,
            y: point.pt.y,
        },
        match_pt: AutoTrackPathMapPosition {
            x: point.match_pt.x,
            y: point.match_pt.y,
        },
    }
}

fn auto_track_path_steps() -> Vec<AutoTrackPathStep> {
    use AutoTrackPathAction::*;
    use AutoTrackPathPhase::*;
    vec![
        AutoTrackPathStep {
            phase: Startup,
            action: AcquireTaskSemaphore,
        },
        AutoTrackPathStep {
            phase: Startup,
            action: ActivateWindow,
        },
        AutoTrackPathStep {
            phase: Startup,
            action: ReadWay2Json,
        },
        AutoTrackPathStep {
            phase: Teleport,
            action: TeleportToFirstPoint,
        },
        AutoTrackPathStep {
            phase: Teleport,
            action: WaitForMiniMap,
        },
        AutoTrackPathStep {
            phase: Calibration,
            action: CalibrateMouseAngleOffset,
        },
        AutoTrackPathStep {
            phase: Tracking,
            action: SelectNearestNextPoint,
        },
        AutoTrackPathStep {
            phase: Tracking,
            action: RotateTowardTarget,
        },
        AutoTrackPathStep {
            phase: Tracking,
            action: HoldForward,
        },
        AutoTrackPathStep {
            phase: StatusRefresh,
            action: RefreshMotionStatus,
        },
        AutoTrackPathStep {
            phase: Jump,
            action: JumpWhileNormal,
        },
        AutoTrackPathStep {
            phase: Tracking,
            action: CancelTracking,
        },
        AutoTrackPathStep {
            phase: Cleanup,
            action: ClearOverlayAndReleaseSemaphore,
        },
    ]
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn string_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<String> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(|value| value.as_str().map(str::to_string))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Default)]
    struct FakeAutoTrackPathRuntime {
        semaphore_available: bool,
        observations: VecDeque<AutoTrackPathTrackingObservation>,
        calls: Vec<String>,
    }

    impl FakeAutoTrackPathRuntime {
        fn with_observations(observations: Vec<AutoTrackPathTrackingObservation>) -> Self {
            Self {
                semaphore_available: true,
                observations: VecDeque::from(observations),
                calls: Vec::new(),
            }
        }

        fn called(&self, prefix: &str) -> bool {
            self.calls.iter().any(|call| call.starts_with(prefix))
        }

        fn call_count(&self, value: &str) -> usize {
            self.calls.iter().filter(|call| *call == value).count()
        }
    }

    impl AutoTrackPathRuntime for FakeAutoTrackPathRuntime {
        fn acquire_auto_track_path_semaphore(&mut self) -> Result<bool> {
            self.calls.push("acquire".to_string());
            Ok(self.semaphore_available)
        }

        fn activate_auto_track_path_window(&mut self) -> Result<()> {
            self.calls.push("activate".to_string());
            Ok(())
        }

        fn teleport_auto_track_path_to(
            &mut self,
            waypoint: &AutoTrackPathWaypointPlan,
        ) -> Result<()> {
            self.calls.push(format!("teleport:{}", waypoint.index));
            Ok(())
        }

        fn observe_auto_track_path(
            &mut self,
            phase: AutoTrackPathObservationPhase,
        ) -> Result<AutoTrackPathTrackingObservation> {
            self.calls.push(format!("observe:{phase:?}"));
            self.observations
                .pop_front()
                .ok_or_else(|| auto_track_path_runtime_error("test observation queue exhausted"))
        }

        fn move_auto_track_path_mouse(&mut self, delta_x: i32, delta_y: i32) -> Result<()> {
            self.calls.push(format!("mouse:{delta_x}:{delta_y}"));
            Ok(())
        }

        fn set_auto_track_path_forward(&mut self, pressed: bool) -> Result<()> {
            self.calls.push(format!("forward:{pressed}"));
            Ok(())
        }

        fn tap_auto_track_path_jump(&mut self) -> Result<()> {
            self.calls.push("jump".to_string());
            Ok(())
        }

        fn cancel_auto_track_path_tracking(&mut self) -> Result<()> {
            self.calls.push("cancel".to_string());
            Ok(())
        }

        fn delay_auto_track_path(&mut self, duration_ms: u64) -> Result<()> {
            self.calls.push(format!("delay:{duration_ms}"));
            Ok(())
        }

        fn clear_auto_track_path_overlay(&mut self) -> Result<()> {
            self.calls.push("clear".to_string());
            Ok(())
        }

        fn release_auto_track_path_semaphore(&mut self) -> Result<()> {
            self.calls.push("release".to_string());
            Ok(())
        }

        fn draw_auto_track_path_tracking_overlay(
            &mut self,
            selection: &AutoTrackPathTargetSelection,
            _observation: &AutoTrackPathTrackingObservation,
        ) -> Result<()> {
            self.calls
                .push(format!("draw:{}", selection.waypoint_index));
            Ok(())
        }
    }

    #[test]
    fn empty_path_returns_empty_status_and_runs_cleanup() {
        let plan = test_plan(Vec::new());
        let mut runtime = FakeAutoTrackPathRuntime {
            semaphore_available: true,
            ..FakeAutoTrackPathRuntime::default()
        };

        let report = execute_auto_track_path_plan(&plan, &mut runtime, 10).unwrap();

        assert_eq!(report.status, AutoTrackPathExecutionStatus::EmptyPath);
        assert!(report.state.cleanup_completed);
        assert!(!report.state.semaphore_acquired);
        assert!(runtime.called("acquire"));
        assert!(runtime.called("activate"));
        assert!(runtime.called("clear"));
        assert!(runtime.called("release"));
        assert!(!runtime.called("teleport"));
    }

    #[test]
    fn two_point_path_executes_until_last_point_and_cancels_tracking() {
        let plan = test_plan(vec![
            waypoint(0, "Normal", 0.0, 0.0),
            waypoint(1, "Fighting", 20.0, 0.0),
        ]);
        let mut runtime = FakeAutoTrackPathRuntime::with_observations(vec![
            observation(None, None, AutoTrackPathMotionStatus::Normal),
            observation(None, Some(0.0), AutoTrackPathMotionStatus::Normal),
            observation(None, Some(10.0), AutoTrackPathMotionStatus::Normal),
            observation(
                Some(AutoTrackPathMapPosition { x: 0.0, y: 0.0 }),
                Some(0.0),
                AutoTrackPathMotionStatus::Normal,
            ),
            observation(None, Some(0.0), AutoTrackPathMotionStatus::Normal),
            observation(None, None, AutoTrackPathMotionStatus::Normal),
            observation(None, None, AutoTrackPathMotionStatus::Normal),
            observation(
                Some(AutoTrackPathMapPosition { x: 20.0, y: 0.0 }),
                Some(0.0),
                AutoTrackPathMotionStatus::Normal,
            ),
        ]);

        let report = execute_auto_track_path_plan(&plan, &mut runtime, 10).unwrap();

        assert_eq!(report.status, AutoTrackPathExecutionStatus::Completed);
        assert!(report.state.tracking_cancelled);
        assert!(report.state.cleanup_completed);
        assert_eq!(report.state.current_waypoint_position, 1);
        assert_eq!(report.state.jump_count, 2);
        assert!(runtime.called("teleport:0"));
        assert!(runtime.called("draw:1"));
        assert!(runtime.called("mouse:500:0"));
        assert!(runtime.called("mouse:0:0"));
        assert!(runtime.called("forward:true"));
        assert!(runtime.called("forward:false"));
        assert_eq!(runtime.call_count("jump"), 2);
        assert!(runtime.called("cancel"));
        assert!(runtime.called("clear"));
        assert!(runtime.called("release"));
        assert!(report.actions.iter().any(|action| {
            action.action == AutoTrackPathAction::CancelTracking
                && action.status == AutoTrackPathActionStatus::Succeeded
        }));
    }

    #[test]
    fn calibration_failure_returns_error_after_cleanup() {
        let plan = test_plan(vec![
            waypoint(0, "Normal", 0.0, 0.0),
            waypoint(1, "Normal", 20.0, 0.0),
        ]);
        let mut runtime = FakeAutoTrackPathRuntime::with_observations(vec![
            observation(None, None, AutoTrackPathMotionStatus::Normal),
            observation(None, Some(10.0), AutoTrackPathMotionStatus::Normal),
            observation(None, Some(10.0), AutoTrackPathMotionStatus::Normal),
        ]);

        let error = execute_auto_track_path_plan(&plan, &mut runtime, 10).unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("angle offset calibration failed")
        ));
        assert!(runtime.called("clear"));
        assert!(runtime.called("release"));
        assert!(runtime.called("forward:true"));
        assert!(runtime.called("forward:false"));
        assert!(!runtime.called("draw"));
    }

    #[test]
    fn jump_is_dispatched_only_when_motion_status_is_normal() {
        let plan = test_plan(vec![
            waypoint(0, "Normal", 0.0, 0.0),
            waypoint(1, "Normal", 20.0, 0.0),
        ]);
        let mut runtime = FakeAutoTrackPathRuntime::with_observations(vec![
            observation(None, None, AutoTrackPathMotionStatus::Normal),
            observation(None, Some(0.0), AutoTrackPathMotionStatus::Normal),
            observation(None, Some(10.0), AutoTrackPathMotionStatus::Normal),
            observation(
                Some(AutoTrackPathMapPosition { x: 0.0, y: 0.0 }),
                Some(0.0),
                AutoTrackPathMotionStatus::Normal,
            ),
            observation(None, Some(0.0), AutoTrackPathMotionStatus::Normal),
            observation(None, None, AutoTrackPathMotionStatus::Fly),
        ]);

        let report = execute_auto_track_path_plan(&plan, &mut runtime, 1).unwrap();

        assert_eq!(
            report.status,
            AutoTrackPathExecutionStatus::IterationLimitReached
        );
        assert_eq!(report.state.motion_status, AutoTrackPathMotionStatus::Fly);
        assert_eq!(report.state.jump_count, 0);
        assert_eq!(runtime.call_count("jump"), 0);
        assert!(report.actions.iter().any(|action| {
            action.action == AutoTrackPathAction::JumpWhileNormal
                && action.status == AutoTrackPathActionStatus::Skipped
        }));
        assert!(runtime.called("forward:false"));
        assert!(runtime.called("clear"));
        assert!(runtime.called("release"));
    }

    fn test_plan(waypoints: Vec<AutoTrackPathWaypointPlan>) -> AutoTrackPathExecutionPlan {
        let key_point_types = vec![
            "KeyPoint".to_string(),
            "Fighting".to_string(),
            "Collection".to_string(),
        ];
        let summary = AutoTrackPathSummary {
            waypoint_count: waypoints.len(),
            first_point: waypoints.first().map(point_plan_from_waypoint),
            last_point: waypoints.last().map(point_plan_from_waypoint),
            key_point_indices: waypoints
                .iter()
                .filter(|point| key_point_types.contains(&point.point_type))
                .map(|point| point.index)
                .collect(),
            key_point_types: key_point_types.clone(),
        };

        AutoTrackPathExecutionPlan {
            task_key: AUTO_TRACK_PATH_TASK_KEY.to_string(),
            display_name: AUTO_TRACK_PATH_DISPLAY_NAME.to_string(),
            capture_size: Size::new(
                AUTO_TRACK_PATH_DEFAULT_CAPTURE_WIDTH,
                AUTO_TRACK_PATH_DEFAULT_CAPTURE_HEIGHT,
            ),
            path_file: AUTO_TRACK_PATH_DEFAULT_PATH_FILE.to_string(),
            config_rule: AutoTrackPathConfigRule {
                map_zoom_enabled: true,
                map_zoom_out_distance: 1000,
                map_zoom_in_distance: 400,
                step_interval_milliseconds: 20,
                max_zoom_level: 5.0,
                min_zoom_level: 2.0,
                tolerance: 200.0,
                max_iterations: 30,
                max_mouse_move: 300,
                map_scale_factor: 2.361,
                hp_restore_duration: 5.0,
            },
            path_summary: summary,
            waypoints,
            startup_rule: AutoTrackPathStartupRule {
                uses_task_semaphore_non_blocking: true,
                activates_game_window: true,
                logs_start: true,
                treats_normal_end_as_manual_interrupt: true,
                clears_draw_content_on_finish: true,
                releases_task_semaphore_on_finish: true,
            },
            teleport_rule: AutoTrackPathTeleportRule {
                teleports_to_first_waypoint: true,
                post_teleport_sleep_ms: 0,
                wait_minimap_retry_attempts: 1,
                wait_minimap_retry_interval_ms: 0,
                post_minimap_detected_sleep_ms: 0,
                paimon_menu_locator_asset: "Common/Element:paimon_menu.png".to_string(),
                mini_map_crop_from_paimon: Rect {
                    x: 24,
                    y: -15,
                    width: 210,
                    height: 210,
                },
            },
            angle_calibration_rule: AutoTrackPathAngleCalibrationRule {
                char_moving_unit: AUTO_TRACK_PATH_CHAR_MOVING_UNIT,
                mouse_move_x: AUTO_TRACK_PATH_CHAR_MOVING_UNIT,
                after_mouse_move_sleep_ms: 0,
                move_forward_hold_ms: 0,
                after_forward_sleep_ms: 0,
                fails_when_angle_offset_zero: true,
            },
            tracking_rule: AutoTrackPathTrackingRule {
                nearest_lookahead_points: 20,
                stop_distance: 10.0,
                key_point_types,
                rotation_unit: AUTO_TRACK_PATH_CHAR_MOVING_UNIT,
                rotation_formula:
                    "(target_angle - current_angle) / angle_offset_unit * CharMovingUnit"
                        .to_string(),
                after_mouse_move_sleep_ms: 0,
                after_angle_recheck_sleep_ms: 0,
                move_forward_after_rotation: true,
                post_forward_sleep_ms: 0,
                release_forward_when_reaching_point: true,
                cancels_track_when_last_point_reached: true,
            },
            status_refresh_rule: AutoTrackPathStatusRefreshRule {
                captures_motion_status: true,
                interval_ms: 0,
                main_ui_required_by_minimap_detection: true,
            },
            jump_rule: AutoTrackPathJumpRule {
                only_jumps_when_motion_normal: true,
                first_jump_interval_ms: 0,
                second_jump_followup_sleep_ms: 0,
                interrupted_motion_sleep_ms: 0,
                non_normal_sleep_ms: 0,
            },
            steps: auto_track_path_steps(),
            executor_ready: true,
            pending_native: Vec::new(),
        }
    }

    fn waypoint(
        index: i32,
        point_type: &str,
        match_x: f64,
        match_y: f64,
    ) -> AutoTrackPathWaypointPlan {
        AutoTrackPathWaypointPlan {
            index,
            point_type: point_type.to_string(),
            pt: AutoTrackPathMapPosition {
                x: match_x,
                y: match_y,
            },
            match_pt: AutoTrackPathMapPosition {
                x: match_x,
                y: match_y,
            },
        }
    }

    fn observation(
        avatar_map_position: Option<AutoTrackPathMapPosition>,
        character_orientation_degrees: Option<f64>,
        motion_status: AutoTrackPathMotionStatus,
    ) -> AutoTrackPathTrackingObservation {
        AutoTrackPathTrackingObservation {
            mini_map_visible: true,
            avatar_map_position,
            character_orientation_degrees,
            motion_status,
        }
    }

    fn point_plan_from_waypoint(waypoint: &AutoTrackPathWaypointPlan) -> AutoTrackPathPointPlan {
        AutoTrackPathPointPlan {
            index: waypoint.index,
            pt_x: waypoint.pt.x,
            pt_y: waypoint.pt.y,
            match_x: waypoint.match_pt.x,
            match_y: waypoint.match_pt.y,
        }
    }
}
