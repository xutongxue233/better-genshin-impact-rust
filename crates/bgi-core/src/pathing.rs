use crate::error::{BgiError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const CONTROL_FILE_NAME: &str = "control.json5";

#[path = "pathing_execution.rs"]
mod pathing_execution;
#[path = "pathing_io.rs"]
mod pathing_io;

pub use pathing_execution::*;
#[cfg(test)]
use pathing_io::merge_pathing_json;
pub use pathing_io::{read_pathing_json_with_control, read_pathing_task};

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
#[path = "pathing_tests.rs"]
mod tests;
