use super::{read_repo_bridge_repo_json, ScriptRepoBridgeIndexNode};
use crate::ScriptRepoError;
use std::path::{Path, PathBuf};

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
            .unwrap_or({
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
