use super::*;
use bgi_input::PostMessageSequence;
use serde_json::Value;

pub(super) fn post_message_result(sequence: PostMessageSequence) -> ScriptHostCallResult {
    ScriptHostCallResult::PostMessageEvents(sequence.events().to_vec())
}

pub(super) fn unknown_method(call: &ScriptHostCall) -> ScriptHostRuntimeError {
    ScriptHostRuntimeError::UnknownHostMethod {
        target: call.target.as_str(),
        method: call.method.clone(),
    }
}

pub(super) fn arg_value<'a>(
    call: &'a ScriptHostCall,
    index: usize,
    expected: &'static str,
) -> Result<&'a Value> {
    call.args
        .get(index)
        .filter(|value| !value.is_null())
        .ok_or_else(|| invalid_arg(call, index, expected))
}

pub(super) fn arg_str<'a>(call: &'a ScriptHostCall, index: usize) -> Result<&'a str> {
    arg_value(call, index, "string")?
        .as_str()
        .ok_or_else(|| invalid_arg(call, index, "string"))
}

pub(super) fn optional_str<'a>(call: &'a ScriptHostCall, index: usize) -> Result<Option<&'a str>> {
    match call.args.get(index) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_str()
            .map(Some)
            .ok_or_else(|| invalid_arg(call, index, "string")),
    }
}

pub(super) fn arg_i32(call: &ScriptHostCall, index: usize) -> Result<i32> {
    let value = arg_value(call, index, "i32")?;
    let number = value
        .as_i64()
        .ok_or_else(|| invalid_arg(call, index, "i32"))?;
    i32::try_from(number).map_err(|_| invalid_arg(call, index, "i32"))
}

pub(super) fn arg_u32(call: &ScriptHostCall, index: usize) -> Result<u32> {
    let value = arg_value(call, index, "u32")?;
    let number = value
        .as_u64()
        .ok_or_else(|| invalid_arg(call, index, "u32"))?;
    u32::try_from(number).map_err(|_| invalid_arg(call, index, "u32"))
}

pub(super) fn arg_u64(call: &ScriptHostCall, index: usize) -> Result<u64> {
    arg_value(call, index, "u64")?
        .as_u64()
        .ok_or_else(|| invalid_arg(call, index, "u64"))
}

pub(super) fn arg_f64_like(call: &ScriptHostCall, index: usize) -> Result<f64> {
    f64_like(arg_value(call, index, "number or numeric string")?)
        .ok_or_else(|| invalid_arg(call, index, "number or numeric string"))
}

pub(super) fn arg_u32_like(call: &ScriptHostCall, index: usize) -> Result<u32> {
    u32_like(arg_value(call, index, "u32 or numeric string")?)
        .ok_or_else(|| invalid_arg(call, index, "u32 or numeric string"))
}

pub(super) fn optional_u32_like(call: &ScriptHostCall, index: usize) -> Result<Option<u32>> {
    match call.args.get(index) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => u32_like(value)
            .map(Some)
            .ok_or_else(|| invalid_arg(call, index, "u32 or numeric string")),
    }
}

pub(super) fn optional_u64(call: &ScriptHostCall, index: usize) -> Result<Option<u64>> {
    match call.args.get(index) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => u64_like(value)
            .map(Some)
            .ok_or_else(|| invalid_arg(call, index, "u64 or numeric string")),
    }
}

pub(super) fn optional_i32(call: &ScriptHostCall, index: usize) -> Result<Option<i32>> {
    match call.args.get(index) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => i32_like(value)
            .map(Some)
            .ok_or_else(|| invalid_arg(call, index, "i32 or numeric string")),
    }
}

pub(super) fn optional_f64(call: &ScriptHostCall, index: usize) -> Result<Option<f64>> {
    match call.args.get(index) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_f64()
            .map(Some)
            .ok_or_else(|| invalid_arg(call, index, "f64")),
    }
}

pub(super) fn optional_bool(call: &ScriptHostCall, index: usize) -> Result<Option<bool>> {
    match call.args.get(index) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_bool()
            .map(Some)
            .ok_or_else(|| invalid_arg(call, index, "bool")),
    }
}

