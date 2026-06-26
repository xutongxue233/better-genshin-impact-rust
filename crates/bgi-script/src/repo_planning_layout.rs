use super::repo_paths::path_mapper;
use super::repo_planning_url::{repo_folder_name, resolve_repo_url};
use super::{
    ScriptRepoLayout, OLD_CENTER_REPO_FOLDER_NAME, REPOS_DIR, REPOS_TEMP_DIR, SUBSCRIPTIONS_DIR,
};
use bgi_core::config::ScriptConfig;
use std::collections::BTreeMap;
use std::path::Path;

pub fn script_repo_layout(
    app_root: impl AsRef<Path>,
    config: &ScriptConfig,
    folder_mapping: &BTreeMap<String, String>,
) -> ScriptRepoLayout {
    let app_root = app_root.as_ref();
    let repos_path = app_root.join(REPOS_DIR);
    let repos_temp_path = app_root.join(REPOS_TEMP_DIR);
    let repo_url = resolve_repo_url(config);
    let repo_folder_name = repo_folder_name(repo_url.as_deref(), folder_mapping);
    let center_repo_path = repos_path.join(&repo_folder_name);
    let subscriptions_path = app_root.join(SUBSCRIPTIONS_DIR);
    let subscription_file_path = subscriptions_path.join(format!("{repo_folder_name}.json"));
    let repo_updated_json_path = center_repo_path.join("repo_updated.json");

    ScriptRepoLayout {
        repos_path: repos_path.clone(),
        repos_temp_path,
        center_repo_path,
        old_center_repo_path: repos_path.join(OLD_CENTER_REPO_FOLDER_NAME),
        folder_mapping_path: repos_path.join("repo_folder_mapping.json"),
        subscriptions_path,
        subscription_file_path,
        repo_updated_json_path,
        path_mapper: path_mapper(app_root),
    }
}
