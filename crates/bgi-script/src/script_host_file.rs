use super::{image_from_mat_value, invalid_arg_for_method, Result, ScriptHostRuntimeError};
use crate::policy::ScriptFilePolicy;
use bgi_vision::{resize_bgr_nearest, BgrImage, Size as VisionSize};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImageMatResizePlan {
    pub width: f64,
    pub height: f64,
    pub interpolation: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImageMatReadPlan {
    pub normalized_path: PathBuf,
    pub color_mode: &'static str,
    pub resize: Option<ImageMatResizePlan>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImageMatWritePlan {
    pub normalized_path: PathBuf,
    pub source: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImageMatReadExecution {
    pub normalized_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub pixel_format: &'static str,
    pub pixels: Vec<u8>,
    pub color_mode: &'static str,
    pub resize: Option<ImageMatResizePlan>,
}

impl ImageMatReadExecution {
    fn from_image(
        normalized_path: PathBuf,
        image: BgrImage,
        resize: Option<ImageMatResizePlan>,
    ) -> Self {
        Self {
            normalized_path,
            width: image.size.width,
            height: image.size.height,
            pixel_format: "BGR24",
            pixels: image.pixels,
            color_mode: "color",
            resize,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImageMatWriteExecution {
    pub normalized_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub pixel_format: &'static str,
    pub bytes_written: u64,
}

#[derive(Debug, Clone)]
pub struct LimitedFileHost {
    policy: ScriptFilePolicy,
}

impl LimitedFileHost {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            policy: ScriptFilePolicy::new(root),
        }
    }

    pub fn with_policy(policy: ScriptFilePolicy) -> Self {
        Self { policy }
    }

    pub fn policy(&self) -> &ScriptFilePolicy {
        &self.policy
    }

    pub fn normalize_path(&self, path: &str) -> Result<PathBuf> {
        Ok(self.policy.normalize_path(path)?)
    }

    pub fn read_path_sync(&self, folder_path: &str) -> Result<Vec<String>> {
        let normalized = self.policy.normalize_path(folder_path)?;
        if !normalized.is_dir() {
            return Ok(Vec::new());
        }
        let root = self.policy.normalize_path(".")?;

        let mut entries = Vec::new();
        for entry in fs::read_dir(&normalized).map_err(|source| ScriptHostRuntimeError::Io {
            path: normalized.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| ScriptHostRuntimeError::Io {
                path: normalized.clone(),
                source,
            })?;
            entries.push(relative_to_root(&root, &entry.path()));
        }
        entries.sort();
        Ok(entries)
    }

    pub fn create_directory(&self, folder_path: &str) -> Result<bool> {
        let normalized = self.policy.normalize_path(folder_path)?;
        fs::create_dir_all(&normalized).map_err(|source| ScriptHostRuntimeError::Io {
            path: normalized,
            source,
        })?;
        Ok(true)
    }

    pub fn is_folder(&self, path: &str) -> Result<bool> {
        Ok(self.policy.normalize_path(path)?.is_dir())
    }

    pub fn is_file(&self, path: &str) -> Result<bool> {
        Ok(self.policy.normalize_path(path)?.is_file())
    }

    pub fn is_exists(&self, path: &str) -> Result<bool> {
        let normalized = self.policy.normalize_path(path)?;
        Ok(normalized.exists())
    }

    pub fn read_text_sync(&self, path: &str) -> Result<String> {
        let normalized = self.policy.normalize_path(path)?;
        fs::read_to_string(&normalized).map_err(|source| ScriptHostRuntimeError::Io {
            path: normalized,
            source,
        })
    }

    pub fn write_text_sync(&self, path: &str, content: &str, append: bool) -> Result<bool> {
        let normalized = self.policy.validate_text_write(path, content)?;
        if let Some(parent) = normalized.parent() {
            fs::create_dir_all(parent).map_err(|source| ScriptHostRuntimeError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        if append && normalized.exists() {
            use std::io::Write;
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(&normalized)
                .map_err(|source| ScriptHostRuntimeError::Io {
                    path: normalized.clone(),
                    source,
                })?;
            file.write_all(content.as_bytes())
                .map_err(|source| ScriptHostRuntimeError::Io {
                    path: normalized,
                    source,
                })?;
        } else {
            fs::write(&normalized, content).map_err(|source| ScriptHostRuntimeError::Io {
                path: normalized,
                source,
            })?;
        }
        Ok(true)
    }

    pub fn read_image_mat_plan_sync(&self, path: &str) -> Result<ImageMatReadPlan> {
        let normalized = self.normalize_image_read_path(path)?;
        Ok(ImageMatReadPlan {
            normalized_path: normalized,
            color_mode: "color",
            resize: None,
        })
    }

    pub fn read_image_mat_sync(&self, path: &str) -> Result<ImageMatReadExecution> {
        let normalized = self.normalize_image_read_path(path)?;
        let image = BgrImage::read(&normalized)?;
        Ok(ImageMatReadExecution::from_image(normalized, image, None))
    }

    pub fn read_image_mat_with_resize_plan_sync(
        &self,
        path: &str,
        width: f64,
        height: f64,
        interpolation: i32,
    ) -> Result<ImageMatReadPlan> {
        let resize = self.validate_image_resize_args(width, height, interpolation)?;
        let normalized = self.normalize_image_read_path(path)?;
        Ok(ImageMatReadPlan {
            normalized_path: normalized,
            color_mode: "color",
            resize: Some(resize),
        })
    }

    pub fn read_image_mat_with_resize_sync(
        &self,
        path: &str,
        width: f64,
        height: f64,
        interpolation: i32,
    ) -> Result<ImageMatReadExecution> {
        let resize = self.validate_image_resize_args(width, height, interpolation)?;
        let normalized = self.normalize_image_read_path(path)?;
        let image = resize_bgr_nearest(
            &BgrImage::read(&normalized)?,
            VisionSize::new(resize.width.round() as u32, resize.height.round() as u32),
        )?;
        Ok(ImageMatReadExecution::from_image(
            normalized,
            image,
            Some(resize),
        ))
    }

    fn validate_image_resize_args(
        &self,
        width: f64,
        height: f64,
        interpolation: i32,
    ) -> Result<ImageMatResizePlan> {
        if width <= 0.0 {
            return Err(invalid_arg_for_method(
                "file.ReadImageMatWithResizeSync",
                1,
                "positive width",
            ));
        }
        if height <= 0.0 {
            return Err(invalid_arg_for_method(
                "file.ReadImageMatWithResizeSync",
                2,
                "positive height",
            ));
        }
        if width.round() < 1.0 || height.round() < 1.0 {
            return Err(invalid_arg_for_method(
                "file.ReadImageMatWithResizeSync",
                1,
                "positive rounded image size",
            ));
        }
        if !(0..=5).contains(&interpolation) {
            return Err(invalid_arg_for_method(
                "file.ReadImageMatWithResizeSync",
                3,
                "OpenCV interpolation value 0..=5",
            ));
        }
        Ok(ImageMatResizePlan {
            width,
            height,
            interpolation,
        })
    }

    fn normalize_image_read_path(&self, path: &str) -> Result<PathBuf> {
        let normalized = self.policy.normalize_path(path)?;
        self.policy.validate_image_extension(&normalized)?;
        Ok(normalized)
    }

    pub fn write_image_plan_sync(&self, path: &str, source: Value) -> Result<ImageMatWritePlan> {
        let normalized = self.policy.normalize_image_write_target(path)?;
        Ok(ImageMatWritePlan {
            normalized_path: normalized,
            source,
        })
    }

    pub fn write_image_sync(&self, path: &str, source: Value) -> Result<ImageMatWriteExecution> {
        let plan = self.write_image_plan_sync(path, source)?;
        let image = image_from_mat_value(&plan.source)?;
        if let Some(parent) = plan.normalized_path.parent() {
            fs::create_dir_all(parent).map_err(|source| ScriptHostRuntimeError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        image.write_png(&plan.normalized_path)?;
        let bytes_written = fs::metadata(&plan.normalized_path)
            .map_err(|source| ScriptHostRuntimeError::Io {
                path: plan.normalized_path.clone(),
                source,
            })?
            .len();
        Ok(ImageMatWriteExecution {
            normalized_path: plan.normalized_path,
            width: image.size.width,
            height: image.size.height,
            pixel_format: "BGR24",
            bytes_written,
        })
    }

    pub fn rename_path_sync(&self, old_path: &str, new_path: &str) -> Result<bool> {
        let old_normalized = self.policy.normalize_path(old_path)?;
        let new_normalized = self.policy.normalize_path(new_path)?;
        if !old_normalized.exists() {
            return Err(ScriptHostRuntimeError::RenameSourceMissing(old_normalized));
        }
        if old_normalized.is_file() {
            self.policy.validate_write_extension(&new_normalized)?;
        }
        if new_normalized.exists() && old_normalized.is_file() != new_normalized.is_file() {
            return Err(ScriptHostRuntimeError::RenameKindMismatch {
                from: old_normalized,
                to: new_normalized,
            });
        }
        if let Some(parent) = new_normalized.parent() {
            fs::create_dir_all(parent).map_err(|source| ScriptHostRuntimeError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::rename(&old_normalized, &new_normalized).map_err(|source| {
            ScriptHostRuntimeError::Io {
                path: old_normalized,
                source,
            }
        })?;
        Ok(true)
    }
}

#[derive(Debug, Clone)]
pub struct StrategyFileHost {
    file_host: LimitedFileHost,
}

impl StrategyFileHost {
    pub fn new(auto_fight_root: impl Into<PathBuf>) -> Self {
        Self {
            file_host: LimitedFileHost::new(auto_fight_root),
        }
    }

    pub fn is_folder(&self, path: &str) -> Result<bool> {
        self.file_host.is_folder(path)
    }

    pub fn is_file(&self, path: &str) -> Result<bool> {
        self.file_host.is_file(path)
    }

    pub fn is_exists(&self, path: &str) -> Result<bool> {
        self.file_host.is_exists(path)
    }

    pub fn read_path_sync(&self, path: &str) -> Result<Vec<String>> {
        self.file_host.read_path_sync(path)
    }
}

fn relative_to_root(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
