use crate::project::ScriptProject;
use bgi_core::{BgiError, Result as BgiResult};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

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

pub fn select_script_groups_from_resume(
    groups: &[ScriptGroup],
    next_group_name: Option<&str>,
) -> (Vec<ScriptGroup>, bool) {
    let Some(next_group_name) = next_group_name.filter(|value| !value.trim().is_empty()) else {
        return (groups.to_vec(), false);
    };

    let Some(start_index) = groups
        .iter()
        .position(|group| group.name == next_group_name)
    else {
        return (groups.to_vec(), false);
    };

    (groups[start_index..].to_vec(), true)
}

pub fn select_script_group_projects_from_resume(
    group: &ScriptGroup,
    resume_pointer: Option<&ScriptGroupResumePointer>,
) -> (Vec<ScriptGroupProject>, bool) {
    let Some(pointer) = resume_pointer.filter(|pointer| pointer.group_name == group.name) else {
        return (group.projects.clone(), false);
    };

    let Some(start_index) = group.projects.iter().position(|project| {
        project.index == pointer.project_index
            && project.folder_name == pointer.folder_name
            && project.name == pointer.project_name
    }) else {
        return (group.projects.clone(), false);
    };

    let projects = group
        .projects
        .iter()
        .enumerate()
        .map(|(index, project)| {
            let mut project = project.clone();
            project.next_flag = Some(index == start_index);
            project.skip_flag = Some(index < start_index);
            project
        })
        .collect();
    (projects, true)
}

pub fn script_group_file_path(root: impl AsRef<Path>, name: &str) -> PathBuf {
    root.as_ref().join(format!("{name}.json"))
}

pub fn read_script_groups(root: impl AsRef<Path>) -> BgiResult<Vec<ScriptGroupFile>> {
    let root = root.as_ref();
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut groups = Vec::new();
    for entry in fs::read_dir(root).map_err(|source| BgiError::io(root, source))? {
        let entry = entry.map_err(|source| BgiError::io(root, source))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let group = read_script_group_file(&path)?;
        groups.push(ScriptGroupFile { path, group });
    }
    groups.sort_by_key(|entry| entry.group.index);
    Ok(groups)
}

pub fn read_script_group_file(path: impl AsRef<Path>) -> BgiResult<ScriptGroup> {
    let path = path.as_ref();
    let content = fs::read_to_string(path).map_err(|source| BgiError::io(path, source))?;
    parse_script_group_json(&content, Some(path))
}

pub fn write_script_group_file(
    root: impl AsRef<Path>,
    group: &mut ScriptGroup,
) -> BgiResult<PathBuf> {
    normalize_project_indexes(group);
    let root = root.as_ref();
    fs::create_dir_all(root).map_err(|source| BgiError::io(root, source))?;
    let path = script_group_file_path(root, &group.name);
    let content = serde_json::to_string_pretty(group)
        .map_err(|source| BgiError::json(Some(path.clone()), source.to_string()))?;
    fs::write(&path, content).map_err(|source| BgiError::io(&path, source))?;
    Ok(path)
}

pub fn parse_script_group_json(content: &str, path: Option<&Path>) -> BgiResult<ScriptGroup> {
    serde_json::from_str(content)
        .map_err(|source| BgiError::json(path.map(Path::to_path_buf), source.to_string()))
}

pub fn create_script_group(root: impl AsRef<Path>, name: &str) -> BgiResult<ScriptGroupFile> {
    let root = root.as_ref();
    let name = sanitize_group_name(name)?;
    fs::create_dir_all(root).map_err(|source| BgiError::io(root, source))?;
    let path = script_group_file_path(root, &name);
    if path.exists() {
        return Err(BgiError::json(
            Some(path),
            format!("script group {name:?} already exists"),
        ));
    }
    let next_index = read_script_groups(root)?
        .iter()
        .map(|entry| entry.group.index)
        .max()
        .unwrap_or(0)
        + 1;
    let mut group = ScriptGroup {
        index: next_index,
        name,
        ..ScriptGroup::default()
    };
    let path = write_script_group_file(root, &mut group)?;
    Ok(ScriptGroupFile { path, group })
}

