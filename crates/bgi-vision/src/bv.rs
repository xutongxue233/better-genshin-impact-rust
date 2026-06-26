use super::{RecognitionObject, Rect, Result, Size, VisionError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
