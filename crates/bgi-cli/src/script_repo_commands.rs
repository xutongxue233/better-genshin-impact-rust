use super::*;

pub(crate) fn script_repo(command: ScriptCommand) -> Result<()> {
    match command {
        ScriptCommand::RepoChannels { json } => {
            let channels = script_repo_channels();
            if json {
                println!("{}", serde_json::to_string_pretty(&channels)?);
            } else {
                for channel in channels {
                    println!("{:<8} {}", channel.name, channel.url);
                }
            }
        }
        ScriptCommand::RepoLayout { json } => {
            let config = AppConfig::default();
            let layout = script_repo_layout(
                ".",
                &config.script_config,
                &std::collections::BTreeMap::new(),
            );
            if json {
                println!("{}", serde_json::to_string_pretty(&layout)?);
            } else {
                println!("repos: {}", layout.repos_path.display());
                println!("center: {}", layout.center_repo_path.display());
                println!("subscriptions: {}", layout.subscription_file_path.display());
            }
        }
        ScriptCommand::RepoUpdatePlan {
            json,
            manual,
            paths,
        } => {
            let config = AppConfig::default();
            let plan = script_repo_update_plan(
                ".",
                &config.script_config,
                &std::collections::BTreeMap::new(),
                paths,
                manual,
            );
            if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                println!("enabled: {}", plan.enabled);
                println!("reason: {}", plan.reason.unwrap_or("-"));
                println!("repo: {}", plan.repo_url.as_deref().unwrap_or("-"));
                println!("subscriptions: {}", plan.subscribed_paths.len());
            }
        }
        ScriptCommand::RepoImportUri {
            uri,
            json,
            clipboard,
        } => {
            let plan = parse_import_uri(&uri, clipboard).map_err(anyhow::Error::msg)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else if let Some(plan) = plan {
                println!("paths: {}", plan.paths.join(","));
                println!("clear_clipboard: {}", plan.clear_clipboard_after_import);
            } else {
                println!("not_import_uri");
            }
        }
        ScriptCommand::RepoImportPlan { json, paths } => {
            let config = AppConfig::default();
            let plan = script_import_plan(
                ".",
                "Repos/bettergi-scripts-list/repo",
                &config.script_config,
                &std::collections::BTreeMap::new(),
                paths,
                Vec::<String>::new(),
                &std::collections::BTreeMap::new(),
            );
            if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                println!("targets: {}", plan.targets.len());
                println!("unknown: {}", plan.unknown_paths.len());
                println!("subscriptions: {}", plan.merged_subscriptions.len());
            }
        }
        ScriptCommand::RepoImportExec {
            repo,
            json,
            git_repo,
            git,
            paths,
        } => {
            let config = AppConfig::default();
            let layout = script_repo_layout(
                ".",
                &config.script_config,
                &std::collections::BTreeMap::new(),
            );
            let existing =
                read_subscription_file(&layout.subscription_file_path).unwrap_or_default();
            let plan = script_import_plan(
                ".",
                &repo,
                &config.script_config,
                &std::collections::BTreeMap::new(),
                paths,
                existing,
                &std::collections::BTreeMap::new(),
            );
            let result = if git_repo {
                let mut runner = SystemGitRunner::new(git);
                execute_repo_import_with_git(&plan, Some(&mut runner))?
            } else {
                execute_file_repo_import(&plan)?
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("imported: {}", result.imported_targets.len());
                println!("unknown: {}", result.skipped_unknown_paths.len());
                println!("subscriptions: {}", result.subscriptions.len());
                println!("dependencies: {}", result.dependency_files_copied.len());
                println!("git_checkouts: {}", result.git_checkouts.len());
            }
        }
        ScriptCommand::RepoZipPlan { zip, json, folder } => {
            let plan = zip_import_plan(".", zip, folder.as_deref());
            if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                println!("target: {}", plan.target_path.display());
                println!("temp: {}", plan.temp_unzip_dir.display());
                println!("marker: {}", plan.repo_updated_json_path.display());
            }
        }
        ScriptCommand::RepoZipExec { zip, json, folder } => {
            let plan = zip_import_plan(".", zip, folder.as_deref());
            let result = execute_zip_repo_import(&plan)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("target: {}", result.target_path.display());
                println!("repo_json: {}", result.repo_json_path.display());
                println!("marker: {}", result.repo_updated_json_path.display());
                println!(
                    "best_overlap: {}",
                    result
                        .best_overlap_ratio
                        .map(|ratio| format!("{ratio:.3}"))
                        .unwrap_or_else(|| "-".to_string())
                );
                println!("marker_generated: {}", result.marker_generated);
            }
        }
        ScriptCommand::RepoGitPlan { repo_url, json } => {
            let plan = git_update_plan(".", repo_url, &std::collections::BTreeMap::new());
            if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                println!("repo: {}", plan.repo_url);
                println!("branch: {}", plan.branch);
                println!("target: {}", plan.repo_path.display());
                println!("marker: {}", plan.repo_updated_json_path.display());
            }
        }
        ScriptCommand::RepoGitUpdate {
            repo_url,
            json,
            git,
        } => {
            let plan = git_update_plan(".", repo_url, &std::collections::BTreeMap::new());
            let mut runner = SystemGitRunner::new(git);
            let result = execute_git_repo_update(&plan, &mut runner)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("target: {}", result.repo_path.display());
                println!("updated: {}", result.updated);
                println!("cloned: {}", result.cloned);
                println!("remote_changed: {}", result.remote_changed);
                println!("marker: {}", result.repo_updated_json_path.display());
            }
        }
        ScriptCommand::RepoGitCheckout {
            repo,
            source,
            destination,
            json,
            git,
            root,
        } => {
            let mut runner = SystemGitRunner::new(git);
            let result = checkout_git_repo_path(&mut runner, &repo, &source, &destination, !root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else if let Some(result) = result {
                println!("source: {}", result.git_tree_path);
                println!("destination: {}", result.destination_path.display());
                println!("directory: {}", result.is_directory);
                println!("files: {}", result.files_written.len());
            } else {
                println!("not_found");
            }
        }
        ScriptCommand::RepoBridgePaths { repo, json, folder } => {
            let paths = script_repo_bridge_paths(".", &repo, folder.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&paths)?);
            } else {
                println!("repo: {}", paths.repo_path.display());
                println!("repo_json: {}", paths.repo_json_path.display());
                println!("subscriptions: {}", paths.subscription_file_path.display());
            }
        }
        ScriptCommand::RepoBridgeJson { repo } => {
            println!("{}", read_repo_bridge_repo_json(&repo)?);
        }
        ScriptCommand::RepoBridgeIndex { repo, json } => {
            let nodes = repo_bridge_index_nodes(&repo)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&nodes)?);
            } else {
                for node in nodes {
                    println!(
                        "{:<8} {:<5} {:<6} {}",
                        node.node_type,
                        if node.importable { "yes" } else { "no" },
                        if node.has_update { "update" } else { "-" },
                        node.path
                    );
                }
            }
        }
        ScriptCommand::RepoBridgeSubscribed { repo, folder } => {
            let paths = script_repo_bridge_paths(".", &repo, folder.as_deref())?;
            println!(
                "{}",
                repo_bridge_subscribed_paths_json(&paths.subscription_file_path)?
            );
        }
        ScriptCommand::RepoBridgeFile {
            repo,
            rel_path,
            json,
            git_repo,
            git,
        } => {
            let response = if git_repo {
                let mut runner = SystemGitRunner::new(git);
                read_repo_bridge_file_with_git(&repo, &rel_path, Some(&mut runner))?
            } else {
                read_repo_bridge_file(&repo, &rel_path)?
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else if let Some(response) = response {
                println!("{}", response.content);
            } else {
                println!("404");
            }
        }
        ScriptCommand::RepoBridgeMarkUpdated { repo, path, json } => {
            let paths = script_repo_bridge_paths(".", &repo, None)?;
            let updated = mark_repo_bridge_path_updated(&paths.repo_json_path, &path)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "updated": updated,
                        "repo_json_path": paths.repo_json_path,
                    }))?
                );
            } else {
                println!("{updated}");
            }
        }
        ScriptCommand::RepoBridgeClearUpdate { repo, json } => {
            let path = clear_repo_bridge_update(&repo)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "repo_updated_json_path": path,
                    }))?
                );
            } else {
                println!("{}", path.display());
            }
        }
        _ => unreachable!("non-repo script command routed to repo handler"),
    }

    Ok(())
}