pub fn rename_script_group(
    root: impl AsRef<Path>,
    old_name: &str,
    new_name: &str,
) -> BgiResult<ScriptGroupFile> {
    let root = root.as_ref();
    let new_name = sanitize_group_name(new_name)?;
    let old_path = script_group_file_path(root, old_name);
    let mut group = read_script_group_file(&old_path)?;
    let new_path = script_group_file_path(root, &new_name);
    if new_path.exists() && old_path != new_path {
        return Err(BgiError::json(
            Some(new_path),
            format!("script group {new_name:?} already exists"),
        ));
    }
    if old_path.exists() && old_path != new_path {
        fs::remove_file(&old_path).map_err(|source| BgiError::io(&old_path, source))?;
    }
    group.name = new_name;
    let path = write_script_group_file(root, &mut group)?;
    Ok(ScriptGroupFile { path, group })
}

pub fn delete_script_group(root: impl AsRef<Path>, name: &str) -> BgiResult<bool> {
    let path = script_group_file_path(root, name);
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_file(&path).map_err(|source| BgiError::io(&path, source))?;
    Ok(true)
}

pub fn update_script_group_project(
    root: impl AsRef<Path>,
    group_name: &str,
    project_index: usize,
    patch: ScriptGroupProjectPatch,
) -> BgiResult<ScriptGroupFile> {
    let root = root.as_ref();
    let path = script_group_file_path(root, group_name);
    let mut group = read_script_group_file(&path)?;
    let project = group.projects.get_mut(project_index).ok_or_else(|| {
        BgiError::json(
            Some(path.clone()),
            format!("script group project index {project_index} was not found"),
        )
    })?;
    patch.apply_to(project);
    let path = write_script_group_file(root, &mut group)?;
    Ok(ScriptGroupFile { path, group })
}

pub fn add_script_group_project(
    root: impl AsRef<Path>,
    group_name: &str,
    mut project: ScriptGroupProject,
) -> BgiResult<ScriptGroupFile> {
    let root = root.as_ref();
    let path = script_group_file_path(root, group_name);
    let mut group = read_script_group_file(&path)?;
    project.index = group.projects.len() as i32 + 1;
    group.projects.push(project);
    let path = write_script_group_file(root, &mut group)?;
    Ok(ScriptGroupFile { path, group })
}

pub fn add_key_mouse_script_project(
    root: impl AsRef<Path>,
    group_name: &str,
    name: impl Into<String>,
) -> BgiResult<ScriptGroupFile> {
    add_script_group_project(root, group_name, ScriptGroupProject::key_mouse(name))
}

pub fn add_pathing_script_project(
    root: impl AsRef<Path>,
    group_name: &str,
    name: impl Into<String>,
    folder_name: impl Into<String>,
) -> BgiResult<ScriptGroupFile> {
    add_script_group_project(
        root,
        group_name,
        ScriptGroupProject::pathing(name, folder_name),
    )
}

pub fn add_shell_script_project(
    root: impl AsRef<Path>,
    group_name: &str,
    command: impl Into<String>,
) -> BgiResult<ScriptGroupFile> {
    add_script_group_project(root, group_name, ScriptGroupProject::shell(command))
}

pub fn remove_script_group_project(
    root: impl AsRef<Path>,
    group_name: &str,
    project_index: usize,
) -> BgiResult<ScriptGroupFile> {
    let root = root.as_ref();
    let path = script_group_file_path(root, group_name);
    let mut group = read_script_group_file(&path)?;
    if project_index >= group.projects.len() {
        return Err(BgiError::json(
            Some(path),
            format!("script group project index {project_index} was not found"),
        ));
    }
    group.projects.remove(project_index);
    let path = write_script_group_file(root, &mut group)?;
    Ok(ScriptGroupFile { path, group })
}

pub fn move_script_group_project(
    root: impl AsRef<Path>,
    group_name: &str,
    project_index: usize,
    target_index: usize,
) -> BgiResult<ScriptGroupFile> {
    let root = root.as_ref();
    let path = script_group_file_path(root, group_name);
    let mut group = read_script_group_file(&path)?;
    let project_count = group.projects.len();
    if project_index >= project_count {
        return Err(BgiError::json(
            Some(path),
            format!("script group project index {project_index} was not found"),
        ));
    }
    if target_index >= project_count {
        return Err(BgiError::json(
            Some(path),
            format!("script group target index {target_index} was not found"),
        ));
    }
    if project_index != target_index {
        let project = group.projects.remove(project_index);
        group.projects.insert(target_index, project);
    }
    let path = write_script_group_file(root, &mut group)?;
    Ok(ScriptGroupFile { path, group })
}

