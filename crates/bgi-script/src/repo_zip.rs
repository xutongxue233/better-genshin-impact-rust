use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use super::repo_files::{
    best_matching_existing_repo, find_file_named, generate_unique_folder_name,
    read_existing_repo_content,
};
use super::repo_paths::{copy_directory, lexical_normalize, path_starts_with, remove_existing};
use super::repo_planning_update::{add_update_markers_to_new_repo, calculate_repo_overlap_ratio};
use super::REPOS_DIR;
use super::{Result, ScriptRepoError, ScriptRepoZipImportExecution, ScriptRepoZipImportPlan};

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
