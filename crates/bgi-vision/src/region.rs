use serde::{Deserialize, Serialize};

use super::{
    crop_bgr_image, resize_bgr_nearest, BgrImage, OcrMatchConfig, RecognitionObject,
    RecognitionType, Rect, Result, Size, VisionBackend, VisionError,
};

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
