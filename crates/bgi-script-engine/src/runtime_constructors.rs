use super::*;

pub(super) fn realtime_timer_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let name = constructor_string_arg(args, 0, context)?;
    let config = constructor_config_arg(args, 1, context)?;
    let object = ObjectInitializer::new(context)
        .property(js_string!("name"), name.clone(), Attribute::all())
        .property(js_string!("Name"), name, Attribute::all())
        .property(js_string!("interval"), 50, Attribute::all())
        .property(js_string!("Interval"), 50, Attribute::all())
        .property(js_string!("config"), config.clone(), Attribute::all())
        .property(js_string!("Config"), config, Attribute::all())
        .build();
    Ok(object.into())
}

pub(super) fn solo_task_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let name = constructor_string_arg(args, 0, context)?;
    let config = constructor_config_arg(args, 1, context)?;
    let object = ObjectInitializer::new(context)
        .property(js_string!("name"), name.clone(), Attribute::all())
        .property(js_string!("Name"), name, Attribute::all())
        .property(js_string!("config"), config.clone(), Attribute::all())
        .property(js_string!("Config"), config, Attribute::all())
        .build();
    Ok(object.into())
}

pub(super) fn auto_skip_config_constructor(
    _: &JsValue,
    _: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    task_param_object(AutoSkipConfigParam::default(), context)
}

pub(super) fn auto_domain_param_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let round_num = constructor_i32_arg(args, 0, context)?.unwrap_or(0);
    let strategy_name = constructor_optional_string_arg(args, 1, context)?;
    task_param_object(
        AutoDomainParam::new(round_num, strategy_name.as_deref()),
        context,
    )
}

pub(super) fn auto_boss_param_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let strategy_name = constructor_optional_string_arg(args, 0, context)?;
    task_param_object(AutoBossParam::new(strategy_name.as_deref()), context)
}

pub(super) fn auto_fight_param_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let strategy_name = constructor_optional_string_arg(args, 0, context)?;
    task_param_object(AutoFightParam::new(strategy_name.as_deref()), context)
}

pub(super) fn auto_ley_line_outcrop_param_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    if args.is_empty()
        || args
            .first()
            .is_some_and(|value| value.is_undefined() || value.is_null())
    {
        return task_param_object(AutoLeyLineOutcropParam::default(), context);
    }
    let count = constructor_i32_arg(args, 0, context)?.unwrap_or(0);
    let country = constructor_optional_string_arg(args, 1, context)?.unwrap_or_default();
    let outcrop_type = constructor_optional_string_arg(args, 2, context)?.unwrap_or_default();
    task_param_object(
        AutoLeyLineOutcropParam::new(count, country, outcrop_type),
        context,
    )
}

pub(super) fn auto_stygian_onslaught_param_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let strategy_name = constructor_optional_string_arg(args, 0, context)?;
    task_param_object(
        AutoStygianOnslaughtParam::new(strategy_name.as_deref()),
        context,
    )
}

pub(super) fn rect_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let x = constructor_i32_arg(args, 0, context)?.unwrap_or(0);
    let y = constructor_i32_arg(args, 1, context)?.unwrap_or(0);
    let width = constructor_i32_arg(args, 2, context)?.unwrap_or(0);
    let height = constructor_i32_arg(args, 3, context)?.unwrap_or(0);
    let rect = VisionRect::new(x, y, width, height)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
    task_param_object(rect, context)
}

pub(super) fn recognition_object_constructor(
    _: &JsValue,
    _: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    task_param_object(VisionRecognitionObject::default(), context)
}

pub(super) fn bv_image_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let template_assert = constructor_optional_string_arg(args, 0, context)?
        .ok_or_else(|| JsNativeError::typ().with_message("BvImage template asset is required"))?;
    let roi = constructor_rect_arg(args, 1, context)?;
    let threshold = constructor_f64_arg(args, 2, context)?.unwrap_or(0.8);
    let image = VisionBvImage::new(template_assert)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
    let recognition_object = image
        .to_recognition_object(roi, threshold)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
    let mut value = to_json_value(&image);
    if let Value::Object(map) = &mut value {
        map.insert(
            "recognition_object".to_string(),
            to_json_value(&recognition_object),
        );
    }
    add_pascal_case_aliases(&mut value);
    JsValue::from_json(&value, context)
}

pub(super) fn bv_locator_constructor(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let object = constructor_recognition_object_arg(args, 0, context)?;
    task_param_object(VisionBvLocator::new(object), context)
}

pub(super) fn bv_page_constructor(
    _: &JsValue,
    _: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    task_param_object(VisionBvPage::default(), context)
}

pub(super) fn task_param_object<T: Serialize>(
    value: T,
    context: &mut Context,
) -> JsResult<JsValue> {
    let mut value = to_json_value(&value);
    add_pascal_case_aliases(&mut value);
    JsValue::from_json(&value, context)
}

pub(super) fn add_pascal_case_aliases(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for value in map.values_mut() {
                add_pascal_case_aliases(value);
            }
            let aliases = map
                .iter()
                .filter_map(|(key, value)| {
                    snake_to_pascal_case_alias(key).map(|alias| (alias, value.clone()))
                })
                .collect::<Vec<_>>();
            for (alias, value) in aliases {
                map.entry(alias).or_insert(value);
            }
        }
        Value::Array(items) => {
            for value in items {
                add_pascal_case_aliases(value);
            }
        }
        _ => {}
    }
}

