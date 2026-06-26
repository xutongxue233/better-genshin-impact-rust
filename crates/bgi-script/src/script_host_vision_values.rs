use super::super::{bool_like, f64_like, i32_like, invalid_arg_for_method, u32_like, Result};
use bgi_vision::{BgrImage, ColorConversion, Rect, Scalar4, Size as VisionSize, TemplateMatchMode};
use serde_json::Value;

pub(crate) fn image_from_mat_value(source: &Value) -> Result<BgrImage> {
    image_from_mat_value_for(source, "file.WriteImageSync", 1)
}

pub(super) fn image_from_mat_value_for(
    source: &Value,
    method: &str,
    index: usize,
) -> Result<BgrImage> {
    let width = value_u32_field(source, &["width", "Width"], method, index)?;
    let height = value_u32_field(source, &["height", "Height"], method, index)?;
    let pixel_format =
        value_str_field(source, &["pixel_format", "pixelFormat", "PixelFormat"]).unwrap_or("BGR24");
    if !pixel_format.eq_ignore_ascii_case("BGR24") {
        return Err(invalid_arg_for_method(method, index, "BGR24 image payload"));
    }
    let pixels = value_u8_vec_field(source, &["pixels", "Pixels"], method, index)?;
    BgrImage::new(VisionSize::new(width, height), pixels).map_err(Into::into)
}

pub(super) fn object_options<'a>(
    options: Option<&'a Value>,
    method: &str,
    index: usize,
) -> Result<Option<&'a serde_json::Map<String, Value>>> {
    match options {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Object(map)) => Ok(Some(map)),
        Some(_) => Err(invalid_arg_for_method(method, index, "options object")),
    }
}

fn value_field<'a>(map: &'a serde_json::Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|key| map.get(*key))
}

pub(super) fn optional_string_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
) -> Option<String> {
    value_field(options?, keys)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

pub(super) fn optional_bool_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<bool>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    bool_like(value)
        .map(Some)
        .ok_or_else(|| invalid_arg_for_method(method, index, "bool field"))
}

pub(super) fn optional_u32_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<u32>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    u32_like(value)
        .map(Some)
        .ok_or_else(|| invalid_arg_for_method(method, index, "u32 field"))
}

pub(super) fn optional_i32_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<i32>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    i32_like(value)
        .map(Some)
        .ok_or_else(|| invalid_arg_for_method(method, index, "i32 field"))
}

pub(super) fn optional_f64_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<f64>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    f64_like(value)
        .map(Some)
        .ok_or_else(|| invalid_arg_for_method(method, index, "number field"))
}

pub(super) fn optional_rect_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<Rect>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    rect_from_value(value, method, index).map(Some)
}

pub(super) fn rect_from_value(value: &Value, method: &str, index: usize) -> Result<Rect> {
    match value {
        Value::Array(values) if values.len() == 4 => Rect::new(
            value_i32_component(&values[0], method, index)?,
            value_i32_component(&values[1], method, index)?,
            value_i32_component(&values[2], method, index)?,
            value_i32_component(&values[3], method, index)?,
        )
        .map_err(Into::into),
        Value::Object(map) => Rect::new(
            required_i32_component(map, &["x", "X"], method, index)?,
            required_i32_component(map, &["y", "Y"], method, index)?,
            required_i32_component(map, &["width", "Width"], method, index)?,
            required_i32_component(map, &["height", "Height"], method, index)?,
        )
        .map_err(Into::into),
        _ => Err(invalid_arg_for_method(
            method,
            index,
            "rect object or [x,y,width,height]",
        )),
    }
}

pub(super) fn required_scalar_field(
    map: &serde_json::Map<String, Value>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Scalar4> {
    let value = value_field(map, keys)
        .ok_or_else(|| invalid_arg_for_method(method, index, "scalar color field"))?;
    scalar_from_value(value, method, index)
}

fn scalar_from_value(value: &Value, method: &str, index: usize) -> Result<Scalar4> {
    match value {
        Value::Array(values) if values.len() == 3 || values.len() == 4 => Ok(Scalar4 {
            v0: value_f64_component(&values[0], method, index)?,
            v1: value_f64_component(&values[1], method, index)?,
            v2: value_f64_component(&values[2], method, index)?,
            v3: values
                .get(3)
                .map(|value| value_f64_component(value, method, index))
                .transpose()?
                .unwrap_or(0.0),
        }),
        Value::Object(map) => Ok(Scalar4 {
            v0: required_f64_component(map, &["v0", "V0"], method, index)?,
            v1: required_f64_component(map, &["v1", "V1"], method, index)?,
            v2: required_f64_component(map, &["v2", "V2"], method, index)?,
            v3: optional_f64_component(map, &["v3", "V3"], method, index)?.unwrap_or(0.0),
        }),
        _ => Err(invalid_arg_for_method(
            method,
            index,
            "scalar color array or object",
        )),
    }
}

pub(super) fn optional_template_match_mode_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<TemplateMatchMode>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    template_match_mode_from_value(value, method, index).map(Some)
}

