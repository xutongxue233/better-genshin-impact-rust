use crate::{checkout_git_repo_path, read_subscription_file, ScriptRepoError, ScriptRepoGitRunner};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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

pub fn repo_bridge_index_nodes(
    repo_path: impl AsRef<Path>,
) -> Result<Vec<ScriptRepoBridgeIndexNode>, ScriptRepoError> {
    let content = read_repo_bridge_repo_json(repo_path)?;
    repo_bridge_index_nodes_from_json(&content)
}

pub fn repo_bridge_index_nodes_from_json(
    content: &str,
) -> Result<Vec<ScriptRepoBridgeIndexNode>, ScriptRepoError> {
    let value = serde_json::from_str::<serde_json::Value>(content).map_err(|source| {
        ScriptRepoError::Json {
            path: PathBuf::from("repo.json"),
            source,
        }
    })?;
    let mut nodes = Vec::new();
    collect_index_nodes(
        value
            .get("indexes")
            .and_then(|value| value.as_array())
            .map(Vec::as_slice),
        "",
        0,
        &mut nodes,
    );
    Ok(nodes)
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

fn collect_index_nodes(
    nodes: Option<&[serde_json::Value]>,
    prefix: &str,
    depth: usize,
    output: &mut Vec<ScriptRepoBridgeIndexNode>,
) {
    let Some(nodes) = nodes else {
        return;
    };
    for node in nodes {
        let Some(object) = node.as_object() else {
            continue;
        };
        let Some(name) = object.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        let path = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{prefix}/{name}")
        };
        let children = object
            .get("children")
            .and_then(|value| value.as_array())
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let node_type = object
            .get("type")
            .and_then(|value| value.as_str())
            .unwrap_or_else(|| {
                if children.is_empty() {
                    "file"
                } else {
                    "directory"
                }
            })
            .to_string();
        output.push(ScriptRepoBridgeIndexNode {
            path: path.clone(),
            name: name.to_string(),
            node_type: node_type.clone(),
            has_update: object
                .get("hasUpdate")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
            last_updated: object
                .get("lastUpdated")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned),
            depth,
            child_count: children.len(),
            importable: is_importable_index_path(&path, &node_type),
        });
        collect_index_nodes(Some(children), &path, depth + 1, output);
    }
}

