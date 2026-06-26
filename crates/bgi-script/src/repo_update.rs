use std::fs;

use super::repo_files::{
    find_file_named, generate_unique_folder_name, read_existing_repo_content,
    rename_or_copy_directory, unique_suffix, write_folder_mapping,
};
use super::repo_git::{
    checkout_repo_json, clone_release_repo, git_current_branch_sha, git_origin_url,
    git_remote_branch_sha, is_git_worktree,
};
use super::repo_paths::remove_existing;
use super::repo_planning_update::{add_update_markers_to_new_repo, calculate_repo_overlap_ratio};
use super::repo_planning_url::derive_base_folder_name;
use super::{
    Result, ScriptRepoError, ScriptRepoGitRunner, ScriptRepoGitUpdateExecution,
    ScriptRepoGitUpdatePlan,
};

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
