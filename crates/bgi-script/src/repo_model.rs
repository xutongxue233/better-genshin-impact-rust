use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

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
