use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{BgrImage, Result, Size, VisionError};

pub const LEGACY_GRID_ICON_INPUT_SIZE: Size = Size {
    width: 125,
    height: 125,
};
pub const LEGACY_GRID_ICON_FEATURE_DIMENSIONS: usize = 64;
pub const LEGACY_GRID_ICON_MAX_DISTANCE_SQUARED: f64 = 100.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridIconPrototype {
    pub name: String,
    pub feature: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridIconPrototypeMatch {
    pub name: String,
    pub distance_squared: f64,
}

pub fn parse_grid_icon_prototype_csv(
    csv: &str,
    feature_dimensions: usize,
) -> Result<Vec<GridIconPrototype>> {
    let mut prototypes = Vec::new();
    let mut names = HashSet::new();
    for (index, line) in csv.lines().enumerate().skip(1) {
        let line_number = index + 1;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut columns = line.split(',');
        let name = columns.next().unwrap_or_default().trim();
        let encoded = columns.next().unwrap_or_default().trim();
        if name.is_empty() || encoded.is_empty() {
            return Err(invalid_grid_icon_csv_line(
                line_number,
                "expected item name and base64 feature columns",
            ));
        }
        if !names.insert(name.to_string()) {
            return Err(invalid_grid_icon_csv_line(
                line_number,
                format!("duplicate item prototype name: {name}"),
            ));
        }
        let bytes = STANDARD.decode(encoded).map_err(|error| {
            invalid_grid_icon_csv_line(line_number, format!("invalid base64 feature: {error}"))
        })?;
        if bytes.len() % std::mem::size_of::<f32>() != 0 {
            return Err(invalid_grid_icon_csv_line(
                line_number,
                "feature byte length must be divisible by 4",
            ));
        }
        let feature = bytes
            .chunks_exact(std::mem::size_of::<f32>())
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect::<Vec<_>>();
        validate_grid_icon_feature_length(feature_dimensions, feature.len())?;
        prototypes.push(GridIconPrototype {
            name: name.to_string(),
            feature,
        });
    }
    Ok(prototypes)
}

pub fn grid_icon_bgr_to_rgb_chw_input(icon: &BgrImage) -> Result<Vec<f32>> {
    if icon.size != LEGACY_GRID_ICON_INPUT_SIZE {
        return Err(VisionError::ImageSizeMismatch {
            expected: LEGACY_GRID_ICON_INPUT_SIZE,
            actual: icon.size,
        });
    }

    let plane_size = icon.size.width as usize * icon.size.height as usize;
    let mut input = vec![0.0; plane_size * 3];
    for index in 0..plane_size {
        let bgr_index = index * 3;
        input[index] = icon.pixels[bgr_index + 2] as f32 / 255.0;
        input[plane_size + index] = icon.pixels[bgr_index + 1] as f32 / 255.0;
        input[plane_size * 2 + index] = icon.pixels[bgr_index] as f32 / 255.0;
    }
    Ok(input)
}

pub fn nearest_grid_icon_prototype(
    feature: &[f32],
    prototypes: &[GridIconPrototype],
    feature_dimensions: usize,
) -> Result<GridIconPrototypeMatch> {
    validate_grid_icon_feature_length(feature_dimensions, feature.len())?;
    if prototypes.is_empty() {
        return Err(VisionError::EmptyGridIconPrototypes);
    }

    let mut nearest: Option<GridIconPrototypeMatch> = None;
    for prototype in prototypes {
        validate_grid_icon_feature_length(feature_dimensions, prototype.feature.len())?;
        let distance_squared = prototype
            .feature
            .iter()
            .zip(feature.iter())
            .map(|(prototype_value, feature_value)| {
                let delta = f64::from(*prototype_value) - f64::from(*feature_value);
                delta * delta
            })
            .sum::<f64>();
        let replaces_current = match nearest.as_ref() {
            Some(current) => distance_squared < current.distance_squared,
            None => true,
        };
        if replaces_current {
            nearest = Some(GridIconPrototypeMatch {
                name: prototype.name.clone(),
                distance_squared,
            });
        }
    }

    nearest.ok_or(VisionError::EmptyGridIconPrototypes)
}

pub fn classify_grid_icon_feature(
    feature: &[f32],
    prototypes: &[GridIconPrototype],
    feature_dimensions: usize,
    max_distance_squared: f64,
) -> Result<Option<GridIconPrototypeMatch>> {
    let nearest = nearest_grid_icon_prototype(feature, prototypes, feature_dimensions)?;
    Ok((nearest.distance_squared < max_distance_squared).then_some(nearest))
}

pub fn grid_icon_star_index_from_logits(logits: &[f32]) -> Result<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (index, logit) in logits.iter().copied().enumerate() {
        let replaces_current = match best {
            Some((_, current)) => logit.total_cmp(&current).is_gt(),
            None => true,
        };
        if replaces_current {
            best = Some((index, logit));
        }
    }
    best.map(|(index, _)| index)
        .ok_or(VisionError::InvalidGridIconFeatureLength {
            expected: 1,
            actual: 0,
        })
}

fn validate_grid_icon_feature_length(expected: usize, actual: usize) -> Result<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(VisionError::InvalidGridIconFeatureLength { expected, actual })
    }
}

fn invalid_grid_icon_csv_line(line: usize, message: impl Into<String>) -> VisionError {
    VisionError::InvalidGridIconPrototypeCsv {
        line,
        message: message.into(),
    }
}
