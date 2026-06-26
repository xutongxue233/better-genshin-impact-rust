use super::{normalize_bridge_path, ScriptRepoBridgeFileKind, ScriptRepoBridgeFileResponse};
use crate::{checkout_git_repo_path, ScriptRepoError, ScriptRepoGitRunner};
use std::fs;
use std::path::{Path, PathBuf};

pub fn read_repo_bridge_file(
    repo_path: impl AsRef<Path>,
    rel_path: &str,
) -> Result<Option<ScriptRepoBridgeFileResponse>, ScriptRepoError> {
    read_repo_bridge_file_with_git::<crate::SystemGitRunner>(repo_path, rel_path, None)
}

pub fn read_repo_bridge_file_with_git<R: ScriptRepoGitRunner>(
    repo_path: impl AsRef<Path>,
    rel_path: &str,
    git_runner: Option<&mut R>,
) -> Result<Option<ScriptRepoBridgeFileResponse>, ScriptRepoError> {
    let repo_path = repo_path.as_ref();
    let decoded = percent_decode_lossy(rel_path);
    let normalized = normalize_bridge_path(&decoded);
    if normalized.is_empty() {
        return Ok(None);
    }

    let extension = Path::new(&normalized)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{}", ext.to_ascii_lowercase()))
        .unwrap_or_default();
    let kind = if is_allowed_text_extension(&extension) {
        ScriptRepoBridgeFileKind::Text
    } else if is_allowed_image_extension(&extension) {
        ScriptRepoBridgeFileKind::ImageBase64
    } else {
        return Ok(None);
    };

    let Some(bytes) = read_repo_bridge_file_bytes(repo_path, &normalized, git_runner)? else {
        return Ok(None);
    };
    if bytes.is_empty() {
        return Ok(None);
    }

    let content = match kind {
        ScriptRepoBridgeFileKind::Text => match String::from_utf8(bytes) {
            Ok(content) => content,
            Err(_) => return Ok(None),
        },
        ScriptRepoBridgeFileKind::ImageBase64 => base64_encode(&bytes),
    };

    Ok(Some(ScriptRepoBridgeFileResponse {
        rel_path: normalized,
        extension,
        kind,
        content,
    }))
}

fn read_repo_bridge_file_bytes<R: ScriptRepoGitRunner>(
    repo_path: &Path,
    normalized: &str,
    git_runner: Option<&mut R>,
) -> Result<Option<Vec<u8>>, ScriptRepoError> {
    let file_repo_root = repo_path.join("repo");
    if file_repo_root.is_dir() {
        let root = lexical_normalize(&file_repo_root);
        let path = lexical_normalize(&root.join(normalized));
        if !path_starts_with(&path, &root) || !path.is_file() {
            return Ok(None);
        }
        return fs::read(&path)
            .map(Some)
            .map_err(|source| ScriptRepoError::Io { path, source });
    }

    let Some(runner) = git_runner else {
        return Ok(None);
    };
    let temp_root = std::env::temp_dir().join(format!("bgi-repo-bridge-{}", unique_suffix()));
    let destination = temp_root.join(normalized);
    let checkout = checkout_git_repo_path(runner, repo_path, normalized, &destination, true)?;
    let result = if checkout.is_some() && destination.is_file() {
        fs::read(&destination)
            .map(Some)
            .map_err(|source| ScriptRepoError::Io {
                path: destination.clone(),
                source,
            })
    } else {
        Ok(None)
    };
    let cleanup = remove_existing(&temp_root);
    match (result, cleanup) {
        (Ok(bytes), Ok(())) => Ok(bytes),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

fn is_allowed_text_extension(extension: &str) -> bool {
    matches!(
        extension,
        ".txt"
            | ".md"
            | ".json"
            | ".js"
            | ".ts"
            | ".vue"
            | ".css"
            | ".html"
            | ".csv"
            | ".xml"
            | ".yaml"
            | ".yml"
            | ".ini"
            | ".config"
    )
}

fn is_allowed_image_extension(extension: &str) -> bool {
    matches!(
        extension,
        ".png" | ".jpg" | ".jpeg" | ".gif" | ".webp" | ".svg" | ".bmp" | ".ico"
    )
}

fn percent_decode_lossy(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or("");
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    output.push(byte);
                    index += 3;
                } else {
                    output.push(bytes[index]);
                    index += 1;
                }
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
    String::from_utf8_lossy(&output).to_string()
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut index = 0;
    while index < bytes.len() {
        let b0 = bytes[index];
        let b1 = bytes.get(index + 1).copied().unwrap_or(0);
        let b2 = bytes.get(index + 2).copied().unwrap_or(0);
        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | b2 as u32;
        output.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);
        if index + 1 < bytes.len() {
            output.push(TABLE[((triple >> 6) & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
        if index + 2 < bytes.len() {
            output.push(TABLE[(triple & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
        index += 3;
    }
    output
}

fn unique_suffix() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| format!("{:x}", duration.as_nanos()))
        .unwrap_or_else(|_| "0".to_string())
}

fn remove_existing(path: &Path) -> Result<(), ScriptRepoError> {
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

fn lexical_normalize(path: &Path) -> PathBuf {
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

fn path_starts_with(path: &Path, root: &Path) -> bool {
    let path = comparable_components(path);
    let root = comparable_components(root);
    path.len() >= root.len() && path.iter().zip(root.iter()).all(|(a, b)| a == b)
}

fn comparable_components(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_ascii_lowercase())
        .collect()
}