pub(super) fn arg_owned_value(call: &ScriptHostCall, index: usize) -> Result<Value> {
    Ok(arg_value(call, index, "object")?.clone())
}

pub(super) fn optional_owned_value(call: &ScriptHostCall, index: usize) -> Option<Value> {
    match call.args.get(index) {
        None | Some(Value::Null) => None,
        Some(value) => Some(value.clone()),
    }
}

pub(super) fn f64_like(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str()?.trim().parse::<f64>().ok())
}

pub(super) fn bool_like(value: &Value) -> Option<bool> {
    if let Some(value) = value.as_bool() {
        return Some(value);
    }
    match value.as_str()?.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" => Some(true),
        "false" | "0" | "no" | "n" => Some(false),
        _ => None,
    }
}

pub(super) fn u64_like(value: &Value) -> Option<u64> {
    if let Some(value) = value.as_u64() {
        return Some(value);
    }
    let value = value.as_str()?.trim();
    if value.is_empty() {
        return None;
    }
    value.parse::<u64>().ok()
}

pub(super) fn u32_like(value: &Value) -> Option<u32> {
    u64_like(value).and_then(|value| u32::try_from(value).ok())
}

pub(super) fn i32_like(value: &Value) -> Option<i32> {
    if let Some(value) = value.as_i64() {
        return i32::try_from(value).ok();
    }
    let value = value.as_str()?.trim();
    if value.is_empty() {
        return None;
    }
    value
        .parse::<i64>()
        .ok()
        .and_then(|value| i32::try_from(value).ok())
}

pub(super) fn legacy_jagged_array_type(element_type: &str, dimensions: u32) -> String {
    let mut type_name = element_type.trim().to_string();
    for _ in 0..dimensions {
        type_name.push_str("[]");
    }
    type_name
}

pub(super) fn timer_plan_from_arg(
    call: &ScriptHostCall,
    index: usize,
    clears_existing_triggers: bool,
) -> Result<RealtimeTimerHostPlan> {
    let value = arg_value(call, index, "timer object")?;
    let Value::Object(map) = value else {
        return Err(invalid_arg(call, index, "timer object"));
    };
    let name = map
        .get("Name")
        .or_else(|| map.get("name"))
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| invalid_arg(call, index, "timer.name"))?;
    let interval_ms = map
        .get("Interval")
        .or_else(|| map.get("interval"))
        .and_then(Value::as_u64)
        .unwrap_or(50);
    let config_value = map.get("Config").or_else(|| map.get("config"));
    let config = if name.eq_ignore_ascii_case("AutoPick") {
        Some(AutoPickExternalConfig::from_value(config_value)?.to_legacy_config_value())
    } else {
        config_value.cloned()
    };
    Ok(RealtimeTimerHostPlan {
        name: name.to_string(),
        interval_ms,
        config,
        clears_existing_triggers,
    })
}

pub(super) fn solo_task_plan_from_arg(
    call: &ScriptHostCall,
    index: usize,
) -> Result<SoloTaskHostPlan> {
    let value = arg_value(call, index, "solo task object")?;
    let Value::Object(map) = value else {
        return Err(invalid_arg(call, index, "solo task object"));
    };
    let name = map
        .get("Name")
        .or_else(|| map.get("name"))
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| invalid_arg(call, index, "soloTask.name"))?;
    let config = map.get("Config").or_else(|| map.get("config")).cloned();
    Ok(SoloTaskHostPlan::new(name, config))
}

pub(super) fn invalid_arg(
    call: &ScriptHostCall,
    index: usize,
    expected: &'static str,
) -> ScriptHostRuntimeError {
    invalid_arg_for_method(
        &format!("{}.{}", call.target.as_str(), call.method),
        index,
        expected,
    )
}

pub(super) fn invalid_arg_for_method(
    method: &str,
    index: usize,
    expected: &'static str,
) -> ScriptHostRuntimeError {
    ScriptHostRuntimeError::InvalidArgument {
        method: method.to_string(),
        index,
        expected,
    }
}
