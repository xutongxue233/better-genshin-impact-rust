use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::PathBuf;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptProjectType {
    Javascript,
    KeyMouse,
    Pathing,
    Shell,
}

impl Default for ScriptProjectType {
    fn default() -> Self {
        Self::Javascript
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptProjectStatus {
    Enabled,
    Disabled,
}

impl Default for ScriptProjectStatus {
    fn default() -> Self {
        Self::Enabled
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptGroup {
    pub index: i32,
    pub name: String,
    pub config: ScriptGroupConfig,
    pub projects: Vec<ScriptGroupProject>,
    #[serde(skip)]
    pub next_flag: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptGroupConfig {
    pub enable_shell_config: bool,
    pub pathing_config: Value,
    pub shell_config: Value,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptGroupProject {
    pub index: i32,
    pub name: String,
    pub folder_name: String,
    #[serde(rename = "type")]
    pub project_type: ScriptProjectType,
    pub status: ScriptProjectStatus,
    pub schedule: String,
    pub run_num: i32,
    pub js_script_settings_object: Option<Value>,
    pub allow_js_notification: Option<bool>,
    pub allow_js_http_hash: Option<String>,
    #[serde(skip)]
    pub next_flag: Option<bool>,
    #[serde(skip)]
    pub skip_flag: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptGroupFile {
    pub path: PathBuf,
    pub group: ScriptGroup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptGroupProjectPatch {
    pub name: Option<String>,
    pub folder_name: Option<String>,
    pub project_type: Option<ScriptProjectType>,
    pub status: Option<ScriptProjectStatus>,
    pub schedule: Option<String>,
    pub run_num: Option<i32>,
    pub allow_js_notification: Option<bool>,
    pub allow_js_http_hash: Option<Option<String>>,
}

impl Default for ScriptGroupProjectPatch {
    fn default() -> Self {
        Self {
            name: None,
            folder_name: None,
            project_type: None,
            status: None,
            schedule: None,
            run_num: None,
            allow_js_notification: None,
            allow_js_http_hash: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AvailableJsScriptProject {
    pub folder_name: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub settings_ui: String,
    pub has_settings_ui: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AvailableKeyMouseScript {
    pub name: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AvailablePathingScript {
    pub name: String,
    pub folder_name: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AvailablePathingTreeNode {
    pub name: String,
    pub relative_path: String,
    pub folder_name: String,
    pub route: Option<AvailablePathingScript>,
    pub children: Vec<AvailablePathingTreeNode>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptGroupResumePointer {
    pub group_name: String,
    pub project_index: i32,
    pub folder_name: String,
    pub project_name: String,
}

impl Default for ScriptGroupProject {
    fn default() -> Self {
        Self {
            index: 0,
            name: String::new(),
            folder_name: String::new(),
            project_type: ScriptProjectType::Javascript,
            status: ScriptProjectStatus::Enabled,
            schedule: String::new(),
            run_num: 1,
            js_script_settings_object: None,
            allow_js_notification: Some(true),
            allow_js_http_hash: Some(String::new()),
            next_flag: Some(false),
            skip_flag: Some(false),
            extra: Map::new(),
        }
    }
}

impl ScriptGroupProject {
    pub fn javascript(name: impl Into<String>, folder_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            folder_name: folder_name.into(),
            project_type: ScriptProjectType::Javascript,
            status: ScriptProjectStatus::Enabled,
            schedule: "Daily".to_string(),
            ..Self::default()
        }
    }

    pub fn key_mouse(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            folder_name: name,
            project_type: ScriptProjectType::KeyMouse,
            status: ScriptProjectStatus::Enabled,
            schedule: "Daily".to_string(),
            ..Self::default()
        }
    }

    pub fn pathing(name: impl Into<String>, folder_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            folder_name: folder_name.into(),
            project_type: ScriptProjectType::Pathing,
            status: ScriptProjectStatus::Enabled,
            schedule: "Daily".to_string(),
            ..Self::default()
        }
    }

    pub fn shell(command: impl Into<String>) -> Self {
        Self {
            name: command.into(),
            project_type: ScriptProjectType::Shell,
            status: ScriptProjectStatus::Enabled,
            schedule: "Daily".to_string(),
            ..Self::default()
        }
    }
}

impl ScriptGroupProjectPatch {
    pub fn apply_to(self, project: &mut ScriptGroupProject) {
        if let Some(name) = self.name {
            project.name = name;
        }
        if let Some(folder_name) = self.folder_name {
            project.folder_name = folder_name;
        }
        if let Some(project_type) = self.project_type {
            project.project_type = project_type;
        }
        if let Some(status) = self.status {
            project.status = status;
        }
        if let Some(schedule) = self.schedule {
            project.schedule = schedule;
        }
        if let Some(run_num) = self.run_num {
            project.run_num = run_num.max(1);
        }
        if let Some(allow_js_notification) = self.allow_js_notification {
            project.allow_js_notification = Some(allow_js_notification);
        }
        if let Some(allow_js_http_hash) = self.allow_js_http_hash {
            project.allow_js_http_hash = allow_js_http_hash;
        }
    }
}
