use std::io;
use std::path::PathBuf;

mod backend;
mod bv;
#[path = "geometry.rs"]
mod geometry;
mod grid;
mod onnx;
#[path = "recognition.rs"]
mod recognition;
#[path = "region.rs"]
mod region;

pub use backend::*;
pub use bv::*;
pub use geometry::*;
pub use grid::*;
pub use onnx::*;
pub use recognition::*;
pub use region::*;

#[derive(Debug, thiserror::Error)]
pub enum VisionError {
    #[error("rectangle dimensions must be non-negative")]
    InvalidRect,
    #[error("threshold must be in the 0.0..=1.0 range")]
    InvalidThreshold,
    #[error("color match count must be greater than zero")]
    InvalidColorMatchCount,
    #[error("template asset reference must be feature:asset")]
    InvalidBvImageAsset,
    #[error("{field} must be greater than zero")]
    NonPositiveDuration { field: &'static str },
    #[error("OCR match requires at least one contains or regex rule")]
    EmptyOcrMatchRules,
    #[error("invalid OCR regex pattern {pattern:?}: {source}")]
    InvalidRegex {
        pattern: String,
        source: regex::Error,
    },
    #[error("BGR image buffer length {actual} does not match expected length {expected}")]
    InvalidImageBuffer { expected: usize, actual: usize },
    #[error("image size mismatch: expected {expected:?}, got {actual:?}")]
    ImageSizeMismatch { expected: Size, actual: Size },
    #[error("failed to decode image {path:?}: {source}")]
    ImageDecode {
        path: Option<PathBuf>,
        #[source]
        source: image::ImageError,
    },
    #[error("failed to read image {path:?}: {source}")]
    ImageRead {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write image {path:?}: {source}")]
    ImageWrite {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },
    #[error("template asset is required for template matching")]
    MissingTemplateAsset,
    #[error("template asset was not registered: {0:?}")]
    TemplateAssetNotRegistered(PathBuf),
    #[error("template is larger than the search image or region")]
    TemplateLargerThanImage,
    #[error("recognition type {0:?} is not supported by this backend")]
    UnsupportedRecognitionType(RecognitionType),
    #[error("vision backend {0} is not implemented yet")]
    BackendNotImplemented(&'static str),
}

pub type Result<T> = std::result::Result<T, VisionError>;

#[cfg(test)]
mod tests;
