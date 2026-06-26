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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoTrackPathPhase {
    Startup,
    Teleport,
    Calibration,
    Tracking,
    StatusRefresh,
    Jump,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
    ClearOverlayAndReleaseSemaphore,
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
        executor_ready: false,
        pending_native: vec![
            "TaskSemaphore, CancellationContext, SystemControl window activation, and DrawContent cleanup".to_string(),
            "TpTask teleport execution, mini-map capture from Paimon menu, map matching, and CharacterOrientation/CameraOrientation computation".to_string(),
            "live tracking loop, mouse movement, W/Space input dispatch, Bv motion status recognition, and cancellation across parallel tasks".to_string(),
        ],
    })
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
