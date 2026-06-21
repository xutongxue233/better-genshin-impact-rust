use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Result<Self> {
        if width < 0 || height < 0 {
            return Err(VisionError::InvalidRect);
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn right(self) -> i32 {
        self.x + self.width
    }

    pub fn bottom(self) -> i32 {
        self.y + self.height
    }

    pub fn center(self) -> Point {
        Point {
            x: self.x + self.width / 2,
            y: self.y + self.height / 2,
        }
    }

    pub fn is_empty(self) -> bool {
        self.x == 0 && self.y == 0 && self.width == 0 && self.height == 0
    }

    pub fn clamp_to(self, size: Size) -> Result<Self> {
        if self.width < 0 || self.height < 0 {
            return Err(VisionError::InvalidRect);
        }

        let max_width = size.width as i32;
        let max_height = size.height as i32;
        let x = self.x.clamp(0, max_width);
        let y = self.y.clamp(0, max_height);
        let right = self.right().clamp(x, max_width);
        let bottom = self.bottom().clamp(y, max_height);
        Self::new(x, y, right - x, bottom - y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecognitionType {
    None,
    TemplateMatch,
    ColorMatch,
    OcrMatch,
    Ocr,
    ColorRangeAndOcr,
    Detect,
}

impl RecognitionType {
    pub const ALL: [Self; 7] = [
        Self::None,
        Self::TemplateMatch,
        Self::ColorMatch,
        Self::OcrMatch,
        Self::Ocr,
        Self::ColorRangeAndOcr,
        Self::Detect,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecognitionTypeInfo {
    pub recognition_type: RecognitionType,
    pub implemented: bool,
    pub notes: &'static str,
}

pub fn recognition_type_infos() -> Vec<RecognitionTypeInfo> {
    RecognitionType::ALL
        .into_iter()
        .map(|recognition_type| RecognitionTypeInfo {
            recognition_type,
            implemented: matches!(
                recognition_type,
                RecognitionType::TemplateMatch | RecognitionType::ColorMatch
            ),
            notes: match recognition_type {
                RecognitionType::None => "Sentinel value kept for legacy config compatibility.",
                RecognitionType::TemplateMatch => {
                    "Pure Rust BGR24 template matcher and image asset decoding exist; OpenCV parity remains pending."
                }
                RecognitionType::ColorMatch => {
                    "Pure Rust BGR24/RGB/Gray/HSV color-range backend exists; full OpenCV parity remains pending."
                }
                RecognitionType::OcrMatch => {
                    "Rust OCR match rule evaluation exists; Paddle/ONNX OCR backend is still pending."
                }
                RecognitionType::Ocr => "Rust OCR result model exists; OCR backend is still pending.",
                RecognitionType::ColorRangeAndOcr => {
                    "Rust request model exists; color extraction plus OCR backend is still pending."
                }
                RecognitionType::Detect => "Detection request model exists; YOLO/ONNX backend is still pending.",
            },
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateMatchMode {
    SqDiff,
    SqDiffNormed,
    CCorr,
    CCorrNormed,
    CCoeff,
    CCoeffNormed,
}

impl Default for TemplateMatchMode {
    fn default() -> Self {
        Self::CCoeffNormed
    }
}

impl TemplateMatchMode {
    pub fn is_lower_better(self) -> bool {
        matches!(self, Self::SqDiff | Self::SqDiffNormed)
    }

    pub fn accepts_legacy_score(self, score: f64, threshold: f64) -> Result<bool> {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(VisionError::InvalidThreshold);
        }

        if self.is_lower_better() {
            Ok(score <= 1.0 - threshold)
        } else {
            Ok(score >= threshold)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorConversion {
    BgrToRgb,
    BgrToHsv,
    BgrToGray,
    BgraToBgr,
}

impl Default for ColorConversion {
    fn default() -> Self {
        Self::BgrToRgb
    }
}

impl ColorConversion {
    pub fn legacy_opencv_code(self) -> i32 {
        match self {
            Self::BgrToRgb => 4,
            Self::BgrToGray => 6,
            Self::BgrToHsv => 40,
            Self::BgraToBgr => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0 };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Scalar4 {
    pub v0: f64,
    pub v1: f64,
    pub v2: f64,
    pub v3: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateMatchConfig {
    pub template_asset: Option<PathBuf>,
    pub template_size: Option<Size>,
    pub threshold: f64,
    pub use_3_channels: bool,
    pub mode: TemplateMatchMode,
    pub use_mask: bool,
    pub mask_color: RgbColor,
    pub draw_on_window: bool,
    pub draw_color: RgbColor,
    pub max_match_count: i32,
    pub use_binary_match: bool,
    pub binary_threshold: u8,
}

impl Default for TemplateMatchConfig {
    fn default() -> Self {
        Self {
            template_asset: None,
            template_size: None,
            threshold: 0.8,
            use_3_channels: false,
            mode: TemplateMatchMode::default(),
            use_mask: false,
            mask_color: RgbColor::GREEN,
            draw_on_window: false,
            draw_color: RgbColor::RED,
            max_match_count: -1,
            use_binary_match: false,
            binary_threshold: 128,
        }
    }
}

impl TemplateMatchConfig {
    pub fn with_asset(path: impl Into<PathBuf>) -> Self {
        Self {
            template_asset: Some(path.into()),
            ..Self::default()
        }
    }

    pub fn validate(&self) -> Result<()> {
        if !(0.0..=1.0).contains(&self.threshold) {
            return Err(VisionError::InvalidThreshold);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorMatchConfig {
    pub conversion: ColorConversion,
    pub lower_color: Scalar4,
    pub upper_color: Scalar4,
    pub match_count: u32,
}

impl Default for ColorMatchConfig {
    fn default() -> Self {
        Self {
            conversion: ColorConversion::default(),
            lower_color: Scalar4::default(),
            upper_color: Scalar4::default(),
            match_count: 1,
        }
    }
}

impl ColorMatchConfig {
    pub fn validate(&self) -> Result<()> {
        if self.match_count == 0 {
            return Err(VisionError::InvalidColorMatchCount);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OcrEngineType {
    Paddle,
    YasModel,
    YapModel,
}

impl OcrEngineType {
    pub const ALL: [Self; 3] = [Self::Paddle, Self::YasModel, Self::YapModel];
}

impl Default for OcrEngineType {
    fn default() -> Self {
        Self::Paddle
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OcrMatchConfig {
    pub engine: OcrEngineType,
    pub replace_dictionary: BTreeMap<String, Vec<String>>,
    pub all_contain_match_text: Vec<String>,
    pub one_contain_match_text: Vec<String>,
    pub regex_match_text: Vec<String>,
    pub text: String,
}

impl Default for OcrMatchConfig {
    fn default() -> Self {
        Self {
            engine: OcrEngineType::default(),
            replace_dictionary: BTreeMap::new(),
            all_contain_match_text: Vec::new(),
            one_contain_match_text: Vec::new(),
            regex_match_text: Vec::new(),
            text: String::new(),
        }
    }
}

impl OcrMatchConfig {
    pub fn normalize_text(value: &str) -> String {
        value.chars().filter(|ch| !ch.is_whitespace()).collect()
    }

    pub fn apply_replacements(&self, value: &str) -> String {
        let mut text = Self::normalize_text(value);
        for (canonical, variants) in &self.replace_dictionary {
            for variant in variants {
                text = text.replace(variant, canonical);
            }
        }
        text
    }

    pub fn matches_text(&self, value: &str) -> Result<bool> {
        if self.all_contain_match_text.is_empty()
            && self.one_contain_match_text.is_empty()
            && self.regex_match_text.is_empty()
        {
            return Err(VisionError::EmptyOcrMatchRules);
        }

        let text = self.apply_replacements(value);
        let all_contains = self
            .all_contain_match_text
            .iter()
            .all(|needle| text.contains(needle));
        let one_contains = self.one_contain_match_text.is_empty()
            || self
                .one_contain_match_text
                .iter()
                .any(|needle| text.contains(needle));
        let regexes_match = self
            .regex_match_text
            .iter()
            .try_fold(true, |matched, pattern| {
                let regex = Regex::new(pattern).map_err(|source| VisionError::InvalidRegex {
                    pattern: pattern.clone(),
                    source,
                })?;
                Ok(matched && regex.is_match(&text))
            })?;

        Ok(all_contains && one_contains && regexes_match)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecognitionObject {
    pub recognition_type: RecognitionType,
    pub region_of_interest: Option<Rect>,
    pub name: Option<String>,
    pub template: TemplateMatchConfig,
    pub color: ColorMatchConfig,
    pub ocr: OcrMatchConfig,
}

impl Default for RecognitionObject {
    fn default() -> Self {
        Self {
            recognition_type: RecognitionType::None,
            region_of_interest: None,
            name: None,
            template: TemplateMatchConfig::default(),
            color: ColorMatchConfig::default(),
            ocr: OcrMatchConfig::default(),
        }
    }
}

impl RecognitionObject {
    pub fn template_match(path: impl Into<PathBuf>) -> Self {
        Self {
            recognition_type: RecognitionType::TemplateMatch,
            template: TemplateMatchConfig::with_asset(path),
            ..Self::default()
        }
    }

    pub fn template_match_in(path: impl Into<PathBuf>, region: Rect) -> Self {
        Self {
            region_of_interest: Some(region),
            ..Self::template_match(path)
        }
    }

    pub fn ocr(region: Rect) -> Self {
        Self {
            recognition_type: RecognitionType::Ocr,
            region_of_interest: Some(region),
            ..Self::default()
        }
    }

    pub fn ocr_match(
        region: Rect,
        match_texts: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let mut ocr = OcrMatchConfig::default();
        ocr.one_contain_match_text = match_texts.into_iter().map(Into::into).collect();
        Self {
            recognition_type: RecognitionType::OcrMatch,
            region_of_interest: Some(region),
            ocr,
            ..Self::default()
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.template.validate()?;
        self.color.validate()?;
        if matches!(self.recognition_type, RecognitionType::OcrMatch) {
            self.ocr.matches_text("")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Region {
    pub rect: Rect,
    pub text: String,
    pub score: Option<f32>,
}

impl Region {
    pub fn empty() -> Self {
        Self {
            rect: Rect::empty(),
            text: String::new(),
            score: None,
        }
    }

    pub fn is_exist(&self) -> bool {
        !self.rect.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrResultRegion {
    pub rect: Rect,
    pub text: String,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrRecognizerResult {
    pub text: String,
    pub score: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OcrResult {
    pub regions: Vec<OcrResultRegion>,
}

impl OcrResult {
    pub fn text(&self) -> String {
        let mut regions = self.regions.clone();
        regions.sort_by_key(|region| (region.rect.center().y, region.rect.center().x));
        regions
            .into_iter()
            .map(|region| region.text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn normalized_text(&self) -> String {
        OcrMatchConfig::normalize_text(&self.text())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecognitionMatch {
    pub recognition_type: RecognitionType,
    pub name: Option<String>,
    pub region: Region,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImageRegionSource {
    Capture,
    MatHandle(String),
    DerivedCrop,
    DerivedScale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageRegionModel {
    pub source: ImageRegionSource,
    pub rect: Rect,
    pub size: Size,
    pub owner: Option<Box<ImageRegionModel>>,
    pub text: String,
}

impl ImageRegionModel {
    pub fn capture(size: Size) -> Self {
        Self {
            source: ImageRegionSource::Capture,
            rect: Rect {
                x: 0,
                y: 0,
                width: size.width as i32,
                height: size.height as i32,
            },
            size,
            owner: None,
            text: String::new(),
        }
    }

    pub fn from_mat_handle(handle: impl Into<String>, size: Size, x: i32, y: i32) -> Self {
        Self {
            source: ImageRegionSource::MatHandle(handle.into()),
            rect: Rect {
                x,
                y,
                width: size.width as i32,
                height: size.height as i32,
            },
            size,
            owner: None,
            text: String::new(),
        }
    }

    pub fn derive_crop(&self, rect: Rect) -> Result<Self> {
        let clamped = rect.clamp_to(self.size)?;
        if clamped.width <= 0 || clamped.height <= 0 {
            return Err(VisionError::InvalidRect);
        }
        Ok(Self {
            source: ImageRegionSource::DerivedCrop,
            rect: clamped,
            size: Size {
                width: clamped.width as u32,
                height: clamped.height as u32,
            },
            owner: Some(Box::new(self.clone())),
            text: String::new(),
        })
    }

    pub fn derive_to_1080p(&self) -> Self {
        if self.size.width <= 1920 {
            return self.clone();
        }

        let scale = self.size.width as f64 / 1920.0;
        let height = (self.size.height as f64 / scale).round() as u32;
        Self {
            source: ImageRegionSource::DerivedScale,
            rect: Rect {
                x: 0,
                y: 0,
                width: 1920,
                height: height as i32,
            },
            size: Size {
                width: 1920,
                height,
            },
            owner: Some(Box::new(self.clone())),
            text: self.text.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageRegion {
    pub image: BgrImage,
    pub model: ImageRegionModel,
}

impl ImageRegion {
    pub fn capture(image: BgrImage) -> Self {
        let model = ImageRegionModel::capture(image.size);
        Self { image, model }
    }

    pub fn from_mat_handle(handle: impl Into<String>, image: BgrImage, x: i32, y: i32) -> Self {
        let model = ImageRegionModel::from_mat_handle(handle, image.size, x, y);
        Self { image, model }
    }

    pub fn derive_crop(&self, rect: Rect) -> Result<Self> {
        let model = self.model.derive_crop(rect)?;
        let image = crop_bgr_image(&self.image, model.rect)?;
        Ok(Self { image, model })
    }

    pub fn derive_to_1080p(&self) -> Result<Self> {
        let model = self.model.derive_to_1080p();
        if model.size == self.image.size {
            return Ok(self.clone());
        }
        let image = resize_bgr_nearest(&self.image, model.size)?;
        Ok(Self { image, model })
    }

    pub fn find<B: VisionBackend>(
        &self,
        backend: &B,
        object: &RecognitionObject,
    ) -> Result<Region> {
        let scoped = self.scoped_object(object)?;
        backend.find(&self.image.pixels, self.image.size, &scoped)
    }

    pub fn find_multi<B: VisionBackend>(
        &self,
        backend: &B,
        object: &RecognitionObject,
    ) -> Result<Vec<Region>> {
        let scoped = self.scoped_object(object)?;
        backend.find_multi(&self.image.pixels, self.image.size, &scoped)
    }

    fn scoped_object(&self, object: &RecognitionObject) -> Result<RecognitionObject> {
        let mut object = object.clone();
        object.region_of_interest = Some(
            object
                .region_of_interest
                .unwrap_or(Rect {
                    x: 0,
                    y: 0,
                    width: self.image.size.width as i32,
                    height: self.image.size.height as i32,
                })
                .clamp_to(self.image.size)?,
        );
        Ok(object)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BvImage {
    pub template_assert: String,
    pub feature_name: String,
    pub asset_name: String,
}

impl BvImage {
    pub fn new(template_assert: impl Into<String>) -> Result<Self> {
        let template_assert = template_assert.into();
        let (feature_name, asset_name) = template_assert
            .split_once(':')
            .ok_or(VisionError::InvalidBvImageAsset)?;
        if feature_name.trim().is_empty() || asset_name.trim().is_empty() {
            return Err(VisionError::InvalidBvImageAsset);
        }
        let feature_name = feature_name.to_string();
        let asset_name = asset_name.to_string();
        Ok(Self {
            template_assert,
            feature_name,
            asset_name,
        })
    }

    pub fn asset_path(&self) -> PathBuf {
        self.asset_path_for_screen(Size::new(1920, 1080))
    }

    pub fn asset_path_for_screen(&self, screen_size: Size) -> PathBuf {
        Path::new("GameTask")
            .join(&self.feature_name)
            .join("Assets")
            .join(format!("{}x{}", screen_size.width, screen_size.height))
            .join(&self.asset_name)
    }

    pub fn to_recognition_object(
        &self,
        roi: Option<Rect>,
        threshold: f64,
    ) -> Result<RecognitionObject> {
        let mut object = RecognitionObject::template_match(self.asset_path());
        object.name = Some(self.template_assert.clone());
        object.region_of_interest = roi;
        object.template.threshold = threshold;
        object.validate()?;
        Ok(object)
    }

    pub fn to_recognition_object_for_screen(
        &self,
        roi: Option<Rect>,
        threshold: f64,
        screen_size: Size,
    ) -> Result<RecognitionObject> {
        let mut object = RecognitionObject::template_match(self.asset_path_for_screen(screen_size));
        object.name = Some(self.template_assert.clone());
        object.region_of_interest = roi;
        object.template.threshold = threshold;
        object.validate()?;
        Ok(object)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BvLocatorOperation {
    FindAll,
    IsExist,
    WaitFor,
    WaitForDisappear,
    Click,
    ClickUntilDisappears,
    DoubleClick,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BvLocator {
    pub recognition_object: RecognitionObject,
    pub timeout_ms: Option<u32>,
    pub retry_interval_ms: Option<u32>,
    pub retry_action: Option<String>,
}

impl BvLocator {
    pub const DEFAULT_TIMEOUT_MS: u32 = 10_000;
    pub const DEFAULT_RETRY_INTERVAL_MS: u32 = 250;

    pub fn new(recognition_object: RecognitionObject) -> Self {
        Self {
            recognition_object,
            timeout_ms: None,
            retry_interval_ms: None,
            retry_action: None,
        }
    }

    pub fn with_roi(mut self, rect: Rect) -> Self {
        self.recognition_object.region_of_interest = Some(rect);
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u32) -> Result<Self> {
        if timeout_ms == 0 {
            return Err(VisionError::NonPositiveDuration { field: "timeout" });
        }
        self.timeout_ms = Some(timeout_ms);
        Ok(self)
    }

    pub fn with_retry_interval(mut self, retry_interval_ms: u32) -> Result<Self> {
        if retry_interval_ms == 0 {
            return Err(VisionError::NonPositiveDuration {
                field: "retry_interval",
            });
        }
        self.retry_interval_ms = Some(retry_interval_ms);
        Ok(self)
    }

    pub fn with_retry_action(mut self, callback_id: impl Into<String>) -> Self {
        self.retry_action = Some(callback_id.into());
        self
    }

    pub fn timeout_or_default(&self, override_timeout_ms: Option<u32>) -> u32 {
        override_timeout_ms
            .or(self.timeout_ms)
            .unwrap_or(Self::DEFAULT_TIMEOUT_MS)
    }

    pub fn retry_interval_or_default(&self) -> u32 {
        self.retry_interval_ms
            .unwrap_or(Self::DEFAULT_RETRY_INTERVAL_MS)
    }

    pub fn plan(&self, operation: BvLocatorOperation, timeout_ms: Option<u32>) -> BvLocatorPlan {
        let timeout_ms = self.timeout_or_default(timeout_ms);
        let retry_interval_ms = self.retry_interval_or_default();
        BvLocatorPlan {
            operation,
            recognition_object: self.recognition_object.clone(),
            timeout_ms,
            retry_interval_ms,
            retry_count: std::cmp::max(1, timeout_ms / retry_interval_ms),
            retry_action: self.retry_action.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BvLocatorPlan {
    pub operation: BvLocatorOperation,
    pub recognition_object: RecognitionObject,
    pub timeout_ms: u32,
    pub retry_interval_ms: u32,
    pub retry_count: u32,
    pub retry_action: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BvPageCommand {
    Screenshot {
        size: Size,
    },
    Wait {
        milliseconds: u32,
    },
    Click1080p {
        x: f64,
        y: f64,
        capture_size: Size,
        screen_x: f64,
        screen_y: f64,
    },
    Ocr {
        locator: BvLocatorPlan,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BvPage {
    pub default_timeout_ms: u32,
    pub default_retry_interval_ms: u32,
    pub capture_size: Size,
}

impl Default for BvPage {
    fn default() -> Self {
        Self {
            default_timeout_ms: 10_000,
            default_retry_interval_ms: 1_000,
            capture_size: Size::new(1920, 1080),
        }
    }
}

impl BvPage {
    pub fn screenshot(&self) -> BvPageCommand {
        BvPageCommand::Screenshot {
            size: self.capture_size,
        }
    }

    pub fn wait(&self, milliseconds: u32) -> Result<BvPageCommand> {
        if milliseconds == 0 {
            return Err(VisionError::NonPositiveDuration {
                field: "milliseconds",
            });
        }
        Ok(BvPageCommand::Wait { milliseconds })
    }

    pub fn locator_for_image(
        &self,
        image: &BvImage,
        roi: Option<Rect>,
        threshold: f64,
    ) -> Result<BvLocator> {
        Ok(BvLocator::new(image.to_recognition_object_for_screen(
            roi,
            threshold,
            self.capture_size,
        )?))
    }

    pub fn locator_for_text(&self, text: impl Into<String>, roi: Option<Rect>) -> BvLocator {
        let mut object = RecognitionObject::ocr(roi.unwrap_or_else(Rect::empty));
        object.ocr.text = text.into();
        BvLocator::new(object)
    }

    pub fn ocr(&self, rect: Option<Rect>) -> BvPageCommand {
        let locator = self.locator_for_text("", rect);
        BvPageCommand::Ocr {
            locator: locator.plan(BvLocatorOperation::FindAll, Some(self.default_timeout_ms)),
        }
    }

    pub fn click_1080p(&self, x: f64, y: f64) -> BvPageCommand {
        let scale = self.capture_size.width as f64 / 1920.0;
        BvPageCommand::Click1080p {
            x,
            y,
            capture_size: self.capture_size,
            screen_x: x * scale,
            screen_y: y * scale,
        }
    }
}

pub trait VisionBackend {
    fn find(&self, image: &[u8], image_size: Size, object: &RecognitionObject) -> Result<Region>;
    fn find_multi(
        &self,
        image: &[u8],
        image_size: Size,
        object: &RecognitionObject,
    ) -> Result<Vec<Region>>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BgrImage {
    pub size: Size,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BgrPixel {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl BgrImage {
    pub fn new(size: Size, pixels: Vec<u8>) -> Result<Self> {
        validate_bgr_len(size, pixels.len())?;
        Ok(Self { size, pixels })
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        Self::decode_with_path(bytes, None)
    }

    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| VisionError::ImageRead {
            path: path.to_path_buf(),
            source,
        })?;
        Self::decode_with_path(&bytes, Some(path.to_path_buf()))
    }

    pub fn write_png(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        image::save_buffer_with_format(
            path,
            &self.to_rgb_bytes(),
            self.size.width,
            self.size.height,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )
        .map_err(|source| VisionError::ImageWrite {
            path: path.to_path_buf(),
            source,
        })
    }

    fn decode_with_path(bytes: &[u8], path: Option<PathBuf>) -> Result<Self> {
        let image = image::load_from_memory(bytes)
            .map_err(|source| VisionError::ImageDecode { path, source })?
            .to_rgb8();
        let size = Size::new(image.width(), image.height());
        let mut pixels = Vec::with_capacity(size.width as usize * size.height as usize * 3);
        for pixel in image.pixels() {
            let [r, g, b] = pixel.0;
            pixels.extend_from_slice(&[b, g, r]);
        }
        Self::new(size, pixels)
    }

    pub fn to_rgb_bytes(&self) -> Vec<u8> {
        let mut rgb = Vec::with_capacity(self.pixels.len());
        for chunk in self.pixels.chunks_exact(3) {
            rgb.extend_from_slice(&[chunk[2], chunk[1], chunk[0]]);
        }
        rgb
    }

    fn pixel(&self, x: u32, y: u32) -> [f64; 3] {
        bgr_pixel(&self.pixels, self.size, x, y)
    }

    pub fn bgr_pixel_at(&self, x: u32, y: u32) -> Option<BgrPixel> {
        if x >= self.size.width || y >= self.size.height {
            return None;
        }
        let index = ((y * self.size.width + x) as usize) * 3;
        Some(BgrPixel {
            b: self.pixels[index],
            g: self.pixels[index + 1],
            r: self.pixels[index + 2],
        })
    }

    pub fn rgb_pixel_at(&self, x: u32, y: u32) -> Option<RgbPixel> {
        self.bgr_pixel_at(x, y).map(|pixel| RgbPixel {
            r: pixel.r,
            g: pixel.g,
            b: pixel.b,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorPlaneImage {
    pub size: Size,
    pub channels: u8,
    pub pixels: Vec<u8>,
}

impl ColorPlaneImage {
    pub fn new(size: Size, channels: u8, pixels: Vec<u8>) -> Result<Self> {
        let expected = size.width as usize * size.height as usize * channels as usize;
        if pixels.len() != expected {
            return Err(VisionError::InvalidImageBuffer {
                expected,
                actual: pixels.len(),
            });
        }
        Ok(Self {
            size,
            channels,
            pixels,
        })
    }

    pub fn channel_values(&self, x: u32, y: u32) -> &[u8] {
        let index = ((y * self.size.width + x) as usize) * self.channels as usize;
        &self.pixels[index..index + self.channels as usize]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorRangeMask {
    pub size: Size,
    pub pixels: Vec<u8>,
    pub matched_count: u32,
}

impl ColorRangeMask {
    pub fn new(size: Size, pixels: Vec<u8>) -> Result<Self> {
        let expected = size.width as usize * size.height as usize;
        if pixels.len() != expected {
            return Err(VisionError::InvalidImageBuffer {
                expected,
                actual: pixels.len(),
            });
        }
        let matched_count = pixels.iter().filter(|value| **value != 0).count() as u32;
        Ok(Self {
            size,
            pixels,
            matched_count,
        })
    }

    pub fn bounding_rect(&self, roi: Option<Rect>) -> Result<Option<Rect>> {
        let region = search_region(roi, self.size)?;
        let mut left = region.right();
        let mut top = region.bottom();
        let mut right = region.x;
        let mut bottom = region.y;
        for y in region.y..region.bottom() {
            for x in region.x..region.right() {
                let index = (y as u32 * self.size.width + x as u32) as usize;
                if self.pixels[index] == 0 {
                    continue;
                }
                left = left.min(x);
                top = top.min(y);
                right = right.max(x + 1);
                bottom = bottom.max(y + 1);
            }
        }
        if right <= left || bottom <= top {
            return Ok(None);
        }
        Ok(Some(Rect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }))
    }
}

#[derive(Debug, Clone, Default)]
pub struct PureRustVisionBackend {
    templates: HashMap<PathBuf, BgrImage>,
    template_roots: Vec<PathBuf>,
}

impl PureRustVisionBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_template(mut self, path: impl Into<PathBuf>, template: BgrImage) -> Self {
        self.templates.insert(path.into(), template);
        self
    }

    pub fn with_template_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.template_roots.push(root.into());
        self
    }

    pub fn register_template(&mut self, path: impl Into<PathBuf>, template: BgrImage) {
        self.templates.insert(path.into(), template);
    }

    pub fn register_template_file(&mut self, path: impl Into<PathBuf>) -> Result<()> {
        let path = path.into();
        let image = BgrImage::read(&path)?;
        self.register_template(path, image);
        Ok(())
    }

    pub fn template_roots(&self) -> &[PathBuf] {
        &self.template_roots
    }

    fn load_template(&self, object: &RecognitionObject) -> Result<BgrImage> {
        let path = object
            .template
            .template_asset
            .as_ref()
            .ok_or(VisionError::MissingTemplateAsset)?;
        if let Some(template) = self.templates.get(path) {
            return Ok(template.clone());
        }
        if path.is_absolute() && path.is_file() {
            return BgrImage::read(path);
        }
        for root in &self.template_roots {
            let candidate = root.join(path);
            if candidate.is_file() {
                return BgrImage::read(candidate);
            }
        }
        Err(VisionError::TemplateAssetNotRegistered(path.clone()))
    }
}

impl VisionBackend for PureRustVisionBackend {
    fn find(&self, image: &[u8], image_size: Size, object: &RecognitionObject) -> Result<Region> {
        Ok(self
            .find_multi(image, image_size, object)?
            .into_iter()
            .next()
            .unwrap_or_else(Region::empty))
    }

    fn find_multi(
        &self,
        image: &[u8],
        image_size: Size,
        object: &RecognitionObject,
    ) -> Result<Vec<Region>> {
        object.validate()?;
        validate_bgr_len(image_size, image.len())?;
        match object.recognition_type {
            RecognitionType::TemplateMatch => {
                let template = self.load_template(object)?;
                let region = search_region(object.region_of_interest, image_size)?;
                find_template_matches(image, image_size, &template, object, region)
            }
            RecognitionType::ColorMatch => find_color_match(image, image_size, object),
            other => Err(VisionError::UnsupportedRecognitionType(other)),
        }
    }
}

fn validate_bgr_len(size: Size, actual: usize) -> Result<()> {
    let expected = size.width as usize * size.height as usize * 3;
    if actual != expected {
        return Err(VisionError::InvalidImageBuffer { expected, actual });
    }
    Ok(())
}

pub fn crop_bgr_image(image: &BgrImage, rect: Rect) -> Result<BgrImage> {
    let rect = rect.clamp_to(image.size)?;
    if rect.width <= 0 || rect.height <= 0 {
        return Err(VisionError::InvalidRect);
    }
    let mut pixels = Vec::with_capacity(rect.width as usize * rect.height as usize * 3);
    for y in rect.y as u32..rect.bottom() as u32 {
        let start = ((y * image.size.width + rect.x as u32) as usize) * 3;
        let end = start + rect.width as usize * 3;
        pixels.extend_from_slice(&image.pixels[start..end]);
    }
    BgrImage::new(Size::new(rect.width as u32, rect.height as u32), pixels)
}

pub fn resize_bgr_nearest(image: &BgrImage, size: Size) -> Result<BgrImage> {
    if size.width == 0 || size.height == 0 {
        return Err(VisionError::InvalidRect);
    }
    let mut pixels = Vec::with_capacity(size.width as usize * size.height as usize * 3);
    for y in 0..size.height {
        let source_y = (y as u64 * image.size.height as u64 / size.height as u64) as u32;
        for x in 0..size.width {
            let source_x = (x as u64 * image.size.width as u64 / size.width as u64) as u32;
            let index = ((source_y * image.size.width + source_x) as usize) * 3;
            pixels.extend_from_slice(&image.pixels[index..index + 3]);
        }
    }
    BgrImage::new(size, pixels)
}

pub fn convert_bgr_image(
    image: &[u8],
    image_size: Size,
    conversion: ColorConversion,
) -> Result<ColorPlaneImage> {
    validate_bgr_len(image_size, image.len())?;
    let pixel_count = image_size.width as usize * image_size.height as usize;
    let channels = match conversion {
        ColorConversion::BgrToGray => 1,
        ColorConversion::BgrToRgb | ColorConversion::BgrToHsv | ColorConversion::BgraToBgr => 3,
    };
    let mut pixels = Vec::with_capacity(pixel_count * channels);
    for chunk in image.chunks_exact(3) {
        let b = chunk[0];
        let g = chunk[1];
        let r = chunk[2];
        match conversion {
            ColorConversion::BgrToRgb => pixels.extend_from_slice(&[r, g, b]),
            ColorConversion::BgrToGray => {
                pixels.push(gray_from_bgr([b as f64, g as f64, r as f64]).round() as u8)
            }
            ColorConversion::BgrToHsv => pixels.extend_from_slice(&bgr_to_opencv_hsv(b, g, r)),
            ColorConversion::BgraToBgr => pixels.extend_from_slice(&[b, g, r]),
        }
    }
    ColorPlaneImage::new(image_size, channels as u8, pixels)
}

pub fn in_range_mask(
    image: &ColorPlaneImage,
    lower: Scalar4,
    upper: Scalar4,
    roi: Option<Rect>,
) -> Result<ColorRangeMask> {
    let region = search_region(roi, image.size)?;
    let mut pixels = vec![0; image.size.width as usize * image.size.height as usize];
    for y in region.y..region.bottom() {
        for x in region.x..region.right() {
            let values = image.channel_values(x as u32, y as u32);
            if scalar_contains(values, lower, upper) {
                pixels[(y as u32 * image.size.width + x as u32) as usize] = 255;
            }
        }
    }
    ColorRangeMask::new(image.size, pixels)
}

fn scalar_contains(values: &[u8], lower: Scalar4, upper: Scalar4) -> bool {
    let lower = [lower.v0, lower.v1, lower.v2, lower.v3];
    let upper = [upper.v0, upper.v1, upper.v2, upper.v3];
    values.iter().enumerate().all(|(index, value)| {
        let value = *value as f64;
        value >= lower[index] && value <= upper[index]
    })
}

fn find_color_match(
    image: &[u8],
    image_size: Size,
    object: &RecognitionObject,
) -> Result<Vec<Region>> {
    let converted = convert_bgr_image(image, image_size, object.color.conversion)?;
    let mask = in_range_mask(
        &converted,
        object.color.lower_color,
        object.color.upper_color,
        object.region_of_interest,
    )?;
    if mask.matched_count < object.color.match_count {
        return Ok(Vec::new());
    }
    let Some(rect) = mask.bounding_rect(object.region_of_interest)? else {
        return Ok(Vec::new());
    };
    Ok(vec![Region {
        rect,
        text: object.name.clone().unwrap_or_default(),
        score: Some(mask.matched_count as f32),
    }])
}

fn search_region(region: Option<Rect>, image_size: Size) -> Result<Rect> {
    region
        .unwrap_or(Rect {
            x: 0,
            y: 0,
            width: image_size.width as i32,
            height: image_size.height as i32,
        })
        .clamp_to(image_size)
}

fn find_template_matches(
    image: &[u8],
    image_size: Size,
    template: &BgrImage,
    object: &RecognitionObject,
    region: Rect,
) -> Result<Vec<Region>> {
    if template.size.width == 0
        || template.size.height == 0
        || region.width < template.size.width as i32
        || region.height < template.size.height as i32
    {
        return Err(VisionError::TemplateLargerThanImage);
    }

    let max_x = region.x + region.width - template.size.width as i32;
    let max_y = region.y + region.height - template.size.height as i32;
    let mut matches = Vec::new();
    for y in region.y..=max_y {
        for x in region.x..=max_x {
            let score = template_match_score(
                image,
                image_size,
                template,
                x as u32,
                y as u32,
                &object.template,
            );
            if object
                .template
                .mode
                .accepts_legacy_score(score, object.template.threshold)?
            {
                matches.push(Region {
                    rect: Rect {
                        x,
                        y,
                        width: template.size.width as i32,
                        height: template.size.height as i32,
                    },
                    text: object.name.clone().unwrap_or_default(),
                    score: Some(score as f32),
                });
            }
        }
    }

    if object.template.mode.is_lower_better() {
        matches.sort_by(|left, right| {
            left.score
                .partial_cmp(&right.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        matches.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    if object.template.max_match_count > 0 {
        matches.truncate(object.template.max_match_count as usize);
    } else if object.template.max_match_count == 0 {
        matches.clear();
    }
    Ok(matches)
}

fn template_match_score(
    image: &[u8],
    image_size: Size,
    template: &BgrImage,
    origin_x: u32,
    origin_y: u32,
    config: &TemplateMatchConfig,
) -> f64 {
    let channels = if config.use_3_channels && !config.use_binary_match {
        3
    } else {
        1
    };
    let mut image_values =
        Vec::with_capacity(template.size.width as usize * template.size.height as usize * channels);
    let mut template_values = Vec::with_capacity(image_values.capacity());
    for ty in 0..template.size.height {
        for tx in 0..template.size.width {
            let image_pixel = bgr_pixel(image, image_size, origin_x + tx, origin_y + ty);
            let template_pixel = template.pixel(tx, ty);
            if config.use_binary_match {
                image_values.push(binary_gray_from_bgr(image_pixel, config.binary_threshold));
                template_values.push(binary_gray_from_bgr(
                    template_pixel,
                    config.binary_threshold,
                ));
            } else if config.use_3_channels {
                image_values.extend_from_slice(&image_pixel);
                template_values.extend_from_slice(&template_pixel);
            } else {
                image_values.push(gray_from_bgr(image_pixel));
                template_values.push(gray_from_bgr(template_pixel));
            }
        }
    }

    match config.mode {
        TemplateMatchMode::SqDiff => sum_sq_diff(&image_values, &template_values),
        TemplateMatchMode::SqDiffNormed => normalized_sq_diff(&image_values, &template_values),
        TemplateMatchMode::CCorr => dot(&image_values, &template_values),
        TemplateMatchMode::CCorrNormed => normalized_dot(&image_values, &template_values),
        TemplateMatchMode::CCoeff => centered_dot(&image_values, &template_values),
        TemplateMatchMode::CCoeffNormed => centered_normalized_dot(&image_values, &template_values),
    }
}

fn bgr_pixel(pixels: &[u8], size: Size, x: u32, y: u32) -> [f64; 3] {
    let index = ((y * size.width + x) as usize) * 3;
    [
        pixels[index] as f64,
        pixels[index + 1] as f64,
        pixels[index + 2] as f64,
    ]
}

fn gray_from_bgr(pixel: [f64; 3]) -> f64 {
    0.114 * pixel[0] + 0.587 * pixel[1] + 0.299 * pixel[2]
}

fn binary_gray_from_bgr(pixel: [f64; 3], threshold: u8) -> f64 {
    if (gray_from_bgr(pixel).round() as u8) >= threshold {
        255.0
    } else {
        0.0
    }
}

fn bgr_to_opencv_hsv(b: u8, g: u8, r: u8) -> [u8; 3] {
    let b = b as f64 / 255.0;
    let g = g as f64 / 255.0;
    let r = r as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let hue_degrees = if delta <= f64::EPSILON {
        0.0
    } else if (max - r).abs() <= f64::EPSILON {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if (max - g).abs() <= f64::EPSILON {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    let saturation = if max <= f64::EPSILON {
        0.0
    } else {
        delta / max
    };
    [
        (hue_degrees / 2.0).round().clamp(0.0, 179.0) as u8,
        (saturation * 255.0).round().clamp(0.0, 255.0) as u8,
        (max * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

fn dot(left: &[f64], right: &[f64]) -> f64 {
    left.iter().zip(right).map(|(a, b)| a * b).sum()
}

fn sum_sq(values: &[f64]) -> f64 {
    values.iter().map(|value| value * value).sum()
}

fn sum_sq_diff(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(a, b)| {
            let diff = a - b;
            diff * diff
        })
        .sum()
}

fn normalized_dot(left: &[f64], right: &[f64]) -> f64 {
    let denom = (sum_sq(left) * sum_sq(right)).sqrt();
    if denom <= f64::EPSILON {
        return 0.0;
    }
    (dot(left, right) / denom).clamp(-1.0, 1.0)
}

fn normalized_sq_diff(left: &[f64], right: &[f64]) -> f64 {
    let denom = (sum_sq(left) * sum_sq(right)).sqrt();
    if denom <= f64::EPSILON {
        return if sum_sq_diff(left, right) <= f64::EPSILON {
            0.0
        } else {
            1.0
        };
    }
    (sum_sq_diff(left, right) / denom).clamp(0.0, 1.0)
}

fn centered_values(values: &[f64]) -> Vec<f64> {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.iter().map(|value| value - mean).collect()
}

fn centered_dot(left: &[f64], right: &[f64]) -> f64 {
    dot(&centered_values(left), &centered_values(right))
}

fn centered_normalized_dot(left: &[f64], right: &[f64]) -> f64 {
    let left = centered_values(left);
    let right = centered_values(right);
    normalized_dot(&left, &right)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    TensorRt,
    Cuda,
    Dml,
    Cpu,
    Dnnl,
    OpenVino,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InferenceDeviceType {
    Cpu,
    GpuDirectMl,
    Gpu,
    OpenVino,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct OnnxProviderSelection {
    pub enable_tensor_rt_cache: bool,
    pub providers: &'static [ProviderType],
}

impl OnnxProviderSelection {
    pub const CPU: Self = Self {
        enable_tensor_rt_cache: false,
        providers: &[ProviderType::Cpu],
    };

    pub fn supports_tensor_rt(self) -> bool {
        self.providers.contains(&ProviderType::TensorRt)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OnnxModel {
    pub rust_name: &'static str,
    pub legacy_registered_name: &'static str,
    pub model_relative_path: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnnxModelLoadSource {
    SourceModel,
    TensorRtAnonymousCache,
    TensorRtNamedCache,
    PreCachedModelPath,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OnnxModelLoadPlan {
    pub model_name: String,
    pub model_path: PathBuf,
    pub source_model_path: PathBuf,
    pub cache_dir: PathBuf,
    pub anonymous_tensor_rt_cache_path: PathBuf,
    pub named_tensor_rt_cache_path: PathBuf,
    pub source: OnnxModelLoadSource,
    pub exists: bool,
    pub will_generate_tensor_rt_cache: bool,
    pub message: String,
}

impl OnnxModel {
    pub fn cache_relative_path(&self, version: &str) -> PathBuf {
        Path::new("Cache")
            .join(version)
            .join("Model")
            .join(self.legacy_registered_name)
    }

    pub fn source_model_path(&self, workspace_root: impl AsRef<Path>) -> PathBuf {
        workspace_root.as_ref().join(self.model_relative_path)
    }

    pub fn cache_dir(&self, workspace_root: impl AsRef<Path>, version: &str) -> PathBuf {
        workspace_root
            .as_ref()
            .join(self.cache_relative_path(version))
    }

    pub fn load_plan(
        &self,
        workspace_root: impl AsRef<Path>,
        version: &str,
        provider_selection: OnnxProviderSelection,
    ) -> OnnxModelLoadPlan {
        let workspace_root = workspace_root.as_ref();
        let source_model_path = self.source_model_path(workspace_root);
        let cache_dir = self.cache_dir(workspace_root, version);
        let trt_dir = cache_dir.join("trt");
        let anonymous_tensor_rt_cache_path = trt_dir.join("_ctx.onnx");
        let named_tensor_rt_cache_path = trt_dir.join(format!(
            "{}_ctx.onnx",
            source_model_path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or(self.rust_name)
        ));
        let cache_prefix = self.cache_relative_path(version);
        let pre_cached = Path::new(self.model_relative_path).starts_with(&cache_prefix)
            && self.model_relative_path.ends_with("_ctx.onnx");

        let (model_path, source, will_generate_tensor_rt_cache) =
            if provider_selection.enable_tensor_rt_cache && provider_selection.supports_tensor_rt()
            {
                if pre_cached {
                    (
                        source_model_path.clone(),
                        OnnxModelLoadSource::PreCachedModelPath,
                        false,
                    )
                } else if anonymous_tensor_rt_cache_path.is_file() {
                    (
                        anonymous_tensor_rt_cache_path.clone(),
                        OnnxModelLoadSource::TensorRtAnonymousCache,
                        false,
                    )
                } else if named_tensor_rt_cache_path.is_file() {
                    (
                        named_tensor_rt_cache_path.clone(),
                        OnnxModelLoadSource::TensorRtNamedCache,
                        false,
                    )
                } else {
                    (
                        source_model_path.clone(),
                        OnnxModelLoadSource::SourceModel,
                        true,
                    )
                }
            } else {
                (
                    source_model_path.clone(),
                    OnnxModelLoadSource::SourceModel,
                    false,
                )
            };
        let exists = model_path.is_file();
        let source = if exists {
            source
        } else {
            OnnxModelLoadSource::Missing
        };
        let message = match source {
            OnnxModelLoadSource::SourceModel if will_generate_tensor_rt_cache => {
                "source model will be loaded and TensorRT cache generation may run".to_string()
            }
            OnnxModelLoadSource::SourceModel => "source model will be loaded".to_string(),
            OnnxModelLoadSource::TensorRtAnonymousCache => {
                "anonymous TensorRT cache model will be loaded".to_string()
            }
            OnnxModelLoadSource::TensorRtNamedCache => {
                "named TensorRT cache model will be loaded".to_string()
            }
            OnnxModelLoadSource::PreCachedModelPath => {
                "model path already points at a TensorRT cache model".to_string()
            }
            OnnxModelLoadSource::Missing => {
                format!("model file is missing: {}", model_path.display())
            }
        };

        OnnxModelLoadPlan {
            model_name: self.rust_name.to_string(),
            model_path,
            source_model_path,
            cache_dir,
            anonymous_tensor_rt_cache_path,
            named_tensor_rt_cache_path,
            source,
            exists,
            will_generate_tensor_rt_cache,
            message,
        }
    }
}

pub fn registered_onnx_models() -> Vec<OnnxModel> {
    vec![
        OnnxModel {
            rust_name: "YapModelTraining",
            legacy_registered_name: "YapModelTraining",
            model_relative_path: "Assets/Model/Yap/model_training.onnx",
        },
        OnnxModel {
            rust_name: "BgiFish",
            legacy_registered_name: "BgiFish",
            model_relative_path: "Assets/Model/Fish/bgi_fish.onnx",
        },
        OnnxModel {
            rust_name: "BgiTree",
            legacy_registered_name: "BgiTree",
            model_relative_path: "Assets/Model/Domain/bgi_tree.onnx",
        },
        OnnxModel {
            rust_name: "BgiWorld",
            legacy_registered_name: "BgiTree",
            model_relative_path: "Assets/Model/World/bgi_world.onnx",
        },
        OnnxModel {
            rust_name: "BgiMine",
            legacy_registered_name: "BgiMine",
            model_relative_path: "Assets/Model/Mine/bgi_mine.onnx",
        },
        OnnxModel {
            rust_name: "BgiAvatarSide",
            legacy_registered_name: "BgiAvatarSide",
            model_relative_path: "Assets/Model/Common/avatar_side_classify_sim.onnx",
        },
        OnnxModel {
            rust_name: "BgiQClassify",
            legacy_registered_name: "BgiQClassify",
            model_relative_path: "Assets/Model/Common/q_classify_sim.onnx",
        },
        OnnxModel {
            rust_name: "SileroVad",
            legacy_registered_name: "SileroVad",
            model_relative_path: "Assets/Model/Vad/silero_vad.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrDetV4",
            legacy_registered_name: "PpOcrDetV4",
            model_relative_path:
                "Assets/Model/PaddleOCR/Det/V4/PP-OCRv4_mobile_det_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrDetV5",
            legacy_registered_name: "PpOcrDetV5",
            model_relative_path:
                "Assets/Model/PaddleOCR/Det/V5/PP-OCRv5_mobile_det_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrDetV6",
            legacy_registered_name: "PpOcrDetV6",
            model_relative_path: "Assets/Model/PaddleOCR/Det/V6/PP-OCRv6_small_det_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV4",
            legacy_registered_name: "PpOcrRecV4",
            model_relative_path:
                "Assets/Model/PaddleOCR/Rec/V4/PP-OCRv4_mobile_rec_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV4En",
            legacy_registered_name: "PpOcrRecV4En",
            model_relative_path:
                "Assets/Model/PaddleOCR/Rec/V4/en_PP-OCRv4_mobile_rec_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV5",
            legacy_registered_name: "PpOcrRecV5",
            model_relative_path:
                "Assets/Model/PaddleOCR/Rec/V5/PP-OCRv5_mobile_rec_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV6",
            legacy_registered_name: "PpOcrRecV6",
            model_relative_path: "Assets/Model/PaddleOCR/Rec/V6/PP-OCRv6_small_rec_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV5Latin",
            legacy_registered_name: "PpOcrRecV5Latin",
            model_relative_path:
                "Assets/Model/PaddleOCR/Rec/V5/latin_PP-OCRv5_mobile_rec_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV5Eslav",
            legacy_registered_name: "PpOcrRecV5Eslav",
            model_relative_path:
                "Assets/Model/PaddleOCR/Rec/V5/eslav_PP-OCRv5_mobile_rec_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV5Korean",
            legacy_registered_name: "PpOcrRecV5Korean",
            model_relative_path:
                "Assets/Model/PaddleOCR/Rec/V5/korean_PP-OCRv5_mobile_rec_infer/slim.onnx",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocr_result_text_orders_regions_top_to_bottom_then_left_to_right() {
        let result = OcrResult {
            regions: vec![
                OcrResultRegion {
                    rect: Rect::new(20, 10, 4, 4).unwrap(),
                    text: "B".to_string(),
                    score: 0.9,
                },
                OcrResultRegion {
                    rect: Rect::new(2, 10, 4, 4).unwrap(),
                    text: "A".to_string(),
                    score: 0.9,
                },
                OcrResultRegion {
                    rect: Rect::new(2, 30, 4, 4).unwrap(),
                    text: "C".to_string(),
                    score: 0.9,
                },
            ],
        };

        assert_eq!(result.text(), "A\nB\nC");
    }

    #[test]
    fn evaluates_legacy_ocr_match_rules() {
        let mut config = OcrMatchConfig::default();
        config
            .replace_dictionary
            .insert("resin".to_string(), vec!["resln".to_string()]);
        config.all_contain_match_text = vec!["originalresin".to_string()];
        config.one_contain_match_text = vec!["160".to_string(), "200".to_string()];
        config.regex_match_text = vec![r"\d{3}".to_string()];

        assert!(config.matches_text("original resln 160").unwrap());
        assert!(!config.matches_text("original resln 20").unwrap());
    }

    #[test]
    fn keeps_template_threshold_semantics() {
        assert!(TemplateMatchMode::CCoeffNormed
            .accepts_legacy_score(0.81, 0.8)
            .unwrap());
        assert!(TemplateMatchMode::SqDiffNormed
            .accepts_legacy_score(0.19, 0.8)
            .unwrap());
        assert!(!TemplateMatchMode::SqDiffNormed
            .accepts_legacy_score(0.21, 0.8)
            .unwrap());
    }

    #[test]
    fn converts_bgr_to_rgb_gray_and_opencv_hsv_planes() {
        let pixels = bgr_pixels(&[[255, 0, 0], [0, 255, 0], [0, 0, 255], [10, 20, 30]]);
        let size = Size::new(4, 1);

        let rgb = convert_bgr_image(&pixels, size, ColorConversion::BgrToRgb).unwrap();
        let gray = convert_bgr_image(&pixels, size, ColorConversion::BgrToGray).unwrap();
        let hsv = convert_bgr_image(&pixels, size, ColorConversion::BgrToHsv).unwrap();

        assert_eq!(rgb.channels, 3);
        assert_eq!(
            rgb.pixels,
            vec![0, 0, 255, 0, 255, 0, 255, 0, 0, 30, 20, 10]
        );
        assert_eq!(gray.channels, 1);
        assert_eq!(gray.pixels, vec![29, 150, 76, 22]);
        assert_eq!(hsv.channel_values(0, 0), &[120, 255, 255]);
        assert_eq!(hsv.channel_values(1, 0), &[60, 255, 255]);
        assert_eq!(hsv.channel_values(2, 0), &[0, 255, 255]);
    }

    #[test]
    fn in_range_mask_counts_roi_pixels_and_bounding_rect() {
        let image = ColorPlaneImage::new(
            Size::new(4, 2),
            3,
            vec![
                0, 0, 0, 255, 0, 0, 255, 0, 0, 0, 0, 0, //
                0, 0, 0, 255, 0, 0, 255, 0, 0, 0, 0, 0,
            ],
        )
        .unwrap();

        let mask = in_range_mask(
            &image,
            Scalar4 {
                v0: 250.0,
                v1: 0.0,
                v2: 0.0,
                v3: 0.0,
            },
            Scalar4 {
                v0: 255.0,
                v1: 5.0,
                v2: 5.0,
                v3: 0.0,
            },
            Some(Rect::new(1, 0, 2, 2).unwrap()),
        )
        .unwrap();

        assert_eq!(mask.matched_count, 4);
        assert_eq!(
            mask.bounding_rect(Some(Rect::new(1, 0, 2, 2).unwrap()))
                .unwrap(),
            Some(Rect::new(1, 0, 2, 2).unwrap())
        );
        assert_eq!(mask.pixels, vec![0, 255, 255, 0, 0, 255, 255, 0]);
    }

    #[test]
    fn pure_rust_color_match_returns_roi_bounding_rect_and_count() {
        let image = BgrImage::new(
            Size::new(4, 3),
            bgr_pixels(&[
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [10, 20, 30],
                [10, 20, 30],
                [0, 0, 0],
                [0, 0, 0],
                [10, 20, 30],
                [10, 20, 30],
                [0, 0, 0],
            ]),
        )
        .unwrap();
        let mut object = RecognitionObject {
            recognition_type: RecognitionType::ColorMatch,
            region_of_interest: Some(Rect::new(1, 1, 2, 2).unwrap()),
            name: Some("warm-block".to_string()),
            ..RecognitionObject::default()
        };
        object.color.conversion = ColorConversion::BgrToRgb;
        object.color.lower_color = Scalar4 {
            v0: 25.0,
            v1: 15.0,
            v2: 5.0,
            v3: 0.0,
        };
        object.color.upper_color = Scalar4 {
            v0: 35.0,
            v1: 25.0,
            v2: 15.0,
            v3: 0.0,
        };
        object.color.match_count = 4;
        let backend = PureRustVisionBackend::new();

        let region = backend.find(&image.pixels, image.size, &object).unwrap();
        object.color.match_count = 5;
        let missing = backend.find(&image.pixels, image.size, &object).unwrap();

        assert_eq!(region.rect, Rect::new(1, 1, 2, 2).unwrap());
        assert_eq!(region.text, "warm-block");
        assert_eq!(region.score, Some(4.0));
        assert!(!missing.is_exist());
    }

    #[test]
    fn image_region_crops_scales_and_runs_recognition_on_pixels() {
        let image = BgrImage::new(
            Size::new(4, 3),
            bgr_pixels(&[
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [10, 10, 10],
                [20, 20, 20],
                [0, 0, 0],
                [0, 0, 0],
                [30, 30, 30],
                [40, 40, 40],
                [0, 0, 0],
            ]),
        )
        .unwrap();
        let region = ImageRegion::capture(image)
            .derive_crop(Rect::new(1, 1, 2, 2).unwrap())
            .unwrap();
        let scaled = region.derive_to_1080p().unwrap();
        let template_path = Path::new("templates").join("block.png");
        let template = BgrImage::new(
            Size::new(2, 2),
            bgr_pixels(&[[10, 10, 10], [20, 20, 20], [30, 30, 30], [40, 40, 40]]),
        )
        .unwrap();
        let backend = PureRustVisionBackend::new().with_template(&template_path, template);
        let mut template_object = RecognitionObject::template_match(&template_path);
        template_object.template.threshold = 0.99;
        let mut color_object = RecognitionObject {
            recognition_type: RecognitionType::ColorMatch,
            ..RecognitionObject::default()
        };
        color_object.color.conversion = ColorConversion::BgrToRgb;
        color_object.color.lower_color = Scalar4 {
            v0: 35.0,
            v1: 35.0,
            v2: 35.0,
            v3: 0.0,
        };
        color_object.color.upper_color = Scalar4 {
            v0: 45.0,
            v1: 45.0,
            v2: 45.0,
            v3: 0.0,
        };

        let template_match = region.find(&backend, &template_object).unwrap();
        let color_match = region.find(&backend, &color_object).unwrap();

        assert_eq!(region.model.rect, Rect::new(1, 1, 2, 2).unwrap());
        assert_eq!(
            region.image.pixels,
            bgr_pixels(&[[10, 10, 10], [20, 20, 20], [30, 30, 30], [40, 40, 40]])
        );
        assert_eq!(scaled.image, region.image);
        assert_eq!(template_match.rect, Rect::new(0, 0, 2, 2).unwrap());
        assert_eq!(color_match.rect, Rect::new(1, 1, 1, 1).unwrap());
        assert_eq!(color_match.score, Some(1.0));
    }

    #[test]
    fn pure_rust_template_match_finds_best_bgr24_match_inside_roi() {
        let image = BgrImage::new(
            Size::new(5, 4),
            bgr_pixels(&[
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [10, 10, 10],
                [20, 20, 20],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [30, 30, 30],
                [40, 40, 40],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
            ]),
        )
        .unwrap();
        let template = BgrImage::new(
            Size::new(2, 2),
            bgr_pixels(&[[10, 10, 10], [20, 20, 20], [30, 30, 30], [40, 40, 40]]),
        )
        .unwrap();
        let template_path = Path::new("templates").join("block.bgr");
        let backend = PureRustVisionBackend::new().with_template(&template_path, template);
        let mut object = RecognitionObject::template_match(&template_path);
        object.name = Some("block".to_string());
        object.region_of_interest = Some(Rect::new(1, 1, 3, 2).unwrap());
        object.template.threshold = 0.99;
        object.template.mode = TemplateMatchMode::CCoeffNormed;

        let region = backend.find(&image.pixels, image.size, &object).unwrap();

        assert_eq!(region.rect, Rect::new(1, 1, 2, 2).unwrap());
        assert_eq!(region.text, "block");
        assert!(region.score.unwrap() >= 0.99);
    }

    #[test]
    fn pure_rust_template_match_applies_binary_threshold() {
        let image = BgrImage::new(
            Size::new(3, 3),
            bgr_pixels(&[
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [220, 220, 220],
                [20, 20, 20],
                [0, 0, 0],
                [20, 20, 20],
                [220, 220, 220],
            ]),
        )
        .unwrap();
        let template = BgrImage::new(
            Size::new(2, 2),
            bgr_pixels(&[[255, 255, 255], [0, 0, 0], [0, 0, 0], [255, 255, 255]]),
        )
        .unwrap();
        let template_path = Path::new("templates").join("binary-block.bgr");
        let backend = PureRustVisionBackend::new().with_template(&template_path, template);
        let mut object = RecognitionObject::template_match(&template_path);
        object.region_of_interest = Some(Rect::new(1, 1, 2, 2).unwrap());
        object.template.threshold = 0.999;
        object.template.mode = TemplateMatchMode::CCoeffNormed;
        object.template.use_binary_match = true;
        object.template.binary_threshold = 200;

        let region = backend.find(&image.pixels, image.size, &object).unwrap();

        assert_eq!(region.rect, Rect::new(1, 1, 2, 2).unwrap());
        assert!(region.score.unwrap() >= 0.999);
    }

    #[test]
    fn pure_rust_template_match_respects_sqdiff_threshold_and_match_limit() {
        let image = BgrImage::new(
            Size::new(4, 2),
            bgr_pixels(&[
                [1, 1, 1],
                [2, 2, 2],
                [1, 1, 1],
                [2, 2, 2],
                [3, 3, 3],
                [4, 4, 4],
                [3, 3, 3],
                [4, 4, 4],
            ]),
        )
        .unwrap();
        let template = BgrImage::new(
            Size::new(2, 2),
            bgr_pixels(&[[1, 1, 1], [2, 2, 2], [3, 3, 3], [4, 4, 4]]),
        )
        .unwrap();
        let template_path = Path::new("templates").join("pair.bgr");
        let backend = PureRustVisionBackend::new().with_template(&template_path, template);
        let mut object = RecognitionObject::template_match(&template_path);
        object.template.mode = TemplateMatchMode::SqDiffNormed;
        object.template.threshold = 1.0;
        object.template.max_match_count = 1;

        let matches = backend
            .find_multi(&image.pixels, image.size, &object)
            .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].rect, Rect::new(0, 0, 2, 2).unwrap());
        assert_eq!(matches[0].score, Some(0.0));
    }

    #[test]
    fn pure_rust_template_match_reports_missing_template_and_bad_buffers() {
        let backend = PureRustVisionBackend::new();
        let object = RecognitionObject::template_match("missing.bgr");

        assert!(matches!(
            backend.find(&[0, 0, 0], Size::new(1, 1), &object),
            Err(VisionError::TemplateAssetNotRegistered(_))
        ));
        assert!(matches!(
            BgrImage::new(Size::new(2, 2), vec![0; 11]).unwrap_err(),
            VisionError::InvalidImageBuffer {
                expected: 12,
                actual: 11
            }
        ));
    }

    #[test]
    fn bgr_image_decodes_and_writes_png_as_packed_bgr() {
        let image = BgrImage::new(Size::new(2, 1), bgr_pixels(&[[1, 2, 3], [4, 5, 6]])).unwrap();
        let path = temp_path("bgr-roundtrip.png");

        image.write_png(&path).unwrap();
        let bytes = fs::read(&path).unwrap();
        let decoded_from_file = BgrImage::read(&path).unwrap();
        let decoded_from_bytes = BgrImage::decode(&bytes).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(decoded_from_file, image);
        assert_eq!(decoded_from_bytes, image);
        assert_eq!(decoded_from_file.to_rgb_bytes(), vec![3, 2, 1, 6, 5, 4]);
    }

    #[test]
    fn bgr_image_samples_bgr_and_rgb_pixels() {
        let image = BgrImage::new(Size::new(2, 1), bgr_pixels(&[[1, 2, 3], [4, 5, 6]])).unwrap();

        assert_eq!(
            image.bgr_pixel_at(1, 0),
            Some(BgrPixel { b: 4, g: 5, r: 6 })
        );
        assert_eq!(
            image.rgb_pixel_at(1, 0),
            Some(RgbPixel { r: 6, g: 5, b: 4 })
        );
        assert_eq!(image.bgr_pixel_at(2, 0), None);
        assert_eq!(image.rgb_pixel_at(0, 1), None);
    }

    #[test]
    fn pure_rust_template_match_loads_template_from_rooted_asset_path() {
        let root = temp_path("template-root");
        let template_path = Path::new("GameTask")
            .join("AutoPick")
            .join("Assets")
            .join("1920x1080")
            .join("F.png");
        let absolute_template_path = root.join(&template_path);
        fs::create_dir_all(absolute_template_path.parent().unwrap()).unwrap();
        let template = BgrImage::new(
            Size::new(2, 2),
            bgr_pixels(&[[10, 20, 30], [40, 50, 60], [70, 80, 90], [100, 110, 120]]),
        )
        .unwrap();
        template.write_png(&absolute_template_path).unwrap();
        let image = BgrImage::new(
            Size::new(3, 3),
            bgr_pixels(&[
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [10, 20, 30],
                [40, 50, 60],
                [0, 0, 0],
                [70, 80, 90],
                [100, 110, 120],
            ]),
        )
        .unwrap();
        let backend = PureRustVisionBackend::new().with_template_root(&root);
        let mut object = RecognitionObject::template_match(&template_path);
        object.template.threshold = 0.99;
        object.template.mode = TemplateMatchMode::CCoeffNormed;
        object.template.use_3_channels = true;

        let region = backend.find(&image.pixels, image.size, &object).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(region.rect, Rect::new(1, 1, 2, 2).unwrap());
        assert!(region.score.unwrap() >= 0.99);
    }

    #[test]
    fn bv_image_preserves_legacy_feature_asset_reference() {
        let image = BvImage::new("AutoPick:F.png").unwrap();
        let roi = Rect::new(10, 20, 30, 40).unwrap();
        let object = image
            .to_recognition_object_for_screen(Some(roi), 0.91, Size::new(2560, 1440))
            .unwrap();

        assert_eq!(image.feature_name, "AutoPick");
        assert_eq!(image.asset_name, "F.png");
        assert_eq!(
            object.template.template_asset,
            Some(
                Path::new("GameTask")
                    .join("AutoPick")
                    .join("Assets")
                    .join("2560x1440")
                    .join("F.png")
            )
        );
        assert_eq!(object.name.as_deref(), Some("AutoPick:F.png"));
        assert_eq!(object.region_of_interest, Some(roi));
        assert_eq!(object.template.threshold, 0.91);

        assert!(matches!(
            BvImage::new("AutoPick").unwrap_err(),
            VisionError::InvalidBvImageAsset
        ));
    }

    #[test]
    fn bv_locator_models_wait_retry_and_retry_action() {
        let object = RecognitionObject::ocr(Rect::new(0, 0, 200, 80).unwrap());
        let locator = BvLocator::new(object)
            .with_timeout(1_500)
            .unwrap()
            .with_retry_interval(200)
            .unwrap()
            .with_retry_action("retry-callback");

        let plan = locator.plan(BvLocatorOperation::ClickUntilDisappears, None);

        assert_eq!(plan.operation, BvLocatorOperation::ClickUntilDisappears);
        assert_eq!(plan.timeout_ms, 1_500);
        assert_eq!(plan.retry_interval_ms, 200);
        assert_eq!(plan.retry_count, 7);
        assert_eq!(plan.retry_action.as_deref(), Some("retry-callback"));

        assert!(matches!(
            BvLocator::new(RecognitionObject::default())
                .with_retry_interval(0)
                .unwrap_err(),
            VisionError::NonPositiveDuration {
                field: "retry_interval"
            }
        ));
    }

    #[test]
    fn bv_page_models_screenshot_ocr_and_1080p_clicks() {
        let page = BvPage {
            capture_size: Size::new(2560, 1440),
            ..BvPage::default()
        };

        assert_eq!(
            page.screenshot(),
            BvPageCommand::Screenshot {
                size: Size::new(2560, 1440)
            }
        );
        assert_eq!(
            page.wait(250).unwrap(),
            BvPageCommand::Wait { milliseconds: 250 }
        );
        assert!(matches!(
            page.wait(0).unwrap_err(),
            VisionError::NonPositiveDuration {
                field: "milliseconds"
            }
        ));

        assert_eq!(
            page.click_1080p(960.0, 540.0),
            BvPageCommand::Click1080p {
                x: 960.0,
                y: 540.0,
                capture_size: Size::new(2560, 1440),
                screen_x: 1280.0,
                screen_y: 720.0
            }
        );

        let BvPageCommand::Ocr { locator } = page.ocr(Some(Rect::new(10, 20, 100, 40).unwrap()))
        else {
            panic!("expected OCR command");
        };
        assert_eq!(locator.operation, BvLocatorOperation::FindAll);
        assert_eq!(
            locator.recognition_object.region_of_interest,
            Some(Rect::new(10, 20, 100, 40).unwrap())
        );
    }

    #[test]
    fn image_region_model_derives_crops_and_1080p_scale() {
        let capture = ImageRegionModel::capture(Size::new(2560, 1440));
        let crop = capture
            .derive_crop(Rect::new(2400, 1300, 400, 400).unwrap())
            .unwrap();

        assert_eq!(crop.source, ImageRegionSource::DerivedCrop);
        assert_eq!(
            crop.rect,
            Rect {
                x: 2400,
                y: 1300,
                width: 160,
                height: 140
            }
        );
        assert_eq!(
            crop.size,
            Size {
                width: 160,
                height: 140
            }
        );
        assert!(crop.owner.is_some());

        let scaled = capture.derive_to_1080p();
        assert_eq!(scaled.source, ImageRegionSource::DerivedScale);
        assert_eq!(scaled.size, Size::new(1920, 1080));
        assert!(scaled.owner.is_some());
    }

    #[test]
    fn preserves_legacy_onnx_registry_names() {
        let models = registered_onnx_models();
        let world = models
            .iter()
            .find(|model| model.rust_name == "BgiWorld")
            .unwrap();
        assert_eq!(world.legacy_registered_name, "BgiTree");
        assert_eq!(
            world.cache_relative_path("9.9.9"),
            Path::new("Cache")
                .join("9.9.9")
                .join("Model")
                .join("BgiTree")
        );
    }

    #[test]
    fn onnx_model_load_plan_reports_missing_source_model() {
        let root = temp_path("onnx-missing");
        let model = registered_onnx_models()
            .into_iter()
            .find(|model| model.rust_name == "BgiAvatarSide")
            .unwrap();

        let plan = model.load_plan(&root, "9.9.9", OnnxProviderSelection::CPU);

        assert_eq!(plan.source, OnnxModelLoadSource::Missing);
        assert!(!plan.exists);
        assert!(plan.model_path.ends_with(
            Path::new("Assets")
                .join("Model")
                .join("Common")
                .join("avatar_side_classify_sim.onnx")
        ));
        assert!(plan.message.contains("model file is missing"));
    }

    #[test]
    fn onnx_model_load_plan_uses_tensor_rt_cache_when_available() {
        let root = temp_path("onnx-cache");
        let model = registered_onnx_models()
            .into_iter()
            .find(|model| model.rust_name == "BgiAvatarSide")
            .unwrap();
        let named_cache = root
            .join(model.cache_relative_path("9.9.9"))
            .join("trt")
            .join("avatar_side_classify_sim_ctx.onnx");
        fs::create_dir_all(named_cache.parent().unwrap()).unwrap();
        fs::write(&named_cache, b"fake").unwrap();

        let plan = model.load_plan(
            &root,
            "9.9.9",
            OnnxProviderSelection {
                enable_tensor_rt_cache: true,
                providers: &[ProviderType::TensorRt, ProviderType::Cpu],
            },
        );
        let _ = fs::remove_dir_all(&root);

        assert_eq!(plan.source, OnnxModelLoadSource::TensorRtNamedCache);
        assert_eq!(plan.model_path, named_cache);
        assert!(plan.exists);
        assert!(!plan.will_generate_tensor_rt_cache);
    }

    #[test]
    fn onnx_model_load_plan_prefers_anonymous_tensor_rt_cache() {
        let root = temp_path("onnx-cache-anonymous");
        let model = registered_onnx_models()
            .into_iter()
            .find(|model| model.rust_name == "BgiAvatarSide")
            .unwrap();
        let cache_dir = root.join(model.cache_relative_path("9.9.9")).join("trt");
        let anonymous_cache = cache_dir.join("_ctx.onnx");
        let named_cache = cache_dir.join("avatar_side_classify_sim_ctx.onnx");
        fs::create_dir_all(&cache_dir).unwrap();
        fs::write(&anonymous_cache, b"fake").unwrap();
        fs::write(&named_cache, b"fake").unwrap();

        let plan = model.load_plan(
            &root,
            "9.9.9",
            OnnxProviderSelection {
                enable_tensor_rt_cache: true,
                providers: &[ProviderType::TensorRt, ProviderType::Cpu],
            },
        );
        let _ = fs::remove_dir_all(&root);

        assert_eq!(plan.source, OnnxModelLoadSource::TensorRtAnonymousCache);
        assert_eq!(plan.model_path, anonymous_cache);
    }

    #[test]
    fn onnx_model_load_plan_falls_back_to_source_without_tensor_rt_provider() {
        let root = temp_path("onnx-source");
        let model = registered_onnx_models()
            .into_iter()
            .find(|model| model.rust_name == "BgiAvatarSide")
            .unwrap();
        let source = root.join(model.model_relative_path);
        let cache = root
            .join(model.cache_relative_path("9.9.9"))
            .join("trt")
            .join("_ctx.onnx");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, b"fake").unwrap();
        fs::create_dir_all(cache.parent().unwrap()).unwrap();
        fs::write(&cache, b"fake").unwrap();

        let plan = model.load_plan(
            &root,
            "9.9.9",
            OnnxProviderSelection {
                enable_tensor_rt_cache: true,
                providers: &[ProviderType::Dml, ProviderType::Cpu],
            },
        );
        let _ = fs::remove_dir_all(&root);

        assert_eq!(plan.source, OnnxModelLoadSource::SourceModel);
        assert_eq!(plan.model_path, source);
        assert!(plan.exists);
        assert!(!plan.will_generate_tensor_rt_cache);
    }

    fn bgr_pixels(values: &[[u8; 3]]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|pixel| pixel.iter().copied())
            .collect()
    }

    fn temp_path(name: &str) -> PathBuf {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("bgi-vision-{id}-{name}"))
    }
}
