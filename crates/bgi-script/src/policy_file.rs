use super::policy_error::{Result, ScriptHostPolicyError};
use super::policy_path::normalize_script_path;
use serde::Serialize;
use std::path::{Path, PathBuf};

pub(crate) const DEFAULT_ALLOWED_FILE_EXTENSIONS: &[&str] = &[
    ".txt", ".json", ".log", ".csv", ".xml", ".html", ".css", ".png", ".jpg", ".jpeg", ".bmp",
    ".tiff", ".webp",
];
pub(crate) const DEFAULT_IMAGE_EXTENSIONS: &[&str] =
    &[".png", ".jpg", ".jpeg", ".bmp", ".tiff", ".webp"];
pub(crate) const DEFAULT_MAX_WRITE_BYTES: u64 = 999 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptFilePolicy {
    pub root: PathBuf,
    pub allowed_extensions: Vec<&'static str>,
    pub image_extensions: Vec<&'static str>,
    pub max_write_bytes: u64,
}

impl ScriptFilePolicy {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            allowed_extensions: DEFAULT_ALLOWED_FILE_EXTENSIONS.to_vec(),
            image_extensions: DEFAULT_IMAGE_EXTENSIONS.to_vec(),
            max_write_bytes: DEFAULT_MAX_WRITE_BYTES,
        }
    }

    pub fn normalize_path(&self, path: &str) -> Result<PathBuf> {
        normalize_script_path(&self.root, path)
    }

    pub fn validate_text_write(&self, path: &str, content: &str) -> Result<PathBuf> {
        let normalized = self.normalize_path(path)?;
        self.validate_write_extension(&normalized)?;
        let actual_bytes = content.len() as u64;
        if actual_bytes > self.max_write_bytes {
            return Err(ScriptHostPolicyError::ContentTooLarge {
                actual_bytes,
                max_bytes: self.max_write_bytes,
            });
        }
        Ok(normalized)
    }

    pub fn normalize_image_write_target(&self, path: &str) -> Result<PathBuf> {
        let path = ensure_image_extension(path, &self.image_extensions);
        let normalized = self.normalize_path(&path)?;
        self.validate_image_extension(&normalized)?;
        Ok(normalized)
    }

    pub fn validate_write_extension(&self, path: &Path) -> Result<()> {
        let extension = normalized_extension(path);
        if self
            .allowed_extensions
            .iter()
            .any(|allowed| *allowed == extension)
        {
            Ok(())
        } else {
            Err(ScriptHostPolicyError::ExtensionNotAllowed(extension))
        }
    }

    pub fn validate_image_extension(&self, path: &Path) -> Result<()> {
        let extension = normalized_extension(path);
        if self
            .image_extensions
            .iter()
            .any(|allowed| *allowed == extension)
        {
            Ok(())
        } else {
            Err(ScriptHostPolicyError::ExtensionNotAllowed(extension))
        }
    }
}

fn normalized_extension(path: &Path) -> String {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!(".{}", extension.to_ascii_lowercase()))
        .unwrap_or_default()
}

fn ensure_image_extension(path: &str, image_extensions: &[&str]) -> String {
    let extension = normalized_extension(Path::new(path));
    if image_extensions.iter().any(|allowed| *allowed == extension) {
        path.to_string()
    } else {
        format!("{path}.png")
    }
}
