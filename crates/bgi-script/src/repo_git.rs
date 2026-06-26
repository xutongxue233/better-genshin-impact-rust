use super::*;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn git_args(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_string()).collect()
}

pub(super) fn git_with_owned_args(
    args: impl IntoIterator<Item = impl Into<String>>,
) -> Vec<String> {
    args.into_iter().map(Into::into).collect()
}

pub(super) fn clone_release_repo(
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

pub(super) fn checkout_repo_json(
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

pub(super) fn git_object_kind(
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

pub(super) fn checkout_git_tree(
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

pub(super) fn is_git_worktree(runner: &mut impl ScriptRepoGitRunner, repo_path: &Path) -> bool {
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

pub(super) fn git_origin_url(
    runner: &mut impl ScriptRepoGitRunner,
    repo_path: &Path,
) -> Result<String> {
    runner
        .run_git(Some(repo_path), &git_args(&["remote", "get-url", "origin"]))
        .map(|output| output.stdout.trim().to_string())
}

pub(super) fn git_current_branch_sha(
    runner: &mut impl ScriptRepoGitRunner,
    repo_path: &Path,
    branch: &str,
) -> Result<String> {
    runner
        .run_git(Some(repo_path), &git_with_owned_args(["rev-parse", branch]))
        .map(|output| output.stdout.trim().to_string())
}

pub(super) fn git_remote_branch_sha(
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
