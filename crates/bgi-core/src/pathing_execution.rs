use super::{PathingSummary, PathingTask, Waypoint};
use crate::{GenshinAction, KeyId};
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingExecutionPlan {
    pub summary: PathingSummary,
    pub map_name: String,
    pub map_match_method: Option<String>,
    pub retry_times: u8,
    pub has_positions: bool,
    pub segment_count: usize,
    pub waypoint_count: usize,
    pub action_count: usize,
    pub expected_fight_count: usize,
    pub autopick_realtime_trigger_enabled: bool,
    pub preflight: PathingPreflightPlan,
    pub movement_contract: PathingMovementContractPlan,
    pub farming: PathingFarmingExecutionPlan,
    pub segments: Vec<PathingSegmentPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PathingTrackConversionContext<'a> {
    pub map_name: &'a str,
    pub map_match_method: Option<&'a str>,
    pub global_index: usize,
    pub segment_index: usize,
    pub segment_waypoint_index: usize,
    pub waypoint_type: &'a str,
    pub action: Option<&'a str>,
    pub route_point: PathingPoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PathingPreflightPlan {
    pub switch_party_before: bool,
    pub validate_game_with_task: bool,
    pub initialize_pathing: bool,
    pub update_current_pathing: bool,
    pub require_16_by_9_resolution: bool,
    pub minimum_width: u32,
    pub minimum_height: u32,
    pub convert_waypoints_for_track: bool,
    pub delay_before_warm_up_ms: u32,
    pub warm_up_navigation: bool,
    pub release_input_after_segment_attempt: bool,
}

impl PathingPreflightPlan {
    fn for_task(task: &PathingTask) -> Self {
        let has_positions = !task.positions.is_empty();
        Self {
            switch_party_before: has_positions,
            validate_game_with_task: has_positions,
            initialize_pathing: has_positions,
            update_current_pathing: has_positions,
            require_16_by_9_resolution: has_positions,
            minimum_width: 1920,
            minimum_height: 1080,
            convert_waypoints_for_track: has_positions,
            delay_before_warm_up_ms: if has_positions { 100 } else { 0 },
            warm_up_navigation: has_positions,
            release_input_after_segment_attempt: has_positions,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingMovementContractPlan {
    pub contract_version: u8,
    pub movement_executor_ready: bool,
    pub native_pathing_completed: bool,
    pub pending_dependencies: Vec<PathingMovementDependency>,
    pub map_name: String,
    pub map_match_method: Option<String>,
    pub preflight: PathingPreflightPlan,
    pub release_input_after_segment_attempt: bool,
    pub segment_count: usize,
    pub waypoint_count: usize,
    pub segments: Vec<PathingMovementSegmentContract>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathingMovementDependency {
    CoordinateConversion,
    MapMatching,
    PositionObservation,
    CameraRotation,
    InputDispatch,
    Teleport,
    LowHpRecovery,
    TrapEscape,
    ActionHandlers,
    MovementTermination,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingMovementSegmentContract {
    pub segment_index: usize,
    pub retry_times: u8,
    pub starts_with_teleport: bool,
    pub pre_teleport_delay_ms: u32,
    pub seed_previous_position: Option<PathingPoint>,
    pub seed_previous_position_coordinate_space: Option<PathingCoordinateSpace>,
    pub seed_previous_position_requires_track_conversion: bool,
    pub release_input_after_attempt: bool,
    pub waypoints: Vec<PathingMovementWaypointContract>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingMovementWaypointContract {
    pub global_index: usize,
    pub segment_index: usize,
    pub segment_waypoint_index: usize,
    pub waypoint_type: String,
    pub move_mode: String,
    pub action: Option<String>,
    pub action_params: Option<String>,
    pub route_point: PathingPoint,
    pub track_point: Option<PathingPoint>,
    pub track_conversion_pending: bool,
    pub effective_target_point: bool,
    pub phase_contracts: Vec<PathingMovementPhaseContract>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingMovementPhaseContract {
    pub phase: PathingWaypointPhase,
    pub target_point: Option<PathingPoint>,
    pub coordinate_space: Option<PathingCoordinateSpace>,
    pub requires_track_conversion: bool,
    pub native_status: PathingNativePhaseStatus,
    pub pending_dependencies: Vec<PathingMovementDependency>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathingNativePhaseStatus {
    Pending,
    ReadyByRuntime,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingFarmingExecutionPlan {
    pub allow_farming_count: bool,
    pub primary_target: String,
    pub normal_mob_count: f64,
    pub elite_mob_count: f64,
    pub expected_fight_count: usize,
}

impl PathingFarmingExecutionPlan {
    fn from_task(task: &PathingTask, expected_fight_count: usize) -> Self {
        Self {
            allow_farming_count: value_bool_alias(
                &task.farming_info,
                &["allow_farming_count", "AllowFarmingCount"],
            )
            .unwrap_or(false),
            primary_target: value_string_alias(
                &task.farming_info,
                &["primary_target", "PrimaryTarget"],
            )
            .unwrap_or_default(),
            normal_mob_count: value_f64_alias(
                &task.farming_info,
                &["normal_mob_count", "NormalMobCount"],
            )
            .unwrap_or(0.0),
            elite_mob_count: value_f64_alias(
                &task.farming_info,
                &["elite_mob_count", "EliteMobCount"],
            )
            .unwrap_or(0.0),
            expected_fight_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingSegmentPlan {
    pub segment_index: usize,
    pub waypoint_count: usize,
    pub starts_with_teleport: bool,
    pub pre_teleport_delay_ms: u32,
    pub seed_previous_position: Option<PathingPoint>,
    pub seed_previous_position_coordinate_space: Option<PathingCoordinateSpace>,
    pub seed_previous_position_requires_track_conversion: bool,
    pub resolves_anomalies_before_attempt: bool,
    pub retry_times: u8,
    pub waypoints: Vec<PathingWaypointPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathingCoordinateSpace {
    RouteJson,
    LegacyTrackMap,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingWaypointPlan {
    pub global_index: usize,
    pub segment_index: usize,
    pub segment_waypoint_index: usize,
    pub x: f64,
    pub y: f64,
    pub route_point: PathingPoint,
    pub track_point: Option<PathingPoint>,
    pub track_conversion_pending: bool,
    pub waypoint_type: String,
    pub move_mode: String,
    pub action: Option<String>,
    pub action_params: Option<String>,
    pub declared_action_use: Option<PathingActionUseWaypointType>,
    pub action_plan: Option<PathingActionPlan>,
    pub effective_target_point: bool,
    pub phases: Vec<PathingWaypointPhase>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum PathingActionPlan {
    LinneaMining(Box<LinneaMiningActionPlan>),
    SetTime(PathingSetTimeActionPlan),
    LogOutput(PathingLogOutputActionPlan),
    CommonJob(PathingCommonJobActionPlan),
    ForceTeleport(PathingForceTeleportActionPlan),
    NahidaCollect(PathingNahidaCollectActionPlan),
    ElementalCollect(PathingElementalCollectActionPlan),
    UseGadget(PathingUseGadgetActionPlan),
    PickAround(PathingPickAroundActionPlan),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LinneaMiningActionPlan {
    pub action_code: String,
    pub raw_params: Option<String>,
    pub mine_count: i32,
    pub scan_rounds: i32,
    pub prefer_right: bool,
    pub avatar_name: String,
    pub switch_avatar_before_mining: bool,
    pub switch_wait_ms: u32,
    pub aiming_mode_action: GenshinAction,
    pub enter_aim_wait_ms: u32,
    pub detection_rule: LinneaMiningDetectionRule,
    pub cluster_rule: LinneaMiningClusterRule,
    pub alignment_rule: LinneaMiningAlignmentRule,
    pub scan_rule: LinneaMiningScanRule,
    pub mine_rule: LinneaMiningMineRule,
    pub cleanup_rule: LinneaMiningCleanupRule,
    pub executor_ready: bool,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LinneaMiningDetectionRule {
    pub model_name: String,
    pub model_relative_path: String,
    pub accepted_label: String,
    pub confidence_threshold: f32,
    pub source: LinneaMiningDetectionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LinneaMiningDetectionSource {
    FullCapture,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LinneaMiningClusterRule {
    pub base_cluster_distance_1080p: f64,
    pub base_cluster_area_1080p: f64,
    pub base_alignment_expansion_1080p: f64,
    pub base_edge_ignore_1080p: f64,
    pub area_ratio_threshold: f64,
    pub prefer_right_when_scan_rounds_gt_one: bool,
    pub target_selection: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LinneaMiningAlignmentRule {
    pub max_inner_retry: u8,
    pub element_sight_refresh_ms: u32,
    pub refresh_release_ms: u32,
    pub refresh_hold_ms: u32,
    pub aim_sensitivity_factor_x: f64,
    pub aim_sensitivity_factor_y: f64,
    pub aim_move_delay_ms: u32,
    pub fallback_shot_on_last_successful_detection: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LinneaMiningScanRule {
    pub middle_button_hold_ms: u32,
    pub middle_button_release_ms: u32,
    pub compensate_detection_hold_ms: u32,
    pub compensate_move_wait_ms: u32,
    pub left_turn_step_1080p: i32,
    pub left_turn_wait_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LinneaMiningMineRule {
    pub compensate_up_pixels: i32,
    pub compensate_up_wait_ms: u32,
    pub attack_button: String,
    pub after_attack_wait_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LinneaMiningCleanupRule {
    pub leave_aiming_mode_action: GenshinAction,
    pub middle_button_up: bool,
    pub clear_vision_drawings: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingSetTimeActionPlan {
    pub action_code: String,
    pub raw_params: Option<String>,
    pub common_job_task_key: String,
    pub hour: Option<i32>,
    pub minute: Option<i32>,
    pub skip_time_adjustment_animation: Option<bool>,
    pub parse_error: Option<String>,
    pub executor_ready: bool,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingLogOutputActionPlan {
    pub action_code: String,
    pub raw_params: Option<String>,
    pub message: String,
    pub executor_ready: bool,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingCommonJobActionPlan {
    pub action_code: String,
    pub raw_params: Option<String>,
    pub common_job_task_key: String,
    pub executor_ready: bool,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingForceTeleportActionPlan {
    pub action_code: String,
    pub raw_params: Option<String>,
    pub force_teleport: bool,
    pub executor_ready: bool,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingUseGadgetActionPlan {
    pub action_code: String,
    pub raw_params: Option<String>,
    pub genshin_action: GenshinAction,
    pub not_wait: bool,
    pub cooldown_ocr_required: bool,
    pub max_wait_seconds: Option<f64>,
    pub max_wait_parse_error: Option<String>,
    pub cooldown_overlong_skip_threshold_seconds: f64,
    pub cooldown_wait_padding_ms: u32,
    pub quick_use_gadget_press_count: u8,
    pub handler_delay_ms: u32,
    pub path_executor_after_action_delay_ms: u32,
    pub executor_ready: bool,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingNahidaCollectActionPlan {
    pub action_code: String,
    pub raw_params: Option<String>,
    pub avatar_name: String,
    pub requires_combat_scenes: bool,
    pub switch_avatar_before_collect: bool,
    pub wait_skill_cooldown_before_collect: bool,
    pub update_skill_cooldown_after_collect: bool,
    pub dpi_scale_required: bool,
    pub lower_view_mouse_move_y: i32,
    pub lower_view_delay_ms: u32,
    pub elemental_skill_action: GenshinAction,
    pub skill_hold_initial_delay_ms: u32,
    pub ground_scan_iterations: u8,
    pub ground_scan_move_x: i32,
    pub ground_scan_move_x_dpi_scaled: bool,
    pub ground_scan_move_y: i32,
    pub ground_scan_move_y_dpi_scaled: bool,
    pub raised_scan_iterations: u8,
    pub raised_scan_move_x: i32,
    pub raised_scan_move_x_dpi_scaled: bool,
    pub raised_scan_initial_move_y: i32,
    pub raised_scan_y_adjust_before_iteration: u8,
    pub raised_scan_y_adjust_delta: i32,
    pub raised_scan_move_y_dpi_scaled: bool,
    pub scan_step_delay_ms: u32,
    pub post_skill_release_cd_update_delay_ms: u32,
    pub after_collect_delay_ms: u32,
    pub restore_view_key: KeyId,
    pub restore_view_delay_ms: u32,
    pub path_executor_after_action_delay_ms: u32,
    pub skill_release_in_finally: bool,
    pub steps: Vec<PathingNahidaCollectStep>,
    pub executor_ready: bool,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingElementalCollectActionPlan {
    pub action_code: String,
    pub raw_params: Option<String>,
    pub element: PathingElementalCollectElement,
    pub element_chinese: String,
    pub requires_combat_scenes: bool,
    pub legacy_preflight_validates_avatar: bool,
    pub scans_team_order: bool,
    pub switch_avatar_before_collect: bool,
    pub normal_attack_preferred_over_skill: bool,
    pub normal_attack_duration_ms: u32,
    pub wait_skill_cooldown_before_skill: bool,
    pub path_executor_after_action_delay_ms: u32,
    pub candidates: Vec<PathingElementalCollectAvatarPlan>,
    pub executor_ready: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathingElementalCollectElement {
    Hydro,
    Electro,
    Anemo,
    Pyro,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingElementalCollectAvatarPlan {
    pub avatar_name: String,
    pub normal_attack: bool,
    pub elemental_skill: bool,
    pub selected_action: PathingElementalCollectAvatarAction,
    pub normal_attack_duration_ms: Option<u32>,
    pub waits_skill_cooldown: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathingElementalCollectAvatarAction {
    NormalAttack,
    ElementalSkill,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PathingNahidaCollectStep {
    GenshinAction {
        action: GenshinAction,
        press: PathingInputPress,
    },
    Key {
        key: KeyId,
        press: PathingInputPress,
    },
    MouseMoveBy {
        dx: i32,
        dy: i32,
        dx_dpi_scaled: bool,
        dy_dpi_scaled: bool,
    },
    Delay {
        milliseconds: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingPickAroundActionPlan {
    pub action_code: String,
    pub raw_params: Option<String>,
    pub turns: i32,
    pub turn_parse_error: Option<String>,
    pub speed: f64,
    pub circle_segments_per_turn: u8,
    pub circle_start_ms: u32,
    pub circle_interval_ms: u32,
    pub circle_time_ms: f64,
    pub view_reset_base_ms: u32,
    pub turn_plans: Vec<PathingPickAroundTurnPlan>,
    pub steps: Vec<PathingPickAroundStep>,
    pub executor_ready: bool,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingPickAroundTurnPlan {
    pub turn_index: u32,
    pub edge_delay_ms: u32,
    pub radius_time_ms: f64,
    pub end_angle_radians: f64,
    pub move_backward_forward_ms: i32,
    pub move_left_forward_ms: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathingInputPress {
    KeyPress,
    KeyDown,
    KeyUp,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PathingPickAroundStep {
    GenshinAction {
        turn_index: u32,
        action: GenshinAction,
        press: PathingInputPress,
    },
    Key {
        turn_index: u32,
        key: KeyId,
        press: PathingInputPress,
    },
    Delay {
        turn_index: u32,
        milliseconds: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PathingPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct LegacyTrackMapCoordinateRule {
    pub origin_x: f64,
    pub origin_y: f64,
    pub block_width_scale: f64,
}

pub fn legacy_track_map_coordinate_rule(map_name: &str) -> Option<LegacyTrackMapCoordinateRule> {
    match map_name.trim() {
        name if name.eq_ignore_ascii_case("Teyvat") => Some(LegacyTrackMapCoordinateRule {
            origin_x: 32768.0,
            origin_y: 16384.0,
            block_width_scale: 2.0,
        }),
        _ => None,
    }
}

pub fn legacy_track_map_point(map_name: &str, point: PathingPoint) -> Option<PathingPoint> {
    let rule = legacy_track_map_coordinate_rule(map_name)?;
    Some(PathingPoint {
        x: rule.origin_x - point.x * rule.block_width_scale,
        y: rule.origin_y - point.y * rule.block_width_scale,
    })
}

pub fn legacy_track_map_point_for_pathing(
    context: PathingTrackConversionContext<'_>,
) -> Option<PathingPoint> {
    legacy_track_map_point(context.map_name, context.route_point)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathingActionUseWaypointType {
    Custom,
    Path,
    Target,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathingWaypointPhase {
    RecoverWhenLowHp,
    HandleTeleport,
    BeforeMoveToTarget,
    FaceTo,
    MoveTo,
    BeforeMoveCloseToTarget,
    MoveCloseTo,
    RunAction,
}

const PATH_EXECUTOR_RETRY_TIMES: u8 = 2;
const PATH_EXECUTOR_PRE_TELEPORT_DELAY_MS: u32 = 1_000;

impl PathingTask {
    pub fn execution_plan(&self) -> PathingExecutionPlan {
        let segments = split_waypoints_for_track(&self.positions);
        self.execution_plan_from_segments(segments)
    }

    pub fn execution_plan_with_legacy_track_converter(&self) -> PathingExecutionPlan {
        self.execution_plan_with_track_converter(legacy_track_map_point_for_pathing)
    }

    pub fn execution_plan_with_track_converter<F>(&self, mut converter: F) -> PathingExecutionPlan
    where
        F: FnMut(PathingTrackConversionContext<'_>) -> Option<PathingPoint>,
    {
        let mut segments = split_waypoints_for_track(&self.positions);
        apply_pathing_track_conversion(
            &mut segments,
            &self.info.map_name,
            self.info.map_match_method.as_deref(),
            &mut converter,
        );
        self.execution_plan_from_segments(segments)
    }

    fn execution_plan_from_segments(
        &self,
        segments: Vec<PathingSegmentPlan>,
    ) -> PathingExecutionPlan {
        let preflight = PathingPreflightPlan::for_task(self);
        let expected_fight_count = self
            .positions
            .iter()
            .filter(|waypoint| action_eq(waypoint.action.as_deref(), "fight"))
            .count();
        let action_count = self
            .positions
            .iter()
            .filter(|waypoint| has_non_empty_action(waypoint.action.as_deref()))
            .count();

        PathingExecutionPlan {
            summary: self.summary(),
            map_name: self.info.map_name.clone(),
            map_match_method: self.info.map_match_method.clone(),
            retry_times: PATH_EXECUTOR_RETRY_TIMES,
            has_positions: !self.positions.is_empty(),
            segment_count: segments.len(),
            waypoint_count: self.positions.len(),
            action_count,
            expected_fight_count,
            autopick_realtime_trigger_enabled: realtime_trigger_enabled(
                &self.config.realtime_triggers,
                "AutoPick",
            ),
            preflight: preflight.clone(),
            movement_contract: PathingMovementContractPlan::from_segments(
                &self.info.map_name,
                self.info.map_match_method.as_deref(),
                &preflight,
                &segments,
            ),
            farming: PathingFarmingExecutionPlan::from_task(self, expected_fight_count),
            segments,
        }
    }
}

fn apply_pathing_track_conversion<F>(
    segments: &mut [PathingSegmentPlan],
    map_name: &str,
    map_match_method: Option<&str>,
    converter: &mut F,
) where
    F: FnMut(PathingTrackConversionContext<'_>) -> Option<PathingPoint>,
{
    for segment in segments {
        for waypoint in &mut segment.waypoints {
            let context = PathingTrackConversionContext {
                map_name,
                map_match_method,
                global_index: waypoint.global_index,
                segment_index: waypoint.segment_index,
                segment_waypoint_index: waypoint.segment_waypoint_index,
                waypoint_type: &waypoint.waypoint_type,
                action: waypoint.action.as_deref(),
                route_point: waypoint.route_point,
            };
            if let Some(track_point) = converter(context) {
                waypoint.track_point = Some(track_point);
                waypoint.track_conversion_pending = false;
            }
        }

        if segment.starts_with_teleport {
            continue;
        }
        if let Some(first_track_point) = segment
            .waypoints
            .first()
            .and_then(|waypoint| waypoint.track_point)
        {
            segment.seed_previous_position = Some(first_track_point);
            segment.seed_previous_position_coordinate_space =
                Some(PathingCoordinateSpace::LegacyTrackMap);
            segment.seed_previous_position_requires_track_conversion = false;
        }
    }
}

fn split_waypoints_for_track(positions: &[Waypoint]) -> Vec<PathingSegmentPlan> {
    if positions.is_empty() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut current = Vec::new();

    for (global_index, waypoint) in positions.iter().enumerate() {
        if waypoint_type_eq(&waypoint.waypoint_type, "teleport") && !current.is_empty() {
            push_pathing_segment(&mut segments, std::mem::take(&mut current));
        }

        current.push((global_index, waypoint));
    }

    push_pathing_segment(&mut segments, current);
    segments
}

fn push_pathing_segment(
    segments: &mut Vec<PathingSegmentPlan>,
    waypoints: Vec<(usize, &Waypoint)>,
) {
    if waypoints.is_empty() {
        return;
    }

    let segment_index = segments.len();
    let starts_with_teleport = waypoints
        .first()
        .map(|(_, waypoint)| waypoint_type_eq(&waypoint.waypoint_type, "teleport"))
        .unwrap_or(false);
    let pre_teleport_delay_ms = pre_teleport_delay_ms(starts_with_teleport, segments.last());
    let seed_previous_position = if starts_with_teleport {
        None
    } else {
        waypoints.first().map(|(_, waypoint)| PathingPoint {
            x: waypoint.x,
            y: waypoint.y,
        })
    };
    let waypoint_count = waypoints.len();
    let waypoints = waypoints
        .into_iter()
        .enumerate()
        .map(|(segment_waypoint_index, (global_index, waypoint))| {
            PathingWaypointPlan::from_waypoint(
                segment_index,
                segment_waypoint_index,
                global_index,
                waypoint,
            )
        })
        .collect();

    segments.push(PathingSegmentPlan {
        segment_index,
        waypoint_count,
        starts_with_teleport,
        pre_teleport_delay_ms,
        seed_previous_position,
        seed_previous_position_coordinate_space: if starts_with_teleport {
            None
        } else {
            Some(PathingCoordinateSpace::RouteJson)
        },
        seed_previous_position_requires_track_conversion: !starts_with_teleport,
        resolves_anomalies_before_attempt: true,
        retry_times: PATH_EXECUTOR_RETRY_TIMES,
        waypoints,
    });
}

fn pre_teleport_delay_ms(
    starts_with_teleport: bool,
    previous_segment: Option<&PathingSegmentPlan>,
) -> u32 {
    if !starts_with_teleport {
        return 0;
    }

    let Some(previous_waypoint) = previous_segment.and_then(|segment| segment.waypoints.last())
    else {
        return 0;
    };
    if legacy_skips_pre_teleport_delay(previous_waypoint) {
        0
    } else {
        PATH_EXECUTOR_PRE_TELEPORT_DELAY_MS
    }
}

fn legacy_skips_pre_teleport_delay(waypoint: &PathingWaypointPlan) -> bool {
    waypoint_type_eq(&waypoint.waypoint_type, "teleport")
        || action_eq(waypoint.action.as_deref(), "fight")
        || action_eq(waypoint.action.as_deref(), "nahida_collect")
        || action_eq(waypoint.action.as_deref(), "pick_around")
}

impl PathingMovementContractPlan {
    fn from_segments(
        map_name: &str,
        map_match_method: Option<&str>,
        preflight: &PathingPreflightPlan,
        segments: &[PathingSegmentPlan],
    ) -> Self {
        let pending_dependencies = aggregate_pathing_movement_pending_dependencies(segments);

        Self {
            contract_version: 1,
            movement_executor_ready: !segments.is_empty() && pending_dependencies.is_empty(),
            native_pathing_completed: false,
            pending_dependencies,
            map_name: map_name.to_string(),
            map_match_method: map_match_method.map(ToOwned::to_owned),
            preflight: preflight.clone(),
            release_input_after_segment_attempt: preflight.release_input_after_segment_attempt,
            segment_count: segments.len(),
            waypoint_count: segments.iter().map(|segment| segment.waypoints.len()).sum(),
            segments: segments
                .iter()
                .map(|segment| {
                    PathingMovementSegmentContract::from_segment(
                        segment,
                        preflight.release_input_after_segment_attempt,
                    )
                })
                .collect(),
        }
    }
}

impl PathingMovementSegmentContract {
    fn from_segment(segment: &PathingSegmentPlan, release_input_after_attempt: bool) -> Self {
        Self {
            segment_index: segment.segment_index,
            retry_times: segment.retry_times,
            starts_with_teleport: segment.starts_with_teleport,
            pre_teleport_delay_ms: segment.pre_teleport_delay_ms,
            seed_previous_position: segment.seed_previous_position,
            seed_previous_position_coordinate_space: segment
                .seed_previous_position_coordinate_space,
            seed_previous_position_requires_track_conversion: segment
                .seed_previous_position_requires_track_conversion,
            release_input_after_attempt,
            waypoints: segment
                .waypoints
                .iter()
                .map(PathingMovementWaypointContract::from_waypoint)
                .collect(),
        }
    }
}

impl PathingMovementWaypointContract {
    fn from_waypoint(waypoint: &PathingWaypointPlan) -> Self {
        Self {
            global_index: waypoint.global_index,
            segment_index: waypoint.segment_index,
            segment_waypoint_index: waypoint.segment_waypoint_index,
            waypoint_type: waypoint.waypoint_type.clone(),
            move_mode: waypoint.move_mode.clone(),
            action: waypoint.action.clone(),
            action_params: waypoint.action_params.clone(),
            route_point: waypoint.route_point,
            track_point: waypoint.track_point,
            track_conversion_pending: waypoint.track_conversion_pending,
            effective_target_point: waypoint.effective_target_point,
            phase_contracts: waypoint
                .phases
                .iter()
                .map(|phase| PathingMovementPhaseContract::from_phase(*phase, waypoint))
                .collect(),
        }
    }
}

impl PathingMovementPhaseContract {
    fn from_phase(phase: PathingWaypointPhase, waypoint: &PathingWaypointPlan) -> Self {
        let target_point = pathing_phase_target_point(phase, waypoint);
        let coordinate_space = target_point.map(|_| {
            if waypoint.track_point.is_some() {
                PathingCoordinateSpace::LegacyTrackMap
            } else {
                PathingCoordinateSpace::RouteJson
            }
        });
        Self {
            phase,
            target_point,
            coordinate_space,
            requires_track_conversion: target_point.is_some() && waypoint.track_conversion_pending,
            native_status: pathing_phase_native_status(phase),
            pending_dependencies: pathing_phase_pending_dependencies(phase, waypoint),
        }
    }
}

const PATHING_MOVEMENT_DEPENDENCY_ORDER: [PathingMovementDependency; 10] = [
    PathingMovementDependency::CoordinateConversion,
    PathingMovementDependency::MapMatching,
    PathingMovementDependency::PositionObservation,
    PathingMovementDependency::CameraRotation,
    PathingMovementDependency::InputDispatch,
    PathingMovementDependency::Teleport,
    PathingMovementDependency::LowHpRecovery,
    PathingMovementDependency::TrapEscape,
    PathingMovementDependency::ActionHandlers,
    PathingMovementDependency::MovementTermination,
];

fn aggregate_pathing_movement_pending_dependencies(
    segments: &[PathingSegmentPlan],
) -> Vec<PathingMovementDependency> {
    PATHING_MOVEMENT_DEPENDENCY_ORDER
        .into_iter()
        .filter(|dependency| {
            segments.iter().any(|segment| {
                segment.waypoints.iter().any(|waypoint| {
                    waypoint.phases.iter().any(|phase| {
                        pathing_phase_pending_dependencies(*phase, waypoint).contains(dependency)
                    })
                })
            })
        })
        .collect()
}

fn pathing_phase_target_point(
    phase: PathingWaypointPhase,
    waypoint: &PathingWaypointPlan,
) -> Option<PathingPoint> {
    match phase {
        PathingWaypointPhase::HandleTeleport
        | PathingWaypointPhase::FaceTo
        | PathingWaypointPhase::MoveTo
        | PathingWaypointPhase::MoveCloseTo
        | PathingWaypointPhase::RunAction => {
            Some(waypoint.track_point.unwrap_or(waypoint.route_point))
        }
        PathingWaypointPhase::RecoverWhenLowHp
        | PathingWaypointPhase::BeforeMoveToTarget
        | PathingWaypointPhase::BeforeMoveCloseToTarget => None,
    }
}

fn pathing_phase_native_status(phase: PathingWaypointPhase) -> PathingNativePhaseStatus {
    match phase {
        PathingWaypointPhase::BeforeMoveToTarget
        | PathingWaypointPhase::BeforeMoveCloseToTarget => PathingNativePhaseStatus::ReadyByRuntime,
        _ => PathingNativePhaseStatus::Pending,
    }
}

fn pathing_phase_pending_dependencies(
    phase: PathingWaypointPhase,
    waypoint: &PathingWaypointPlan,
) -> Vec<PathingMovementDependency> {
    match phase {
        PathingWaypointPhase::RecoverWhenLowHp => vec![
            PathingMovementDependency::LowHpRecovery,
            PathingMovementDependency::PositionObservation,
        ],
        PathingWaypointPhase::HandleTeleport => vec![
            PathingMovementDependency::Teleport,
            PathingMovementDependency::MapMatching,
            PathingMovementDependency::CoordinateConversion,
        ]
        .into_iter()
        .filter(|dependency| {
            *dependency != PathingMovementDependency::CoordinateConversion
                || waypoint.track_conversion_pending
        })
        .collect(),
        PathingWaypointPhase::BeforeMoveToTarget
        | PathingWaypointPhase::BeforeMoveCloseToTarget => Vec::new(),
        PathingWaypointPhase::FaceTo => vec![
            PathingMovementDependency::CoordinateConversion,
            PathingMovementDependency::PositionObservation,
            PathingMovementDependency::CameraRotation,
        ]
        .into_iter()
        .filter(|dependency| {
            *dependency != PathingMovementDependency::CoordinateConversion
                || waypoint.track_conversion_pending
        })
        .collect(),
        PathingWaypointPhase::MoveTo | PathingWaypointPhase::MoveCloseTo => vec![
            PathingMovementDependency::CoordinateConversion,
            PathingMovementDependency::PositionObservation,
            PathingMovementDependency::InputDispatch,
            PathingMovementDependency::MovementTermination,
            PathingMovementDependency::TrapEscape,
        ]
        .into_iter()
        .filter(|dependency| {
            *dependency != PathingMovementDependency::CoordinateConversion
                || waypoint.track_conversion_pending
        })
        .collect(),
        PathingWaypointPhase::RunAction => pathing_action_pending_dependencies(waypoint),
    }
}

fn pathing_action_pending_dependencies(
    waypoint: &PathingWaypointPlan,
) -> Vec<PathingMovementDependency> {
    match waypoint.action_plan.as_ref() {
        Some(PathingActionPlan::SetTime(plan)) if plan.executor_ready => Vec::new(),
        Some(PathingActionPlan::LogOutput(plan)) if plan.executor_ready => Vec::new(),
        Some(PathingActionPlan::CommonJob(plan)) if plan.executor_ready => Vec::new(),
        Some(PathingActionPlan::ForceTeleport(plan)) if plan.executor_ready => Vec::new(),
        Some(PathingActionPlan::UseGadget(plan)) if plan.executor_ready => {
            vec![PathingMovementDependency::InputDispatch]
        }
        Some(PathingActionPlan::PickAround(plan)) if plan.executor_ready => {
            vec![PathingMovementDependency::InputDispatch]
        }
        Some(PathingActionPlan::NahidaCollect(_)) => vec![
            PathingMovementDependency::InputDispatch,
            PathingMovementDependency::ActionHandlers,
        ],
        Some(PathingActionPlan::ElementalCollect(_)) => vec![
            PathingMovementDependency::InputDispatch,
            PathingMovementDependency::ActionHandlers,
        ],
        Some(PathingActionPlan::LinneaMining(plan)) if plan.executor_ready => {
            vec![PathingMovementDependency::ActionHandlers]
        }
        _ => vec![PathingMovementDependency::ActionHandlers],
    }
}

impl PathingWaypointPlan {
    fn from_waypoint(
        segment_index: usize,
        segment_waypoint_index: usize,
        global_index: usize,
        waypoint: &Waypoint,
    ) -> Self {
        let action = normalized_action(waypoint.action.as_deref()).map(ToOwned::to_owned);
        let declared_action_use = action
            .as_deref()
            .and_then(declared_action_use_waypoint_type);
        let action_plan = action
            .as_deref()
            .and_then(|action| pathing_action_plan(action, waypoint.action_params.as_deref()));
        let effective_target_point = effective_target_point(waypoint, action.as_deref());
        let phases = waypoint_phases(waypoint, action.as_deref(), effective_target_point);

        Self {
            global_index,
            segment_index,
            segment_waypoint_index,
            x: waypoint.x,
            y: waypoint.y,
            route_point: PathingPoint {
                x: waypoint.x,
                y: waypoint.y,
            },
            track_point: None,
            track_conversion_pending: true,
            waypoint_type: waypoint.waypoint_type.clone(),
            move_mode: waypoint.move_mode.clone(),
            action,
            action_params: waypoint.action_params.clone(),
            declared_action_use,
            action_plan,
            effective_target_point,
            phases,
        }
    }
}

fn pathing_action_plan(action: &str, action_params: Option<&str>) -> Option<PathingActionPlan> {
    if action.eq_ignore_ascii_case("linnea_mining") {
        Some(PathingActionPlan::LinneaMining(Box::new(
            plan_linnea_mining_action(action_params),
        )))
    } else if action.eq_ignore_ascii_case("set_time") {
        Some(PathingActionPlan::SetTime(plan_set_time_action(
            action_params,
        )))
    } else if action.eq_ignore_ascii_case("log_output") {
        Some(PathingActionPlan::LogOutput(plan_log_output_action(
            action_params,
        )))
    } else if action.eq_ignore_ascii_case("exit_and_relogin") {
        Some(PathingActionPlan::CommonJob(
            plan_common_job_pathing_action("exit_and_relogin", action_params, "Relogin"),
        ))
    } else if action.eq_ignore_ascii_case("wonderland_cycle") {
        Some(PathingActionPlan::CommonJob(
            plan_common_job_pathing_action("wonderland_cycle", action_params, "WonderlandCycle"),
        ))
    } else if action.eq_ignore_ascii_case("force_tp") {
        Some(PathingActionPlan::ForceTeleport(
            plan_force_teleport_action(action_params),
        ))
    } else if action.eq_ignore_ascii_case("nahida_collect") {
        Some(PathingActionPlan::NahidaCollect(
            plan_nahida_collect_action(action_params),
        ))
    } else if let Some(element) = elemental_collect_element_from_action(action) {
        Some(PathingActionPlan::ElementalCollect(
            plan_elemental_collect_action(action, action_params, element),
        ))
    } else if action.eq_ignore_ascii_case("use_gadget") {
        Some(PathingActionPlan::UseGadget(plan_use_gadget_action(
            action_params,
        )))
    } else if action.eq_ignore_ascii_case("pick_around") {
        Some(PathingActionPlan::PickAround(plan_pick_around_action(
            action_params,
        )))
    } else {
        None
    }
}

const NAHIDA_COLLECT_AVATAR_NAME: &str = "纳西妲";
const NAHIDA_COLLECT_LOWER_VIEW_MOUSE_MOVE_Y: i32 = 10_000;
const NAHIDA_COLLECT_LOWER_VIEW_DELAY_MS: u32 = 200;
const NAHIDA_COLLECT_SKILL_HOLD_INITIAL_DELAY_MS: u32 = 200;
const NAHIDA_COLLECT_GROUND_SCAN_ITERATIONS: u8 = 15;
const NAHIDA_COLLECT_GROUND_SCAN_MOVE_X: i32 = 400;
const NAHIDA_COLLECT_GROUND_SCAN_MOVE_Y: i32 = 500;
const NAHIDA_COLLECT_RAISED_SCAN_ITERATIONS: u8 = 60;
const NAHIDA_COLLECT_RAISED_SCAN_MOVE_X: i32 = 400;
const NAHIDA_COLLECT_RAISED_SCAN_INITIAL_MOVE_Y: i32 = -30;
const NAHIDA_COLLECT_RAISED_SCAN_Y_ADJUST_BEFORE_ITERATION: u8 = 20;
const NAHIDA_COLLECT_RAISED_SCAN_Y_ADJUST_DELTA: i32 = -20;
const NAHIDA_COLLECT_SCAN_STEP_DELAY_MS: u32 = 30;
const NAHIDA_COLLECT_POST_SKILL_RELEASE_CD_UPDATE_DELAY_MS: u32 = 200;
const NAHIDA_COLLECT_AFTER_COLLECT_DELAY_MS: u32 = 800;
const NAHIDA_COLLECT_RESTORE_VIEW_DELAY_MS: u32 = 1_000;
const NAHIDA_COLLECT_PATH_EXECUTOR_AFTER_ACTION_DELAY_MS: u32 = 1_000;

fn plan_nahida_collect_action(action_params: Option<&str>) -> PathingNahidaCollectActionPlan {
    PathingNahidaCollectActionPlan {
        action_code: "nahida_collect".to_string(),
        raw_params: action_params.map(ToOwned::to_owned),
        avatar_name: NAHIDA_COLLECT_AVATAR_NAME.to_string(),
        requires_combat_scenes: true,
        switch_avatar_before_collect: true,
        wait_skill_cooldown_before_collect: true,
        update_skill_cooldown_after_collect: true,
        dpi_scale_required: true,
        lower_view_mouse_move_y: NAHIDA_COLLECT_LOWER_VIEW_MOUSE_MOVE_Y,
        lower_view_delay_ms: NAHIDA_COLLECT_LOWER_VIEW_DELAY_MS,
        elemental_skill_action: GenshinAction::ElementalSkill,
        skill_hold_initial_delay_ms: NAHIDA_COLLECT_SKILL_HOLD_INITIAL_DELAY_MS,
        ground_scan_iterations: NAHIDA_COLLECT_GROUND_SCAN_ITERATIONS,
        ground_scan_move_x: NAHIDA_COLLECT_GROUND_SCAN_MOVE_X,
        ground_scan_move_x_dpi_scaled: true,
        ground_scan_move_y: NAHIDA_COLLECT_GROUND_SCAN_MOVE_Y,
        ground_scan_move_y_dpi_scaled: false,
        raised_scan_iterations: NAHIDA_COLLECT_RAISED_SCAN_ITERATIONS,
        raised_scan_move_x: NAHIDA_COLLECT_RAISED_SCAN_MOVE_X,
        raised_scan_move_x_dpi_scaled: true,
        raised_scan_initial_move_y: NAHIDA_COLLECT_RAISED_SCAN_INITIAL_MOVE_Y,
        raised_scan_y_adjust_before_iteration:
            NAHIDA_COLLECT_RAISED_SCAN_Y_ADJUST_BEFORE_ITERATION,
        raised_scan_y_adjust_delta: NAHIDA_COLLECT_RAISED_SCAN_Y_ADJUST_DELTA,
        raised_scan_move_y_dpi_scaled: true,
        scan_step_delay_ms: NAHIDA_COLLECT_SCAN_STEP_DELAY_MS,
        post_skill_release_cd_update_delay_ms: NAHIDA_COLLECT_POST_SKILL_RELEASE_CD_UPDATE_DELAY_MS,
        after_collect_delay_ms: NAHIDA_COLLECT_AFTER_COLLECT_DELAY_MS,
        restore_view_key: KeyId::MOUSE_MIDDLE_BUTTON,
        restore_view_delay_ms: NAHIDA_COLLECT_RESTORE_VIEW_DELAY_MS,
        path_executor_after_action_delay_ms: NAHIDA_COLLECT_PATH_EXECUTOR_AFTER_ACTION_DELAY_MS,
        skill_release_in_finally: true,
        steps: build_nahida_collect_steps(),
        executor_ready: false,
        notes:
            "Pathing nahida_collect action is modeled as the legacy Nahida long-hold ElementalSkill scan sequence; live combat-scene avatar selection, skill-cooldown tracking, DPI-aware mouse dispatch, and sequence-safe input dispatch remain pending."
                .to_string(),
    }
}

fn build_nahida_collect_steps() -> Vec<PathingNahidaCollectStep> {
    let mut steps = vec![
        PathingNahidaCollectStep::MouseMoveBy {
            dx: 0,
            dy: NAHIDA_COLLECT_LOWER_VIEW_MOUSE_MOVE_Y,
            dx_dpi_scaled: false,
            dy_dpi_scaled: false,
        },
        PathingNahidaCollectStep::Delay {
            milliseconds: NAHIDA_COLLECT_LOWER_VIEW_DELAY_MS,
        },
        PathingNahidaCollectStep::GenshinAction {
            action: GenshinAction::ElementalSkill,
            press: PathingInputPress::KeyDown,
        },
        PathingNahidaCollectStep::Delay {
            milliseconds: NAHIDA_COLLECT_SKILL_HOLD_INITIAL_DELAY_MS,
        },
    ];

    for _ in 0..NAHIDA_COLLECT_GROUND_SCAN_ITERATIONS {
        steps.push(PathingNahidaCollectStep::MouseMoveBy {
            dx: NAHIDA_COLLECT_GROUND_SCAN_MOVE_X,
            dy: NAHIDA_COLLECT_GROUND_SCAN_MOVE_Y,
            dx_dpi_scaled: true,
            dy_dpi_scaled: false,
        });
        steps.push(PathingNahidaCollectStep::Delay {
            milliseconds: NAHIDA_COLLECT_SCAN_STEP_DELAY_MS,
        });
    }

    for iteration in 1..=NAHIDA_COLLECT_RAISED_SCAN_ITERATIONS {
        let dy = if iteration >= NAHIDA_COLLECT_RAISED_SCAN_Y_ADJUST_BEFORE_ITERATION {
            NAHIDA_COLLECT_RAISED_SCAN_INITIAL_MOVE_Y + NAHIDA_COLLECT_RAISED_SCAN_Y_ADJUST_DELTA
        } else {
            NAHIDA_COLLECT_RAISED_SCAN_INITIAL_MOVE_Y
        };
        steps.push(PathingNahidaCollectStep::MouseMoveBy {
            dx: NAHIDA_COLLECT_RAISED_SCAN_MOVE_X,
            dy,
            dx_dpi_scaled: true,
            dy_dpi_scaled: true,
        });
        steps.push(PathingNahidaCollectStep::Delay {
            milliseconds: NAHIDA_COLLECT_SCAN_STEP_DELAY_MS,
        });
    }

    steps.push(PathingNahidaCollectStep::GenshinAction {
        action: GenshinAction::ElementalSkill,
        press: PathingInputPress::KeyUp,
    });
    steps.push(PathingNahidaCollectStep::Delay {
        milliseconds: NAHIDA_COLLECT_POST_SKILL_RELEASE_CD_UPDATE_DELAY_MS,
    });
    steps.push(PathingNahidaCollectStep::Delay {
        milliseconds: NAHIDA_COLLECT_AFTER_COLLECT_DELAY_MS,
    });
    steps.push(PathingNahidaCollectStep::Key {
        key: KeyId::MOUSE_MIDDLE_BUTTON,
        press: PathingInputPress::KeyPress,
    });
    steps.push(PathingNahidaCollectStep::Delay {
        milliseconds: NAHIDA_COLLECT_RESTORE_VIEW_DELAY_MS,
    });
    steps
}

const ELEMENTAL_COLLECT_NORMAL_ATTACK_DURATION_MS: u32 = 100;
const ELEMENTAL_COLLECT_PATH_EXECUTOR_AFTER_ACTION_DELAY_MS: u32 = 1_000;

fn elemental_collect_element_from_action(action: &str) -> Option<PathingElementalCollectElement> {
    if action.eq_ignore_ascii_case("hydro_collect") {
        Some(PathingElementalCollectElement::Hydro)
    } else if action.eq_ignore_ascii_case("electro_collect") {
        Some(PathingElementalCollectElement::Electro)
    } else if action.eq_ignore_ascii_case("anemo_collect") {
        Some(PathingElementalCollectElement::Anemo)
    } else if action.eq_ignore_ascii_case("pyro_collect") {
        Some(PathingElementalCollectElement::Pyro)
    } else {
        None
    }
}

fn plan_elemental_collect_action(
    action_code: &str,
    action_params: Option<&str>,
    element: PathingElementalCollectElement,
) -> PathingElementalCollectActionPlan {
    PathingElementalCollectActionPlan {
        action_code: action_code.to_string(),
        raw_params: action_params.map(ToOwned::to_owned),
        element,
        element_chinese: elemental_collect_element_chinese(element).to_string(),
        requires_combat_scenes: true,
        legacy_preflight_validates_avatar: matches!(
            element,
            PathingElementalCollectElement::Hydro
                | PathingElementalCollectElement::Electro
                | PathingElementalCollectElement::Anemo
        ),
        scans_team_order: true,
        switch_avatar_before_collect: true,
        normal_attack_preferred_over_skill: true,
        normal_attack_duration_ms: ELEMENTAL_COLLECT_NORMAL_ATTACK_DURATION_MS,
        wait_skill_cooldown_before_skill: true,
        path_executor_after_action_delay_ms: ELEMENTAL_COLLECT_PATH_EXECUTOR_AFTER_ACTION_DELAY_MS,
        candidates: elemental_collect_candidates(element)
            .iter()
            .map(|candidate| PathingElementalCollectAvatarPlan {
                avatar_name: candidate.name.to_string(),
                normal_attack: candidate.normal_attack,
                elemental_skill: candidate.elemental_skill,
                selected_action: if candidate.normal_attack {
                    PathingElementalCollectAvatarAction::NormalAttack
                } else {
                    PathingElementalCollectAvatarAction::ElementalSkill
                },
                normal_attack_duration_ms: candidate
                    .normal_attack
                    .then_some(ELEMENTAL_COLLECT_NORMAL_ATTACK_DURATION_MS),
                waits_skill_cooldown: !candidate.normal_attack && candidate.elemental_skill,
            })
            .collect(),
        executor_ready: false,
        notes:
            "Pathing elemental collect action is modeled with the legacy element-specific avatar table and team-order selection rule; live combat-scene avatar selection, skill-cooldown tracking, and input dispatch remain pending."
                .to_string(),
    }
}

fn elemental_collect_element_chinese(element: PathingElementalCollectElement) -> &'static str {
    match element {
        PathingElementalCollectElement::Hydro => "水",
        PathingElementalCollectElement::Electro => "雷",
        PathingElementalCollectElement::Anemo => "风",
        PathingElementalCollectElement::Pyro => "火",
    }
}

#[derive(Debug, Clone, Copy)]
struct ElementalCollectCandidate {
    name: &'static str,
    normal_attack: bool,
    elemental_skill: bool,
}

const fn elemental_collect_candidate(
    name: &'static str,
    normal_attack: bool,
    elemental_skill: bool,
) -> ElementalCollectCandidate {
    ElementalCollectCandidate {
        name,
        normal_attack,
        elemental_skill,
    }
}

fn elemental_collect_candidates(
    element: PathingElementalCollectElement,
) -> &'static [ElementalCollectCandidate] {
    const HYDRO_CANDIDATES: [ElementalCollectCandidate; 10] = [
        elemental_collect_candidate("芭芭拉", true, true),
        elemental_collect_candidate("莫娜", true, false),
        elemental_collect_candidate("珊瑚宫心海", true, true),
        elemental_collect_candidate("玛拉妮", true, false),
        elemental_collect_candidate("那维莱特", true, true),
        elemental_collect_candidate("芙宁娜", true, false),
        elemental_collect_candidate("妮露", false, true),
        elemental_collect_candidate("坎蒂斯", false, true),
        elemental_collect_candidate("行秋", false, true),
        elemental_collect_candidate("神里绫人", false, true),
    ];
    const ELECTRO_CANDIDATES: [ElementalCollectCandidate; 8] = [
        elemental_collect_candidate("丽莎", true, true),
        elemental_collect_candidate("八重神子", true, false),
        elemental_collect_candidate("瓦雷莎", true, false),
        elemental_collect_candidate("雷电将军", false, true),
        elemental_collect_candidate("久岐忍", false, true),
        elemental_collect_candidate("北斗", false, true),
        elemental_collect_candidate("菲谢尔", false, true),
        elemental_collect_candidate("雷泽", false, true),
    ];
    const ANEMO_CANDIDATES: [ElementalCollectCandidate; 11] = [
        elemental_collect_candidate("砂糖", true, true),
        elemental_collect_candidate("鹿野院平藏", true, true),
        elemental_collect_candidate("流浪者", true, false),
        elemental_collect_candidate("闲云", true, false),
        elemental_collect_candidate("蓝砚", true, false),
        elemental_collect_candidate("枫原万叶", false, true),
        elemental_collect_candidate("珐露珊", false, true),
        elemental_collect_candidate("琳妮特", false, true),
        elemental_collect_candidate("温迪", false, true),
        elemental_collect_candidate("琴", false, true),
        elemental_collect_candidate("早柚", false, true),
    ];
    const PYRO_CANDIDATES: [ElementalCollectCandidate; 12] = [
        elemental_collect_candidate("烟绯", true, true),
        elemental_collect_candidate("迪卢克", false, true),
        elemental_collect_candidate("可莉", true, true),
        elemental_collect_candidate("班尼特", false, true),
        elemental_collect_candidate("香菱", false, true),
        elemental_collect_candidate("托马", false, true),
        elemental_collect_candidate("胡桃", false, true),
        elemental_collect_candidate("迪希雅", false, true),
        elemental_collect_candidate("夏沃蕾", false, true),
        elemental_collect_candidate("辛焱", false, true),
        elemental_collect_candidate("林尼", false, true),
        elemental_collect_candidate("宵宫", false, true),
    ];

    match element {
        PathingElementalCollectElement::Hydro => &HYDRO_CANDIDATES,
        PathingElementalCollectElement::Electro => &ELECTRO_CANDIDATES,
        PathingElementalCollectElement::Anemo => &ANEMO_CANDIDATES,
        PathingElementalCollectElement::Pyro => &PYRO_CANDIDATES,
    }
}

fn plan_set_time_action(action_params: Option<&str>) -> PathingSetTimeActionPlan {
    let raw_params = action_params.map(ToOwned::to_owned);
    let mut plan = PathingSetTimeActionPlan {
        action_code: "set_time".to_string(),
        raw_params,
        common_job_task_key: "SetTime".to_string(),
        hour: None,
        minute: None,
        skip_time_adjustment_animation: None,
        parse_error: None,
        executor_ready: false,
        notes: "Pathing set_time action is parsed into the SetTime common-job executor contract."
            .to_string(),
    };

    let Some(action_params) = action_params
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        plan.parse_error = Some("set_time action parameters are empty".to_string());
        return plan;
    };
    let parts = action_params.split(':').collect::<Vec<_>>();
    if parts.len() < 2 {
        plan.parse_error = Some("set_time action parameters must be HH:mm[:skip]".to_string());
        return plan;
    }

    let hour = match parts[0].trim().parse::<i32>() {
        Ok(hour) => hour,
        Err(_) => {
            plan.parse_error = Some(format!("invalid set_time hour: {}", parts[0].trim()));
            return plan;
        }
    };
    let minute = match parts[1].trim().parse::<i32>() {
        Ok(minute) => minute,
        Err(_) => {
            plan.parse_error = Some(format!("invalid set_time minute: {}", parts[1].trim()));
            return plan;
        }
    };
    let skip_time_adjustment_animation = if parts.len() < 3 {
        true
    } else {
        match parse_legacy_bool(parts[2].trim()) {
            Some(value) => value,
            None => {
                plan.parse_error = Some(format!("invalid set_time skip flag: {}", parts[2].trim()));
                return plan;
            }
        }
    };

    plan.hour = Some(hour);
    plan.minute = Some(minute);
    plan.skip_time_adjustment_animation = Some(skip_time_adjustment_animation);
    plan.executor_ready = true;
    plan
}

fn plan_log_output_action(action_params: Option<&str>) -> PathingLogOutputActionPlan {
    PathingLogOutputActionPlan {
        action_code: "log_output".to_string(),
        raw_params: action_params.map(ToOwned::to_owned),
        message: action_params.unwrap_or_default().to_string(),
        executor_ready: true,
        notes: "Pathing log_output action is preserved as a Rust action-boundary report."
            .to_string(),
    }
}

fn plan_common_job_pathing_action(
    action_code: &str,
    action_params: Option<&str>,
    common_job_task_key: &str,
) -> PathingCommonJobActionPlan {
    PathingCommonJobActionPlan {
        action_code: action_code.to_string(),
        raw_params: action_params.map(ToOwned::to_owned),
        common_job_task_key: common_job_task_key.to_string(),
        executor_ready: true,
        notes: format!(
            "Pathing {action_code} action is mapped to the {common_job_task_key} common-job executor contract; legacy action_params are preserved but not consumed by this handler."
        ),
    }
}

fn plan_force_teleport_action(action_params: Option<&str>) -> PathingForceTeleportActionPlan {
    PathingForceTeleportActionPlan {
        action_code: "force_tp".to_string(),
        raw_params: action_params.map(ToOwned::to_owned),
        force_teleport: true,
        executor_ready: false,
        notes: "Pathing force_tp action is represented as a force-teleport intent for the HandleTeleport phase; native TpTask dispatch and final navigation state updates remain pending."
            .to_string(),
    }
}

fn plan_use_gadget_action(action_params: Option<&str>) -> PathingUseGadgetActionPlan {
    let raw_params = action_params.map(ToOwned::to_owned);
    let trimmed_params = action_params.map(str::trim).unwrap_or_default();
    let not_wait = trimmed_params.to_ascii_lowercase().contains("not_wait");
    let cooldown_ocr_required = !not_wait;
    let (max_wait_seconds, max_wait_parse_error) = if !cooldown_ocr_required {
        (None, None)
    } else if trimmed_params.is_empty() {
        (Some(100.0), None)
    } else {
        match trimmed_params.parse::<f64>() {
            Ok(value) => (Some(value), None),
            Err(_) => (
                Some(0.0),
                Some(format!(
                    "use_gadget max wait parameter is not numeric and falls back to 0 seconds like legacy double.TryParse: {trimmed_params}"
                )),
            ),
        }
    };

    PathingUseGadgetActionPlan {
        action_code: "use_gadget".to_string(),
        raw_params,
        genshin_action: GenshinAction::QuickUseGadget,
        not_wait,
        cooldown_ocr_required,
        max_wait_seconds,
        max_wait_parse_error,
        cooldown_overlong_skip_threshold_seconds: 100.0,
        cooldown_wait_padding_ms: 100,
        quick_use_gadget_press_count: 2,
        handler_delay_ms: 300,
        path_executor_after_action_delay_ms: 1_000,
        executor_ready: not_wait,
        notes: if not_wait {
            "Pathing use_gadget not_wait branch is modeled as two QuickUseGadget presses followed by the legacy handler and PathExecutor delays; cooldown OCR is skipped."
        } else {
            "Pathing use_gadget default branch is modeled with legacy cooldown OCR wait rules, but live execution remains pending until the cooldown OCR probe is migrated."
        }
        .to_string(),
    }
}

const PICK_AROUND_SPEED: f64 = 1.1;
const PICK_AROUND_CIRCLE_SEGMENTS_PER_TURN: u8 = 6;
const PICK_AROUND_CIRCLE_START_MS: u32 = 600;
const PICK_AROUND_CIRCLE_INTERVAL_MS: u32 = 400;
const PICK_AROUND_CIRCLE_TIME_MS: f64 = 33_000.0;
const PICK_AROUND_VIEW_RESET_BASE_MS: u32 = 350;

fn plan_pick_around_action(action_params: Option<&str>) -> PathingPickAroundActionPlan {
    let raw_params = action_params.map(ToOwned::to_owned);
    let (turns, turn_parse_error) = parse_pick_around_turns(action_params);
    let (turn_plans, steps) = build_pick_around_turns(turns, PICK_AROUND_SPEED);

    PathingPickAroundActionPlan {
        action_code: "pick_around".to_string(),
        raw_params,
        turns,
        turn_parse_error,
        speed: PICK_AROUND_SPEED,
        circle_segments_per_turn: PICK_AROUND_CIRCLE_SEGMENTS_PER_TURN,
        circle_start_ms: PICK_AROUND_CIRCLE_START_MS,
        circle_interval_ms: PICK_AROUND_CIRCLE_INTERVAL_MS,
        circle_time_ms: PICK_AROUND_CIRCLE_TIME_MS,
        view_reset_base_ms: PICK_AROUND_VIEW_RESET_BASE_MS,
        turn_plans,
        steps,
        executor_ready: true,
        notes:
            "Pathing pick_around action is modeled as the legacy circular middle-click pickup movement sequence; sequence-safe desktop input dispatch remains pending."
                .to_string(),
    }
}

fn parse_pick_around_turns(action_params: Option<&str>) -> (i32, Option<String>) {
    let Some(raw_params) = action_params else {
        return (1, None);
    };
    let trimmed = raw_params.trim();
    if trimmed.is_empty() {
        return (1, None);
    }
    match trimmed.parse::<i32>() {
        Ok(turns) => (turns, None),
        Err(_) => (
            1,
            Some(format!(
                "pick_around turns parameter is not an integer and falls back to 1 like legacy int.TryParse: {trimmed}"
            )),
        ),
    }
}

fn build_pick_around_turns(
    turns: i32,
    speed: f64,
) -> (Vec<PathingPickAroundTurnPlan>, Vec<PathingPickAroundStep>) {
    let mut turn_plans = Vec::new();
    let mut steps = Vec::new();
    if turns <= 0 {
        return (turn_plans, steps);
    }

    let calculator = PickAroundCircularMotionCalculator::new(speed);
    let mut old_radius_time_ms = 0.0_f64;
    let mut angle = 0.0_f64;
    for turn_index in 0..turns as u32 {
        let circle = calculator.circle_info(turn_index);
        let move_left_time_ms = circle.radius_time_ms - old_radius_time_ms * angle.cos();
        let move_backward_time_ms = old_radius_time_ms * angle.sin();
        let move_backward_forward_ms = round_midpoint_to_even_i32(move_backward_time_ms) + 200;
        let move_left_forward_ms = round_midpoint_to_even_i32(move_left_time_ms);
        turn_plans.push(PathingPickAroundTurnPlan {
            turn_index,
            edge_delay_ms: round_midpoint_to_even_u32(circle.edge_time_ms),
            radius_time_ms: circle.radius_time_ms,
            end_angle_radians: circle.end_angle_radians,
            move_backward_forward_ms,
            move_left_forward_ms,
        });
        push_pick_around_move_to_next_start_point(
            &mut steps,
            turn_index,
            move_backward_forward_ms,
            move_left_forward_ms,
        );
        push_pick_around_move_circle(
            &mut steps,
            turn_index,
            round_midpoint_to_even_u32(circle.edge_time_ms),
        );
        old_radius_time_ms = circle.radius_time_ms;
        angle = circle.end_angle_radians;
    }

    (turn_plans, steps)
}

#[derive(Debug, Clone, Copy)]
struct PickAroundCircleInfo {
    edge_time_ms: f64,
    radius_time_ms: f64,
    end_angle_radians: f64,
}

struct PickAroundCircularMotionCalculator {
    speed: f64,
    mix_angle: f64,
    mix_x: f64,
    mix_y: f64,
}

impl PickAroundCircularMotionCalculator {
    fn new(speed: f64) -> Self {
        let view_reset_time_ms = PICK_AROUND_VIEW_RESET_BASE_MS as f64 * speed;
        let mix_angle = (view_reset_time_ms / PICK_AROUND_CIRCLE_TIME_MS + 1.0 / 4.0)
            * 2.0
            * std::f64::consts::PI;
        let (mix_x, mix_y) = Self::arc_point(view_reset_time_ms / mix_angle, mix_angle);
        Self {
            speed,
            mix_angle,
            mix_x,
            mix_y,
        }
    }

    fn circle_info(&self, index: u32) -> PickAroundCircleInfo {
        let edge_time_ms = PICK_AROUND_CIRCLE_START_MS as f64
            + index as f64 * PICK_AROUND_CIRCLE_INTERVAL_MS as f64;
        let angle = (edge_time_ms / PICK_AROUND_CIRCLE_TIME_MS + 1.0 / 4.0) * std::f64::consts::PI;
        let radius_time = PICK_AROUND_CIRCLE_TIME_MS / (2.0 * std::f64::consts::PI);
        let (rest_x, rest_y) = Self::arc_point(radius_time, 2.0 * angle - self.mix_angle);
        let x = self.mix_x - rest_x;
        let y = self.mix_y + rest_y;
        let small_radius_time = (x.powi(2) + y.powi(2)).sqrt() / (2.0 * angle.sin());
        let end_angle = angle - self.mix_angle + x.atan2(y) + std::f64::consts::PI / 2.0;
        PickAroundCircleInfo {
            edge_time_ms: edge_time_ms / self.speed,
            radius_time_ms: small_radius_time / self.speed,
            end_angle_radians: end_angle,
        }
    }

    fn arc_point(radius: f64, angle: f64) -> (f64, f64) {
        (radius * (1.0 - angle.cos()), radius * angle.sin())
    }
}

fn push_pick_around_move_to_next_start_point(
    steps: &mut Vec<PathingPickAroundStep>,
    turn_index: u32,
    move_backward_forward_ms: i32,
    move_left_forward_ms: i32,
) {
    push_pick_around_middle_click(steps, turn_index);
    push_pick_around_delay(steps, turn_index, 500);
    push_pick_around_move_after_turn(
        steps,
        turn_index,
        GenshinAction::MoveBackward,
        move_backward_forward_ms,
    );
    push_pick_around_move_after_turn(
        steps,
        turn_index,
        GenshinAction::MoveLeft,
        move_left_forward_ms,
    );
}

fn push_pick_around_move_after_turn(
    steps: &mut Vec<PathingPickAroundStep>,
    turn_index: u32,
    action: GenshinAction,
    forward_ms: i32,
) {
    push_pick_around_action(steps, turn_index, action, PathingInputPress::KeyPress);
    push_pick_around_delay(steps, turn_index, 200);
    push_pick_around_middle_click(steps, turn_index);
    push_pick_around_delay(steps, turn_index, 500);
    if forward_ms > 0 {
        push_pick_around_action(
            steps,
            turn_index,
            GenshinAction::MoveForward,
            PathingInputPress::KeyDown,
        );
        push_pick_around_delay(steps, turn_index, forward_ms as u32);
        push_pick_around_action(
            steps,
            turn_index,
            GenshinAction::MoveForward,
            PathingInputPress::KeyUp,
        );
        push_pick_around_delay(steps, turn_index, 200);
    }
}

fn push_pick_around_move_circle(
    steps: &mut Vec<PathingPickAroundStep>,
    turn_index: u32,
    edge_delay_ms: u32,
) {
    push_pick_around_action(
        steps,
        turn_index,
        GenshinAction::MoveLeft,
        PathingInputPress::KeyDown,
    );
    push_pick_around_delay(steps, turn_index, 30);
    for _ in 0..PICK_AROUND_CIRCLE_SEGMENTS_PER_TURN {
        push_pick_around_middle_click(steps, turn_index);
        push_pick_around_delay(steps, turn_index, edge_delay_ms);
    }
    push_pick_around_action(
        steps,
        turn_index,
        GenshinAction::MoveLeft,
        PathingInputPress::KeyUp,
    );
    push_pick_around_delay(steps, turn_index, 200);
}

fn push_pick_around_middle_click(steps: &mut Vec<PathingPickAroundStep>, turn_index: u32) {
    steps.push(PathingPickAroundStep::Key {
        turn_index,
        key: KeyId::MOUSE_MIDDLE_BUTTON,
        press: PathingInputPress::KeyPress,
    });
}

fn push_pick_around_action(
    steps: &mut Vec<PathingPickAroundStep>,
    turn_index: u32,
    action: GenshinAction,
    press: PathingInputPress,
) {
    steps.push(PathingPickAroundStep::GenshinAction {
        turn_index,
        action,
        press,
    });
}

fn push_pick_around_delay(
    steps: &mut Vec<PathingPickAroundStep>,
    turn_index: u32,
    milliseconds: u32,
) {
    steps.push(PathingPickAroundStep::Delay {
        turn_index,
        milliseconds,
    });
}

fn round_midpoint_to_even_u32(value: f64) -> u32 {
    round_midpoint_to_even_i32(value).max(0) as u32
}

fn round_midpoint_to_even_i32(value: f64) -> i32 {
    if !value.is_finite() {
        return 0;
    }
    let floor = value.floor();
    let fraction = value - floor;
    let rounded = if (fraction - 0.5).abs() < f64::EPSILON {
        if (floor as i64).rem_euclid(2) == 0 {
            floor
        } else {
            floor + 1.0
        }
    } else {
        value.round()
    };
    rounded as i32
}

fn parse_legacy_bool(value: &str) -> Option<bool> {
    if value.eq_ignore_ascii_case("true") {
        Some(true)
    } else if value.eq_ignore_ascii_case("false") {
        Some(false)
    } else {
        None
    }
}

pub fn plan_linnea_mining_action(action_params: Option<&str>) -> LinneaMiningActionPlan {
    let (mine_count, scan_rounds) = parse_linnea_mining_params(action_params);
    LinneaMiningActionPlan {
        action_code: "linnea_mining".to_string(),
        raw_params: action_params.map(ToOwned::to_owned),
        mine_count,
        scan_rounds,
        prefer_right: scan_rounds > 1,
        avatar_name: "莉奈娅".to_string(),
        switch_avatar_before_mining: true,
        switch_wait_ms: 500,
        aiming_mode_action: GenshinAction::SwitchAimingMode,
        enter_aim_wait_ms: 400,
        detection_rule: LinneaMiningDetectionRule {
            model_name: "BgiMine".to_string(),
            model_relative_path: "Assets/Model/Mine/bgi_mine.onnx".to_string(),
            accepted_label: "ore".to_string(),
            confidence_threshold: 0.70,
            source: LinneaMiningDetectionSource::FullCapture,
        },
        cluster_rule: LinneaMiningClusterRule {
            base_cluster_distance_1080p: 400.0,
            base_cluster_area_1080p: 1_800.0,
            base_alignment_expansion_1080p: 3.0,
            base_edge_ignore_1080p: 200.0,
            area_ratio_threshold: 4.0,
            prefer_right_when_scan_rounds_gt_one: true,
            target_selection:
                "nearest cluster to screen center; when prefer_right and cluster has >=2 rects, choose the rightmost of the two nearest rects"
                    .to_string(),
        },
        alignment_rule: LinneaMiningAlignmentRule {
            max_inner_retry: 7,
            element_sight_refresh_ms: 3_000,
            refresh_release_ms: 100,
            refresh_hold_ms: 1_500,
            aim_sensitivity_factor_x: 0.45,
            aim_sensitivity_factor_y: 0.80,
            aim_move_delay_ms: 150,
            fallback_shot_on_last_successful_detection: true,
        },
        scan_rule: LinneaMiningScanRule {
            middle_button_hold_ms: 1_500,
            middle_button_release_ms: 300,
            compensate_detection_hold_ms: 1_500,
            compensate_move_wait_ms: 800,
            left_turn_step_1080p: -250,
            left_turn_wait_ms: 800,
        },
        mine_rule: LinneaMiningMineRule {
            compensate_up_pixels: -25,
            compensate_up_wait_ms: 10,
            attack_button: "LeftMouse".to_string(),
            after_attack_wait_ms: 2_000,
        },
        cleanup_rule: LinneaMiningCleanupRule {
            leave_aiming_mode_action: GenshinAction::SwitchAimingMode,
            middle_button_up: true,
            clear_vision_drawings: true,
        },
        executor_ready: false,
        notes:
            "Linnea mining action parameters, avatar requirement, YOLO model, clustering, aiming, scan, mining, and cleanup are modeled; live avatar switching, capture, ONNX inference, mouse input, and overlay execution remain pending."
                .to_string(),
    }
}

fn parse_linnea_mining_params(action_params: Option<&str>) -> (i32, i32) {
    let mut mine_count = -1;
    let mut scan_rounds = -1;

    if let Some(action_params) = action_params {
        for part in action_params.split(',') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(value) = trimmed.strip_prefix("mines=") {
                if let Ok(value) = value.parse::<i32>() {
                    mine_count = clamp_linnea_mining_count(value);
                }
            } else if let Some(value) = trimmed.strip_prefix("rounds=") {
                if let Ok(value) = value.parse::<i32>() {
                    scan_rounds = clamp_linnea_mining_count(value);
                }
            } else if trimmed.len() >= "mines=".len()
                && trimmed[.."mines=".len()].eq_ignore_ascii_case("mines=")
            {
                if let Ok(value) = trimmed["mines=".len()..].parse::<i32>() {
                    mine_count = clamp_linnea_mining_count(value);
                }
            } else if trimmed.len() >= "rounds=".len()
                && trimmed[.."rounds=".len()].eq_ignore_ascii_case("rounds=")
            {
                if let Ok(value) = trimmed["rounds=".len()..].parse::<i32>() {
                    scan_rounds = clamp_linnea_mining_count(value);
                }
            } else if let Ok(value) = trimmed.parse::<i32>() {
                if mine_count == -1 {
                    mine_count = clamp_linnea_mining_count(value);
                } else if scan_rounds == -1 {
                    scan_rounds = clamp_linnea_mining_count(value);
                }
            }
        }
    }

    if mine_count == -1 {
        mine_count = 1;
    }
    if scan_rounds == -1 {
        scan_rounds = 1;
    }
    if scan_rounds < mine_count {
        scan_rounds = mine_count;
    }

    (mine_count, scan_rounds)
}

fn clamp_linnea_mining_count(value: i32) -> i32 {
    if value <= 0 {
        1
    } else if value > 999 {
        999
    } else {
        value
    }
}

fn waypoint_phases(
    waypoint: &Waypoint,
    action: Option<&str>,
    effective_target_point: bool,
) -> Vec<PathingWaypointPhase> {
    let mut phases = vec![PathingWaypointPhase::RecoverWhenLowHp];
    if waypoint_type_eq(&waypoint.waypoint_type, "teleport") {
        phases.push(PathingWaypointPhase::HandleTeleport);
        return phases;
    }

    phases.push(PathingWaypointPhase::BeforeMoveToTarget);
    if waypoint_type_eq(&waypoint.waypoint_type, "orientation") {
        phases.push(PathingWaypointPhase::FaceTo);
    } else if !action_eq(action, "up_down_grab_leaf") {
        phases.push(PathingWaypointPhase::MoveTo);
    }

    phases.push(PathingWaypointPhase::BeforeMoveCloseToTarget);
    if effective_target_point {
        phases.push(PathingWaypointPhase::MoveCloseTo);
    }
    if action.is_some() {
        phases.push(PathingWaypointPhase::RunAction);
    }
    phases
}

fn effective_target_point(waypoint: &Waypoint, action: Option<&str>) -> bool {
    if waypoint_type_eq(&waypoint.waypoint_type, "orientation")
        || action_eq(action, "up_down_grab_leaf")
    {
        return false;
    }

    // Legacy ActionEnum.GetEnumByCode currently enumerates only stop_flying, so other declared
    // action target/path overrides are metadata until the old behavior is intentionally fixed.
    if let Some(action_use) = legacy_action_use_waypoint_type(action) {
        if action_use != PathingActionUseWaypointType::Custom {
            return action_use == PathingActionUseWaypointType::Target;
        }
    }

    waypoint_type_eq(&waypoint.waypoint_type, "target")
}

fn declared_action_use_waypoint_type(action: &str) -> Option<PathingActionUseWaypointType> {
    match action {
        "fight" => Some(PathingActionUseWaypointType::Path),
        "hydro_collect" | "electro_collect" | "anemo_collect" | "pyro_collect" => {
            Some(PathingActionUseWaypointType::Target)
        }
        "stop_flying" | "force_tp" | "nahida_collect" | "pick_around" | "up_down_grab_leaf"
        | "combat_script" | "mining" | "linnea_mining" | "log_output" | "fishing"
        | "exit_and_relogin" | "wonderland_cycle" | "set_time" | "use_gadget"
        | "pick_up_collect" => Some(PathingActionUseWaypointType::Custom),
        _ => None,
    }
}

fn legacy_action_use_waypoint_type(action: Option<&str>) -> Option<PathingActionUseWaypointType> {
    match action? {
        "stop_flying" => Some(PathingActionUseWaypointType::Custom),
        _ => None,
    }
}

fn realtime_trigger_enabled(triggers: &Map<String, Value>, name: &str) -> bool {
    triggers.get(name).and_then(Value::as_bool).unwrap_or(false)
}

fn normalized_action(action: Option<&str>) -> Option<&str> {
    let action = action?.trim();
    if action.is_empty() {
        None
    } else {
        Some(action)
    }
}

fn has_non_empty_action(action: Option<&str>) -> bool {
    normalized_action(action).is_some()
}

fn action_eq(action: Option<&str>, expected: &str) -> bool {
    normalized_action(action)
        .map(|action| action.eq_ignore_ascii_case(expected))
        .unwrap_or(false)
}

fn waypoint_type_eq(actual: &str, expected: &str) -> bool {
    actual.eq_ignore_ascii_case(expected)
}

fn value_bool_alias(value: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_bool))
}

fn value_f64_alias(value: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_f64))
}

fn value_string_alias(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(ToOwned::to_owned)
}
