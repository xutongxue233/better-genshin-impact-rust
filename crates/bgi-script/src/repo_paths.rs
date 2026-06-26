use super::{Result, ScriptRepoError, ScriptRepoPathKind};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn expand_top_level_paths(
    paths: impl IntoIterator<Item = impl Into<String>>,
    available_top_level_children: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut result = Vec::new();
    for path in normalize_subscription_paths(paths) {
        if ScriptRepoPathKind::from_prefix(&path).is_some() {
            if let Some(children) = available_top_level_children.get(&path) {
                for child in children {
                    result.push(format!("{path}/{}", child.trim_matches('/')));
                }
                continue;
            }
        }
        result.push(path);
    }
    normalize_subscription_paths(result)
}

pub fn normalize_subscription_paths(
    paths: impl IntoIterator<Item = impl Into<String>>,
) -> Vec<String> {
    let mut seen = BTreeSet::new();
    for path in paths {
        let normalized = normalize_repo_path(&path.into());
        if !normalized.is_empty() {
            seen.insert(normalized);
        }
    }
    seen.into_iter().collect()
}

pub fn merge_subscription_paths(
    existing: impl IntoIterator<Item = impl Into<String>>,
    added: impl IntoIterator<Item = impl Into<String>>,
) -> Vec<String> {
    let mut merged = existing
        .into_iter()
        .map(Into::into)
        .collect::<Vec<String>>();
    merged.extend(added.into_iter().map(Into::into));
    normalize_subscription_paths(merged)
}

pub fn first_folder_and_remaining_path(path: &str) -> (String, PathBuf) {
    let normalized = normalize_repo_path(path);
    let mut parts = normalized.split('/').filter(|part| !part.is_empty());
    let first = parts.next().unwrap_or_default().to_string();
    let remaining = parts.collect::<Vec<_>>().join("/");
    (first, PathBuf::from(remaining))
}

pub(crate) fn path_mapper(app_root: &Path) -> BTreeMap<ScriptRepoPathKind, PathBuf> {
    BTreeMap::from([
        (
            ScriptRepoPathKind::Pathing,
            app_root.join(ScriptRepoPathKind::Pathing.user_relative_root()),
        ),
        (
            ScriptRepoPathKind::Js,
            app_root.join(ScriptRepoPathKind::Js.user_relative_root()),
        ),
        (
            ScriptRepoPathKind::Combat,
            app_root.join(ScriptRepoPathKind::Combat.user_relative_root()),
        ),
        (
            ScriptRepoPathKind::Tcg,
            app_root.join(ScriptRepoPathKind::Tcg.user_relative_root()),
        ),
    ])
}

pub(crate) fn normalize_repo_path(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .split('/')
        .filter(|part| !part.trim().is_empty() && *part != "." && *part != "..")
        .collect::<Vec<_>>()
        .join("/")
}

pub(crate) fn canonical_or_lexical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| lexical_normalize(path))
}

pub(crate) fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::Normal(part) => normalized.push(part),
            std::path::Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            std::path::Component::RootDir => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

pub(crate) fn validate_child_path(root: &Path, relative: &str) -> Result<PathBuf> {
    let candidate = lexical_normalize(&root.join(normalize_repo_path(relative)));
    if !path_starts_with(&candidate, root) {
        return Err(ScriptRepoError::PathEscapesRoot {
            path: candidate,
            root: root.to_path_buf(),
        });
    }
    Ok(candidate)
}

pub(crate) fn path_starts_with(path: &Path, root: &Path) -> bool {
    let path = comparable_components(path);
    let root = comparable_components(root);
    path.len() >= root.len() && path.iter().zip(root.iter()).all(|(a, b)| a == b)
}

fn comparable_components(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_ascii_lowercase())
        .collect()
}

pub(crate) fn remove_existing(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|source| ScriptRepoError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    } else if path.exists() {
        fs::remove_file(path).map_err(|source| ScriptRepoError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

pub(crate) fn copy_repo_path(source: &Path, destination: &Path) -> Result<()> {
    if source.is_dir() {
        copy_directory(source, destination)
    } else if source.is_file() {
        copy_file(source, destination)
    } else if source.exists() {
        Err(ScriptRepoError::UnsupportedSource(source.to_path_buf()))
    } else {
        Err(ScriptRepoError::MissingSource(source.to_path_buf()))
    }
}

pub(crate) fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).map_err(|source_error| ScriptRepoError::Io {
        path: destination.to_path_buf(),
        source: source_error,
    })?;
    for entry in fs::read_dir(source).map_err(|source_error| ScriptRepoError::Io {
        path: source.to_path_buf(),
        source: source_error,
    })? {
        let entry = entry.map_err(|source_error| ScriptRepoError::Io {
            path: source.to_path_buf(),
            source: source_error,
        })?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        copy_repo_path(&from, &to)?;
    }
    Ok(())
}

fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|source_error| ScriptRepoError::Io {
            path: parent.to_path_buf(),
            source: source_error,
        })?;
    }
    fs::copy(source, destination).map_err(|source_error| ScriptRepoError::Io {
        path: destination.to_path_buf(),
        source: source_error,
    })?;
    Ok(())
}

pub(crate) fn percent_decode(value: &str) -> std::result::Result<String, String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3])
                    .map_err(|_| "invalid percent escape".to_string())?;
                let byte = u8::from_str_radix(hex, 16)
                    .map_err(|_| "invalid percent escape".to_string())?;
                output.push(byte);
                index += 3;
            }
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(output).map_err(|_| "percent-decoded value must be UTF-8".to_string())
}

pub(crate) fn base64_decode(value: &str) -> std::result::Result<Vec<u8>, String> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = Vec::new();
    let mut buffer = 0_u32;
    let mut bits = 0_u8;

    for byte in value.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        if byte == b'=' {
            break;
        }
        let Some(position) = TABLE.iter().position(|candidate| *candidate == byte) else {
            return Err("import payload is not valid base64".to_string());
        };
        buffer = (buffer << 6) | position as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push(((buffer >> bits) & 0xff) as u8);
        }
    }

    Ok(output)
}
