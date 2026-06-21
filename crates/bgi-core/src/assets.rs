use crate::error::{BgiError, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
}

impl ScreenSize {
    pub const FULL_HD: Self = Self {
        width: 1920,
        height: 1080,
    };

    pub fn folder_name(self) -> String {
        format!("{}x{}", self.width, self.height)
    }
}

impl Default for ScreenSize {
    fn default() -> Self {
        Self::FULL_HD
    }
}

#[derive(Debug, Clone)]
pub struct AssetResolver {
    workspace_root: PathBuf,
}

impl AssetResolver {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }

    pub fn source_root(&self) -> Result<PathBuf> {
        let direct = self.workspace_root.join("GameTask");
        if direct.is_dir() {
            return Ok(self.workspace_root.clone());
        }

        let nested = self.workspace_root.join("BetterGenshinImpact");
        if nested.join("GameTask").is_dir() {
            return Ok(nested);
        }

        Err(BgiError::InvalidWorkspace(self.workspace_root.clone()))
    }

    pub fn feature_asset_dir(&self, feature: &str, screen_size: ScreenSize) -> Result<PathBuf> {
        let source_root = self.source_root()?;
        let base = source_root.join("GameTask").join(feature).join("Assets");
        let preferred = base.join(screen_size.folder_name());

        if preferred.is_dir() {
            return Ok(preferred);
        }

        let fallback = base.join(ScreenSize::FULL_HD.folder_name());
        if fallback.is_dir() {
            return Ok(fallback);
        }

        Err(BgiError::AssetNotFound {
            feature: feature.to_string(),
            name: "<asset-dir>".to_string(),
            searched: vec![preferred, fallback],
        })
    }

    pub fn resolve_feature_asset(
        &self,
        feature: &str,
        asset_name: &str,
        screen_size: ScreenSize,
    ) -> Result<PathBuf> {
        let asset_dir = self.feature_asset_dir(feature, screen_size)?;
        let candidate = asset_dir.join(asset_name);
        if candidate.is_file() {
            return Ok(candidate);
        }

        Err(BgiError::AssetNotFound {
            feature: feature.to_string(),
            name: asset_name.to_string(),
            searched: vec![candidate],
        })
    }

    pub fn known_feature_asset_dirs(&self) -> Result<Vec<String>> {
        let task_root = self.source_root()?.join("GameTask");
        let mut features = Vec::new();

        for entry in std::fs::read_dir(&task_root)
            .map_err(|source| BgiError::io(task_root.clone(), source))?
        {
            let entry = entry.map_err(|source| BgiError::io(task_root.clone(), source))?;
            let path = entry.path();
            if !path.join("Assets").is_dir() {
                continue;
            }

            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                features.push(name.to_string());
            }
        }

        features.sort();
        Ok(features)
    }

    pub fn list_feature_assets(
        &self,
        feature: &str,
        screen_size: ScreenSize,
    ) -> Result<Vec<PathBuf>> {
        let asset_dir = self.feature_asset_dir(feature, screen_size)?;
        let mut assets = Vec::new();

        for entry in WalkDir::new(&asset_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if entry.file_type().is_file() {
                assets.push(entry.path().to_path_buf());
            }
        }

        assets.sort();
        Ok(assets)
    }
}

pub fn normalize_workspace_root(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref().to_path_buf()
}
