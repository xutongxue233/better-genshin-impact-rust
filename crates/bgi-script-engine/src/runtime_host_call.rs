use super::super::{EngineState, ExecutedHostCall, Result, ScriptEngineError};
use super::settle_promise_value_for_callback;
use bgi_script::{ScriptHostCall, ScriptHostCallResult, ScriptHostTarget};
use boa_engine::{
    js_string,
    object::{JsObject, ObjectInitializer},
    Context, JsError, JsNativeError, JsResult, JsString, JsValue,
};
use serde::Serialize;
use serde_json::{json, Value};

pub(crate) fn call_host(
    state: &EngineState,
    target: ScriptHostTarget,
    method: &str,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let args = args_to_json(args, context)?;
    let call = ScriptHostCall::new(target, method, args.clone());
    let result = state
        .host
        .borrow_mut()
        .call(call)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
    let json_result = host_result_to_json(&result);
    state.host_calls.borrow_mut().push(ExecutedHostCall {
        target,
        method: method.to_string(),
        args,
        result: json_result.clone(),
    });
    JsValue::from_json(&json_result, context)
}

pub(crate) fn call_key_mouse_hook_host(
    state: &EngineState,
    method: &str,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let args = key_mouse_hook_args_to_json(state, method, args, context)?;
    let call = ScriptHostCall::new(ScriptHostTarget::KeyMouseHook, method, args.clone());
    let result = state
        .host
        .borrow_mut()
        .call(call)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
    let json_result = host_result_to_json(&result);
    state.host_calls.borrow_mut().push(ExecutedHostCall {
        target: ScriptHostTarget::KeyMouseHook,
        method: method.to_string(),
        args,
        result: json_result.clone(),
    });

    if method.eq_ignore_ascii_case("dispatchEvent") {
        if let ScriptHostCallResult::KeyMouseHookDispatches(dispatches) = &result {
            dispatch_key_mouse_hook_callbacks(dispatches, context)?;
        }
    }

    JsValue::from_json(&json_result, context)
}

pub(crate) fn key_mouse_hook_args_to_json(
    state: &EngineState,
    method: &str,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<Vec<Value>> {
    if is_key_mouse_hook_registration_method(method) {
        let mut converted = Vec::with_capacity(args.len());
        if let Some(first) = args.first() {
            if let Some(callback) = first.as_callable() {
                let callback_id = register_callback(state, callback.clone(), context)?;
                converted.push(Value::String(callback_id));
            } else if first.is_undefined() {
                converted.push(Value::Null);
            } else {
                converted.push(first.to_json(context)?);
            }
        }
        for arg in args.iter().skip(1) {
            converted.push(if arg.is_undefined() {
                Value::Null
            } else {
                arg.to_json(context)?
            });
        }
        return Ok(converted);
    }

    args_to_json(args, context)
}

pub(crate) fn is_key_mouse_hook_registration_method(method: &str) -> bool {
    matches!(
        method,
        "onKeyDown"
            | "OnKeyDown"
            | "onKeyUp"
            | "OnKeyUp"
            | "onMouseDown"
            | "OnMouseDown"
            | "onMouseUp"
            | "OnMouseUp"
            | "onMouseMove"
            | "OnMouseMove"
            | "onMouseWheel"
            | "OnMouseWheel"
    )
}

pub(crate) fn register_callback(
    state: &EngineState,
    callback: JsObject,
    context: &mut Context,
) -> JsResult<String> {
    ensure_callback_registry(context)?;
    let id = {
        let mut next = state.next_callback_id.borrow_mut();
        let id = format!("callback-{}", *next);
        *next = next.saturating_add(1);
        id
    };
    let registry = callback_registry(context)?;
    registry.set(JsString::from(id.as_str()), callback, true, context)?;
    Ok(id)
}

pub(crate) fn dispatch_key_mouse_hook_callbacks(
    dispatches: &[bgi_script::KeyMouseHookDispatch],
    context: &mut Context,
) -> JsResult<()> {
    let registry = callback_registry(context)?;
    for dispatch in dispatches {
        let value = registry.get(JsString::from(dispatch.listener_id.as_str()), context)?;
        let Some(callback) = value.as_callable() else {
            continue;
        };
        let args = dispatch
            .args
            .iter()
            .map(|value| JsValue::from_json(value, context))
            .collect::<JsResult<Vec<_>>>()?;
        let result = callback.call(&JsValue::Undefined, &args, context)?;
        settle_promise_value_for_callback(result, context)?;
    }
    Ok(())
}

pub(crate) fn ensure_callback_registry(context: &mut Context) -> JsResult<()> {
    let global = context.global_object().clone();
    if global
        .get(js_string!("__bettergiCallbacks"), context)?
        .as_object()
        .is_some()
    {
        return Ok(());
    }
    let registry = ObjectInitializer::new(context).build();
    global.set(js_string!("__bettergiCallbacks"), registry, true, context)?;
    Ok(())
}

pub(crate) fn callback_registry(context: &mut Context) -> JsResult<JsObject> {
    ensure_callback_registry(context)?;
    context
        .global_object()
        .clone()
        .get(js_string!("__bettergiCallbacks"), context)?
        .as_object()
        .cloned()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("callback registry is not an object")
                .into()
        })
}