fn template_match_mode_from_value(
    value: &Value,
    method: &str,
    index: usize,
) -> Result<TemplateMatchMode> {
    if let Some(number) = i32_like(value) {
        return match number {
            0 => Ok(TemplateMatchMode::SqDiff),
            1 => Ok(TemplateMatchMode::SqDiffNormed),
            2 => Ok(TemplateMatchMode::CCorr),
            3 => Ok(TemplateMatchMode::CCorrNormed),
            4 => Ok(TemplateMatchMode::CCoeff),
            5 => Ok(TemplateMatchMode::CCoeffNormed),
            _ => Err(invalid_arg_for_method(
                method,
                index,
                "template match mode 0..=5",
            )),
        };
    }
    let Some(value) = value.as_str() else {
        return Err(invalid_arg_for_method(method, index, "template match mode"));
    };
    match normalize_enum_token(value).as_str() {
        "sqdiff" | "templatemodessqdiff" => Ok(TemplateMatchMode::SqDiff),
        "sqdiffnormed" | "templatemodessqdiffnormed" => Ok(TemplateMatchMode::SqDiffNormed),
        "ccorr" | "templatemodesccorr" => Ok(TemplateMatchMode::CCorr),
        "ccorrnormed" | "templatemodesccorrnormed" => Ok(TemplateMatchMode::CCorrNormed),
        "ccoeff" | "templatemodesccoeff" => Ok(TemplateMatchMode::CCoeff),
        "ccoeffnormed" | "templatemodesccoeffnormed" => Ok(TemplateMatchMode::CCoeffNormed),
        _ => Err(invalid_arg_for_method(method, index, "template match mode")),
    }
}

pub(super) fn optional_color_conversion_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<ColorConversion>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    color_conversion_from_value(value, method, index).map(Some)
}

fn color_conversion_from_value(
    value: &Value,
    method: &str,
    index: usize,
) -> Result<ColorConversion> {
    if let Some(number) = i32_like(value) {
        return match number {
            1 => Ok(ColorConversion::BgraToBgr),
            4 => Ok(ColorConversion::BgrToRgb),
            6 => Ok(ColorConversion::BgrToGray),
            40 => Ok(ColorConversion::BgrToHsv),
            _ => Err(invalid_arg_for_method(
                method,
                index,
                "OpenCV color conversion code 1, 4, 6, or 40",
            )),
        };
    }
    let Some(value) = value.as_str() else {
        return Err(invalid_arg_for_method(method, index, "color conversion"));
    };
    match normalize_enum_token(value).as_str() {
        "bgrtorgb" | "bgr2rgb" | "rgb" => Ok(ColorConversion::BgrToRgb),
        "bgrtohsv" | "bgr2hsv" | "hsv" => Ok(ColorConversion::BgrToHsv),
        "bgrtogray" | "bgr2gray" | "gray" | "grey" => Ok(ColorConversion::BgrToGray),
        "bgratobgr" | "bgra2bgr" => Ok(ColorConversion::BgraToBgr),
        _ => Err(invalid_arg_for_method(method, index, "color conversion")),
    }
}

fn normalize_enum_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn required_i32_component(
    map: &serde_json::Map<String, Value>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<i32> {
    let value = value_field(map, keys)
        .ok_or_else(|| invalid_arg_for_method(method, index, "i32 component"))?;
    value_i32_component(value, method, index)
}

fn required_f64_component(
    map: &serde_json::Map<String, Value>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<f64> {
    let value = value_field(map, keys)
        .ok_or_else(|| invalid_arg_for_method(method, index, "number component"))?;
    value_f64_component(value, method, index)
}

fn optional_f64_component(
    map: &serde_json::Map<String, Value>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<f64>> {
    value_field(map, keys)
        .map(|value| value_f64_component(value, method, index))
        .transpose()
}

fn value_i32_component(value: &Value, method: &str, index: usize) -> Result<i32> {
    i32_like(value).ok_or_else(|| invalid_arg_for_method(method, index, "i32 component"))
}

fn value_f64_component(value: &Value, method: &str, index: usize) -> Result<f64> {
    f64_like(value).ok_or_else(|| invalid_arg_for_method(method, index, "number component"))
}

fn value_u32_field(source: &Value, keys: &[&str], method: &str, index: usize) -> Result<u32> {
    let value = keys
        .iter()
        .find_map(|key| source.get(*key))
        .ok_or_else(|| invalid_arg_for_method(method, index, "image width/height fields"))?;
    u32_like(value).ok_or_else(|| invalid_arg_for_method(method, index, "u32 field"))
}

fn value_str_field<'a>(source: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| source.get(*key))
        .and_then(Value::as_str)
}

fn value_u8_vec_field(
    source: &Value,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Vec<u8>> {
    let value = keys
        .iter()
        .find_map(|key| source.get(*key))
        .ok_or_else(|| invalid_arg_for_method(method, index, "image pixels field"))?;
    let pixels = value
        .as_array()
        .ok_or_else(|| invalid_arg_for_method(method, index, "u8 pixel array"))?;
    pixels
        .iter()
        .map(|value| {
            let byte = value
                .as_u64()
                .ok_or_else(|| invalid_arg_for_method(method, index, "u8 pixel array"))?;
            u8::try_from(byte).map_err(|_| invalid_arg_for_method(method, index, "u8 pixel array"))
        })
        .collect()
}