pub(super) fn snake_to_pascal_case_alias(name: &str) -> Option<String> {
    if name.is_empty() || name.chars().any(|ch| ch.is_ascii_uppercase()) {
        return None;
    }
    let mut alias = String::with_capacity(name.len());
    for part in name.split('_') {
        if part.is_empty() {
            continue;
        }
        let mut chars = part.chars();
        let first = chars.next()?;
        alias.push(first.to_ascii_uppercase());
        alias.push_str(chars.as_str());
    }
    (!alias.is_empty() && alias != name).then_some(alias)
}

pub(super) fn constructor_string_arg(
    args: &[JsValue],
    index: usize,
    context: &mut Context,
) -> JsResult<JsValue> {
    let Some(value) = args.get(index) else {
        return Ok(JsValue::Null);
    };
    if value.is_undefined() || value.is_null() {
        return Ok(JsValue::Null);
    }
    Ok(JsValue::from(value.to_string(context)?))
}

pub(super) fn constructor_config_arg(
    args: &[JsValue],
    index: usize,
    context: &mut Context,
) -> JsResult<JsValue> {
    let Some(value) = args.get(index) else {
        return Ok(JsValue::Null);
    };
    if value.is_undefined() {
        return Ok(JsValue::Null);
    }
    JsValue::from_json(&value.to_json(context)?, context)
}

pub(super) fn constructor_optional_string_arg(
    args: &[JsValue],
    index: usize,
    context: &mut Context,
) -> JsResult<Option<String>> {
    let Some(value) = args.get(index) else {
        return Ok(None);
    };
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    Ok(Some(value.to_string(context)?.to_std_string_escaped()))
}

pub(super) fn constructor_i32_arg(
    args: &[JsValue],
    index: usize,
    _context: &mut Context,
) -> JsResult<Option<i32>> {
    let Some(value) = args.get(index) else {
        return Ok(None);
    };
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    let Some(number) = value.as_number() else {
        return Err(JsNativeError::typ()
            .with_message(format!("constructor argument {index} must be number"))
            .into());
    };
    if !number.is_finite() || number < i32::MIN as f64 || number > i32::MAX as f64 {
        return Err(JsNativeError::typ()
            .with_message(format!("constructor argument {index} is outside i32 range"))
            .into());
    }
    let truncated = number.trunc();
    if (number - truncated).abs() > f64::EPSILON {
        return Err(JsNativeError::typ()
            .with_message(format!("constructor argument {index} must be an integer"))
            .into());
    }
    Ok(Some(truncated as i32))
}

pub(super) fn constructor_f64_arg(
    args: &[JsValue],
    index: usize,
    _context: &mut Context,
) -> JsResult<Option<f64>> {
    let Some(value) = args.get(index) else {
        return Ok(None);
    };
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    let Some(number) = value.as_number() else {
        return Err(JsNativeError::typ()
            .with_message(format!("constructor argument {index} must be number"))
            .into());
    };
    if !number.is_finite() {
        return Err(JsNativeError::typ()
            .with_message(format!("constructor argument {index} must be finite"))
            .into());
    }
    Ok(Some(number))
}

pub(super) fn constructor_rect_arg(
    args: &[JsValue],
    index: usize,
    context: &mut Context,
) -> JsResult<Option<VisionRect>> {
    let Some(value) = args.get(index) else {
        return Ok(None);
    };
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    let json = value.to_json(context)?;
    rect_from_json(&json, index).map(Some)
}

pub(super) fn constructor_recognition_object_arg(
    args: &[JsValue],
    index: usize,
    context: &mut Context,
) -> JsResult<VisionRecognitionObject> {
    let Some(value) = args.get(index) else {
        return Ok(VisionRecognitionObject::default());
    };
    if value.is_undefined() || value.is_null() {
        return Ok(VisionRecognitionObject::default());
    }
    let json = value.to_json(context)?;
    let json = json
        .as_object()
        .and_then(|map| {
            map.get("recognition_object")
                .or_else(|| map.get("RecognitionObject"))
                .cloned()
        })
        .unwrap_or(json);
    serde_json::from_value(json).map_err(|err| {
        JsNativeError::typ()
            .with_message(format!(
                "constructor argument {index} must be RecognitionObject: {err}"
            ))
            .into()
    })
}

pub(super) fn rect_from_json(value: &Value, index: usize) -> JsResult<VisionRect> {
    let Value::Object(map) = value else {
        return Err(JsNativeError::typ()
            .with_message(format!("constructor argument {index} must be Rect object"))
            .into());
    };
    let x = json_i32_field(map, &["x", "X"]).unwrap_or(0);
    let y = json_i32_field(map, &["y", "Y"]).unwrap_or(0);
    let width = json_i32_field(map, &["width", "Width"]).unwrap_or(0);
    let height = json_i32_field(map, &["height", "Height"]).unwrap_or(0);
    VisionRect::new(x, y, width, height)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()).into())
}

pub(super) fn json_i32_field(map: &serde_json::Map<String, Value>, names: &[&str]) -> Option<i32> {
    names
        .iter()
        .filter_map(|name| map.get(*name))
        .find_map(|value| value.as_i64().and_then(|number| i32::try_from(number).ok()))
}