fn is_importable_index_path(path: &str, node_type: &str) -> bool {
    if node_type != "directory" {
        return false;
    }
    let first = path.split('/').next().unwrap_or_default();
    matches!(first, "js" | "pathing" | "combat" | "tcg")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScriptRepoGitCommandOutput, ScriptRepoGitRunner};
    use std::collections::BTreeMap;

    #[derive(Debug, Default)]
    struct BridgeGitRunner {
        objects: BTreeMap<String, (String, Vec<u8>)>,
        trees: BTreeMap<String, String>,
    }

    impl ScriptRepoGitRunner for BridgeGitRunner {
        fn run_git(
            &mut self,
            _cwd: Option<&Path>,
            args: &[String],
        ) -> Result<ScriptRepoGitCommandOutput, ScriptRepoError> {
            let bytes = match args
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .as_slice()
            {
                ["cat-file", "-t", object] => self
                    .objects
                    .get(*object)
                    .map(|(kind, _)| kind.clone().into_bytes())
                    .ok_or_else(|| ScriptRepoError::MissingSource(PathBuf::from(object)))?,
                ["ls-tree", object] => self
                    .trees
                    .get(*object)
                    .cloned()
                    .map(String::into_bytes)
                    .ok_or_else(|| ScriptRepoError::MissingSource(PathBuf::from(object)))?,
                ["show", object] => self
                    .objects
                    .get(*object)
                    .map(|(_, bytes)| bytes.clone())
                    .ok_or_else(|| ScriptRepoError::MissingSource(PathBuf::from(object)))?,
                _ => Vec::new(),
            };
            Ok(ScriptRepoGitCommandOutput {
                stdout: String::from_utf8_lossy(&bytes).trim().to_string(),
                stderr: String::new(),
                stdout_bytes: bytes,
            })
        }
    }

    fn test_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        fs::remove_dir_all(&root).unwrap_or(());
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn bridge_paths_prefer_repo_updated_json_and_subscription_file() {
        let root = test_root("bgi-repo-bridge-paths");
        let repo = root.join("Repos/repo");
        fs::create_dir_all(&repo).unwrap();
        fs::write(repo.join("repo.json"), "{}").unwrap();
        fs::write(repo.join("repo_updated.json"), "{\"updated\":true}").unwrap();

        let paths = script_repo_bridge_paths(&root, &repo, None).unwrap();

        assert!(paths.repo_json_path.ends_with("repo_updated.json"));
        assert!(paths.subscription_file_path.ends_with("repo.json"));
        assert!(paths.user_config_path.ends_with("User/config.json"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bridge_file_reads_text_and_image_from_file_repo_with_path_guard() {
        let root = test_root("bgi-repo-bridge-file");
        let repo_root = root.join("Repos/repo/repo/js/demo");
        fs::create_dir_all(&repo_root).unwrap();
        fs::write(repo_root.join("main.js"), "console.log('ok');").unwrap();
        fs::write(repo_root.join("icon.png"), [1_u8, 2, 3, 4]).unwrap();

        let text = read_repo_bridge_file(root.join("Repos/repo"), "js%2Fdemo%2Fmain.js")
            .unwrap()
            .unwrap();
        let image = read_repo_bridge_file(root.join("Repos/repo"), "js/demo/icon.png")
            .unwrap()
            .unwrap();

        assert_eq!(text.kind, ScriptRepoBridgeFileKind::Text);
        assert_eq!(text.content, "console.log('ok');");
        assert_eq!(image.kind, ScriptRepoBridgeFileKind::ImageBase64);
        assert_eq!(image.content, "AQIDBA==");
        assert!(
            read_repo_bridge_file(root.join("Repos/repo"), "../config.json")
                .unwrap()
                .is_none()
        );
        assert!(
            read_repo_bridge_file(root.join("Repos/repo"), "js/demo/main.exe")
                .unwrap()
                .is_none()
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bridge_file_reads_git_repo_blob() {
        let root = test_root("bgi-repo-bridge-git-file");
        let repo = root.join("Repos/repo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        let mut runner = BridgeGitRunner::default();
        runner.objects.insert(
            "HEAD:repo/js/demo/icon.png".to_string(),
            ("blob".to_string(), vec![0, 255, 1]),
        );

        let response = read_repo_bridge_file_with_git(&repo, "js/demo/icon.png", Some(&mut runner))
            .unwrap()
            .unwrap();

        assert_eq!(response.kind, ScriptRepoBridgeFileKind::ImageBase64);
        assert_eq!(response.content, "AP8B");

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bridge_mark_path_resets_update_flags_recursively() {
        let root = test_root("bgi-repo-bridge-mark");
        let repo_json = root.join("repo_updated.json");
        fs::write(
            &repo_json,
            r#"{"indexes":[{"name":"js","hasUpdate":true,"children":[{"name":"demo","hasUpdate":true,"children":[{"name":"main","hasUpdate":true}]}]},{"name":"pathing","hasUpdate":true}]}"#,
        )
        .unwrap();

        assert!(mark_repo_bridge_path_updated(&repo_json, "js/demo").unwrap());
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&repo_json).unwrap()).unwrap();

        assert_eq!(value["indexes"][0]["hasUpdate"], true);
        assert_eq!(value["indexes"][0]["children"][0]["hasUpdate"], false);
        assert_eq!(
            value["indexes"][0]["children"][0]["children"][0]["hasUpdate"],
            false
        );
        assert_eq!(value["indexes"][1]["hasUpdate"], true);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bridge_index_nodes_flatten_repo_json_for_import_ui() {
        let nodes = repo_bridge_index_nodes_from_json(
            r#"{"indexes":[{"name":"js","type":"directory","hasUpdate":true,"children":[{"name":"demo","type":"directory","lastUpdated":"2024-01-01","children":[{"name":"main.js","type":"file"}]}]},{"name":"misc","type":"directory","children":[]}]}"#,
        )
        .unwrap();

        assert_eq!(nodes.len(), 4);
        assert_eq!(nodes[0].path, "js");
        assert!(nodes[0].importable);
        assert_eq!(nodes[1].path, "js/demo");
        assert_eq!(nodes[1].depth, 1);
        assert_eq!(nodes[1].last_updated.as_deref(), Some("2024-01-01"));
        assert!(nodes[1].importable);
        assert_eq!(nodes[2].path, "js/demo/main.js");
        assert!(!nodes[2].importable);
        assert_eq!(nodes[3].path, "misc");
        assert!(!nodes[3].importable);
    }

    #[test]
    fn bridge_clear_update_copies_original_repo_json() {
        let root = test_root("bgi-repo-bridge-clear");
        let repo = root.join("Repos/repo");
        fs::create_dir_all(repo.join("nested")).unwrap();
        fs::write(repo.join("nested/repo.json"), "{\"original\":true}").unwrap();
        fs::write(repo.join("repo_updated.json"), "{\"updated\":true}").unwrap();

        let target = clear_repo_bridge_update(&repo).unwrap();

        assert_eq!(target, repo.join("repo_updated.json"));
        assert_eq!(
            fs::read_to_string(repo.join("repo_updated.json")).unwrap(),
            "{\"original\":true}"
        );

        fs::remove_dir_all(root).unwrap();
    }
}
