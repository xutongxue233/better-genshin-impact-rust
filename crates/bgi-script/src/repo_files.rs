use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::repo_paths::{copy_directory, remove_existing};
use super::repo_planning_update::calculate_repo_overlap_ratio;
use super::{Result, ScriptRepoError};

#[derive(Debug, Clone)]
pub(super) struct ExistingRepoCandidate {
    pub(super) folder_name: String,
    pub(super) path: PathBuf,
    pub(super) overlap_ratio: f64,
}

pub(super) fn find_file_named(root: &Path, file_name: &str) -> Result<Option<PathBuf>> {
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

pub(super) fn read_existing_repo_content(target_path: &Path) -> Result<Option<String>> {
    let repo_updated = target_path.join("repo_updated.json");
    if repo_updated.exists() {
        return fs::read_to_string(&repo_updated)
            .map(Some)
            .map_err(|source| ScriptRepoError::Io {
                path: repo_updated,
                source,
            });
    }

    let Some(repo_json) = find_file_named(target_path, "repo.json")? else {
        return Ok(None);
    };
    fs::read_to_string(&repo_json)
        .map(Some)
        .map_err(|source| ScriptRepoError::Io {
            path: repo_json,
            source,
        })
}

pub(super) fn best_matching_existing_repo(
    repos_path: &Path,
    new_repo_content: &str,
) -> Option<ExistingRepoCandidate> {
    if !repos_path.exists() {
        return None;
    }

    let mut best: Option<ExistingRepoCandidate> = None;
    let entries = fs::read_dir(repos_path).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(folder_name) = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(ToOwned::to_owned)
        else {
            continue;
        };
        if folder_name.eq_ignore_ascii_case("Temp") {
            continue;
        }

        let Ok(Some(existing_content)) = read_existing_repo_content(&path) else {
            continue;
        };
        let overlap_ratio = calculate_repo_overlap_ratio(&existing_content, new_repo_content);
        if overlap_ratio <= 0.0 {
            continue;
        }
        if best
            .as_ref()
            .map(|candidate| overlap_ratio > candidate.overlap_ratio)
            .unwrap_or(true)
        {
            best = Some(ExistingRepoCandidate {
                folder_name,
                path,
                overlap_ratio,
            });
        }
    }

    best
}

pub(super) fn generate_unique_folder_name(repos_path: &Path, base_name: &str) -> String {
    for index in 1..100 {
        let candidate = format!("{base_name}_{index}");
        if !repos_path.join(&candidate).exists() {
            return candidate;
        }
    }
    let ticks = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{base_name}_{ticks}")
}

pub(crate) fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("{:x}", duration.as_nanos()))
        .unwrap_or_else(|_| "0".to_string())
}

pub(super) fn write_folder_mapping(path: &Path, repo_url: &str, folder_name: &str) -> Result<()> {
    let mut mapping = if path.exists() {
        let content = fs::read_to_string(path).map_err(|source| ScriptRepoError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        serde_json::from_str::<BTreeMap<String, String>>(&content).map_err(|source| {
            ScriptRepoError::Json {
                path: path.to_path_buf(),
                source,
            }
        })?
    } else {
        BTreeMap::new()
    };
    mapping.insert(
        repo_url.trim_end_matches('/').to_string(),
        folder_name.to_string(),
    );
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ScriptRepoError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let content =
        serde_json::to_string_pretty(&mapping).map_err(|source| ScriptRepoError::Json {
            path: path.to_path_buf(),
            source,
        })?;
    fs::write(path, content).map_err(|source| ScriptRepoError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub(super) fn rename_or_copy_directory(source: &Path, destination: &Path) -> Result<()> {
    remove_existing(destination)?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|source_error| ScriptRepoError::Io {
            path: parent.to_path_buf(),
            source: source_error,
        })?;
    }
    match fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(_) => {
            copy_directory(source, destination)?;
            remove_existing(source)
        }
    }
}