pub(crate) fn join_js_args(args: &[JsValue], context: &mut Context) -> JsResult<String> {
    let mut parts = Vec::with_capacity(args.len());
    for arg in args {
        parts.push(arg.to_string(context)?.to_std_string_escaped());
    }
    Ok(parts.join(" "))
}

pub(crate) fn args_to_json(args: &[JsValue], context: &mut Context) -> JsResult<Vec<Value>> {
    args.iter()
        .map(|value| {
            if value.is_undefined() {
                Ok(Value::Null)
            } else {
                value.to_json(context)
            }
        })
        .collect()
}

pub(crate) fn host_result_to_json(result: &ScriptHostCallResult) -> Value {
    match result {
        ScriptHostCallResult::None => Value::Null,
        ScriptHostCallResult::Bool(value) => Value::Bool(*value),
        ScriptHostCallResult::Integer(value) => json!(value),
        ScriptHostCallResult::String(value) => json!(value),
        ScriptHostCallResult::StringList(value) => json!(value),
        ScriptHostCallResult::GameMetrics(value) => to_json_value(value),
        ScriptHostCallResult::CaptureGameRegionPlan(value) => to_json_value(value),
        ScriptHostCallResult::CaptureGameRegionExecution(value) => to_json_value(value),
        ScriptHostCallResult::AvatarRecognitionPlan(value) => to_json_value(value),
        ScriptHostCallResult::ImageMatReadPlan(value) => to_json_value(value),
        ScriptHostCallResult::ImageMatWritePlan(value) => to_json_value(value),
        ScriptHostCallResult::ImageMatReadExecution(value) => to_json_value(value),
        ScriptHostCallResult::ImageMatWriteExecution(value) => to_json_value(value),
        ScriptHostCallResult::VisionRecognitionExecution(value) => to_json_value(value),
        ScriptHostCallResult::VisionImageMatExecution(value) => to_json_value(value),
        ScriptHostCallResult::CustomHostFunctionCommand(value) => to_json_value(value),
        ScriptHostCallResult::InputEvents(value) => to_json_value(value),
        ScriptHostCallResult::InputExecution(value) => to_json_value(value),
        ScriptHostCallResult::PostMessageEvents(value) => to_json_value(value),
        ScriptHostCallResult::HttpRequestPlan(value) => to_json_value(value),
        ScriptHostCallResult::HttpExecution(value) => to_json_value(value),
        ScriptHostCallResult::DispatcherCommand(value) => to_json_value(value),
        ScriptHostCallResult::DispatcherCommands(value) => to_json_value(value),
        ScriptHostCallResult::GenshinCommand(value) => to_json_value(value),
        ScriptHostCallResult::GenshinCommands(value) => to_json_value(value),
        ScriptHostCallResult::PathingPlan(value) => to_json_value(value),
        ScriptHostCallResult::PathingExecution(value) => to_json_value(value),
        ScriptHostCallResult::KeyMousePlan(value) => to_json_value(value),
        ScriptHostCallResult::KeyMouseExecution(value) => to_json_value(value),
        ScriptHostCallResult::HtmlMaskCommand(value) => to_json_value(value),
        ScriptHostCallResult::HtmlMaskSnapshot(value) => to_json_value(value),
        ScriptHostCallResult::KeyMouseHookCommand(value) => to_json_value(value),
        ScriptHostCallResult::KeyMouseHookDispatches(value) => to_json_value(value),
        ScriptHostCallResult::KeyMouseHookSnapshot(value) => to_json_value(value),
        ScriptHostCallResult::LogRecords(value) => to_json_value(value),
        ScriptHostCallResult::NotificationExecution(value) => to_json_value(value),
        ScriptHostCallResult::NotificationRecords(value) => to_json_value(value),
    }
}

pub(crate) fn to_json_value<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

pub(crate) fn value_to_json_option(
    value: &JsValue,
    context: &mut Context,
) -> Result<Option<Value>> {
    if value.is_undefined() {
        return Ok(None);
    }
    value
        .to_json(context)
        .map(Some)
        .map_err(|err| ScriptEngineError::ValueConversion(js_error_to_string(err, context)))
}

pub(crate) fn result_to_display(value: &JsValue, context: &mut Context) -> Result<String> {
    if value.is_undefined() {
        return Ok("undefined".to_string());
    }
    value
        .to_string(context)
        .map(|value| value.to_std_string_escaped())
        .map_err(|err| ScriptEngineError::ValueConversion(js_error_to_string(err, context)))
}

pub(crate) fn js_value_to_string(value: &JsValue, context: &mut Context) -> String {
    value
        .to_string(context)
        .map(|message| message.to_std_string_escaped())
        .unwrap_or_else(|_| value.display().to_string())
}

pub(crate) fn js_error_to_string(error: JsError, context: &mut Context) -> String {
    error
        .to_opaque(context)
        .to_string(context)
        .map(|message| message.to_std_string_escaped())
        .unwrap_or_else(|_| error.to_string())
}
