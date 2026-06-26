use super::repo_paths::{copy_repo_path, validate_child_path};
use super::*;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn read_saved_files_for_source(source: &Path) -> Result<Vec<String>> {
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

pub(super) fn backup_saved_files(
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

pub(super) fn restore_saved_files(
    destination: &Path,
    backups: Vec<(PathBuf, PathBuf)>,
) -> Result<()> {
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

pub(super) fn copy_js_package_dependencies(
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
pub(super) struct MaterializedGitSource {
    pub(super) root: PathBuf,
    pub(super) path: PathBuf,
}

pub(super) fn materialize_git_repo_source(
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

pub(super) fn copy_js_package_dependencies_from_git(
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
