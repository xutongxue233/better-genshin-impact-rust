use super::policy_error::{Result, ScriptHostPolicyError};
use std::path::{Component, Path, PathBuf};

pub(crate) fn normalize_script_path(root: &Path, path: &str) -> Result<PathBuf> {
    if path.trim().is_empty() {
        return Err(ScriptHostPolicyError::EmptyPath);
    }

    if let Some(file_name) = Path::new(path).file_name().and_then(|name| name.to_str()) {
        if contains_invalid_windows_file_name_char(file_name) {
            return Err(ScriptHostPolicyError::InvalidPathCharacter(
                file_name.to_string(),
            ));
        }
    }

    let root = absolute_lexical_path(root);
    let requested = Path::new(path);
    let candidate = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        root.join(requested)
    };
    let normalized = lexical_normalize(&candidate);

    if !path_starts_with(&normalized, &root) {
        return Err(ScriptHostPolicyError::PathTraversal {
            path: normalized,
            root,
        });
    }

    Ok(normalized)
}

fn contains_invalid_windows_file_name_char(value: &str) -> bool {
    value
        .chars()
        .any(|ch| matches!(ch, '<' | '>' | ':' | '"' | '|' | '?' | '*') || ch.is_control())
}

fn absolute_lexical_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        lexical_normalize(path)
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        lexical_normalize(&cwd.join(path))
    }
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
        }
    }

    normalized
}

fn path_starts_with(path: &Path, root: &Path) -> bool {
    let path_components = comparable_components(path);
    let root_components = comparable_components(root);
    path_components.len() >= root_components.len()
        && path_components
            .iter()
            .zip(root_components.iter())
            .all(|(left, right)| left == right)
}

fn comparable_components(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| {
            let value = component.as_os_str().to_string_lossy().to_string();
            if cfg!(windows) {
                value.to_ascii_lowercase()
            } else {
                value
            }
        })
        .collect()
}
