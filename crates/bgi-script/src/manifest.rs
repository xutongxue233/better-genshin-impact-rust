use bgi_core::{BgiError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Author {
    pub name: String,
    pub link: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Manifest {
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    pub bgi_version: Option<String>,
    pub description: String,
    pub authors: Vec<Author>,
    pub main: String,
    pub settings_ui: String,
    pub scripts: Vec<String>,
    pub library: Vec<String>,
    pub saved_files: Vec<String>,
    pub http_allowed_urls: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            manifest_version: 1,
            name: String::new(),
            version: String::new(),
            bgi_version: None,
            description: String::new(),
            authors: Vec::new(),
            main: String::new(),
            settings_ui: String::new(),
            scripts: Vec::new(),
            library: Vec::new(),
            saved_files: Vec::new(),
            http_allowed_urls: Vec::new(),
            extra: Map::new(),
        }
    }
}

impl Manifest {
    pub fn from_json(json: &str) -> Result<Self> {
        json5::from_str(json).map_err(|err| BgiError::json(None::<PathBuf>, err.to_string()))
    }

    pub fn read_from(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| BgiError::io(path, source))?;
        Self::from_json(&text).map_err(|err| match err {
            BgiError::Json { message, .. } => BgiError::json(Some(path), message),
            other => other,
        })
    }

    pub fn validate_in_project(
        &self,
        project_dir: impl AsRef<Path>,
    ) -> std::result::Result<(), ManifestError> {
        let project_dir = project_dir.as_ref();
        if self.name.trim().is_empty() {
            return Err(ManifestError::MissingName);
        }

        if self.version.trim().is_empty() {
            return Err(ManifestError::MissingVersion);
        }

        if self.main.trim().is_empty() {
            return Err(ManifestError::MissingMain);
        }

        let main_path = project_dir.join(&self.main);
        if !main_path.is_file() {
            return Err(ManifestError::MainScriptNotFound(main_path));
        }

        Ok(())
    }

    pub fn short_description(&self) -> String {
        let mut lines: Vec<&str> = self.description.lines().collect();
        if lines.len() > 6 {
            lines.truncate(6);
            lines.push("...");
        }
        lines.join("\n")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("manifest.json: name is required")]
    MissingName,
    #[error("manifest.json: version is required")]
    MissingVersion,
    #[error("manifest.json: main script is required")]
    MissingMain,
    #[error("main script was not found at {0:?}")]
    MainScriptNotFound(PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_deserializes_legacy_shape() {
        let manifest = Manifest::from_json(
            r#"{
                "manifestVersion": 1,
                "name": "sample",
                "version": "1.0.0",
                "description": "line1\nline2",
                "authors": [{ "name": "tester", "link": "https://example.com" }],
                "main": "main.js",
                "httpAllowedUrls": ["https://example.com"]
            }"#,
        )
        .unwrap();

        assert_eq!(manifest.name, "sample");
        assert_eq!(manifest.authors[0].name, "tester");
        assert_eq!(manifest.http_allowed_urls, vec!["https://example.com"]);
    }
}
