use std::fs;
use std::path::Path;

use super::repo_paths::normalize_subscription_paths;
use super::{Result, ScriptRepoError};

pub fn read_subscription_file(path: impl AsRef<Path>) -> Result<Vec<String>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).map_err(|source| ScriptRepoError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    let paths =
        serde_json::from_str::<Vec<String>>(&content).map_err(|source| ScriptRepoError::Json {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(normalize_subscription_paths(paths))
}

pub fn write_subscription_file(path: impl AsRef<Path>, paths: &[String]) -> Result<()> {
    let path = path.as_ref();
    let paths = normalize_subscription_paths(paths.iter().cloned());
    if paths.is_empty() {
        if path.exists() {
            fs::remove_file(path).map_err(|source| ScriptRepoError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        }
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ScriptRepoError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let content = serde_json::to_string_pretty(&paths).map_err(|source| ScriptRepoError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    fs::write(path, content).map_err(|source| ScriptRepoError::Io {
        path: path.to_path_buf(),
        source,
    })
}