pub fn available_js_script_projects(
    scripts_root: impl AsRef<Path>,
) -> BgiResult<Vec<AvailableJsScriptProject>> {
    let scripts_root = scripts_root.as_ref();
    if !scripts_root.exists() {
        return Ok(Vec::new());
    }
    let mut projects = Vec::new();
    for entry in fs::read_dir(scripts_root).map_err(|source| BgiError::io(scripts_root, source))? {
        let entry = entry.map_err(|source| BgiError::io(scripts_root, source))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(folder_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if let Ok(project) = ScriptProject::load(scripts_root, folder_name.to_string()) {
            let description = project.manifest.short_description();
            projects.push(AvailableJsScriptProject {
                folder_name: folder_name.to_string(),
                name: project.manifest.name,
                version: project.manifest.version,
                description,
                settings_ui: project.manifest.settings_ui.clone(),
                has_settings_ui: project
                    .layout
                    .settings_ui_path
                    .as_ref()
                    .is_some_and(|path| path.is_file()),
            });
        }
    }
    projects.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(projects)
}

pub fn available_key_mouse_scripts(
    key_mouse_root: impl AsRef<Path>,
) -> BgiResult<Vec<AvailableKeyMouseScript>> {
    let key_mouse_root = key_mouse_root.as_ref();
    if !key_mouse_root.exists() {
        return Ok(Vec::new());
    }
    let mut scripts = Vec::new();
    collect_files_recursively(key_mouse_root, key_mouse_root, &mut |path| {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return;
        };
        let relative_path = path_to_forward_slash(
            path.strip_prefix(key_mouse_root)
                .map(Path::to_path_buf)
                .unwrap_or_else(|_| PathBuf::from(name)),
        );
        scripts.push(AvailableKeyMouseScript {
            name: name.to_string(),
            relative_path,
        });
    })?;
    scripts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(scripts)
}

pub fn available_pathing_scripts(
    pathing_root: impl AsRef<Path>,
) -> BgiResult<Vec<AvailablePathingScript>> {
    let pathing_root = pathing_root.as_ref();
    if !pathing_root.exists() {
        return Ok(Vec::new());
    }
    let mut scripts = Vec::new();
    collect_files_recursively(pathing_root, pathing_root, &mut |path| {
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            return;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return;
        };
        let folder_name = path
            .parent()
            .and_then(|parent| parent.strip_prefix(pathing_root).ok())
            .map(path_to_forward_slash)
            .unwrap_or_default();
        let relative_path = path_to_forward_slash(
            path.strip_prefix(pathing_root)
                .map(Path::to_path_buf)
                .unwrap_or_else(|_| PathBuf::from(name)),
        );
        scripts.push(AvailablePathingScript {
            name: name.to_string(),
            folder_name,
            relative_path,
        });
    })?;
    scripts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(scripts)
}

pub fn available_pathing_tree(
    pathing_root: impl AsRef<Path>,
) -> BgiResult<AvailablePathingTreeNode> {
    let scripts = available_pathing_scripts(pathing_root)?;
    let mut root = AvailablePathingTreeNode {
        name: "AutoPathing".to_string(),
        ..AvailablePathingTreeNode::default()
    };
    for script in scripts {
        insert_pathing_route(&mut root, script);
    }
    sort_pathing_tree(&mut root);
    Ok(root)
}

fn insert_pathing_route(root: &mut AvailablePathingTreeNode, script: AvailablePathingScript) {
    let mut node = root;
    let parts = script
        .relative_path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    for folder in parts.iter().take(parts.len().saturating_sub(1)) {
        let next_path = if node.relative_path.is_empty() {
            (*folder).to_string()
        } else {
            format!("{}/{}", node.relative_path, folder)
        };
        let child_index = node
            .children
            .iter()
            .position(|child| child.route.is_none() && child.name == *folder)
            .unwrap_or_else(|| {
                node.children.push(AvailablePathingTreeNode {
                    name: (*folder).to_string(),
                    relative_path: next_path,
                    folder_name: node.relative_path.clone(),
                    route: None,
                    children: Vec::new(),
                });
                node.children.len() - 1
            });
        node = &mut node.children[child_index];
    }
    node.children.push(AvailablePathingTreeNode {
        name: script.name.clone(),
        relative_path: script.relative_path.clone(),
        folder_name: script.folder_name.clone(),
        route: Some(script),
        children: Vec::new(),
    });
}

