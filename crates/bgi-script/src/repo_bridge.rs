use crate::{read_subscription_file, ScriptRepoError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[path = "repo_bridge_file.rs"]
mod repo_bridge_file;
#[path = "repo_bridge_index.rs"]
mod repo_bridge_index;

pub use repo_bridge_file::{read_repo_bridge_file, read_repo_bridge_file_with_git};
pub use repo_bridge_index::{repo_bridge_index_nodes, repo_bridge_index_nodes_from_json};

pub const REPO_BRIDGE_NOT_FOUND: &str = "404";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptRepoBridgeFileKind {
    Text,
    ImageBase64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptRepoBridgePaths {
    pub repo_path: PathBuf,
    pub repo_json_path: PathBuf,
    pub repo_updated_json_path: PathBuf,
    pub subscription_file_path: PathBuf,
    pub user_config_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptRepoBridgeFileResponse {
    pub rel_path: String,
    pub extension: String,
    pub kind: ScriptRepoBridgeFileKind,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptRepoBridgeIndexNode {
    pub path: String,
    pub name: String,
    pub node_type: String,
    pub has_update: bool,
    pub last_updated: Option<String>,
    pub depth: usize,
    pub child_count: usize,
    pub importable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptRepoBridgeGuideState {
    pub guide_status: bool,
}

pub fn script_repo_bridge_paths(
    app_root: impl AsRef<Path>,
    repo_path: impl AsRef<Path>,
    repo_folder_name: Option<&str>,
) -> Result<ScriptRepoBridgePaths, ScriptRepoError> {
    let app_root = app_root.as_ref();
    let repo_path = repo_path.as_ref().to_path_buf();
    let folder_name = repo_folder_name
        .filter(|name| !name.trim().is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            repo_path
                .file_name()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| crate::DEFAULT_REPO_FOLDER_NAME.to_string());
    let repo_json_path = repo_bridge_repo_json_path(&repo_path)?;

    Ok(ScriptRepoBridgePaths {
        repo_updated_json_path: repo_path.join("repo_updated.json"),
        subscription_file_path: app_root
            .join(crate::SUBSCRIPTIONS_DIR)
            .join(format!("{folder_name}.json")),
        user_config_path: app_root.join("User/config.json"),
        repo_path,
        repo_json_path,
    })
}

pub fn repo_bridge_repo_json_path(repo_path: impl AsRef<Path>) -> Result<PathBuf, ScriptRepoError> {
    let repo_path = repo_path.as_ref();
    let updated = repo_path.join("repo_updated.json");
    if updated.exists() {
        return Ok(updated);
    }
    find_file_named(repo_path, "repo.json")?
        .ok_or_else(|| ScriptRepoError::MissingRepoJson(repo_path.to_path_buf()))
}

pub fn read_repo_bridge_repo_json(repo_path: impl AsRef<Path>) -> Result<String, ScriptRepoError> {
    let path = repo_bridge_repo_json_path(repo_path)?;
    fs::read_to_string(&path).map_err(|source| ScriptRepoError::Io { path, source })
}

pub fn read_repo_bridge_user_config(app_root: impl AsRef<Path>) -> Result<String, ScriptRepoError> {
    let path = app_root.as_ref().join("User/config.json");
    fs::read_to_string(&path).map_err(|source| ScriptRepoError::Io { path, source })
}

pub fn repo_bridge_subscribed_paths_json(
    subscription_file_path: impl AsRef<Path>,
) -> Result<String, ScriptRepoError> {
    let paths = read_subscription_file(subscription_file_path)?;
    serde_json::to_string(&paths).map_err(|source| ScriptRepoError::Json {
        path: PathBuf::from(crate::SUBSCRIPTIONS_DIR),
        source,
    })
}

pub fn clear_repo_bridge_update(repo_path: impl AsRef<Path>) -> Result<PathBuf, ScriptRepoError> {
    let repo_path = repo_path.as_ref();
    let original = find_file_named(repo_path, "repo.json")?
        .ok_or_else(|| ScriptRepoError::MissingRepoJson(repo_path.to_path_buf()))?;
    let target = repo_path.join("repo_updated.json");
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|source| ScriptRepoError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::copy(&original, &target).map_err(|source| ScriptRepoError::Io {
        path: target.clone(),
        source,
    })?;
    Ok(target)
}

pub fn mark_repo_bridge_path_updated(
    repo_json_path: impl AsRef<Path>,
    path: &str,
) -> Result<bool, ScriptRepoError> {
    let repo_json_path = repo_json_path.as_ref();
    let content = fs::read_to_string(repo_json_path).map_err(|source| ScriptRepoError::Io {
        path: repo_json_path.to_path_buf(),
        source,
    })?;
    let mut value = serde_json::from_str::<serde_json::Value>(&content).map_err(|source| {
        ScriptRepoError::Json {
            path: repo_json_path.to_path_buf(),
            source,
        }
    })?;
    let Some(indexes) = value
        .get_mut("indexes")
        .and_then(|value| value.as_array_mut())
    else {
        return Ok(false);
    };

    let parts = normalize_bridge_path(path)
        .split('/')
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if !parts.is_empty() {
        reset_path_recursively(indexes, &parts, 0);
    }

    let content = serde_json::to_string_pretty(&value).map_err(|source| ScriptRepoError::Json {
        path: repo_json_path.to_path_buf(),
        source,
    })?;
    fs::write(repo_json_path, content).map_err(|source| ScriptRepoError::Io {
        path: repo_json_path.to_path_buf(),
        source,
    })?;
    Ok(true)
}

fn reset_path_recursively(
    nodes: &mut [serde_json::Value],
    path_parts: &[String],
    current_index: usize,
) {
    if current_index >= path_parts.len() {
        return;
    }
    for node in nodes {
        let Some(object) = node.as_object_mut() else {
            continue;
        };
        if object
            .get("name")
            .and_then(|value| value.as_str())
            .map(|name| name == path_parts[current_index])
            .unwrap_or(false)
        {
            if current_index == path_parts.len() - 1 {
                reset_has_update_flag(object);
            } else if let Some(children) = object
                .get_mut("children")
                .and_then(|value| value.as_array_mut())
            {
                reset_path_recursively(children, path_parts, current_index + 1);
            }
            break;
        }
    }
}

fn reset_has_update_flag(object: &mut serde_json::Map<String, serde_json::Value>) {
    if object.get("hasUpdate").and_then(|value| value.as_bool()) == Some(true) {
        object.insert("hasUpdate".to_string(), serde_json::Value::Bool(false));
    }
    if let Some(children) = object
        .get_mut("children")
        .and_then(|value| value.as_array_mut())
    {
        for child in children {
            if let Some(child_object) = child.as_object_mut() {
                reset_has_update_flag(child_object);
            }
        }
    }
}

fn find_file_named(root: &Path, file_name: &str) -> Result<Option<PathBuf>, ScriptRepoError> {
    if !root.exists() {
        return Ok(None);
    }
    for entry in fs::read_dir(root).map_err(|source| ScriptRepoError::Io {
        path: root.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| ScriptRepoError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.eq_ignore_ascii_case("repo_updated.json"))
            .unwrap_or(false)
        {
            continue;
        }
        if path.is_dir() {
            if let Some(found) = find_file_named(&path, file_name)? {
                return Ok(Some(found));
            }
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.eq_ignore_ascii_case(file_name))
            .unwrap_or(false)
        {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn normalize_bridge_path(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .split('/')
        .filter(|part| !part.trim().is_empty() && *part != "." && *part != "..")
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
#[path = "repo_bridge_tests.rs"]
mod tests;
