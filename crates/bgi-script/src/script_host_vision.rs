use super::{invalid_arg_for_method, Result};
use bgi_vision::{
    BgrImage, ColorMatchConfig, ImageRegion, ImageRegionModel, PureRustVisionBackend,
    RecognitionObject, RecognitionType, Region,
};
use serde::Serialize;
use serde_json::Value;

#[path = "script_host_vision_values.rs"]
mod vision_values;

pub(crate) use vision_values::image_from_mat_value;
use vision_values::{
    image_from_mat_value_for, object_options, optional_bool_field, optional_color_conversion_field,
    optional_f64_field, optional_i32_field, optional_rect_field, optional_string_field,
    optional_template_match_mode_field, optional_u32_field, rect_from_value, required_scalar_field,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VisionRecognitionExecution {
    pub recognition_type: RecognitionType,
    pub image_region: ImageRegionModel,
    pub first: Region,
    pub matches: Vec<Region>,
    pub matched_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VisionImageMatExecution {
    pub image_region: ImageRegionModel,
    pub width: u32,
    pub height: u32,
    pub pixel_format: &'static str,
    pub pixels: Vec<u8>,
    pub color_mode: &'static str,
}

impl VisionImageMatExecution {
    fn from_image_region(region: ImageRegion) -> Self {
        Self {
            width: region.image.size.width,
            height: region.image.size.height,
            pixel_format: "BGR24",
            pixels: region.image.pixels,
            color_mode: "color",
            image_region: region.model,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct VisionHost;

impl VisionHost {
    const INLINE_TEMPLATE_ASSET: &'static str = "__script_inline_template__";

    pub fn crop(&self, image: Value, rect: Value) -> Result<VisionImageMatExecution> {
        const METHOD: &str = "vision.crop";

        let image = image_from_mat_value_for(&image, METHOD, 0)?;
        let rect = rect_from_value(&rect, METHOD, 1)?;
        let region = ImageRegion::from_mat_handle("script-mat", image, 0, 0).derive_crop(rect)?;
        Ok(VisionImageMatExecution::from_image_region(region))
    }

    pub fn to_1080p(&self, image: Value) -> Result<VisionImageMatExecution> {
        const METHOD: &str = "vision.to1080p";

        let image = image_from_mat_value_for(&image, METHOD, 0)?;
        let region = ImageRegion::from_mat_handle("script-mat", image, 0, 0).derive_to_1080p()?;
        Ok(VisionImageMatExecution::from_image_region(region))
    }

    pub fn find_template(
        &self,
        image: Value,
        template: Value,
        options: Option<Value>,
    ) -> Result<VisionRecognitionExecution> {
        const METHOD: &str = "vision.findTemplate";

        let image = image_from_mat_value_for(&image, METHOD, 0)?;
        let template = image_from_mat_value_for(&template, METHOD, 1)?;
        let options = object_options(options.as_ref(), METHOD, 2)?;

        let mut object = RecognitionObject::template_match(Self::INLINE_TEMPLATE_ASSET);
        object.name = optional_string_field(options, &["name", "Name"]);
        object.region_of_interest = optional_rect_field(
            options,
            &[
                "roi",
                "ROI",
                "regionOfInterest",
                "region_of_interest",
                "RegionOfInterest",
            ],
            METHOD,
            2,
        )?;
        object.template.template_size = Some(template.size);
        if let Some(threshold) =
            optional_f64_field(options, &["threshold", "Threshold"], METHOD, 2)?
        {
            object.template.threshold = threshold;
        }
        if let Some(use_3_channels) = optional_bool_field(
            options,
            &["use3Channels", "use_3_channels", "Use3Channels"],
            METHOD,
            2,
        )? {
            object.template.use_3_channels = use_3_channels;
        }
        if let Some(mode) =
            optional_template_match_mode_field(options, &["mode", "Mode"], METHOD, 2)?
        {
            object.template.mode = mode;
        }
        if let Some(max_match_count) = optional_i32_field(
            options,
            &["maxMatchCount", "max_match_count", "MaxMatchCount"],
            METHOD,
            2,
        )? {
            object.template.max_match_count = max_match_count;
        }
        object.validate()?;

        let mut backend = PureRustVisionBackend::new();
        backend.register_template(Self::INLINE_TEMPLATE_ASSET, template);
        self.execute(image, object, &backend)
    }

    pub fn find_color(
        &self,
        image: Value,
        options: Option<Value>,
    ) -> Result<VisionRecognitionExecution> {
        const METHOD: &str = "vision.findColor";

        let image = image_from_mat_value_for(&image, METHOD, 0)?;
        let options = object_options(options.as_ref(), METHOD, 1)?
            .ok_or_else(|| invalid_arg_for_method(METHOD, 1, "color match options object"))?;

        let mut object = RecognitionObject {
            recognition_type: RecognitionType::ColorMatch,
            ..RecognitionObject::default()
        };
        object.name = optional_string_field(Some(options), &["name", "Name"]);
        object.region_of_interest = optional_rect_field(
            Some(options),
            &[
                "roi",
                "ROI",
                "regionOfInterest",
                "region_of_interest",
                "RegionOfInterest",
            ],
            METHOD,
            1,
        )?;
        object.color = ColorMatchConfig {
            conversion: optional_color_conversion_field(
                Some(options),
                &[
                    "conversion",
                    "Conversion",
                    "colorConversion",
                    "ColorConversion",
                ],
                METHOD,
                1,
            )?
            .unwrap_or_default(),
            lower_color: required_scalar_field(
                options,
                &["lowerColor", "lower_color", "LowerColor", "lower"],
                METHOD,
                1,
            )?,
            upper_color: required_scalar_field(
                options,
                &["upperColor", "upper_color", "UpperColor", "upper"],
                METHOD,
                1,
            )?,
            match_count: optional_u32_field(
                Some(options),
                &["matchCount", "match_count", "MatchCount"],
                METHOD,
                1,
            )?
            .unwrap_or(1),
        };
        object.validate()?;

        let backend = PureRustVisionBackend::new();
        self.execute(image, object, &backend)
    }

    fn execute(
        &self,
        image: BgrImage,
        object: RecognitionObject,
        backend: &PureRustVisionBackend,
    ) -> Result<VisionRecognitionExecution> {
        let recognition_type = object.recognition_type;
        let image_region = ImageRegion::from_mat_handle("script-mat", image, 0, 0);
        let matches = image_region.find_multi(backend, &object)?;
        let first = matches.first().cloned().unwrap_or_else(Region::empty);
        Ok(VisionRecognitionExecution {
            recognition_type,
            image_region: image_region.model,
            matched_count: matches.len(),
            first,
            matches,
        })
    }
}
