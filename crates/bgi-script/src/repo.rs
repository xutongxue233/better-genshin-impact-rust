#[cfg(test)]
use std::collections::BTreeMap;
#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::path::{Path, PathBuf};

pub const DEFAULT_REPO_FOLDER_NAME: &str = "bettergi-scripts-list";
pub const OLD_CENTER_REPO_FOLDER_NAME: &str = "bettergi-scripts-list-main";
pub const REPOS_DIR: &str = "Repos";
pub const REPOS_TEMP_DIR: &str = "Repos/Temp";
pub const SUBSCRIPTIONS_DIR: &str = "User/Subscriptions";
pub const IMPORT_URI_PREFIX: &str = "bettergi://script?import=";

#[path = "repo_checkout.rs"]
mod repo_checkout;
#[path = "repo_dependencies.rs"]
mod repo_dependencies;
#[path = "repo_files.rs"]
mod repo_files;
#[path = "repo_git.rs"]
mod repo_git;
#[path = "repo_import.rs"]
mod repo_import;
#[path = "repo_model.rs"]
mod repo_model;
#[path = "repo_paths.rs"]
mod repo_paths;
#[path = "repo_planning.rs"]
mod repo_planning;
#[path = "repo_planning_import.rs"]
mod repo_planning_import;
#[path = "repo_planning_layout.rs"]
mod repo_planning_layout;
#[path = "repo_planning_update.rs"]
mod repo_planning_update;
#[path = "repo_planning_url.rs"]
mod repo_planning_url;
#[path = "repo_subscription.rs"]
mod repo_subscription;
#[path = "repo_update.rs"]
mod repo_update;
#[path = "repo_zip.rs"]
mod repo_zip;

pub use repo_checkout::*;
pub(crate) use repo_files::unique_suffix;
#[cfg(test)]
use repo_git::*;
pub use repo_import::*;
pub use repo_model::*;
pub use repo_paths::{
    expand_top_level_paths, first_folder_and_remaining_path, merge_subscription_paths,
    normalize_subscription_paths,
};
use repo_paths::{lexical_normalize, normalize_repo_path, remove_existing};
pub use repo_planning::*;
pub use repo_subscription::*;
pub use repo_update::*;
pub use repo_zip::*;

#[cfg(test)]
#[path = "repo_tests.rs"]
mod tests;