fn sort_pathing_tree(node: &mut AvailablePathingTreeNode) {
    node.children.sort_by(
        |left, right| match (left.route.is_some(), right.route.is_some()) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => left.name.cmp(&right.name),
        },
    );
    for child in &mut node.children {
        sort_pathing_tree(child);
    }
}

fn normalize_project_indexes(group: &mut ScriptGroup) {
    for (index, project) in group.projects.iter_mut().enumerate() {
        project.index = index as i32 + 1;
    }
}

fn collect_files_recursively(
    root: &Path,
    current: &Path,
    visit: &mut impl FnMut(&Path),
) -> BgiResult<()> {
    for entry in fs::read_dir(current).map_err(|source| BgiError::io(current, source))? {
        let entry = entry.map_err(|source| BgiError::io(current, source))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursively(root, &path, visit)?;
        } else if path.is_file() {
            let normalized = path.strip_prefix(root).unwrap_or(&path);
            if !normalized
                .components()
                .any(|component| component.as_os_str().to_string_lossy().starts_with('.'))
            {
                visit(&path);
            }
        }
    }
    Ok(())
}

fn path_to_forward_slash(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .components()
        .filter_map(|component| component.as_os_str().to_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>()
        .join("/")
}

fn sanitize_group_name(name: &str) -> BgiResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(BgiError::json(
            None::<PathBuf>,
            "script group name is empty",
        ));
    }
    if name.chars().any(|ch| {
        matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || ch.is_control()
    }) {
        return Err(BgiError::json(
            None::<PathBuf>,
            format!("script group name contains invalid path characters: {name}"),
        ));
    }
    Ok(name.to_string())
}

pub fn type_description(project_type: &ScriptProjectType) -> &'static str {
    match project_type {
        ScriptProjectType::Javascript => "JS Script",
        ScriptProjectType::KeyMouse => "Key/Mouse Script",
        ScriptProjectType::Pathing => "Map Pathing",
        ScriptProjectType::Shell => "Shell",
    }
}

