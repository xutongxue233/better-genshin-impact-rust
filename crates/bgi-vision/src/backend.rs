use crate::{
    RecognitionObject, RecognitionType, Rect, Region, Result, Size, TemplateMatchConfig,
    TemplateMatchMode, VisionError,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[path = "backend_image.rs"]
mod backend_image;

use backend_image::{
    bgr_pixel, binary_gray_from_bgr, gray_from_bgr, search_region, validate_bgr_len,
};
pub use backend_image::{
    convert_bgr_image, crop_bgr_image, in_range_mask, resize_bgr_nearest, BgrImage, BgrPixel,
    ColorPlaneImage, ColorRangeMask, RgbPixel,
};

pub trait VisionBackend {
    fn find(&self, image: &[u8], image_size: Size, object: &RecognitionObject) -> Result<Region>;
    fn find_multi(
        &self,
        image: &[u8],
        image_size: Size,
        object: &RecognitionObject,
    ) -> Result<Vec<Region>>;
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
