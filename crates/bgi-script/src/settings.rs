use crate::group::{parse_script_group_json, write_script_group_file, ScriptProjectType};
use crate::manifest::Manifest;
use crate::project::ScriptProject;
use bgi_core::{BgiError, Result as BgiResult};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ScriptSettingKind {
    Separator,
    InputText,
    Select,
    Checkbox,
    MultiCheckbox,
    CascadeSelect,
    Unknown,
}

impl ScriptSettingKind {
    pub fn from_code(code: &str) -> Self {
        match code {
            "separator" => Self::Separator,
            "input-text" => Self::InputText,
            "select" => Self::Select,
            "checkbox" => Self::Checkbox,
            "multi-checkbox" => Self::MultiCheckbox,
            "cascade-select" => Self::CascadeSelect,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptSettingItem {
    pub name: String,
    #[serde(rename = "type")]
    pub setting_type: String,
    pub label: String,
    pub options: Option<Vec<String>>,
    pub cascade_options: Option<BTreeMap<String, Vec<String>>>,
    pub default: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for ScriptSettingItem {
    fn default() -> Self {
        Self {
            name: String::new(),
            setting_type: String::new(),
            label: String::new(),
            options: None,
            cascade_options: None,
            default: None,
            extra: Map::new(),
        }
    }
}

impl ScriptSettingItem {
    pub fn kind(&self) -> ScriptSettingKind {
        ScriptSettingKind::from_code(&self.setting_type)
    }

    pub fn apply_default(&self, settings: &mut Map<String, Value>) {
        if self.name.trim().is_empty() || settings.contains_key(&self.name) {
            return;
        }

        match self.kind() {
            ScriptSettingKind::InputText
            | ScriptSettingKind::Select
            | ScriptSettingKind::CascadeSelect => {
                if let Some(default) = self.default.as_ref().map(default_to_string) {
                    settings.insert(self.name.clone(), Value::String(default));
                }
            }
            ScriptSettingKind::Checkbox => {
                if let Some(value) = self.default.as_ref().and_then(default_to_bool) {
                    settings.insert(self.name.clone(), Value::Bool(value));
                }
            }
            ScriptSettingKind::MultiCheckbox => {
                let values = self
                    .default
                    .as_ref()
                    .and_then(default_to_string_array)
                    .unwrap_or_default();
                settings.insert(
                    self.name.clone(),
                    Value::Array(values.into_iter().map(Value::String).collect()),
                );
            }
            ScriptSettingKind::Separator | ScriptSettingKind::Unknown => {}
        }
    }

    pub fn clean_invalid_multi_checkbox_value(&self, settings: &mut Map<String, Value>) -> usize {
        if self.kind() != ScriptSettingKind::MultiCheckbox {
            return 0;
        }

        let Some(options) = &self.options else {
            return 0;
        };
        let allowed = options.iter().map(String::as_str).collect::<BTreeSet<_>>();
        let Some(Value::Array(values)) = settings.get_mut(&self.name) else {
            return 0;
        };

        let before = values.len();
        values.retain(|value| value.as_str().is_some_and(|text| allowed.contains(text)));
        before.saturating_sub(values.len())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScriptSettingsSchema {
    pub items: Vec<ScriptSettingItem>,
}

impl ScriptSettingsSchema {
    pub fn from_json(json: &str) -> BgiResult<Self> {
        let items = json5::from_str(json)
            .map_err(|err| BgiError::json(None::<PathBuf>, err.to_string()))?;
        Ok(Self { items })
    }

    pub fn read_from(path: impl AsRef<Path>) -> BgiResult<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| BgiError::io(path, source))?;
        Self::from_json(&text).map_err(|err| match err {
            BgiError::Json { message, .. } => BgiError::json(Some(path), message),
            other => other,
        })
    }

    pub fn read_from_project(
        project_dir: impl AsRef<Path>,
        manifest: &Manifest,
    ) -> BgiResult<Option<Self>> {
        if manifest.settings_ui.trim().is_empty() {
            return Ok(None);
        }

        let path = project_dir.as_ref().join(&manifest.settings_ui);
        if !path.is_file() {
            return Ok(None);
        }

        Self::read_from(path).map(Some)
    }

    pub fn apply_defaults(&self, settings: &mut Map<String, Value>) {
        for item in &self.items {
            item.apply_default(settings);
        }
    }

    pub fn clean_invalid_values(&self, settings: &mut Map<String, Value>) -> usize {
        self.items
            .iter()
            .map(|item| item.clean_invalid_multi_checkbox_value(settings))
            .sum()
    }

    pub fn kinds(&self) -> Vec<ScriptSettingKind> {
        self.items.iter().map(ScriptSettingItem::kind).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptSettingsSummary {
    pub supported_types: Vec<ScriptSettingKind>,
    pub defaulted_types: Vec<ScriptSettingKind>,
    pub cleans_multi_checkbox_options: bool,
    pub preserves_unknown_fields: bool,
}

impl Default for ScriptSettingsSummary {
    fn default() -> Self {
        Self {
            supported_types: vec![
                ScriptSettingKind::Separator,
                ScriptSettingKind::InputText,
                ScriptSettingKind::Select,
                ScriptSettingKind::Checkbox,
                ScriptSettingKind::MultiCheckbox,
                ScriptSettingKind::CascadeSelect,
            ],
            defaulted_types: vec![
                ScriptSettingKind::InputText,
                ScriptSettingKind::Select,
                ScriptSettingKind::Checkbox,
                ScriptSettingKind::MultiCheckbox,
                ScriptSettingKind::CascadeSelect,
            ],
            cleans_multi_checkbox_options: true,
            preserves_unknown_fields: true,
        }
    }
}

pub fn script_settings_summary() -> ScriptSettingsSummary {
    ScriptSettingsSummary::default()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScriptSettingsDocument {
    pub scripts_root: PathBuf,
    pub project_folder_name: String,
    pub project_path: PathBuf,
    pub manifest_path: PathBuf,
    pub settings_ui_path: Option<PathBuf>,
    pub manifest: Manifest,
    pub schema: Option<ScriptSettingsSchema>,
    pub values: Value,
    pub defaults_applied: bool,
    pub cleaned_invalid_values: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScriptGroupSettingsSaveResult {
    pub group_path: PathBuf,
    pub group_name: String,
    pub project_index: usize,
    pub project_folder_name: String,
    pub settings: Value,
    pub cleaned_invalid_values: usize,
}

pub fn read_script_settings_document(
    scripts_root: impl AsRef<Path>,
    project_folder_name: impl Into<String>,
    existing_settings: Option<Value>,
) -> BgiResult<ScriptSettingsDocument> {
    let scripts_root = scripts_root.as_ref().to_path_buf();
    let project_folder_name = project_folder_name.into();
    let project =
        ScriptProject::load(&scripts_root, project_folder_name.clone()).map_err(|error| {
            BgiError::io(
                scripts_root.join(&project_folder_name),
                std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()),
            )
        })?;
    let schema =
        ScriptSettingsSchema::read_from_project(&project.layout.project_path, &project.manifest)?;
    let mut settings =
        object_from_value(existing_settings.unwrap_or_else(|| Value::Object(Map::new())));
    let before_defaults = settings.clone();
    let cleaned_invalid_values = if let Some(schema) = &schema {
        schema.apply_defaults(&mut settings);
        schema.clean_invalid_values(&mut settings)
    } else {
        0
    };
    let defaults_applied = settings != before_defaults;

    Ok(ScriptSettingsDocument {
        scripts_root,
        project_folder_name,
        project_path: project.layout.project_path,
        manifest_path: project.layout.manifest_path,
        settings_ui_path: project.layout.settings_ui_path,
        manifest: project.manifest,
        schema,
        values: Value::Object(settings),
        defaults_applied,
        cleaned_invalid_values,
    })
}

pub fn save_script_group_project_settings(
    script_group_path: impl AsRef<Path>,
    group_name: &str,
    project_index: usize,
    scripts_root: impl AsRef<Path>,
    settings: Value,
) -> BgiResult<ScriptGroupSettingsSaveResult> {
    let group_path = script_group_path
        .as_ref()
        .join(format!("{group_name}.json"));
    let content =
        fs::read_to_string(&group_path).map_err(|source| BgiError::io(&group_path, source))?;
    let mut group = parse_script_group_json(&content, Some(&group_path))?;
    let project = group.projects.get_mut(project_index).ok_or_else(|| {
        BgiError::json(
            Some(group_path.clone()),
            format!("script group project index {project_index} was not found"),
        )
    })?;
    if project.project_type != ScriptProjectType::Javascript {
        return Err(BgiError::json(
            Some(group_path.clone()),
            "only JavaScript script projects can store custom settings",
        ));
    }

    let document =
        read_script_settings_document(scripts_root, project.folder_name.clone(), Some(settings))?;
    project.js_script_settings_object = Some(document.values.clone());

    let group_path = write_script_group_file(&script_group_path, &mut group)?;

    Ok(ScriptGroupSettingsSaveResult {
        group_path,
        group_name: group.name,
        project_index,
        project_folder_name: document.project_folder_name,
        settings: document.values,
        cleaned_invalid_values: document.cleaned_invalid_values,
    })
}

fn object_from_value(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        _ => Map::new(),
    }
}

fn default_to_string(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Null => String::new(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

fn default_to_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(value) => Some(*value),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn default_to_string_array(value: &Value) -> Option<Vec<String>> {
    let Value::Array(values) = value else {
        return None;
    };

    Some(
        values
            .iter()
            .filter_map(|value| value.as_str().map(ToString::to_string))
            .collect(),
    )
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
