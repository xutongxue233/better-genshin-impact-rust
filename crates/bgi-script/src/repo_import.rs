use std::path::PathBuf;

use super::repo_dependencies::{
    backup_saved_files, copy_js_package_dependencies, copy_js_package_dependencies_from_git,
    materialize_git_repo_source, read_saved_files_for_source, restore_saved_files,
};
use super::repo_git::is_git_worktree;
use super::repo_paths::{
    canonical_or_lexical, copy_repo_path, remove_existing, validate_child_path,
};
use super::{
    checkout_git_repo_path, write_subscription_file, Result, ScriptImportPlan, ScriptRepoError,
    ScriptRepoGitRunner, ScriptRepoImportExecution, SystemGitRunner,
};

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
