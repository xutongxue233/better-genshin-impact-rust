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
    fallback_roots: Vec<PathBuf>,
}

impl AssetResolver {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            fallback_roots: Vec::new(),
        }
    }

    pub fn with_fallback_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.fallback_roots.push(root.into());
        self
    }

    pub fn source_root(&self) -> Result<PathBuf> {
        self.source_roots()
            .into_iter()
            .next()
            .ok_or_else(|| BgiError::InvalidWorkspace(self.workspace_root.clone()))
    }

    pub fn feature_asset_dir(&self, feature: &str, screen_size: ScreenSize) -> Result<PathBuf> {
        let source_roots = self.source_roots();
        if source_roots.is_empty() {
            return Err(BgiError::InvalidWorkspace(self.workspace_root.clone()));
        }

        let mut searched = Vec::new();
        for source_root in source_roots {
            let base = source_root.join("GameTask").join(feature).join("Assets");
            let preferred = base.join(screen_size.folder_name());
            if preferred.is_dir() {
                return Ok(preferred);
            }
            searched.push(preferred);

            let fallback = base.join(ScreenSize::FULL_HD.folder_name());
            if fallback.is_dir() {
                return Ok(fallback);
            }
            searched.push(fallback);
        }

        Err(BgiError::AssetNotFound {
            feature: feature.to_string(),
            name: "<asset-dir>".to_string(),
            searched,
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
        let source_root = self.source_root()?;
        let task_root = source_root.join("GameTask");
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

    fn source_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        if self.workspace_root.join("GameTask").is_dir() {
            roots.push(self.workspace_root.clone());
        }
        for root in &self.fallback_roots {
            if root.join("GameTask").is_dir() && !roots.iter().any(|existing| existing == root) {
                roots.push(root.clone());
            }
        }
        roots
    }
}

pub fn normalize_workspace_root(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref().to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn asset_resolver_uses_explicit_fallback_roots_without_legacy_source_tree() {
        let root = temp_root("asset-fallback");
        let fallback = root.join("rust-assets");
        let nested_source = root.join("legacy-source-tree");
        let fallback_asset_dir = fallback.join("GameTask/Foo/Assets/1920x1080");
        let nested_asset_dir = nested_source.join("GameTask/Foo/Assets/1920x1080");
        fs::create_dir_all(&fallback_asset_dir).unwrap();
        fs::create_dir_all(&nested_asset_dir).unwrap();
        fs::write(fallback_asset_dir.join("asset.png"), "rust").unwrap();
        fs::write(nested_asset_dir.join("asset.png"), "nested").unwrap();

        let resolver = AssetResolver::new(&root).with_fallback_root(&fallback);
        let resolved = resolver
            .resolve_feature_asset("Foo", "asset.png", ScreenSize::FULL_HD)
            .unwrap();

        assert!(resolved.starts_with(&fallback));
        let without_fallback = AssetResolver::new(&root)
            .resolve_feature_asset("Foo", "asset.png", ScreenSize::FULL_HD)
            .unwrap_err();
        assert!(matches!(
            without_fallback,
            BgiError::InvalidWorkspace(path) if path == root
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn asset_resolver_reports_invalid_workspace_without_any_source_root() {
        let root = temp_root("asset-invalid");
        fs::create_dir_all(&root).unwrap();

        let error = AssetResolver::new(&root)
            .feature_asset_dir("Foo", ScreenSize::FULL_HD)
            .unwrap_err();

        assert!(matches!(error, BgiError::InvalidWorkspace(_)));
        fs::remove_dir_all(root).unwrap();
    }

    fn temp_root(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("bgi-{name}-{suffix}"))
    }
}
