use bgi_core::config::ScriptConfig;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_REPO_FOLDER_NAME: &str = "bettergi-scripts-list";
pub const OLD_CENTER_REPO_FOLDER_NAME: &str = "bettergi-scripts-list-main";
pub const REPOS_DIR: &str = "Repos";
pub const REPOS_TEMP_DIR: &str = "Repos/Temp";
pub const SUBSCRIPTIONS_DIR: &str = "User/Subscriptions";
pub const IMPORT_URI_PREFIX: &str = "bettergi://script?import=";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ScriptRepoPathKind {
    Pathing,
    Js,
    Combat,
    Tcg,
}

impl ScriptRepoPathKind {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Pathing => "pathing",
            Self::Js => "js",
            Self::Combat => "combat",
            Self::Tcg => "tcg",
        }
    }

    pub fn user_relative_root(self) -> &'static str {
        match self {
            Self::Pathing => "User/AutoPathing",
            Self::Js => "User/JsScript",
            Self::Combat => "User/AutoFight",
            Self::Tcg => "User/AutoGeniusInvokation",
        }
    }

    pub fn from_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            value if value.eq_ignore_ascii_case("pathing") => Some(Self::Pathing),
            value if value.eq_ignore_ascii_case("js") => Some(Self::Js),
            value if value.eq_ignore_ascii_case("combat") => Some(Self::Combat),
            value if value.eq_ignore_ascii_case("tcg") => Some(Self::Tcg),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptRepoChannel {
    pub name: &'static str,
    pub url: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptRepoLayout {
    pub repos_path: PathBuf,
    pub repos_temp_path: PathBuf,
    pub center_repo_path: PathBuf,
    pub old_center_repo_path: PathBuf,
    pub folder_mapping_path: PathBuf,
    pub subscriptions_path: PathBuf,
    pub subscription_file_path: PathBuf,
    pub repo_updated_json_path: PathBuf,
    pub path_mapper: BTreeMap<ScriptRepoPathKind, PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptRepoUpdatePlan {
    pub enabled: bool,
    pub reason: Option<&'static str>,
    pub manual: bool,
    pub repo_url: Option<String>,
    pub repo_folder_name: String,
    pub repo_path: PathBuf,
    pub subscription_file_path: PathBuf,
    pub subscribed_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptImportUriPlan {
    pub uri: String,
    pub path_json: String,
    pub paths: Vec<String>,
    pub clear_clipboard_after_import: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptRepoPathTarget {
    pub source_path: String,
    pub kind: ScriptRepoPathKind,
    pub remaining_path: PathBuf,
    pub destination_path: PathBuf,
    pub preserves_saved_files: bool,
    pub resolves_js_dependencies: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptImportPlan {
    pub repo_path: PathBuf,
    pub paths: Vec<String>,
    pub expanded_paths: Vec<String>,
    pub targets: Vec<ScriptRepoPathTarget>,
    pub unknown_paths: Vec<String>,
    pub subscription_file_path: PathBuf,
    pub merged_subscriptions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScriptRepoZipImportPlan {
    pub zip_path: PathBuf,
    pub temp_unzip_dir: PathBuf,
    pub target_folder_name: String,
    pub target_path: PathBuf,
    pub repo_updated_json_path: PathBuf,
    pub overlap_threshold: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScriptRepoZipImportExecution {
    pub zip_path: PathBuf,
    pub repo_json_path: PathBuf,
    pub target_folder_name: String,
    pub target_path: PathBuf,
    pub repo_updated_json_path: PathBuf,
    pub best_overlap_ratio: Option<f64>,
    pub matched_existing_folder: Option<String>,
    pub old_repo_overlap_ratio: Option<f64>,
    pub marker_generated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScriptRepoGitUpdatePlan {
    pub repo_url: String,
    pub branch: String,
    pub repos_path: PathBuf,
    pub folder_mapping_path: PathBuf,
    pub repo_folder_name: String,
    pub repo_path: PathBuf,
    pub repo_updated_json_path: PathBuf,
    pub overlap_threshold: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScriptRepoGitUpdateExecution {
    pub repo_url: String,
    pub branch: String,
    pub repo_folder_name: String,
    pub repo_path: PathBuf,
    pub repo_updated_json_path: PathBuf,
    pub updated: bool,
    pub cloned: bool,
    pub remote_changed: bool,
    pub created_new_folder: bool,
    pub fallback_reclone: bool,
    pub marker_generated: bool,
    pub old_repo_overlap_ratio: Option<f64>,
    pub current_commit: Option<String>,
    pub remote_commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptRepoGitCheckout {
    pub repo_path: PathBuf,
    pub source_path: String,
    pub git_tree_path: String,
    pub destination_path: PathBuf,
    pub is_directory: bool,
    pub files_written: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptRepoGitCommandOutput {
    pub stdout: String,
    pub stderr: String,
    #[serde(skip_serializing)]
    pub stdout_bytes: Vec<u8>,
}

pub trait ScriptRepoGitRunner {
    fn run_git(
        &mut self,
        cwd: Option<&Path>,
        args: &[String],
    ) -> Result<ScriptRepoGitCommandOutput>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemGitRunner {
    git_program: PathBuf,
}

impl Default for SystemGitRunner {
    fn default() -> Self {
        Self {
            git_program: PathBuf::from("git"),
        }
    }
}

impl SystemGitRunner {
    pub fn new(git_program: impl Into<PathBuf>) -> Self {
        Self {
            git_program: git_program.into(),
        }
    }
}

impl ScriptRepoGitRunner for SystemGitRunner {
    fn run_git(
        &mut self,
        cwd: Option<&Path>,
        args: &[String],
    ) -> Result<ScriptRepoGitCommandOutput> {
        let mut command = Command::new(&self.git_program);
        command.args(args);
        if let Some(cwd) = cwd {
            command.current_dir(cwd);
        }
        let output = command.output().map_err(|source| ScriptRepoError::GitIo {
            cwd: cwd.map(Path::to_path_buf),
            args: args.to_vec(),
            source,
        })?;
        if !output.status.success() {
            return Err(ScriptRepoError::GitCommand {
                cwd: cwd.map(Path::to_path_buf),
                args: args.to_vec(),
                status: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }
        Ok(ScriptRepoGitCommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            stdout_bytes: output.stdout,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ScriptRepoError {
    #[error("script repo IO failed at {path:?}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("script repo JSON failed at {path:?}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("script repo source path does not exist: {0:?}")]
    MissingSource(PathBuf),
    #[error("script repo target has unsupported file kind: {0:?}")]
    UnsupportedSource(PathBuf),
    #[error("script repo zip failed at {path:?}: {source}")]
    Zip {
        path: PathBuf,
        source: zip::result::ZipError,
    },
    #[error(
        "script repo git command failed in {cwd:?}: git {args:?}; status={status}; stderr={stderr}"
    )]
    GitCommand {
        cwd: Option<PathBuf>,
        args: Vec<String>,
        status: i32,
        stdout: String,
        stderr: String,
    },
    #[error("script repo git command could not start in {cwd:?}: git {args:?}: {source}")]
    GitIo {
        cwd: Option<PathBuf>,
        args: Vec<String>,
        source: io::Error,
    },
    #[error("script repo git remote branch was not found: {0}")]
    MissingGitBranch(String),
    #[error("script repo zip entry escapes extraction root: {0}")]
    ZipEntryEscapesRoot(String),
    #[error("script repo archive does not contain repo.json under {0:?}")]
    MissingRepoJson(PathBuf),
    #[error("script repo path escapes its root: {path:?} outside {root:?}")]
    PathEscapesRoot { path: PathBuf, root: PathBuf },
}

pub type Result<T> = std::result::Result<T, ScriptRepoError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptRepoImportExecution {
    pub imported_targets: Vec<ScriptRepoPathTarget>,
    pub skipped_unknown_paths: Vec<String>,
    pub subscription_file_path: PathBuf,
    pub subscriptions: Vec<String>,
    pub dependency_files_copied: Vec<PathBuf>,
    pub git_checkouts: Vec<ScriptRepoGitCheckout>,
}

pub fn script_repo_channels() -> Vec<ScriptRepoChannel> {
    vec![
        ScriptRepoChannel {
            name: "CNB",
            url: "https://cnb.cool/bettergi/bettergi-scripts-list",
        },
        ScriptRepoChannel {
            name: "GitCode",
            url: "https://gitcode.com/huiyadanli/bettergi-scripts-list",
        },
        ScriptRepoChannel {
            name: "GitHub",
            url: "https://github.com/babalae/bettergi-scripts-list",
        },
    ]
}

pub fn resolve_repo_url(config: &ScriptConfig) -> Option<String> {
    if config.selected_channel_name.is_empty() {
        return Some(script_repo_channels()[0].url.to_string());
    }
    if config.selected_channel_name == "自定义" {
        let custom_url = config.custom_repo_url.trim();
        if !custom_url.is_empty() && custom_url != "https://example.com/custom-repo" {
            return Some(custom_url.to_string());
        }
        return None;
    }

    script_repo_channels()
        .into_iter()
        .find(|channel| channel.name == config.selected_channel_name)
        .map(|channel| channel.url.to_string())
        .or_else(|| Some("https://cnb.cool/bettergi/bettergi-scripts-list".to_string()))
}

pub fn repo_folder_name(
    repo_url: Option<&str>,
    folder_mapping: &BTreeMap<String, String>,
) -> String {
    let Some(repo_url) = repo_url.filter(|value| !value.trim().is_empty()) else {
        return DEFAULT_REPO_FOLDER_NAME.to_string();
    };
    let trimmed_url = repo_url.trim_end_matches('/');
    if let Some(saved) = folder_mapping
        .get(trimmed_url)
        .filter(|value| !value.trim().is_empty())
    {
        return saved.clone();
    }
    derive_base_folder_name(trimmed_url)
}

pub fn derive_base_folder_name(repo_url: &str) -> String {
    let trimmed_url = repo_url.trim_end_matches('/');
    let last_segment = trimmed_url
        .rsplit('/')
        .find(|segment| !segment.trim().is_empty())
        .unwrap_or(DEFAULT_REPO_FOLDER_NAME);
    let without_git = if last_segment
        .get(last_segment.len().saturating_sub(4)..)
        .map(|suffix| suffix.eq_ignore_ascii_case(".git"))
        .unwrap_or(false)
    {
        &last_segment[..last_segment.len().saturating_sub(4)]
    } else {
        last_segment
    };
    sanitize_folder_name(without_git)
}

pub fn sanitize_folder_name(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || ch.is_control()
            {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        DEFAULT_REPO_FOLDER_NAME.to_string()
    } else {
        sanitized
    }
}

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

pub fn execute_zip_repo_import(
    plan: &ScriptRepoZipImportPlan,
) -> Result<ScriptRepoZipImportExecution> {
    let cleanup_root = plan
        .temp_unzip_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| plan.temp_unzip_dir.clone());
    let result = execute_zip_repo_import_inner(plan);
    let cleanup_result = remove_existing(&cleanup_root);

    match (result, cleanup_result) {
        (Ok(execution), Ok(())) => Ok(execution),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

pub fn execute_git_repo_update(
    plan: &ScriptRepoGitUpdatePlan,
    runner: &mut impl ScriptRepoGitRunner,
) -> Result<ScriptRepoGitUpdateExecution> {
    fs::create_dir_all(&plan.repos_path).map_err(|source| ScriptRepoError::Io {
        path: plan.repos_path.clone(),
        source,
    })?;
    execute_git_repo_update_inner(plan, runner)
}

pub fn checkout_git_repo_path(
    runner: &mut impl ScriptRepoGitRunner,
    repo_path: impl AsRef<Path>,
    source_path: &str,
    destination_path: impl AsRef<Path>,
    from_repo_subdir: bool,
) -> Result<Option<ScriptRepoGitCheckout>> {
    let repo_path = repo_path.as_ref();
    let destination_path = destination_path.as_ref();
    let normalized_source = normalize_repo_path(source_path);
    if normalized_source.is_empty() {
        return Ok(None);
    }
    let git_tree_path = if from_repo_subdir {
        normalize_repo_path(&format!("repo/{normalized_source}"))
    } else {
        normalized_source.clone()
    };
    let object = format!("HEAD:{git_tree_path}");
    let Ok(kind) = git_object_kind(runner, repo_path, &object) else {
        return Ok(None);
    };

    remove_existing(destination_path)?;
    let mut files_written = Vec::new();
    if kind == "blob" {
        let output = runner.run_git(
            Some(repo_path),
            &git_with_owned_args(["show", object.as_str()]),
        )?;
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).map_err(|source| ScriptRepoError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(destination_path, output.stdout_bytes).map_err(|source| ScriptRepoError::Io {
            path: destination_path.to_path_buf(),
            source,
        })?;
        files_written.push(destination_path.to_path_buf());
    } else if kind == "tree" {
        checkout_git_tree(
            runner,
            repo_path,
            &git_tree_path,
            destination_path,
            &mut files_written,
        )?;
    } else {
        return Ok(None);
    }

    Ok(Some(ScriptRepoGitCheckout {
        repo_path: repo_path.to_path_buf(),
        source_path: normalized_source,
        git_tree_path,
        destination_path: destination_path.to_path_buf(),
        is_directory: kind == "tree",
        files_written,
    }))
}

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

pub fn execute_file_repo_import(plan: &ScriptImportPlan) -> Result<ScriptRepoImportExecution> {
    execute_repo_import_with_git(plan, None::<&mut SystemGitRunner>)
}

pub fn execute_repo_import_with_git<R: ScriptRepoGitRunner>(
    plan: &ScriptImportPlan,
    mut git_runner: Option<&mut R>,
) -> Result<ScriptRepoImportExecution> {
    let mut imported_targets = Vec::new();
    let mut dependency_files_copied = Vec::new();
    let mut git_checkouts = Vec::new();
    let mut temp_materialized_sources = Vec::new();
    let repo_root = canonical_or_lexical(&plan.repo_path);
    let use_git_repo = git_runner
        .as_deref_mut()
        .map(|runner| is_git_worktree(runner, &repo_root))
        .unwrap_or(false);

    for target in &plan.targets {
        let source = validate_child_path(&repo_root, &target.source_path)?;
        let destination = target.destination_path.clone();

        let materialized_source = if use_git_repo {
            materialize_git_repo_source(
                git_runner
                    .as_deref_mut()
                    .expect("git runner present when git repo is enabled"),
                &repo_root,
                &target.source_path,
            )?
        } else {
            None
        };
        let source_for_metadata = materialized_source
            .as_ref()
            .map(|source| source.path.as_path())
            .unwrap_or(&source);

        let saved_files = if target.preserves_saved_files && destination.exists() {
            read_saved_files_for_source(source_for_metadata)?
        } else {
            Vec::new()
        };
        let backups = backup_saved_files(&destination, &saved_files)?;

        remove_existing(&destination)?;
        if use_git_repo {
            if let Some(checkout) = checkout_git_repo_path(
                git_runner
                    .as_deref_mut()
                    .expect("git runner present when git repo is enabled"),
                &repo_root,
                &target.source_path,
                &destination,
                true,
            )? {
                git_checkouts.push(checkout);
            } else {
                return Err(ScriptRepoError::MissingSource(PathBuf::from(
                    &target.source_path,
                )));
            }
        } else {
            copy_repo_path(&source, &destination)?;
        }
        restore_saved_files(&destination, backups)?;

        if target.resolves_js_dependencies {
            if use_git_repo {
                let copied = copy_js_package_dependencies_from_git(
                    git_runner
                        .as_deref_mut()
                        .expect("git runner present when git repo is enabled"),
                    &repo_root,
                    &destination,
                )?;
                dependency_files_copied.extend(copied);
            } else {
                dependency_files_copied
                    .extend(copy_js_package_dependencies(&repo_root, &destination)?);
            }
        }

        imported_targets.push(target.clone());
        if let Some(source) = materialized_source {
            temp_materialized_sources.push(source.root);
        }
    }

    for temp_source in temp_materialized_sources {
        remove_existing(&temp_source)?;
    }

    write_subscription_file(&plan.subscription_file_path, &plan.merged_subscriptions)?;

    Ok(ScriptRepoImportExecution {
        imported_targets,
        skipped_unknown_paths: plan.unknown_paths.clone(),
        subscription_file_path: plan.subscription_file_path.clone(),
        subscriptions: plan.merged_subscriptions.clone(),
        dependency_files_copied,
        git_checkouts,
    })
}

fn execute_git_repo_update_inner(
    plan: &ScriptRepoGitUpdatePlan,
    runner: &mut impl ScriptRepoGitRunner,
) -> Result<ScriptRepoGitUpdateExecution> {
    let mut execution = ScriptRepoGitUpdateExecution {
        repo_url: plan.repo_url.clone(),
        branch: plan.branch.clone(),
        repo_folder_name: plan.repo_folder_name.clone(),
        repo_path: plan.repo_path.clone(),
        repo_updated_json_path: plan.repo_updated_json_path.clone(),
        updated: false,
        cloned: false,
        remote_changed: false,
        created_new_folder: false,
        fallback_reclone: false,
        marker_generated: false,
        old_repo_overlap_ratio: None,
        current_commit: None,
        remote_commit: None,
    };

    let mut repo_path = plan.repo_path.clone();
    let mut repo_folder_name = plan.repo_folder_name.clone();
    let old_repo_content = read_existing_repo_content(&repo_path)?;

    if !repo_path.exists() {
        clone_release_repo(runner, &plan.repo_url, &repo_path, &plan.branch)?;
        write_folder_mapping(&plan.folder_mapping_path, &plan.repo_url, &repo_folder_name)?;
        execution.updated = true;
        execution.cloned = true;
    } else if !is_git_worktree(runner, &repo_path) {
        remove_existing(&repo_path)?;
        clone_release_repo(runner, &plan.repo_url, &repo_path, &plan.branch)?;
        write_folder_mapping(&plan.folder_mapping_path, &plan.repo_url, &repo_folder_name)?;
        execution.updated = true;
        execution.cloned = true;
        execution.fallback_reclone = true;
    } else {
        let origin_url = git_origin_url(runner, &repo_path)?;
        if origin_url.trim_end_matches('/') != plan.repo_url {
            execution.remote_changed = true;
            let temp_path = plan
                .repos_path
                .join(format!("{repo_folder_name}_temp_{}", unique_suffix()));
            clone_release_repo(runner, &plan.repo_url, &temp_path, &plan.branch)?;
            let new_content = find_file_named(&temp_path, "repo.json")?
                .and_then(|path| fs::read_to_string(path).ok());
            let overlap = old_repo_content
                .as_deref()
                .zip(new_content.as_deref())
                .map(|(old, new)| calculate_repo_overlap_ratio(old, new))
                .unwrap_or(0.0);
            execution.old_repo_overlap_ratio = Some(overlap);

            if overlap >= plan.overlap_threshold {
                remove_existing(&repo_path)?;
                rename_or_copy_directory(&temp_path, &repo_path)?;
                write_folder_mapping(&plan.folder_mapping_path, &plan.repo_url, &repo_folder_name)?;
            } else {
                let base_name = derive_base_folder_name(&plan.repo_url);
                repo_folder_name = generate_unique_folder_name(&plan.repos_path, &base_name);
                repo_path = plan.repos_path.join(&repo_folder_name);
                rename_or_copy_directory(&temp_path, &repo_path)?;
                write_folder_mapping(&plan.folder_mapping_path, &plan.repo_url, &repo_folder_name)?;
                execution.created_new_folder = true;
            }
            execution.updated = true;
            execution.cloned = true;
        } else {
            let remote_commit = git_remote_branch_sha(runner, &plan.repo_url, &plan.branch)?;
            let current_commit = git_current_branch_sha(runner, &repo_path, &plan.branch).ok();
            execution.remote_commit = Some(remote_commit.clone());
            execution.current_commit = current_commit.clone();
            if current_commit.as_deref() == Some(remote_commit.as_str()) {
                checkout_repo_json(runner, &repo_path, &plan.branch)?;
            } else {
                remove_existing(&repo_path)?;
                clone_release_repo(runner, &plan.repo_url, &repo_path, &plan.branch)?;
                write_folder_mapping(&plan.folder_mapping_path, &plan.repo_url, &repo_folder_name)?;
                execution.updated = true;
                execution.cloned = true;
            }
        }
    }

    execution.repo_folder_name = repo_folder_name;
    execution.repo_path = repo_path.clone();
    execution.repo_updated_json_path = repo_path.join("repo_updated.json");
    let new_repo_content =
        find_file_named(&repo_path, "repo.json")?.and_then(|path| fs::read_to_string(path).ok());
    if let Some(new_repo_content) = new_repo_content {
        let updated_content = if let Some(old_content) = old_repo_content.as_deref() {
            let overlap = calculate_repo_overlap_ratio(old_content, &new_repo_content);
            execution.old_repo_overlap_ratio = Some(overlap);
            if overlap >= plan.overlap_threshold {
                execution.marker_generated = true;
                add_update_markers_to_new_repo(old_content, &new_repo_content)
            } else {
                new_repo_content
            }
        } else {
            new_repo_content
        };
        fs::write(&execution.repo_updated_json_path, updated_content).map_err(|source| {
            ScriptRepoError::Io {
                path: execution.repo_updated_json_path.clone(),
                source,
            }
        })?;
    }

    Ok(execution)
}

fn execute_zip_repo_import_inner(
    plan: &ScriptRepoZipImportPlan,
) -> Result<ScriptRepoZipImportExecution> {
    let cleanup_root = plan
        .temp_unzip_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| plan.temp_unzip_dir.clone());
    remove_existing(&cleanup_root)?;
    fs::create_dir_all(&plan.temp_unzip_dir).map_err(|source| ScriptRepoError::Io {
        path: plan.temp_unzip_dir.clone(),
        source,
    })?;

    extract_zip_archive(&plan.zip_path, &plan.temp_unzip_dir)?;

    let repo_json_path = find_file_named(&plan.temp_unzip_dir, "repo.json")?
        .ok_or_else(|| ScriptRepoError::MissingRepoJson(plan.temp_unzip_dir.clone()))?;
    let repo_dir = repo_json_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| plan.temp_unzip_dir.clone());
    let repo_json_relative = repo_json_path
        .strip_prefix(&repo_dir)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| PathBuf::from("repo.json"));
    let new_repo_content =
        fs::read_to_string(&repo_json_path).map_err(|source| ScriptRepoError::Io {
            path: repo_json_path.clone(),
            source,
        })?;

    let repos_path = plan
        .target_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(REPOS_DIR));
    let best_match = best_matching_existing_repo(&repos_path, &new_repo_content);
    let best_overlap_ratio = best_match.as_ref().map(|candidate| candidate.overlap_ratio);
    let matched_existing_folder = best_match
        .as_ref()
        .map(|candidate| candidate.folder_name.clone());

    let mut target_folder_name = plan.target_folder_name.clone();
    let mut target_path = repos_path.join(&target_folder_name);
    let mut old_repo_content = None;

    if let Some(candidate) = best_match
        .as_ref()
        .filter(|candidate| candidate.overlap_ratio >= plan.overlap_threshold)
    {
        target_folder_name = candidate.folder_name.clone();
        target_path = candidate.path.clone();
        old_repo_content = read_existing_repo_content(&target_path)?;
        remove_existing(&target_path)?;
    } else if best_match.is_some() {
        if plan.target_path.exists() {
            target_folder_name = generate_unique_folder_name(&repos_path, &plan.target_folder_name);
            target_path = repos_path.join(&target_folder_name);
        }
    } else if target_path.exists() {
        old_repo_content = read_existing_repo_content(&target_path)?;
        remove_existing(&target_path)?;
    }

    copy_directory(&repo_dir, &target_path)?;

    let repo_updated_json_path = target_path.join("repo_updated.json");
    let mut marker_generated = false;
    let mut old_repo_overlap_ratio = None;
    let updated_content = if let Some(old_content) = old_repo_content.as_deref() {
        let overlap = calculate_repo_overlap_ratio(old_content, &new_repo_content);
        old_repo_overlap_ratio = Some(overlap);
        if overlap >= plan.overlap_threshold {
            marker_generated = true;
            add_update_markers_to_new_repo(old_content, &new_repo_content)
        } else {
            new_repo_content.clone()
        }
    } else {
        new_repo_content.clone()
    };

    fs::write(&repo_updated_json_path, updated_content).map_err(|source| ScriptRepoError::Io {
        path: repo_updated_json_path.clone(),
        source,
    })?;

    Ok(ScriptRepoZipImportExecution {
        zip_path: plan.zip_path.clone(),
        repo_json_path: target_path.join(repo_json_relative),
        target_folder_name,
        target_path,
        repo_updated_json_path,
        best_overlap_ratio,
        matched_existing_folder,
        old_repo_overlap_ratio,
        marker_generated,
    })
}

#[derive(Debug, Clone)]
struct ExistingRepoCandidate {
    folder_name: String,
    path: PathBuf,
    overlap_ratio: f64,
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

fn extract_zip_archive(zip_path: &Path, destination: &Path) -> Result<()> {
    let file = File::open(zip_path).map_err(|source| ScriptRepoError::Io {
        path: zip_path.to_path_buf(),
        source,
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|source| ScriptRepoError::Zip {
        path: zip_path.to_path_buf(),
        source,
    })?;
    let destination_root = lexical_normalize(destination);

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|source| ScriptRepoError::Zip {
                path: zip_path.to_path_buf(),
                source,
            })?;
        let Some(enclosed_name) = entry.enclosed_name().map(|path| path.to_path_buf()) else {
            return Err(ScriptRepoError::ZipEntryEscapesRoot(
                entry.name().to_string(),
            ));
        };
        let output_path = lexical_normalize(&destination_root.join(enclosed_name));
        if !path_starts_with(&output_path, &destination_root) {
            return Err(ScriptRepoError::ZipEntryEscapesRoot(
                entry.name().to_string(),
            ));
        }

        if entry.is_dir() {
            fs::create_dir_all(&output_path).map_err(|source| ScriptRepoError::Io {
                path: output_path.clone(),
                source,
            })?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|source| ScriptRepoError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let mut output = File::create(&output_path).map_err(|source| ScriptRepoError::Io {
            path: output_path.clone(),
            source,
        })?;
        io::copy(&mut entry, &mut output).map_err(|source| ScriptRepoError::Io {
            path: output_path,
            source,
        })?;
    }

    Ok(())
}

fn find_file_named(root: &Path, file_name: &str) -> Result<Option<PathBuf>> {
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

fn read_existing_repo_content(target_path: &Path) -> Result<Option<String>> {
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

fn best_matching_existing_repo(
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

fn generate_unique_folder_name(repos_path: &Path, base_name: &str) -> String {
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

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("{:x}", duration.as_nanos()))
        .unwrap_or_else(|_| "0".to_string())
}

fn git_args(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_string()).collect()
}

fn git_with_owned_args(args: impl IntoIterator<Item = impl Into<String>>) -> Vec<String> {
    args.into_iter().map(Into::into).collect()
}

fn clone_release_repo(
    runner: &mut impl ScriptRepoGitRunner,
    repo_url: &str,
    repo_path: &Path,
    branch: &str,
) -> Result<()> {
    remove_existing(repo_path)?;
    fs::create_dir_all(repo_path).map_err(|source| ScriptRepoError::Io {
        path: repo_path.to_path_buf(),
        source,
    })?;
    runner.run_git(Some(repo_path), &git_args(&["init"]))?;
    runner.run_git(
        Some(repo_path),
        &git_with_owned_args(["remote", "add", "origin", repo_url]),
    )?;
    runner.run_git(
        Some(repo_path),
        &git_with_owned_args([
            "-c",
            "http.proxy=",
            "-c",
            "https.proxy=",
            "fetch",
            "--depth",
            "1",
            "--no-tags",
            "origin",
            &format!("+refs/heads/{branch}:refs/remotes/origin/{branch}"),
        ]),
    )?;
    runner.run_git(
        Some(repo_path),
        &git_with_owned_args(["branch", "-f", branch, &format!("origin/{branch}")]),
    )?;
    runner.run_git(
        Some(repo_path),
        &git_with_owned_args(["symbolic-ref", "HEAD", &format!("refs/heads/{branch}")]),
    )?;
    checkout_repo_json(runner, repo_path, branch)?;
    Ok(())
}

fn checkout_repo_json(
    runner: &mut impl ScriptRepoGitRunner,
    repo_path: &Path,
    branch: &str,
) -> Result<()> {
    let output = runner.run_git(
        Some(repo_path),
        &git_with_owned_args(["show", &format!("{branch}:repo.json")]),
    )?;
    let repo_json_path = repo_path.join("repo.json");
    fs::write(&repo_json_path, output.stdout_bytes).map_err(|source| ScriptRepoError::Io {
        path: repo_json_path,
        source,
    })
}

fn git_object_kind(
    runner: &mut impl ScriptRepoGitRunner,
    repo_path: &Path,
    object: &str,
) -> Result<String> {
    runner
        .run_git(
            Some(repo_path),
            &git_with_owned_args(["cat-file", "-t", object]),
        )
        .map(|output| output.stdout.trim().to_string())
}

fn checkout_git_tree(
    runner: &mut impl ScriptRepoGitRunner,
    repo_path: &Path,
    tree_path: &str,
    destination_path: &Path,
    files_written: &mut Vec<PathBuf>,
) -> Result<()> {
    fs::create_dir_all(destination_path).map_err(|source| ScriptRepoError::Io {
        path: destination_path.to_path_buf(),
        source,
    })?;
    let output = runner.run_git(
        Some(repo_path),
        &git_with_owned_args(["ls-tree", &format!("HEAD:{tree_path}")]),
    )?;
    for line in output.stdout.lines().filter(|line| !line.trim().is_empty()) {
        let Some(entry) = parse_git_ls_tree_line(line) else {
            continue;
        };
        let child_tree_path = normalize_repo_path(&format!("{tree_path}/{}", entry.name));
        let child_destination = destination_path.join(&entry.name);
        if entry.kind == "blob" {
            let content = runner.run_git(
                Some(repo_path),
                &git_with_owned_args(["show", &format!("HEAD:{child_tree_path}")]),
            )?;
            if let Some(parent) = child_destination.parent() {
                fs::create_dir_all(parent).map_err(|source| ScriptRepoError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            fs::write(&child_destination, content.stdout_bytes).map_err(|source| {
                ScriptRepoError::Io {
                    path: child_destination.clone(),
                    source,
                }
            })?;
            files_written.push(child_destination);
        } else if entry.kind == "tree" {
            checkout_git_tree(
                runner,
                repo_path,
                &child_tree_path,
                &child_destination,
                files_written,
            )?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitTreeEntry {
    kind: String,
    name: String,
}

fn parse_git_ls_tree_line(line: &str) -> Option<GitTreeEntry> {
    let (metadata, name) = line.split_once('\t')?;
    let mut parts = metadata.split_whitespace();
    let _mode = parts.next()?;
    let kind = parts.next()?.to_string();
    let _sha = parts.next()?;
    Some(GitTreeEntry {
        kind,
        name: name.to_string(),
    })
}

fn is_git_worktree(runner: &mut impl ScriptRepoGitRunner, repo_path: &Path) -> bool {
    if !repo_path.exists() || repo_path.join("repo").is_dir() {
        return false;
    }
    runner
        .run_git(
            Some(repo_path),
            &git_args(&["rev-parse", "--is-inside-work-tree"]),
        )
        .map(|output| output.stdout.trim() == "true")
        .unwrap_or(false)
}

fn git_origin_url(runner: &mut impl ScriptRepoGitRunner, repo_path: &Path) -> Result<String> {
    runner
        .run_git(Some(repo_path), &git_args(&["remote", "get-url", "origin"]))
        .map(|output| output.stdout.trim().to_string())
}

fn git_current_branch_sha(
    runner: &mut impl ScriptRepoGitRunner,
    repo_path: &Path,
    branch: &str,
) -> Result<String> {
    runner
        .run_git(Some(repo_path), &git_with_owned_args(["rev-parse", branch]))
        .map(|output| output.stdout.trim().to_string())
}

fn git_remote_branch_sha(
    runner: &mut impl ScriptRepoGitRunner,
    repo_url: &str,
    branch: &str,
) -> Result<String> {
    let output = runner.run_git(
        None,
        &git_with_owned_args(["ls-remote", "--heads", repo_url, branch]),
    )?;
    let Some((sha, _)) = output
        .stdout
        .lines()
        .filter_map(|line| line.split_once(char::is_whitespace))
        .find(|(_, reference)| reference.trim_end() == format!("refs/heads/{branch}"))
    else {
        return Err(ScriptRepoError::MissingGitBranch(branch.to_string()));
    };
    Ok(sha.to_string())
}

fn write_folder_mapping(path: &Path, repo_url: &str, folder_name: &str) -> Result<()> {
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

fn rename_or_copy_directory(source: &Path, destination: &Path) -> Result<()> {
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

fn path_mapper(app_root: &Path) -> BTreeMap<ScriptRepoPathKind, PathBuf> {
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

fn normalize_repo_path(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .split('/')
        .filter(|part| !part.trim().is_empty() && *part != "." && *part != "..")
        .collect::<Vec<_>>()
        .join("/")
}

fn canonical_or_lexical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| lexical_normalize(path))
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

fn validate_child_path(root: &Path, relative: &str) -> Result<PathBuf> {
    let candidate = lexical_normalize(&root.join(normalize_repo_path(relative)));
    if !path_starts_with(&candidate, root) {
        return Err(ScriptRepoError::PathEscapesRoot {
            path: candidate,
            root: root.to_path_buf(),
        });
    }
    Ok(candidate)
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

fn remove_existing(path: &Path) -> Result<()> {
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

fn copy_repo_path(source: &Path, destination: &Path) -> Result<()> {
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

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
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

fn read_saved_files_for_source(source: &Path) -> Result<Vec<String>> {
    let manifest_path = if source.is_dir() {
        source.join("manifest.json")
    } else {
        source
            .parent()
            .map(|parent| parent.join("manifest.json"))
            .unwrap_or_else(|| PathBuf::from("manifest.json"))
    };
    if !manifest_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&manifest_path).map_err(|source| ScriptRepoError::Io {
        path: manifest_path.clone(),
        source,
    })?;
    let value: serde_json::Value =
        serde_json::from_str(&content).map_err(|source| ScriptRepoError::Json {
            path: manifest_path.clone(),
            source,
        })?;
    let saved = value
        .get("saved_files")
        .or_else(|| value.get("savedFiles"))
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(saved)
}

fn backup_saved_files(
    destination: &Path,
    saved_files: &[String],
) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut backups = Vec::new();
    if saved_files.is_empty() || !destination.exists() {
        return Ok(backups);
    }
    let backup_root = destination
        .parent()
        .unwrap_or(destination)
        .join(".bgi_saved_backup");
    remove_existing(&backup_root)?;

    for saved in saved_files {
        let saved = saved.trim();
        if saved.is_empty() {
            continue;
        }
        let relative = saved.trim_end_matches(['/', '\\']);
        let source = lexical_normalize(&destination.join(relative));
        if !source.exists() {
            continue;
        }
        let backup = backup_root.join(relative);
        copy_repo_path(&source, &backup)?;
        backups.push((backup, PathBuf::from(relative)));
    }
    Ok(backups)
}

fn restore_saved_files(destination: &Path, backups: Vec<(PathBuf, PathBuf)>) -> Result<()> {
    let backup_root = destination
        .parent()
        .unwrap_or(destination)
        .join(".bgi_saved_backup");
    for (backup, relative) in backups {
        if backup.exists() {
            copy_repo_path(&backup, &destination.join(relative))?;
        }
    }
    remove_existing(&backup_root)?;
    Ok(())
}

fn copy_js_package_dependencies(repo_root: &Path, destination: &Path) -> Result<Vec<PathBuf>> {
    let base_dir = if destination.is_file() {
        destination.parent().unwrap_or(destination).to_path_buf()
    } else {
        destination.to_path_buf()
    };
    let mut copied = Vec::new();
    let mut queue = js_files_under(destination)?;
    let mut processed = BTreeSet::new();

    while let Some(file) = queue.pop() {
        if !processed.insert(file.clone()) {
            continue;
        }
        let content = fs::read_to_string(&file).unwrap_or_default();
        for package_path in package_imports(&content, &base_dir, &file) {
            let source = validate_child_path(repo_root, &package_path)?;
            let target = base_dir.join(&package_path);
            if !target.exists() && source.exists() {
                copy_repo_path(&source, &target)?;
                copied.push(target.clone());
                if target.extension().and_then(|ext| ext.to_str()) == Some("js") {
                    queue.push(target);
                }
            } else if target.exists()
                && target.extension().and_then(|ext| ext.to_str()) == Some("js")
            {
                queue.push(target);
            }
        }
    }
    Ok(copied)
}

#[derive(Debug, Clone)]
struct MaterializedGitSource {
    root: PathBuf,
    path: PathBuf,
}

fn materialize_git_repo_source(
    runner: &mut impl ScriptRepoGitRunner,
    repo_root: &Path,
    source_path: &str,
) -> Result<Option<MaterializedGitSource>> {
    let temp_root = std::env::temp_dir().join(format!("bgi-git-source-{}", unique_suffix()));
    let destination = temp_root.join(normalize_repo_path(source_path));
    match checkout_git_repo_path(runner, repo_root, source_path, &destination, true)? {
        Some(_) => Ok(Some(MaterializedGitSource {
            root: temp_root,
            path: destination,
        })),
        None => Ok(None),
    }
}

fn copy_js_package_dependencies_from_git(
    runner: &mut impl ScriptRepoGitRunner,
    repo_root: &Path,
    destination: &Path,
) -> Result<Vec<PathBuf>> {
    let base_dir = if destination.is_file() {
        destination.parent().unwrap_or(destination).to_path_buf()
    } else {
        destination.to_path_buf()
    };
    let mut copied = Vec::new();
    let mut queue = js_files_under(destination)?;
    let mut processed = BTreeSet::new();

    while let Some(file) = queue.pop() {
        if !processed.insert(file.clone()) {
            continue;
        }
        let content = fs::read_to_string(&file).unwrap_or_default();
        for package_path in package_imports(&content, &base_dir, &file) {
            let target = base_dir.join(&package_path);
            if !target.exists() {
                if checkout_git_repo_path(runner, repo_root, &package_path, &target, false)?
                    .is_some()
                {
                    copied.push(target.clone());
                    if target.extension().and_then(|ext| ext.to_str()) == Some("js") {
                        queue.push(target);
                    }
                }
            } else if target.extension().and_then(|ext| ext.to_str()) == Some("js") {
                queue.push(target);
            }
        }
    }
    Ok(copied)
}

fn js_files_under(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(
            (path.extension().and_then(|ext| ext.to_str()) == Some("js"))
                .then(|| path.to_path_buf())
                .into_iter()
                .collect(),
        );
    }
    let mut files = Vec::new();
    if path.is_dir() {
        collect_js_files(path, &mut files)?;
    }
    Ok(files)
}

fn collect_js_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path).map_err(|source| ScriptRepoError::Io {
        path: path.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| ScriptRepoError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_js_files(&entry_path, files)?;
        } else if entry_path.extension().and_then(|ext| ext.to_str()) == Some("js") {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn package_imports(content: &str, base_dir: &Path, current_file: &Path) -> Vec<String> {
    let mut imports = Vec::new();
    for quote in ['"', '\''] {
        let parts = content.split(quote).collect::<Vec<_>>();
        for candidate in parts.iter().skip(1).step_by(2) {
            if let Some(index) = candidate.to_ascii_lowercase().find("packages/") {
                imports.push(normalize_repo_path(&candidate[index..]));
            } else if candidate.starts_with('.') {
                let local_packages = base_dir.join("packages");
                if current_file.starts_with(&local_packages) {
                    let current_dir = current_file.parent().unwrap_or(base_dir);
                    if let Ok(relative) = current_dir.join(candidate).strip_prefix(base_dir) {
                        let normalized = normalize_repo_path(&relative.to_string_lossy());
                        if normalized
                            .get(..9)
                            .map(|prefix| prefix.eq_ignore_ascii_case("packages/"))
                            .unwrap_or(false)
                        {
                            imports.push(normalized);
                        }
                    }
                }
            }
        }
    }
    normalize_subscription_paths(imports)
}

fn percent_decode(value: &str) -> std::result::Result<String, String> {
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

fn base64_decode(value: &str) -> std::result::Result<Vec<u8>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn test_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        fs::remove_dir_all(&root).unwrap_or(());
        fs::create_dir_all(&root).unwrap();
        root
    }
    fn script_config() -> ScriptConfig {
        ScriptConfig::default()
    }

    #[derive(Debug, Default)]
    struct RecordingGitRunner {
        commands: Vec<(Option<PathBuf>, Vec<String>)>,
        repo_url: String,
        remote_sha: String,
        current_sha: String,
        origin_url: String,
        repo_json: String,
        objects: BTreeMap<String, (String, String)>,
        binary_objects: BTreeMap<String, (String, Vec<u8>)>,
        trees: BTreeMap<String, String>,
    }

    impl RecordingGitRunner {
        fn new(repo_url: &str, repo_json: &str) -> Self {
            Self {
                commands: Vec::new(),
                repo_url: repo_url.to_string(),
                remote_sha: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                current_sha: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
                origin_url: repo_url.to_string(),
                repo_json: repo_json.to_string(),
                objects: BTreeMap::new(),
                binary_objects: BTreeMap::new(),
                trees: BTreeMap::new(),
            }
        }
    }

    impl ScriptRepoGitRunner for RecordingGitRunner {
        fn run_git(
            &mut self,
            cwd: Option<&Path>,
            args: &[String],
        ) -> Result<ScriptRepoGitCommandOutput> {
            self.commands
                .push((cwd.map(Path::to_path_buf), args.to_vec()));
            let stdout_bytes = match args
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .as_slice()
            {
                ["rev-parse", "--is-inside-work-tree"] => b"true".to_vec(),
                ["remote", "get-url", "origin"] => self.origin_url.clone().into_bytes(),
                ["ls-remote", "--heads", url, branch] => {
                    assert_eq!(*url, self.repo_url);
                    format!("{}\trefs/heads/{branch}", self.remote_sha).into_bytes()
                }
                ["rev-parse", "release"] => self.current_sha.clone().into_bytes(),
                ["show", "release:repo.json"] => self.repo_json.clone().into_bytes(),
                ["cat-file", "-t", object] => self
                    .binary_objects
                    .get(*object)
                    .map(|(kind, _)| kind.clone())
                    .or_else(|| self.objects.get(*object).map(|(kind, _)| kind.clone()))
                    .ok_or_else(|| ScriptRepoError::MissingSource(PathBuf::from(object)))?
                    .into_bytes(),
                ["ls-tree", object] => self
                    .trees
                    .get(*object)
                    .cloned()
                    .ok_or_else(|| ScriptRepoError::MissingSource(PathBuf::from(object)))?
                    .into_bytes(),
                ["show", object] => self
                    .binary_objects
                    .get(*object)
                    .map(|(_, content)| content.clone())
                    .or_else(|| {
                        self.objects
                            .get(*object)
                            .map(|(_, content)| content.clone().into_bytes())
                    })
                    .ok_or_else(|| ScriptRepoError::MissingSource(PathBuf::from(object)))?,
                _ => Vec::new(),
            };
            Ok(ScriptRepoGitCommandOutput {
                stdout: String::from_utf8_lossy(&stdout_bytes).trim().to_string(),
                stderr: String::new(),
                stdout_bytes,
            })
        }
    }
    #[test]
    fn repo_url_and_folder_name_follow_legacy_channel_rules() {
        let config = script_config();
        assert_eq!(
            resolve_repo_url(&config).as_deref(),
            Some("https://cnb.cool/bettergi/bettergi-scripts-list")
        );

        let mut custom = script_config();
        custom.selected_channel_name = "自定义".to_string();
        custom.custom_repo_url = "https://example.com/custom-repo".to_string();
        assert_eq!(resolve_repo_url(&custom), None);
        custom.custom_repo_url = "https://host/owner/custom.git".to_string();
        assert_eq!(
            resolve_repo_url(&custom).as_deref(),
            Some("https://host/owner/custom.git")
        );

        let mut mapping = BTreeMap::new();
        mapping.insert(
            "https://host/owner/custom.git".to_string(),
            "mapped-folder".to_string(),
        );
        assert_eq!(
            repo_folder_name(Some("https://host/owner/custom.git"), &mapping),
            "mapped-folder"
        );
        assert_eq!(
            repo_folder_name(
                Some("https://github.com/babalae/bettergi-scripts-list.git"),
                &BTreeMap::new()
            ),
            "bettergi-scripts-list"
        );
    }

    #[test]
    fn script_repo_layout_preserves_legacy_paths() {
        let layout = script_repo_layout("C:/BetterGI", &script_config(), &BTreeMap::new());
        assert!(layout.repos_path.ends_with("Repos"));
        assert!(layout.repos_temp_path.ends_with("Repos/Temp"));
        assert!(layout.center_repo_path.ends_with(DEFAULT_REPO_FOLDER_NAME));
        assert!(layout
            .old_center_repo_path
            .ends_with(OLD_CENTER_REPO_FOLDER_NAME));
        assert!(layout
            .subscription_file_path
            .ends_with("bettergi-scripts-list.json"));
        assert!(layout
            .path_mapper
            .get(&ScriptRepoPathKind::Js)
            .unwrap()
            .ends_with("User/JsScript"));
    }

    #[test]
    fn import_uri_decodes_base64_url_encoded_path_json() {
        let path_json = serde_json::to_string(&vec!["js/demo", "pathing/route"]).unwrap();
        let encoded = "WyJqcy9kZW1vIiwicGF0aGluZy9yb3V0ZSJd";
        let plan = parse_import_uri(&format!("bettergi://script?import={encoded}"), true)
            .unwrap()
            .unwrap();

        assert_eq!(plan.path_json, path_json);
        assert_eq!(plan.paths, vec!["js/demo", "pathing/route"]);
        assert!(plan.clear_clipboard_after_import);
        assert!(parse_import_uri("https://example.com", false)
            .unwrap()
            .is_none());
    }

    #[test]
    fn import_plan_maps_repo_prefixes_to_user_destinations_and_subscriptions() {
        let mut children = BTreeMap::new();
        children.insert(
            "js".to_string(),
            vec!["alpha".to_string(), "beta".to_string()],
        );

        let plan = script_import_plan(
            "C:/BetterGI",
            "C:/BetterGI/Repos/bettergi-scripts-list/repo",
            &script_config(),
            &BTreeMap::new(),
            ["js", "combat/team", "unknown/path"],
            ["pathing/old", "js/alpha"],
            &children,
        );

        assert_eq!(
            plan.expanded_paths,
            vec!["combat/team", "js/alpha", "js/beta", "unknown/path"]
        );
        assert_eq!(plan.unknown_paths, vec!["unknown/path"]);
        assert_eq!(plan.targets.len(), 3);
        let js_target = plan
            .targets
            .iter()
            .find(|target| target.source_path == "js/alpha")
            .unwrap();
        assert!(js_target.destination_path.ends_with("User/JsScript/alpha"));
        assert!(js_target.preserves_saved_files);
        assert!(js_target.resolves_js_dependencies);
        assert_eq!(
            plan.merged_subscriptions,
            vec![
                "combat/team",
                "js",
                "js/alpha",
                "pathing/old",
                "unknown/path"
            ]
        );
    }

    #[test]
    fn update_plan_respects_auto_update_switch_custom_repo_and_subscriptions() {
        let mut config = script_config();
        config.selected_channel_name = "自定义".to_string();
        let plan =
            script_repo_update_plan("C:/BetterGI", &config, &BTreeMap::new(), ["js/demo"], false);
        assert!(!plan.enabled);
        assert_eq!(plan.reason, Some("repo_url_unresolved"));

        config.custom_repo_url = "https://host/repo.git".to_string();
        let plan =
            script_repo_update_plan("C:/BetterGI", &config, &BTreeMap::new(), ["js/demo"], false);
        assert!(!plan.enabled);
        assert_eq!(plan.reason, Some("auto_update_disabled"));

        config.auto_update_subscribed_scripts = true;
        let plan =
            script_repo_update_plan("C:/BetterGI", &config, &BTreeMap::new(), ["js/demo"], false);
        assert!(plan.enabled);
        assert_eq!(plan.repo_folder_name, "repo");
        assert_eq!(plan.subscribed_paths, vec!["js/demo"]);

        let manual = script_repo_update_plan(
            "C:/BetterGI",
            &config,
            &BTreeMap::new(),
            Vec::<String>::new(),
            true,
        );
        assert!(!manual.enabled);
        assert_eq!(manual.reason, Some("no_subscribed_paths"));
    }

    #[test]
    fn zip_import_plan_preserves_legacy_temp_and_marker_paths() {
        let plan = zip_import_plan("C:/BetterGI", "D:/repo.zip", Some("bad:name"));
        assert!(plan.temp_unzip_dir.ends_with("Repos/Temp/importZipFile"));
        assert_eq!(plan.target_folder_name, "bad_name");
        assert!(plan
            .repo_updated_json_path
            .ends_with("bad_name/repo_updated.json"));
        assert_eq!(plan.overlap_threshold, 0.5);
    }

    #[test]
    fn git_update_plan_uses_release_branch_and_folder_mapping() {
        let mut mapping = BTreeMap::new();
        mapping.insert(
            "https://host/owner/custom.git".to_string(),
            "mapped".to_string(),
        );
        let plan = git_update_plan("C:/BetterGI", "https://host/owner/custom.git/", &mapping);

        assert_eq!(plan.repo_url, "https://host/owner/custom.git");
        assert_eq!(plan.branch, "release");
        assert_eq!(plan.repo_folder_name, "mapped");
        assert!(plan.repo_path.ends_with("Repos/mapped"));
        assert!(plan
            .folder_mapping_path
            .ends_with("Repos/repo_folder_mapping.json"));
    }

    #[test]
    fn git_update_exec_clones_missing_repo_and_writes_marker_and_mapping() {
        let root = test_root("bgi-script-repo-git-clone");
        let repo_json = r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[]}]}"#;
        let mut runner = RecordingGitRunner::new("https://host/repo.git", repo_json);
        let plan = git_update_plan(&root, "https://host/repo.git", &BTreeMap::new());

        let result = execute_git_repo_update(&plan, &mut runner).unwrap();

        assert!(result.updated);
        assert!(result.cloned);
        assert!(result.repo_path.join("repo.json").exists());
        assert!(result.repo_updated_json_path.exists());
        assert!(runner
            .commands
            .iter()
            .any(|(_, args)| args.iter().any(|arg| arg == "--depth")
                && args.iter().any(|arg| arg == "--no-tags")
                && args.iter().any(|arg| arg.contains("refs/heads/release"))));
        let mapping = fs::read_to_string(root.join("Repos/repo_folder_mapping.json")).unwrap();
        assert!(mapping.contains("https://host/repo.git"));
        assert!(mapping.contains("repo"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn git_update_exec_skips_clone_when_release_sha_matches() {
        let root = test_root("bgi-script-repo-git-current");
        let repo_path = root.join("Repos/repo");
        fs::create_dir_all(repo_path.join(".git")).unwrap();
        fs::write(
            repo_path.join("repo.json"),
            r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[]}]}"#,
        )
        .unwrap();
        let mut runner = RecordingGitRunner::new(
            "https://host/repo.git",
            r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[]}]}"#,
        );
        runner.current_sha = runner.remote_sha.clone();
        let plan = git_update_plan(&root, "https://host/repo.git", &BTreeMap::new());

        let result = execute_git_repo_update(&plan, &mut runner).unwrap();

        assert!(!result.updated);
        assert!(!result.cloned);
        assert_eq!(result.current_commit, Some(runner.remote_sha.clone()));
        assert!(runner.commands.iter().any(|(_, args)| args
            == &git_args(&["ls-remote", "--heads", "https://host/repo.git", "release"])));
        assert!(!runner
            .commands
            .iter()
            .any(|(_, args)| args.iter().any(|arg| arg == "fetch")));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn git_update_exec_replaces_remote_changed_repo_when_overlap_is_high() {
        let root = test_root("bgi-script-repo-git-remote-changed");
        let repo_path = root.join("Repos/new-repo");
        fs::create_dir_all(repo_path.join(".git")).unwrap();
        fs::write(
            repo_path.join("repo.json"),
            r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[{"name":"demo","type":"directory","lastUpdated":"2024-01-01","children":[]}]}]}"#,
        )
        .unwrap();
        let new_repo_json = r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[{"name":"demo","type":"directory","lastUpdated":"2024-02-01","children":[]}]}]}"#;
        let mut runner = RecordingGitRunner::new("https://host/new-repo.git", new_repo_json);
        runner.origin_url = "https://host/old-repo.git".to_string();
        let plan = git_update_plan(&root, "https://host/new-repo.git", &BTreeMap::new());

        let result = execute_git_repo_update(&plan, &mut runner).unwrap();

        assert!(result.updated);
        assert!(result.remote_changed);
        assert!(!result.created_new_folder);
        assert_eq!(result.repo_folder_name, "new-repo");
        assert!(result.old_repo_overlap_ratio.unwrap() >= 0.5);
        assert!(result.marker_generated);
        let marker = fs::read_to_string(result.repo_updated_json_path).unwrap();
        let marker: serde_json::Value = serde_json::from_str(&marker).unwrap();
        assert_eq!(marker["indexes"][0]["hasUpdate"], true);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn git_checkout_path_exports_repo_subtree_to_destination() {
        let root = test_root("bgi-script-repo-git-checkout");
        let repo_path = root.join("Repos/repo");
        fs::create_dir_all(repo_path.join(".git")).unwrap();
        let mut runner = RecordingGitRunner::new("https://host/repo.git", "{}");
        runner.objects.insert(
            "HEAD:repo/js/demo".to_string(),
            ("tree".to_string(), String::new()),
        );
        runner.trees.insert(
            "HEAD:repo/js/demo".to_string(),
            "100644 blob aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\tmain.js\n040000 tree bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\tlib".to_string(),
        );
        runner.objects.insert(
            "HEAD:repo/js/demo/main.js".to_string(),
            ("blob".to_string(), "import './lib/helper.js';".to_string()),
        );
        runner.trees.insert(
            "HEAD:repo/js/demo/lib".to_string(),
            "100644 blob cccccccccccccccccccccccccccccccccccccccc\thelper.js".to_string(),
        );
        runner.objects.insert(
            "HEAD:repo/js/demo/lib/helper.js".to_string(),
            ("blob".to_string(), "export default 1;".to_string()),
        );

        let destination = root.join("User/JsScript/demo");
        let checkout =
            checkout_git_repo_path(&mut runner, &repo_path, "js/demo", &destination, true)
                .unwrap()
                .unwrap();

        assert!(checkout.is_directory);
        assert_eq!(checkout.git_tree_path, "repo/js/demo");
        assert_eq!(checkout.files_written.len(), 2);
        assert_eq!(
            fs::read_to_string(destination.join("main.js")).unwrap(),
            "import './lib/helper.js';"
        );
        assert_eq!(
            fs::read_to_string(destination.join("lib/helper.js")).unwrap(),
            "export default 1;"
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn git_checkout_path_can_export_root_package_dependency() {
        let root = test_root("bgi-script-repo-git-root-checkout");
        let repo_path = root.join("Repos/repo");
        fs::create_dir_all(repo_path.join(".git")).unwrap();
        let mut runner = RecordingGitRunner::new("https://host/repo.git", "{}");
        runner.objects.insert(
            "HEAD:packages/lib/helper.js".to_string(),
            ("blob".to_string(), "export default 2;".to_string()),
        );

        let destination = root.join("User/JsScript/demo/packages/lib/helper.js");
        let checkout = checkout_git_repo_path(
            &mut runner,
            &repo_path,
            "packages/lib/helper.js",
            &destination,
            false,
        )
        .unwrap()
        .unwrap();

        assert!(!checkout.is_directory);
        assert_eq!(checkout.git_tree_path, "packages/lib/helper.js");
        assert_eq!(
            fs::read_to_string(destination).unwrap(),
            "export default 2;"
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn git_checkout_path_preserves_binary_blob_bytes() {
        let root = test_root("bgi-script-repo-git-binary-checkout");
        let repo_path = root.join("Repos/repo");
        fs::create_dir_all(repo_path.join(".git")).unwrap();
        let mut runner = RecordingGitRunner::new("https://host/repo.git", "{}");
        let bytes = vec![0, 159, 146, 150, 255, 10, 13, 0];
        runner.binary_objects.insert(
            "HEAD:repo/js/demo/icon.png".to_string(),
            ("blob".to_string(), bytes.clone()),
        );

        let destination = root.join("User/JsScript/demo/icon.png");
        let checkout = checkout_git_repo_path(
            &mut runner,
            &repo_path,
            "js/demo/icon.png",
            &destination,
            true,
        )
        .unwrap()
        .unwrap();

        assert!(!checkout.is_directory);
        assert_eq!(fs::read(destination).unwrap(), bytes);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn git_repo_import_uses_git_tree_and_preserves_saved_files_and_packages() {
        let root = test_root("bgi-script-repo-git-import");
        let repo_path = root.join("Repos/repo");
        fs::create_dir_all(repo_path.join(".git")).unwrap();
        let mut runner = RecordingGitRunner::new("https://host/repo.git", "{}");
        runner.objects.insert(
            "HEAD:repo/js/demo".to_string(),
            ("tree".to_string(), String::new()),
        );
        runner.trees.insert(
            "HEAD:repo/js/demo".to_string(),
            "100644 blob aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\tmanifest.json\n100644 blob bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\tmain.js".to_string(),
        );
        runner.objects.insert(
            "HEAD:repo/js/demo/manifest.json".to_string(),
            (
                "blob".to_string(),
                r#"{"manifest_version":1,"name":"Demo","version":"1.0.0","main":"main.js","saved_files":["config.json"]}"#.to_string(),
            ),
        );
        runner.objects.insert(
            "HEAD:repo/js/demo/main.js".to_string(),
            (
                "blob".to_string(),
                r#"import helper from "packages/lib/helper.js"; console.log(helper);"#.to_string(),
            ),
        );
        runner.objects.insert(
            "HEAD:packages/lib/helper.js".to_string(),
            ("blob".to_string(), "export default 9;".to_string()),
        );

        let user_script_dir = root.join("User/JsScript/demo");
        fs::create_dir_all(&user_script_dir).unwrap();
        fs::write(user_script_dir.join("config.json"), r#"{"keep":true}"#).unwrap();
        fs::write(user_script_dir.join("old.js"), "old").unwrap();

        let plan = script_import_plan(
            &root,
            &repo_path,
            &script_config(),
            &BTreeMap::new(),
            ["js/demo"],
            Vec::<String>::new(),
            &BTreeMap::new(),
        );
        let result = execute_repo_import_with_git(&plan, Some(&mut runner)).unwrap();

        assert_eq!(result.imported_targets.len(), 1);
        assert_eq!(result.git_checkouts.len(), 1);
        assert_eq!(result.dependency_files_copied.len(), 1);
        assert_eq!(
            fs::read_to_string(user_script_dir.join("config.json")).unwrap(),
            r#"{"keep":true}"#
        );
        assert!(!user_script_dir.join("old.js").exists());
        assert!(user_script_dir.join("main.js").exists());
        assert_eq!(
            fs::read_to_string(user_script_dir.join("packages/lib/helper.js")).unwrap(),
            "export default 9;"
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn subscription_path_helpers_normalize_dedupe_and_split() {
        assert_eq!(
            normalize_subscription_paths([" js/demo ", "js\\demo", "./pathing/a", "../bad"]),
            vec!["bad", "js/demo", "pathing/a"]
        );
        let (first, rest) = first_folder_and_remaining_path("js/demo/main.js");
        assert_eq!(first, "js");
        assert_eq!(rest, PathBuf::from("demo/main.js"));
        assert_eq!(
            merge_subscription_paths(["js/a"], ["js/a", "pathing/b"]),
            vec!["js/a", "pathing/b"]
        );
    }

    #[test]
    fn subscription_file_read_write_normalizes_and_deletes_empty_lists() {
        let root = test_root("bgi-script-repo-subscriptions");
        let path = root.join("User/Subscriptions/bettergi-scripts-list.json");

        write_subscription_file(&path, &["js/demo".to_string(), "js\\demo".to_string()]).unwrap();
        assert_eq!(read_subscription_file(&path).unwrap(), vec!["js/demo"]);

        write_subscription_file(&path, &[]).unwrap();
        assert!(!path.exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn file_repo_import_copies_targets_preserves_saved_files_and_copies_packages() {
        let root = test_root("bgi-script-repo-import-exec");
        let repo_root = root.join("Repos/bettergi-scripts-list/repo");
        let script_repo_dir = repo_root.join("js/demo");
        fs::create_dir_all(&script_repo_dir).unwrap();
        fs::create_dir_all(repo_root.join("packages/lib")).unwrap();
        fs::write(
            script_repo_dir.join("manifest.json"),
            r#"{"manifest_version":1,"name":"Demo","version":"1.0.0","main":"main.js","saved_files":["config.json"]}"#,
        )
        .unwrap();
        fs::write(
            script_repo_dir.join("main.js"),
            r#"import helper from "packages/lib/helper.js"; console.log(helper);"#,
        )
        .unwrap();
        fs::write(
            repo_root.join("packages/lib/helper.js"),
            "export default 1;",
        )
        .unwrap();

        let user_script_dir = root.join("User/JsScript/demo");
        fs::create_dir_all(&user_script_dir).unwrap();
        fs::write(user_script_dir.join("config.json"), r#"{"keep":true}"#).unwrap();
        fs::write(user_script_dir.join("old.js"), "old").unwrap();

        let plan = script_import_plan(
            &root,
            &repo_root,
            &script_config(),
            &BTreeMap::new(),
            ["js/demo"],
            Vec::<String>::new(),
            &BTreeMap::new(),
        );
        let result = execute_file_repo_import(&plan).unwrap();

        assert_eq!(result.imported_targets.len(), 1);
        assert_eq!(result.subscriptions, vec!["js/demo"]);
        assert!(root
            .join("User/Subscriptions/bettergi-scripts-list.json")
            .exists());
        assert_eq!(
            fs::read_to_string(user_script_dir.join("config.json")).unwrap(),
            r#"{"keep":true}"#
        );
        assert!(!user_script_dir.join("old.js").exists());
        assert!(user_script_dir.join("main.js").exists());
        assert!(user_script_dir.join("packages/lib/helper.js").exists());
        assert_eq!(result.dependency_files_copied.len(), 1);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn repo_overlap_ratio_uses_directory_overlap_coefficient() {
        let old = r#"{
          "indexes": [
            {"name":"js","type":"directory","children":[
              {"name":"demo","type":"directory","children":[]}
            ]},
            {"name":"pathing","type":"directory","children":[]}
          ]
        }"#;
        let new = r#"{
          "indexes": [
            {"name":"js","type":"directory","children":[
              {"name":"demo","type":"directory","children":[]},
              {"name":"new","type":"directory","children":[]}
            ]},
            {"name":"combat","type":"directory","children":[]}
          ]
        }"#;

        assert_eq!(
            repo_directory_paths(old).unwrap(),
            vec!["js", "js/demo", "pathing"]
        );
        assert!((calculate_repo_overlap_ratio(old, new) - (2.0 / 3.0)).abs() < f64::EPSILON);
        assert_eq!(calculate_repo_overlap_ratio("not-json", new), -1.0);
    }

    #[test]
    fn repo_update_markers_preserve_old_flags_and_mark_newer_or_new_nodes() {
        let old = r#"{
          "indexes": [{
            "name": "js",
            "type": "directory",
            "lastUpdated": "2024-01-01 00:00:00",
            "children": [
              {"name":"demo","type":"directory","lastUpdated":"2024-01-02 00:00:00","children":[]},
              {"name":"flagged","type":"directory","hasUpdate":"true","lastUpdated":"2024-01-01 00:00:00","children":[]}
            ]
          }]
        }"#;
        let new = r#"{
          "indexes": [{
            "name": "js",
            "type": "directory",
            "lastUpdated": "2024-01-01 00:00:00",
            "children": [
              {"name":"demo","type":"directory","lastUpdated":"2024-02-01 00:00:00","children":[]},
              {"name":"flagged","type":"directory","lastUpdated":"2024-01-01 00:00:00","children":[]},
              {"name":"fresh","type":"directory","lastUpdated":"2024-03-01 00:00:00","children":[]}
            ]
          }]
        }"#;

        let marked = add_update_markers_to_new_repo(old, new);
        let value: serde_json::Value = serde_json::from_str(&marked).unwrap();
        let js = &value["indexes"][0];
        assert_eq!(js["hasUpdate"], true);
        assert_eq!(js["lastUpdated"], "2024-03-01 00:00:00");
        assert_eq!(js["children"][0]["hasUpdate"], true);
        assert_eq!(js["children"][1]["hasUpdate"], true);
        assert_eq!(js["children"][2]["hasUpdate"], true);
    }

    #[test]
    fn zip_repo_import_extracts_matches_existing_repo_and_generates_marker() {
        let root = test_root("bgi-script-repo-zip-exec");
        let existing_repo = root.join("Repos/existing");
        fs::create_dir_all(existing_repo.join("repo/js/demo")).unwrap();
        fs::write(
            existing_repo.join("repo.json"),
            r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[{"name":"demo","type":"directory","lastUpdated":"2024-01-01","children":[]}]}]}"#,
        )
        .unwrap();

        let source_repo = root.join("source/repo");
        fs::create_dir_all(source_repo.join("js/demo")).unwrap();
        fs::write(source_repo.join("js/demo/main.js"), "console.log('demo');").unwrap();
        fs::write(
            source_repo.join("repo.json"),
            r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[{"name":"demo","type":"directory","lastUpdated":"2024-02-01","children":[]}]}]}"#,
        )
        .unwrap();
        let zip_path = root.join("repo.zip");
        create_test_zip(&zip_path, &source_repo, "packed").unwrap();

        let plan = zip_import_plan(&root, &zip_path, Some("bettergi-scripts-list"));
        let result = execute_zip_repo_import(&plan).unwrap();

        assert_eq!(result.target_folder_name, "existing");
        assert!(result.marker_generated);
        assert!(result
            .best_overlap_ratio
            .map(|ratio| ratio >= 0.5)
            .unwrap_or(false));
        assert!(result.target_path.join("js/demo/main.js").exists());
        assert!(result.repo_updated_json_path.exists());
        assert!(!root.join("Repos/Temp").exists());

        let marker = fs::read_to_string(result.repo_updated_json_path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&marker).unwrap();
        assert_eq!(value["indexes"][0]["hasUpdate"], true);
        assert_eq!(value["indexes"][0]["children"][0]["hasUpdate"], true);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn file_repo_import_reports_missing_sources() {
        let root = test_root("bgi-script-repo-import-missing");
        let repo_root = root.join("Repos/bettergi-scripts-list/repo");
        fs::create_dir_all(&repo_root).unwrap();
        let plan = script_import_plan(
            &root,
            &repo_root,
            &script_config(),
            &BTreeMap::new(),
            ["pathing/missing"],
            Vec::<String>::new(),
            &BTreeMap::new(),
        );
        let error = execute_file_repo_import(&plan).unwrap_err();
        assert!(matches!(error, ScriptRepoError::MissingSource(_)));
        fs::remove_dir_all(root).unwrap();
    }

    fn create_test_zip(zip_path: &Path, source_root: &Path, archive_root: &str) -> Result<()> {
        let file = File::create(zip_path).map_err(|source| ScriptRepoError::Io {
            path: zip_path.to_path_buf(),
            source,
        })?;
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        add_zip_directory(&mut writer, source_root, source_root, archive_root, options)?;
        writer.finish().map_err(|source| ScriptRepoError::Zip {
            path: zip_path.to_path_buf(),
            source,
        })?;
        Ok(())
    }

    fn add_zip_directory(
        writer: &mut zip::ZipWriter<File>,
        source_root: &Path,
        current: &Path,
        archive_root: &str,
        options: zip::write::SimpleFileOptions,
    ) -> Result<()> {
        for entry in fs::read_dir(current).map_err(|source| ScriptRepoError::Io {
            path: current.to_path_buf(),
            source,
        })? {
            let entry = entry.map_err(|source| ScriptRepoError::Io {
                path: current.to_path_buf(),
                source,
            })?;
            let path = entry.path();
            let relative = path.strip_prefix(source_root).unwrap();
            let archive_name =
                normalize_repo_path(&format!("{archive_root}/{}", relative.to_string_lossy()));
            if path.is_dir() {
                writer
                    .add_directory(format!("{archive_name}/"), options)
                    .map_err(|source| ScriptRepoError::Zip {
                        path: path.clone(),
                        source,
                    })?;
                add_zip_directory(writer, source_root, &path, archive_root, options)?;
            } else {
                writer
                    .start_file(&archive_name, options)
                    .map_err(|source| ScriptRepoError::Zip {
                        path: path.clone(),
                        source,
                    })?;
                let bytes = fs::read(&path).map_err(|source| ScriptRepoError::Io {
                    path: path.clone(),
                    source,
                })?;
                writer
                    .write_all(&bytes)
                    .map_err(|source| ScriptRepoError::Io {
                        path: path.clone(),
                        source,
                    })?;
            }
        }
        Ok(())
    }
}
