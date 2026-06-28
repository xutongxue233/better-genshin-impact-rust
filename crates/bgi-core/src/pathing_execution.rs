use super::{PathingSummary, PathingTask, Waypoint};
use crate::GenshinAction;
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
    pub farming: PathingFarmingExecutionPlan,
    pub segments: Vec<PathingSegmentPlan>,
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PathingPoint {
    pub x: f64,
    pub y: f64,
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

impl PathingTask {
    pub fn execution_plan(&self) -> PathingExecutionPlan {
        let segments = split_waypoints_for_track(&self.positions);
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
            preflight: PathingPreflightPlan::for_task(self),
            farming: PathingFarmingExecutionPlan::from_task(self, expected_fight_count),
            segments,
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
    } else {
        None
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
