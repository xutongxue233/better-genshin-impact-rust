use super::repo_git::{checkout_git_tree, git_object_kind, git_with_owned_args};
use super::repo_paths::{normalize_repo_path, remove_existing};
use super::{Result, ScriptRepoError, ScriptRepoGitCheckout, ScriptRepoGitRunner};
use std::fs;
use std::path::Path;

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
