use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use super::{Rect, Result, Size, VisionError};

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
