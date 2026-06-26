use bgi_core::{BgiError, Result as BgiResult};
use std::fs;
use std::path::{Path, PathBuf};

#[path = "group_discovery.rs"]
mod group_discovery;
#[path = "group_model.rs"]
mod group_model;

pub use group_discovery::*;
pub use group_model::*;

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

fn normalize_project_indexes(group: &mut ScriptGroup) {
    for (index, project) in group.projects.iter_mut().enumerate() {
        project.index = index as i32 + 1;
    }
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
#[path = "group_tests.rs"]
mod tests;
