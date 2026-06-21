use crate::error::{BgiError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const CONTROL_FILE_NAME: &str = "control.json5";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct PathingTask {
    #[serde(skip)]
    pub file_name: Option<String>,
    #[serde(skip)]
    pub full_path: Option<PathBuf>,
    #[serde(alias = "Info")]
    pub info: PathingTaskInfo,
    #[serde(alias = "Config")]
    pub config: PathingTaskConfig,
    #[serde(alias = "FarmingInfo")]
    pub farming_info: Value,
    #[serde(alias = "Positions")]
    pub positions: Vec<Waypoint>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for PathingTask {
    fn default() -> Self {
        Self {
            file_name: None,
            full_path: None,
            info: PathingTaskInfo::default(),
            config: PathingTaskConfig::default(),
            farming_info: Value::Object(Map::new()),
            positions: Vec::new(),
            extra: Map::new(),
        }
    }
}

impl PathingTask {
    pub fn from_json(text: &str) -> Result<Self> {
        json5::from_str(text).map_err(|err| BgiError::json(None::<PathBuf>, err.to_string()))
    }

    pub fn has_action(&self, action_name: &str) -> bool {
        self.positions.iter().any(|waypoint| {
            waypoint
                .action
                .as_deref()
                .map(|action| action.eq_ignore_ascii_case(action_name))
                .unwrap_or(false)
        })
    }

    pub fn action_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        let mut seen = HashSet::new();

        for waypoint in &self.positions {
            let Some(action) = waypoint.action.as_deref() else {
                continue;
            };
            let action = action.trim();
            if action.is_empty() {
                continue;
            }
            if seen.insert(action.to_string()) {
                names.push(action.to_string());
            }
        }

        names
    }

    pub fn summary(&self) -> PathingSummary {
        PathingSummary {
            name: self.info.name.clone(),
            task_type: self.info.task_type.clone(),
            type_description: pathing_task_type_description(&self.info.task_type).to_string(),
            map_name: self.info.map_name.clone(),
            waypoint_count: self.positions.len(),
            actions: self.action_names(),
            realtime_triggers: self
                .config
                .realtime_triggers
                .iter()
                .filter_map(|(name, enabled)| match enabled.as_bool() {
                    Some(true) => Some(name.clone()),
                    _ => None,
                })
                .collect(),
        }
    }

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct PathingTaskInfo {
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub bgi_version: Option<String>,
    #[serde(rename = "type")]
    pub task_type: String,
    pub order: i32,
    pub tags: Vec<String>,
    pub enable_monster_loot_split: bool,
    pub map_name: String,
    pub map_match_method: Option<String>,
    pub items: Vec<MaterialInfo>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for PathingTaskInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            author: None,
            version: None,
            bgi_version: None,
            task_type: String::new(),
            order: 0,
            tags: Vec::new(),
            enable_monster_loot_split: false,
            map_name: "Teyvat".to_string(),
            map_match_method: None,
            items: Vec::new(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct PathingTaskConfig {
    pub realtime_triggers: Map<String, Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for PathingTaskConfig {
    fn default() -> Self {
        let mut realtime_triggers = Map::new();
        realtime_triggers.insert("AutoPick".to_string(), Value::Bool(true));
        Self {
            realtime_triggers,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct Waypoint {
    pub x: f64,
    pub y: f64,
    pub point_ext_params: ExtParams,
    #[serde(rename = "type")]
    pub waypoint_type: String,
    pub move_mode: String,
    pub action: Option<String>,
    pub action_params: Option<String>,
    pub items: Vec<MaterialInfo>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Waypoint {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            point_ext_params: ExtParams::default(),
            waypoint_type: "path".to_string(),
            move_mode: "walk".to_string(),
            action: None,
            action_params: None,
            items: Vec::new(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct ExtParams {
    pub misidentification: Misidentification,
    pub description: String,
    pub monster_tag: Option<String>,
    pub enable_monster_loot_split: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for ExtParams {
    fn default() -> Self {
        Self {
            misidentification: Misidentification::default(),
            description: String::new(),
            monster_tag: None,
            enable_monster_loot_split: false,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct Misidentification {
    #[serde(rename = "type")]
    pub kind: Vec<String>,
    pub handling_mode: String,
    pub arrival_time: i32,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Misidentification {
    fn default() -> Self {
        Self {
            kind: vec!["unrecognized".to_string()],
            handling_mode: "previousDetectedPoint".to_string(),
            arrival_time: 0,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct MaterialInfo {
    pub monster: Option<String>,
    pub material: Option<String>,
    pub count: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathingSummary {
    pub name: String,
    pub task_type: String,
    pub type_description: String,
    pub map_name: String,
    pub waypoint_count: usize,
    pub actions: Vec<String>,
    pub realtime_triggers: Vec<String>,
}

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
    pub require_16_by_9_resolution: bool,
    pub minimum_width: u32,
    pub minimum_height: u32,
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
            require_16_by_9_resolution: has_positions,
            minimum_width: 1920,
            minimum_height: 1080,
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
    pub resolves_anomalies_before_attempt: bool,
    pub retry_times: u8,
    pub waypoints: Vec<PathingWaypointPlan>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingWaypointPlan {
    pub global_index: usize,
    pub segment_index: usize,
    pub segment_waypoint_index: usize,
    pub x: f64,
    pub y: f64,
    pub waypoint_type: String,
    pub move_mode: String,
    pub action: Option<String>,
    pub action_params: Option<String>,
    pub declared_action_use: Option<PathingActionUseWaypointType>,
    pub effective_target_point: bool,
    pub phases: Vec<PathingWaypointPhase>,
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
        let effective_target_point = effective_target_point(waypoint, action.as_deref());
        let phases = waypoint_phases(waypoint, action.as_deref(), effective_target_point);

        Self {
            global_index,
            segment_index,
            segment_waypoint_index,
            x: waypoint.x,
            y: waypoint.y,
            waypoint_type: waypoint.waypoint_type.clone(),
            move_mode: waypoint.move_mode.clone(),
            action,
            action_params: waypoint.action_params.clone(),
            declared_action_use,
            effective_target_point,
            phases,
        }
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

pub fn read_pathing_task(path: impl AsRef<Path>) -> Result<PathingTask> {
    let path = path.as_ref();
    let text = read_pathing_json_with_control(path)?;
    let mut task: PathingTask =
        json5::from_str(&text).map_err(|err| BgiError::json(Some(path), err.to_string()))?;
    task.file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned);
    task.full_path = Some(path.to_path_buf());
    Ok(task)
}

pub fn read_pathing_json_with_control(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let original_text = fs::read_to_string(path).map_err(|source| BgiError::io(path, source))?;
    let Some(parent) = path.parent() else {
        return Ok(original_text);
    };

    let control_path = parent.join(CONTROL_FILE_NAME);
    if !control_path.exists() {
        return Ok(original_text);
    }

    let control = read_control_json(&control_path)?;
    let mut original: Value = json5::from_str(&original_text)
        .map_err(|err| BgiError::json(Some(path), err.to_string()))?;
    let name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    merge_pathing_json(&control, &mut original, name);
    serde_json::to_string_pretty(&original)
        .map_err(|err| BgiError::json(Some(path), err.to_string()))
}

fn read_control_json(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path).map_err(|source| BgiError::io(path, source))?;
    let value: Value =
        json5::from_str(&text).map_err(|err| BgiError::json(Some(path), err.to_string()))?;

    if let Some(reference) = value.get("ref").and_then(Value::as_str) {
        let referenced = path
            .parent()
            .map(|parent| parent.join(reference))
            .unwrap_or_else(|| PathBuf::from(reference));
        return read_control_json(&referenced);
    }

    Ok(value)
}

fn merge_pathing_json(control: &Value, original: &mut Value, name: &str) {
    if let Some(global_cover) = control.get("global_cover") {
        merge_value(global_cover, original);
    }

    let Some(json_list) = control.get("json_list").and_then(Value::as_array) else {
        return;
    };

    for item in json_list {
        let item_name = item.get("name").and_then(Value::as_str);
        if item_name != Some(name) {
            continue;
        }

        if let Some(cover) = item.get("cover") {
            merge_value(cover, original);
        }
        break;
    }
}

fn merge_value(control: &Value, target: &mut Value) {
    let Value::Object(control) = control else {
        *target = control.clone();
        return;
    };

    if !target.is_object() {
        *target = Value::Object(control.clone());
    }

    let Some(target) = target.as_object_mut() else {
        return;
    };

    let mut skip_keys = HashSet::new();
    process_special_instructions(control, target, &mut skip_keys);

    for (key, control_value) in control {
        if skip_keys.contains(key) {
            continue;
        }

        if control_value.is_object() {
            if let Some(Value::Object(_)) = target.get(key) {
                if let Some(target_value) = target.get_mut(key) {
                    merge_value(control_value, target_value);
                    continue;
                }
            }
        }

        target.insert(key.clone(), control_value.clone());
    }
}

fn process_special_instructions(
    control: &Map<String, Value>,
    target: &mut Map<String, Value>,
    skip_keys: &mut HashSet<String>,
) {
    if let Some(Value::Array(keys)) = control.get("_obj_cover") {
        for item in keys {
            let Some(prop_name) = item.as_str() else {
                continue;
            };
            if let Some(value) = control.get(prop_name) {
                target.insert(prop_name.to_string(), value.clone());
                skip_keys.insert(prop_name.to_string());
            }
        }
        skip_keys.insert("_obj_cover".to_string());
    }

    if let Some(Value::Array(keys)) = control.get("_arr_add") {
        for item in keys {
            let Some(prop_name) = item.as_str() else {
                continue;
            };
            let Some(Value::Array(source_array)) = control.get(prop_name) else {
                continue;
            };

            let merged = match target.get(prop_name).and_then(Value::as_array) {
                Some(target_array) => merge_arrays(source_array, target_array),
                None => Value::Array(source_array.clone()),
            };
            target.insert(prop_name.to_string(), merged);
            skip_keys.insert(prop_name.to_string());
        }
        skip_keys.insert("_arr_add".to_string());
    }
}

fn merge_arrays(source: &[Value], target: &[Value]) -> Value {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    for item in target.iter().chain(source.iter()) {
        let key = serde_json::to_string(item).unwrap_or_else(|_| format!("{item:?}"));
        if seen.insert(key) {
            result.push(item.clone());
        }
    }

    Value::Array(result)
}

pub fn waypoint_type_description(code: &str) -> &str {
    match code {
        "path" => "path",
        "target" => "target",
        "teleport" => "teleport",
        "orientation" => "orientation",
        other => other,
    }
}

pub fn move_mode_description(code: &str) -> &str {
    match code {
        "walk" => "walk",
        "run" => "run",
        "dash" => "dash",
        "climb" => "climb",
        "fly" => "fly",
        "jump" => "jump",
        "swim" => "swim",
        other => other,
    }
}

pub fn pathing_task_type_description(code: &str) -> &str {
    match code {
        "collect" => "collect",
        "mining" => "mining",
        "farming" => "farming",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_existing_snake_case_pathing_json() {
        let json = r#"{
            "info": {
                "name": "sample",
                "type": "collect",
                "map_name": "Teyvat",
                "bgi_version": "0.45.0"
            },
            "positions": [
                {
                    "id": 1,
                    "x": 4527.51,
                    "y": 4825.12,
                    "type": "path",
                    "move_mode": "dash",
                    "action": "",
                    "action_params": ""
                },
                {
                    "x": 4517.81,
                    "y": 4866.36,
                    "action": "pick_around"
                }
            ]
        }"#;

        let task = PathingTask::from_json(json).unwrap();
        assert_eq!(task.info.name, "sample");
        assert_eq!(task.positions.len(), 2);
        assert!(task.has_action("pick_around"));
        assert_eq!(task.summary().waypoint_count, 2);
    }

    #[test]
    fn merges_control_json_like_legacy_json_merger() {
        let control: Value = json5::from_str(
            r#"{
                global_cover: { config: { realtime_triggers: { AutoPick: false } } },
                json_list: [
                    { name: "route", cover: { info: { author: "tester" } } }
                ]
            }"#,
        )
        .unwrap();
        let mut original: Value = json5::from_str(
            r#"{
                info: { name: "route", type: "collect" },
                config: { realtime_triggers: { AutoPick: true } },
                positions: []
            }"#,
        )
        .unwrap();

        merge_pathing_json(&control, &mut original, "route");
        assert_eq!(original["config"]["realtime_triggers"]["AutoPick"], false);
        assert_eq!(original["info"]["author"], "tester");
    }

    #[test]
    fn pathing_execution_plan_splits_segments_like_legacy_executor() {
        let task = PathingTask::from_json(
            r#"{
                "info": {
                    "name": "route",
                    "type": "farming",
                    "map_name": "Teyvat",
                    "map_match_method": "featureMatch"
                },
                "config": { "realtime_triggers": { "AutoPick": true } },
                "farming_info": {
                    "allow_farming_count": true,
                    "primary_target": "elite",
                    "elite_mob_count": 2
                },
                "positions": [
                    { "x": 1.0, "y": 2.0, "type": "path", "move_mode": "dash" },
                    { "x": 3.0, "y": 4.0, "type": "target", "action": "fight" },
                    { "x": 5.0, "y": 6.0, "type": "teleport" },
                    { "x": 7.0, "y": 8.0, "type": "orientation", "action": "log_output" },
                    { "x": 9.0, "y": 10.0, "type": "path", "action": "up_down_grab_leaf" }
                ]
            }"#,
        )
        .unwrap();

        let plan = task.execution_plan();

        assert_eq!(plan.map_match_method.as_deref(), Some("featureMatch"));
        assert_eq!(plan.retry_times, 2);
        assert!(plan.has_positions);
        assert!(plan.preflight.switch_party_before);
        assert!(plan.autopick_realtime_trigger_enabled);
        assert_eq!(plan.segment_count, 2);
        assert_eq!(plan.waypoint_count, 5);
        assert_eq!(plan.action_count, 3);
        assert_eq!(plan.expected_fight_count, 1);
        assert!(plan.farming.allow_farming_count);
        assert_eq!(plan.farming.primary_target, "elite");
        assert_eq!(plan.farming.elite_mob_count, 2.0);

        assert_eq!(plan.segments[0].waypoint_count, 2);
        assert!(!plan.segments[0].starts_with_teleport);
        assert_eq!(
            plan.segments[0].seed_previous_position,
            Some(PathingPoint { x: 1.0, y: 2.0 })
        );
        assert_eq!(plan.segments[1].waypoint_count, 3);
        assert!(plan.segments[1].starts_with_teleport);
        assert_eq!(plan.segments[1].seed_previous_position, None);

        let target = &plan.segments[0].waypoints[1];
        assert_eq!(target.global_index, 1);
        assert_eq!(
            target.declared_action_use,
            Some(PathingActionUseWaypointType::Path)
        );
        assert!(target.effective_target_point);
        assert!(target.phases.contains(&PathingWaypointPhase::MoveCloseTo));
        assert!(target.phases.contains(&PathingWaypointPhase::RunAction));

        let teleport = &plan.segments[1].waypoints[0];
        assert_eq!(
            teleport.phases,
            vec![
                PathingWaypointPhase::RecoverWhenLowHp,
                PathingWaypointPhase::HandleTeleport
            ]
        );

        let orientation = &plan.segments[1].waypoints[1];
        assert!(orientation.phases.contains(&PathingWaypointPhase::FaceTo));
        assert!(!orientation
            .phases
            .contains(&PathingWaypointPhase::MoveCloseTo));

        let leaf = &plan.segments[1].waypoints[2];
        assert!(!leaf.phases.contains(&PathingWaypointPhase::MoveTo));
        assert!(!leaf.phases.contains(&PathingWaypointPhase::MoveCloseTo));
        assert!(leaf.phases.contains(&PathingWaypointPhase::RunAction));
    }

    #[test]
    fn empty_pathing_execution_plan_skips_preflight_like_legacy_executor() {
        let task = PathingTask::from_json(
            r#"{
                "info": { "name": "empty", "type": "collect", "map_name": "Teyvat" },
                "positions": []
            }"#,
        )
        .unwrap();

        let plan = task.execution_plan();

        assert!(!plan.has_positions);
        assert_eq!(plan.segment_count, 0);
        assert_eq!(plan.waypoint_count, 0);
        assert!(!plan.preflight.switch_party_before);
        assert!(!plan.preflight.validate_game_with_task);
        assert!(!plan.preflight.warm_up_navigation);
        assert_eq!(plan.segments, Vec::<PathingSegmentPlan>::new());
    }
}
