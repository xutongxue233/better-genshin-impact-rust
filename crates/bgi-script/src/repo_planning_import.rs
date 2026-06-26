use super::repo_paths::{
    base64_decode, expand_top_level_paths, first_folder_and_remaining_path,
    merge_subscription_paths, normalize_subscription_paths, path_mapper, percent_decode,
};
use super::repo_planning_layout::script_repo_layout;
use super::repo_planning_url::sanitize_folder_name;
use super::{
    ScriptImportPlan, ScriptImportUriPlan, ScriptRepoPathKind, ScriptRepoPathTarget,
    ScriptRepoZipImportPlan, DEFAULT_REPO_FOLDER_NAME, IMPORT_URI_PREFIX, REPOS_DIR,
    REPOS_TEMP_DIR,
};
use bgi_core::config::ScriptConfig;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn parse_import_uri(
    uri: &str,
    from_clipboard: bool,
) -> std::result::Result<Option<ScriptImportUriPlan>, String> {
    let trimmed = uri.trim();
    if !trimmed
        .get(..IMPORT_URI_PREFIX.len())
        .map(|prefix| prefix.eq_ignore_ascii_case(IMPORT_URI_PREFIX))
        .unwrap_or(false)
    {
        return Ok(None);
    }

    let query = trimmed
        .split_once('?')
        .map(|(_, query)| query)
        .unwrap_or_default();
    let encoded_import = query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(key, _)| key.eq_ignore_ascii_case("import"))
        .map(|(_, value)| value)
        .ok_or_else(|| "missing import query parameter".to_string())?;

    let decoded_once = percent_decode(encoded_import)?;
    let base64_decoded = base64_decode(&decoded_once)?;
    let decoded_utf8 = String::from_utf8(base64_decoded)
        .map_err(|_| "import payload must be UTF-8".to_string())?;
    let path_json = percent_decode(&decoded_utf8)?;
    let paths: Vec<String> = serde_json::from_str(&path_json)
        .map_err(|source| format!("import path JSON is invalid: {source}"))?;
    if paths.is_empty() {
        return Err("import path JSON is empty".to_string());
    }

    Ok(Some(ScriptImportUriPlan {
        uri: trimmed.to_string(),
        path_json,
        paths: normalize_subscription_paths(paths),
        clear_clipboard_after_import: from_clipboard,
    }))
}

pub fn script_import_plan(
    app_root: impl AsRef<Path>,
    repo_path: impl Into<PathBuf>,
    config: &ScriptConfig,
    folder_mapping: &BTreeMap<String, String>,
    paths: impl IntoIterator<Item = impl Into<String>>,
    existing_subscriptions: impl IntoIterator<Item = impl Into<String>>,
    available_top_level_children: &BTreeMap<String, Vec<String>>,
) -> ScriptImportPlan {
    let app_root = app_root.as_ref();
    let repo_path = repo_path.into();
    let layout = script_repo_layout(app_root, config, folder_mapping);
    let paths = normalize_subscription_paths(paths);
    let expanded_paths = expand_top_level_paths(paths.clone(), available_top_level_children);
    let mapper = path_mapper(app_root);
    let mut targets = Vec::new();
    let mut unknown_paths = Vec::new();

    for path in &expanded_paths {
        let (prefix, remaining_path) = first_folder_and_remaining_path(path);
        let Some(kind) = ScriptRepoPathKind::from_prefix(&prefix) else {
            unknown_paths.push(path.clone());
            continue;
        };
        let Some(user_root) = mapper.get(&kind) else {
            unknown_paths.push(path.clone());
            continue;
        };
        targets.push(ScriptRepoPathTarget {
            source_path: path.clone(),
            kind,
            remaining_path: remaining_path.clone(),
            destination_path: user_root.join(&remaining_path),
            preserves_saved_files: kind == ScriptRepoPathKind::Js,
            resolves_js_dependencies: kind == ScriptRepoPathKind::Js,
        });
    }

    ScriptImportPlan {
        repo_path,
        paths: paths.clone(),
        expanded_paths,
        targets,
        unknown_paths,
        subscription_file_path: layout.subscription_file_path,
        merged_subscriptions: merge_subscription_paths(existing_subscriptions, paths),
    }
}

pub fn zip_import_plan(
    app_root: impl AsRef<Path>,
    zip_path: impl Into<PathBuf>,
    target_folder_name: Option<&str>,
) -> ScriptRepoZipImportPlan {
    let app_root = app_root.as_ref();
    let repos_path = app_root.join(REPOS_DIR);
    let repos_temp_path = app_root.join(REPOS_TEMP_DIR);
    let target_folder_name = target_folder_name
        .map(sanitize_folder_name)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| DEFAULT_REPO_FOLDER_NAME.to_string());
    let target_path = repos_path.join(&target_folder_name);
    ScriptRepoZipImportPlan {
        zip_path: zip_path.into(),
        temp_unzip_dir: repos_temp_path.join("importZipFile"),
        repo_updated_json_path: target_path.join("repo_updated.json"),
        target_folder_name,
        target_path,
        overlap_threshold: 0.5,
    }
}
