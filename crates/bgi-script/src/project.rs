use crate::manifest::{Manifest, ManifestError};
use crate::policy::ScriptHostPolicyError;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ScriptProjectError {
    #[error("script directory was not found at {0:?}")]
    DirectoryNotFound(PathBuf),
    #[error("manifest.json was not found at {0:?}")]
    ManifestNotFound(PathBuf),
    #[error("manifest validation failed: {0}")]
    Manifest(#[from] ManifestError),
    #[error("main script is empty at {0:?}")]
    EmptyMainScript(PathBuf),
    #[error("module import {specifier:?} could not be resolved")]
    ModuleNotFound {
        specifier: String,
        referrer: Option<PathBuf>,
    },
    #[error(
        "module import resolved to unsupported file type at {0:?}; only .js modules are executable"
    )]
    UnsupportedModuleExtension(PathBuf),
    #[error("path policy rejected module path: {0}")]
    Policy(#[from] ScriptHostPolicyError),
    #[error("I/O error at {path:?}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, ScriptProjectError>;

#[path = "project_loader.rs"]
mod project_loader;

pub use project_loader::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptProjectLoaderSummary {
    pub default_search_paths: Vec<&'static str>,
    pub package_alias_rewrite: &'static str,
    pub module_detection: Vec<&'static str>,
    pub resource_import_rewrites: Vec<&'static str>,
    pub supported_module_extensions: Vec<&'static str>,
    pub module_resolution_order: Vec<&'static str>,
    pub caches_loaded_modules: bool,
}

impl Default for ScriptProjectLoaderSummary {
    fn default() -> Self {
        Self {
            default_search_paths: vec![".", "./packages"],
            package_alias_rewrite: "../../../packages -> packages",
            module_detection: vec![
                "manifest.library is non-empty",
                "code contains import ",
                "code contains export ",
            ],
            resource_import_rewrites: vec![
                "default image import -> file.ReadImageMatSync",
                "default non-JS import -> file.ReadTextSync",
            ],
            supported_module_extensions: vec![".js"],
            module_resolution_order: vec![
                "packages/ alias",
                "relative to referrer",
                "manifest library and ./packages search path",
                "script root",
                "stripped relative fallback under script root",
            ],
            caches_loaded_modules: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptProjectLayout {
    pub folder_name: String,
    pub project_path: PathBuf,
    pub manifest_path: PathBuf,
    pub main_script_path: PathBuf,
    pub settings_ui_path: Option<PathBuf>,
    pub search_paths: Vec<PathBuf>,
}

impl ScriptProjectLayout {
    pub fn new(
        scripts_root: impl AsRef<Path>,
        folder_name: impl Into<String>,
        manifest: &Manifest,
    ) -> Self {
        let folder_name = folder_name.into();
        let project_path = scripts_root.as_ref().join(&folder_name);
        let manifest_path = project_path.join("manifest.json");
        let main_script_path = project_path.join(&manifest.main);
        let settings_ui_path = (!manifest.settings_ui.trim().is_empty())
            .then(|| project_path.join(&manifest.settings_ui));

        let search_paths = normalized_search_paths(&project_path, manifest);

        Self {
            folder_name,
            project_path,
            manifest_path,
            main_script_path,
            settings_ui_path,
            search_paths,
        }
    }

    pub fn validate_existing(&self, manifest: &Manifest) -> Result<()> {
        if !self.project_path.is_dir() {
            return Err(ScriptProjectError::DirectoryNotFound(
                self.project_path.clone(),
            ));
        }
        if !self.manifest_path.is_file() {
            return Err(ScriptProjectError::ManifestNotFound(
                self.manifest_path.clone(),
            ));
        }
        manifest.validate_in_project(&self.project_path)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptProject {
    pub layout: ScriptProjectLayout,
    pub manifest: Manifest,
}

impl ScriptProject {
    pub fn load(scripts_root: impl AsRef<Path>, folder_name: impl Into<String>) -> Result<Self> {
        let folder_name = folder_name.into();
        let project_path = scripts_root.as_ref().join(&folder_name);
        if !project_path.is_dir() {
            return Err(ScriptProjectError::DirectoryNotFound(project_path));
        }

        let manifest_path = project_path.join("manifest.json");
        if !manifest_path.is_file() {
            return Err(ScriptProjectError::ManifestNotFound(manifest_path));
        }

        let manifest =
            Manifest::read_from(&manifest_path).map_err(|err| ScriptProjectError::Io {
                path: manifest_path.clone(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()),
            })?;
        let layout = ScriptProjectLayout::new(scripts_root, folder_name, &manifest);
        layout.validate_existing(&manifest)?;

        Ok(Self { layout, manifest })
    }

    pub fn read_main_code(&self) -> Result<String> {
        let code = fs::read_to_string(&self.layout.main_script_path).map_err(|source| {
            ScriptProjectError::Io {
                path: self.layout.main_script_path.clone(),
                source,
            }
        })?;
        if code.is_empty() {
            return Err(ScriptProjectError::EmptyMainScript(
                self.layout.main_script_path.clone(),
            ));
        }
        Ok(code)
    }

    pub fn execution_mode_for_code(&self, code: &str) -> ScriptCodeExecutionMode {
        execution_mode_for_code(&self.manifest, code)
    }

    pub fn loader_plan_for_code(&self, code: &str) -> ModuleLoaderPlan {
        ModuleLoaderPlan::from_project(
            self.layout.project_path.clone(),
            self.layout.search_paths.clone(),
            self.layout.main_script_path.clone(),
            code,
        )
    }

    pub fn module_loader(&self) -> Result<ScriptModuleLoader> {
        ScriptModuleLoader::new(
            self.layout.project_path.clone(),
            self.layout.search_paths.clone(),
        )
    }
}

#[cfg(test)]
#[path = "project_tests.rs"]
mod tests;
