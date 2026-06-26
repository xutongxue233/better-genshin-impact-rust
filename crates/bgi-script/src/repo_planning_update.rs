use super::repo_paths::normalize_subscription_paths;
use super::repo_planning_layout::script_repo_layout;
use super::repo_planning_url::{repo_folder_name, resolve_repo_url};
use super::{ScriptRepoGitUpdatePlan, ScriptRepoUpdatePlan, DEFAULT_REPO_FOLDER_NAME, REPOS_DIR};
use bgi_core::config::ScriptConfig;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub fn script_repo_update_plan(
    app_root: impl AsRef<Path>,
    config: &ScriptConfig,
    folder_mapping: &BTreeMap<String, String>,
    subscribed_paths: impl IntoIterator<Item = impl Into<String>>,
    manual: bool,
) -> ScriptRepoUpdatePlan {
    let layout = script_repo_layout(app_root, config, folder_mapping);
    let repo_url = resolve_repo_url(config);
    let subscribed_paths = normalize_subscription_paths(subscribed_paths);
    let enabled = manual || config.auto_update_subscribed_scripts;
    let reason = if repo_url.is_none() {
        Some("repo_url_unresolved")
    } else if !enabled {
        Some("auto_update_disabled")
    } else if subscribed_paths.is_empty() {
        Some("no_subscribed_paths")
    } else {
        None
    };

    ScriptRepoUpdatePlan {
        enabled: reason.is_none(),
        reason,
        manual,
        repo_url,
        repo_folder_name: layout
            .center_repo_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(DEFAULT_REPO_FOLDER_NAME)
            .to_string(),
        repo_path: layout.center_repo_path,
        subscription_file_path: layout.subscription_file_path,
        subscribed_paths,
    }
}

pub fn git_update_plan(
    app_root: impl AsRef<Path>,
    repo_url: impl Into<String>,
    folder_mapping: &BTreeMap<String, String>,
) -> ScriptRepoGitUpdatePlan {
    let app_root = app_root.as_ref();
    let repo_url = repo_url.into();
    let trimmed_url = repo_url.trim_end_matches('/').to_string();
    let repo_folder_name = repo_folder_name(Some(&trimmed_url), folder_mapping);
    let repos_path = app_root.join(REPOS_DIR);
    let repo_path = repos_path.join(&repo_folder_name);
    ScriptRepoGitUpdatePlan {
        repo_url: trimmed_url,
        branch: "release".to_string(),
        repos_path: repos_path.clone(),
        folder_mapping_path: repos_path.join("repo_folder_mapping.json"),
        repo_folder_name,
        repo_updated_json_path: repo_path.join("repo_updated.json"),
        repo_path,
        overlap_threshold: 0.5,
    }
}

pub fn repo_directory_paths(content: &str) -> std::result::Result<Vec<String>, serde_json::Error> {
    let value = serde_json::from_str::<serde_json::Value>(content)?;
    let mut paths = BTreeSet::new();
    collect_directory_paths(
        value.get("indexes").and_then(|value| value.as_array()),
        "",
        &mut paths,
    );
    Ok(paths.into_iter().collect())
}

pub fn calculate_repo_overlap_ratio(old_content: &str, new_content: &str) -> f64 {
    let Ok(old_paths) = repo_directory_paths(old_content) else {
        return -1.0;
    };
    let Ok(new_paths) = repo_directory_paths(new_content) else {
        return -1.0;
    };

    if old_paths.is_empty() && new_paths.is_empty() {
        return 1.0;
    }
    if old_paths.is_empty() || new_paths.is_empty() {
        return 0.0;
    }

    let old_paths = old_paths.into_iter().collect::<BTreeSet<_>>();
    let new_paths = new_paths.into_iter().collect::<BTreeSet<_>>();
    let intersection = old_paths.intersection(&new_paths).count();
    let min_count = old_paths.len().min(new_paths.len());
    if min_count == 0 {
        0.0
    } else {
        intersection as f64 / min_count as f64
    }
}

pub fn add_update_markers_to_new_repo(old_content: &str, new_content: &str) -> String {
    let Ok(old_json) = serde_json::from_str::<serde_json::Value>(old_content) else {
        return new_content.to_string();
    };
    let Ok(mut new_json) = serde_json::from_str::<serde_json::Value>(new_content) else {
        return new_content.to_string();
    };

    if let (Some(old_indexes), Some(new_indexes)) = (
        old_json.get("indexes").and_then(|value| value.as_array()),
        new_json
            .get_mut("indexes")
            .and_then(|value| value.as_array_mut()),
    ) {
        for new_index in new_indexes {
            mark_node_updates(new_index, old_indexes);
        }
    }

    serde_json::to_string_pretty(&new_json).unwrap_or_else(|_| new_content.to_string())
}

