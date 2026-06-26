use crate::{ColorConversion, Rect, Result, Scalar4, Size, VisionError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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

    pub(super) fn pixel(&self, x: u32, y: u32) -> [f64; 3] {
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

pub(super) fn validate_bgr_len(size: Size, actual: usize) -> Result<()> {
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

pub(super) fn search_region(region: Option<Rect>, image_size: Size) -> Result<Rect> {
    region
        .unwrap_or(Rect {
            x: 0,
            y: 0,
            width: image_size.width as i32,
            height: image_size.height as i32,
        })
        .clamp_to(image_size)
}

pub(super) fn bgr_pixel(pixels: &[u8], size: Size, x: u32, y: u32) -> [f64; 3] {
    let index = ((y * size.width + x) as usize) * 3;
    [
        pixels[index] as f64,
        pixels[index + 1] as f64,
        pixels[index + 2] as f64,
    ]
}

pub(super) fn gray_from_bgr(pixel: [f64; 3]) -> f64 {
    0.114 * pixel[0] + 0.587 * pixel[1] + 0.299 * pixel[2]
}

pub(super) fn binary_gray_from_bgr(pixel: [f64; 3], threshold: u8) -> f64 {
    if (gray_from_bgr(pixel).round() as u8) >= threshold {
        255.0
    } else {
        0.0
    }
}

fn scalar_contains(values: &[u8], lower: Scalar4, upper: Scalar4) -> bool {
    let lower = [lower.v0, lower.v1, lower.v2, lower.v3];
    let upper = [upper.v0, upper.v1, upper.v2, upper.v3];
    values.iter().enumerate().all(|(index, value)| {
        let value = *value as f64;
        value >= lower[index] && value <= upper[index]
    })
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