pub fn schedule_description(code: &str) -> &'static str {
    match code {
        "Daily" => "Daily",
        "EveryTwoDays" => "Every two days",
        "Monday" => "Monday",
        "Tuesday" => "Tuesday",
        "Wednesday" => "Wednesday",
        "Thursday" => "Thursday",
        "Friday" => "Friday",
        "Saturday" => "Saturday",
        "Sunday" => "Sunday",
        _ => "Custom",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn pathing_project_matches_legacy_defaults() {
        let project = ScriptGroupProject::pathing("route.json", "folder");
        assert_eq!(project.project_type, ScriptProjectType::Pathing);
        assert_eq!(project.status, ScriptProjectStatus::Enabled);
        assert_eq!(project.schedule, "Daily");
    }

    #[test]
    fn script_group_crud_preserves_legacy_json_shape_and_indexes() {
        let root = temp_root("group-crud");
        let created = create_script_group(&root, "Daily").unwrap();
        assert_eq!(created.group.name, "Daily");
        assert_eq!(created.group.index, 1);
        assert!(created.path.ends_with("Daily.json"));

        let with_project = add_script_group_project(
            &root,
            "Daily",
            ScriptGroupProject::javascript("Demo", "demo-folder"),
        )
        .unwrap();
        assert_eq!(with_project.group.projects[0].index, 1);
        assert_eq!(with_project.group.projects[0].schedule, "Daily");

        let updated = update_script_group_project(
            &root,
            "Daily",
            0,
            ScriptGroupProjectPatch {
                status: Some(ScriptProjectStatus::Disabled),
                run_num: Some(0),
                allow_js_notification: Some(false),
                ..ScriptGroupProjectPatch::default()
            },
        )
        .unwrap();
        assert_eq!(
            updated.group.projects[0].status,
            ScriptProjectStatus::Disabled
        );
        assert_eq!(updated.group.projects[0].run_num, 1);
        assert_eq!(updated.group.projects[0].allow_js_notification, Some(false));

        let renamed = rename_script_group(&root, "Daily", "Nightly").unwrap();
        assert_eq!(renamed.group.name, "Nightly");
        assert!(!root.join("Daily.json").exists());
        assert!(root.join("Nightly.json").exists());

        let empty = remove_script_group_project(&root, "Nightly", 0).unwrap();
        assert!(empty.group.projects.is_empty());
        assert!(delete_script_group(&root, "Nightly").unwrap());
        assert!(!delete_script_group(&root, "Nightly").unwrap());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn script_group_project_move_reorders_and_normalizes_indexes() {
        let root = temp_root("group-move");
        create_script_group(&root, "Daily").unwrap();
        add_script_group_project(&root, "Daily", ScriptGroupProject::javascript("One", "one"))
            .unwrap();
        add_script_group_project(&root, "Daily", ScriptGroupProject::key_mouse("Two")).unwrap();
        add_script_group_project(&root, "Daily", ScriptGroupProject::shell("echo three")).unwrap();

        let moved = move_script_group_project(&root, "Daily", 2, 0).unwrap();

        let names = moved
            .group
            .projects
            .iter()
            .map(|project| project.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["echo three", "One", "Two"]);
        assert_eq!(
            moved
                .group
                .projects
                .iter()
                .map(|project| project.index)
                .collect::<Vec<_>>(),
            [1, 2, 3]
        );

        let unchanged = move_script_group_project(&root, "Daily", 1, 1).unwrap();
        assert_eq!(unchanged.group.projects[1].name, "One");

        let error = move_script_group_project(&root, "Daily", 0, 3)
            .expect_err("target beyond the end should fail");
        assert!(error.to_string().contains("target index 3"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resume_group_selection_starts_from_marked_group() {
        let groups = vec![
            ScriptGroup {
                name: "A".to_string(),
                ..ScriptGroup::default()
            },
            ScriptGroup {
                name: "B".to_string(),
                ..ScriptGroup::default()
            },
            ScriptGroup {
                name: "C".to_string(),
                ..ScriptGroup::default()
            },
        ];

        let (selected, consumed) = select_script_groups_from_resume(&groups, Some("B"));

        assert!(consumed);
        assert_eq!(
            selected
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["B", "C"]
        );

        let (fallback, consumed) = select_script_groups_from_resume(&groups, Some("Missing"));
        assert!(!consumed);
        assert_eq!(fallback.len(), 3);
    }

    #[test]
    fn resume_project_selection_marks_previous_projects_skipped() {
        let group = ScriptGroup {
            name: "Daily".to_string(),
            projects: vec![
                ScriptGroupProject {
                    index: 1,
                    name: "First".to_string(),
                    folder_name: "first".to_string(),
                    ..ScriptGroupProject::default()
                },
                ScriptGroupProject {
                    index: 2,
                    name: "Second".to_string(),
                    folder_name: "second".to_string(),
                    ..ScriptGroupProject::default()
                },
                ScriptGroupProject {
                    index: 3,
                    name: "Third".to_string(),
                    folder_name: "third".to_string(),
                    ..ScriptGroupProject::default()
                },
            ],
            ..ScriptGroup::default()
        };
        let pointer = ScriptGroupResumePointer {
            group_name: "Daily".to_string(),
            project_index: 2,
            folder_name: "second".to_string(),
            project_name: "Second".to_string(),
        };

        let (projects, consumed) = select_script_group_projects_from_resume(&group, Some(&pointer));

        assert!(consumed);
        assert_eq!(projects.len(), 3);
        assert_eq!(projects[0].skip_flag, Some(true));
        assert_eq!(projects[0].next_flag, Some(false));
        assert_eq!(projects[1].skip_flag, Some(false));
        assert_eq!(projects[1].next_flag, Some(true));
        assert_eq!(projects[2].skip_flag, Some(false));
        assert_eq!(projects[2].next_flag, Some(false));
    }

    #[test]
    fn resume_project_selection_falls_back_when_pointer_does_not_match() {
        let group = ScriptGroup {
            name: "Daily".to_string(),
            projects: vec![ScriptGroupProject::javascript("Demo", "demo")],
            ..ScriptGroup::default()
        };
        let pointer = ScriptGroupResumePointer {
            group_name: "Other".to_string(),
            project_index: 1,
            folder_name: "demo".to_string(),
            project_name: "Demo".to_string(),
        };

        let (projects, consumed) = select_script_group_projects_from_resume(&group, Some(&pointer));

        assert!(!consumed);
        assert_eq!(projects, group.projects);
    }

    #[test]
    fn available_js_script_projects_reads_valid_manifest_projects() {
        let root = temp_root("available-js");
        let scripts_root = root.join("User/JsScript");
        let project_root = scripts_root.join("demo");
        fs::create_dir_all(&project_root).unwrap();
        fs::write(
            project_root.join("manifest.json"),
            r#"{
                "name": "Demo",
                "version": "1.0.0",
                "description": "line1\nline2",
                "main": "main.js",
                "settingsUi": "settings.json"
            }"#,
        )
        .unwrap();
        fs::write(project_root.join("main.js"), "export default 1;").unwrap();
        fs::write(project_root.join("settings.json"), "[]").unwrap();

        let projects = available_js_script_projects(&scripts_root).unwrap();

        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].folder_name, "demo");
        assert_eq!(projects[0].name, "Demo");
        assert!(projects[0].has_settings_ui);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn key_mouse_pathing_and_shell_projects_match_legacy_add_defaults() {
        let root = temp_root("group-add-kinds");
        create_script_group(&root, "Daily").unwrap();

        let group = add_key_mouse_script_project(&root, "Daily", "macro.json").unwrap();
        assert_eq!(group.group.projects[0].index, 1);
        assert_eq!(
            group.group.projects[0].project_type,
            ScriptProjectType::KeyMouse
        );
        assert_eq!(group.group.projects[0].name, "macro.json");
        assert_eq!(group.group.projects[0].folder_name, "macro.json");

        let group =
            add_pathing_script_project(&root, "Daily", "route.json", "liyue/mining").unwrap();
        assert_eq!(
            group.group.projects[1].project_type,
            ScriptProjectType::Pathing
        );
        assert_eq!(group.group.projects[1].name, "route.json");
        assert_eq!(group.group.projects[1].folder_name, "liyue/mining");

        let group = add_shell_script_project(&root, "Daily", "echo ok").unwrap();
        assert_eq!(
            group.group.projects[2].project_type,
            ScriptProjectType::Shell
        );
        assert_eq!(group.group.projects[2].name, "echo ok");
        assert_eq!(group.group.projects[2].folder_name, "");
        assert_eq!(
            group
                .group
                .projects
                .iter()
                .map(|project| project.index)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn key_mouse_and_pathing_scans_preserve_relative_paths() {
        let root = temp_root("available-non-js");
        let key_mouse_root = root.join("User/KeyMouseScript");
        let pathing_root = root.join("User/AutoPathing");
        fs::create_dir_all(key_mouse_root.join("combat")).unwrap();
        fs::create_dir_all(pathing_root.join("liyue/mining")).unwrap();
        fs::write(key_mouse_root.join("combat/macro.json"), "{}").unwrap();
        fs::write(pathing_root.join("root.json"), "{}").unwrap();
        fs::write(pathing_root.join("liyue/mining/route.json"), "{}").unwrap();
        fs::write(pathing_root.join("liyue/mining/readme.txt"), "skip").unwrap();

        let key_mouse = available_key_mouse_scripts(&key_mouse_root).unwrap();
        let pathing = available_pathing_scripts(&pathing_root).unwrap();
        let pathing_tree = available_pathing_tree(&pathing_root).unwrap();

        assert_eq!(
            key_mouse,
            vec![AvailableKeyMouseScript {
                name: "macro.json".to_string(),
                relative_path: "combat/macro.json".to_string(),
            }]
        );
        assert_eq!(
            pathing,
            vec![
                AvailablePathingScript {
                    name: "route.json".to_string(),
                    folder_name: "liyue/mining".to_string(),
                    relative_path: "liyue/mining/route.json".to_string(),
                },
                AvailablePathingScript {
                    name: "root.json".to_string(),
                    folder_name: "".to_string(),
                    relative_path: "root.json".to_string(),
                },
            ]
        );
        assert_eq!(pathing_tree.name, "AutoPathing");
        assert_eq!(pathing_tree.children.len(), 2);
        assert_eq!(pathing_tree.children[0].name, "liyue");
        assert!(pathing_tree.children[0].route.is_none());
        assert_eq!(pathing_tree.children[0].children[0].name, "mining");
        let nested_route = &pathing_tree.children[0].children[0].children[0];
        assert_eq!(nested_route.relative_path, "liyue/mining/route.json");
        assert_eq!(
            nested_route.route.as_ref().unwrap().folder_name,
            "liyue/mining"
        );
        assert_eq!(pathing_tree.children[1].relative_path, "root.json");
        assert!(pathing_tree.children[1].route.is_some());

        fs::remove_dir_all(root).unwrap();
    }

    fn temp_root(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("bgi-{name}-{suffix}"))
    }
}