fn collect_directory_paths(
    nodes: Option<&Vec<serde_json::Value>>,
    prefix: &str,
    paths: &mut BTreeSet<String>,
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
        if object.get("type").and_then(|value| value.as_str()) != Some("directory") {
            continue;
        }

        let full_path = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{prefix}/{name}")
        };
        paths.insert(full_path.clone());

        collect_directory_paths(
            object.get("children").and_then(|value| value.as_array()),
            &full_path,
            paths,
        );
    }
}

fn mark_node_updates(new_node: &mut serde_json::Value, old_nodes: &[serde_json::Value]) -> bool {
    let Some(new_object) = new_node.as_object_mut() else {
        return false;
    };
    let Some(new_name) = new_object
        .get("name")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
    else {
        return false;
    };

    let old_node = old_nodes.iter().find(|node| {
        node.get("name")
            .and_then(|value| value.as_str())
            .map(|old_name| old_name == new_name)
            .unwrap_or(false)
    });

    let mut has_direct_update = false;
    let mut has_child_update = false;

    if let Some(old_node) = old_node {
        if is_truthy(old_node.get("hasUpdate")) {
            new_object.insert("hasUpdate".to_string(), serde_json::Value::Bool(true));
            has_direct_update = true;
        }

        let old_time = parse_last_updated(old_node.get("lastUpdated"));
        let new_time = parse_last_updated(new_object.get("lastUpdated"));
        if new_time > old_time {
            new_object.insert("hasUpdate".to_string(), serde_json::Value::Bool(true));
            has_direct_update = true;
        }
    } else {
        new_object.insert("hasUpdate".to_string(), serde_json::Value::Bool(true));
        has_direct_update = true;
    }

    let old_children = old_node
        .and_then(|node| node.get("children"))
        .and_then(|value| value.as_array())
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let parent_time = parse_last_updated(new_object.get("lastUpdated"));
    let mut newest_leaf_last_updated: Option<(Vec<i64>, String)> = None;

    if let Some(new_children) = new_object
        .get_mut("children")
        .and_then(|value| value.as_array_mut())
    {
        for new_child in new_children {
            let child_has_update = mark_node_updates(new_child, old_children);
            if !child_has_update {
                continue;
            }

            has_child_update = true;
            if !is_leaf_node(new_child) || !is_truthy(new_child.get("hasUpdate")) {
                continue;
            }

            has_direct_update = true;
            let child_time = parse_last_updated(new_child.get("lastUpdated"));
            if child_time > parent_time {
                if let Some(last_updated) = new_child
                    .get("lastUpdated")
                    .and_then(|value| value.as_str())
                    .map(ToOwned::to_owned)
                {
                    if newest_leaf_last_updated
                        .as_ref()
                        .map(|(current_time, _)| child_time > *current_time)
                        .unwrap_or(true)
                    {
                        newest_leaf_last_updated = Some((child_time, last_updated));
                    }
                }
            }
        }
    }

    if has_direct_update && has_child_update {
        new_object.insert("hasUpdate".to_string(), serde_json::Value::Bool(true));
    } else if has_direct_update {
        new_object.insert("hasUpdate".to_string(), serde_json::Value::Bool(true));
    }
    if let Some((_, last_updated)) = newest_leaf_last_updated {
        new_object.insert(
            "lastUpdated".to_string(),
            serde_json::Value::String(last_updated),
        );
    }

    has_direct_update || has_child_update
}

fn is_leaf_node(node: &serde_json::Value) -> bool {
    node.get("children")
        .and_then(|value| value.as_array())
        .map(|children| children.is_empty())
        .unwrap_or(true)
}

fn is_truthy(value: Option<&serde_json::Value>) -> bool {
    match value {
        Some(serde_json::Value::Bool(value)) => *value,
        Some(serde_json::Value::String(value)) => value.eq_ignore_ascii_case("true"),
        _ => false,
    }
}

fn parse_last_updated(value: Option<&serde_json::Value>) -> Vec<i64> {
    let Some(value) = value.and_then(|value| value.as_str()) else {
        return vec![0];
    };
    let mut numbers = Vec::new();
    let mut current = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
        } else if !current.is_empty() {
            if let Ok(number) = current.parse::<i64>() {
                numbers.push(number);
            }
            current.clear();
        }
    }
    if !current.is_empty() {
        if let Ok(number) = current.parse::<i64>() {
            numbers.push(number);
        }
    }
    if numbers.is_empty() {
        vec![0]
    } else {
        numbers
    }
}
