use super::{
    AvailableJsScriptProject, AvailableKeyMouseScript, AvailablePathingScript,
    AvailablePathingTreeNode,
};
use crate::project::ScriptProject;
use bgi_core::{BgiError, Result as BgiResult};
use std::fs;
use std::path::{Path, PathBuf};

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
