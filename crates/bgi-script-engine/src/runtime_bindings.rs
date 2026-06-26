use super::*;

#[path = "runtime_constructors.rs"]
mod constructors;
#[path = "runtime_bindings_descriptors.rs"]
mod descriptors;
#[path = "runtime_bindings_helpers.rs"]
mod helpers;
#[path = "runtime_host_call.rs"]
mod host_call;
#[path = "runtime_bindings_promise.rs"]
mod promise;

use constructors::*;
use descriptors::{
    MethodBinding, FILE_METHODS, GLOBAL_HOST_FUNCTIONS, KEY_MOUSE_HOOK_METHODS, LOG_METHODS,
    SIMPLE_HOST_OBJECTS_AFTER_KEY_MOUSE_HOOK, SIMPLE_HOST_OBJECTS_BEFORE_KEY_MOUSE_HOOK,
};
use helpers::pascal_case_alias;
pub(crate) use host_call::*;
pub(super) use promise::{
    settle_js_promise, settle_promise_value, settle_promise_value_for_callback,
};

pub(super) fn register_console(context: &mut Context, state: EngineState) -> Result<()> {
    let console = ObjectInitializer::new(context)
        .function(
            native_closure({
                let state = state.clone();
                move |_, args, context| {
                    let message = join_js_args(args, context)?;
                    state.console.borrow_mut().push(message);
                    Ok(JsValue::Undefined)
                }
            }),
            js_string!("log"),
            1,
        )
        .build();
    context
        .register_global_property(js_string!("console"), console, Attribute::all())
        .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))
}

pub(super) fn register_log_host(context: &mut Context, state: EngineState) -> Result<()> {
    let mut initializer = ObjectInitializer::new(context);
    for method in LOG_METHODS {
        initializer.function(
            host_function(state.clone(), ScriptHostTarget::Log, method.name),
            JsString::from(method.name),
            method.length,
        );
        if let Some(alias) = pascal_case_alias(method.name) {
            initializer.function(
                host_function(state.clone(), ScriptHostTarget::Log, method.name),
                JsString::from(alias.as_str()),
                method.length,
            );
        }
    }
    let log = initializer.build();
    context
        .register_global_property(js_string!("log"), log, Attribute::all())
        .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))
}

pub(super) fn register_global_host_functions(
    context: &mut Context,
    state: EngineState,
) -> Result<()> {
    for function in GLOBAL_HOST_FUNCTIONS {
        context
            .register_global_builtin_callable(
                JsString::from(function.name),
                function.length,
                host_function(state.clone(), ScriptHostTarget::Global, function.name),
            )
            .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))?;
        if let Some(alias) = pascal_case_alias(function.name) {
            context
                .register_global_builtin_callable(
                    JsString::from(alias.as_str()),
                    function.length,
                    host_function(state.clone(), ScriptHostTarget::Global, function.name),
                )
                .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))?;
        }
    }
    Ok(())
}

pub(super) fn register_host_objects(context: &mut Context, state: EngineState) -> Result<()> {
    register_file_object(context, state.clone())?;
    register_script_type_constructors(context)?;

    for binding in SIMPLE_HOST_OBJECTS_BEFORE_KEY_MOUSE_HOOK {
        register_simple_object(
            context,
            state.clone(),
            JsString::from(binding.global_name),
            binding.target,
            binding.methods,
        )?;
    }
    register_key_mouse_hook_object(context, state.clone())?;

    for binding in SIMPLE_HOST_OBJECTS_AFTER_KEY_MOUSE_HOOK {
        register_simple_object(
            context,
            state.clone(),
            JsString::from(binding.global_name),
            binding.target,
            binding.methods,
        )?;
    }
    Ok(())
}

pub(super) fn register_file_object(context: &mut Context, state: EngineState) -> Result<()> {
    let mut initializer = ObjectInitializer::new(context);
    for method in FILE_METHODS {
        initializer.function(
            host_function(state.clone(), ScriptHostTarget::File, method.host_method),
            JsString::from(method.property_name),
            method.length,
        );
    }
    let file = initializer.build();
    context
        .register_global_property(js_string!("file"), file, Attribute::all())
        .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))
}

pub(super) fn register_script_type_constructors(context: &mut Context) -> Result<()> {
    for (name, length, function) in [
        (
            js_string!("RealtimeTimer"),
            2,
            NativeFunction::from_fn_ptr(realtime_timer_constructor),
        ),
        (
            js_string!("SoloTask"),
            2,
            NativeFunction::from_fn_ptr(solo_task_constructor),
        ),
        (
            js_string!("AutoSkipConfig"),
            0,
            NativeFunction::from_fn_ptr(auto_skip_config_constructor),
        ),
        (
            js_string!("AutoDomainParam"),
            2,
            NativeFunction::from_fn_ptr(auto_domain_param_constructor),
        ),
        (
            js_string!("AutoBossParam"),
            1,
            NativeFunction::from_fn_ptr(auto_boss_param_constructor),
        ),
        (
            js_string!("AutoFightParam"),
            1,
            NativeFunction::from_fn_ptr(auto_fight_param_constructor),
        ),
        (
            js_string!("AutoLeyLineOutcropParam"),
            3,
            NativeFunction::from_fn_ptr(auto_ley_line_outcrop_param_constructor),
        ),
        (
            js_string!("AutoStygianOnslaughtParam"),
            1,
            NativeFunction::from_fn_ptr(auto_stygian_onslaught_param_constructor),
        ),
        (
            js_string!("Rect"),
            4,
            NativeFunction::from_fn_ptr(rect_constructor),
        ),
        (
            js_string!("RecognitionObject"),
            0,
            NativeFunction::from_fn_ptr(recognition_object_constructor),
        ),
        (
            js_string!("BvImage"),
            3,
            NativeFunction::from_fn_ptr(bv_image_constructor),
        ),
        (
            js_string!("BvLocator"),
            1,
            NativeFunction::from_fn_ptr(bv_locator_constructor),
        ),
        (
            js_string!("BvPage"),
            0,
            NativeFunction::from_fn_ptr(bv_page_constructor),
        ),
    ] {
        let constructor = FunctionObjectBuilder::new(context.realm(), function)
            .name(name.clone())
            .length(length)
            .constructor(true)
            .build();
        context
            .register_global_property(name, constructor, Attribute::all())
            .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))?;
    }
    Ok(())
}

fn register_simple_object(
    context: &mut Context,
    state: EngineState,
    global_name: JsString,
    target: ScriptHostTarget,
    methods: &[MethodBinding],
) -> Result<()> {
    let mut initializer = ObjectInitializer::new(context);
    for method in methods {
        initializer.function(
            host_function(state.clone(), target, method.name),
            JsString::from(method.name),
            method.length,
        );
        if let Some(alias) = pascal_case_alias(method.name) {
            initializer.function(
                host_function(state.clone(), target, method.name),
                JsString::from(alias.as_str()),
                method.length,
            );
        }
    }
    let object = initializer.build();
    context
        .register_global_property(global_name, object, Attribute::all())
        .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))
}

pub(super) fn register_key_mouse_hook_object(
    context: &mut Context,
    state: EngineState,
) -> Result<()> {
    ensure_callback_registry(context)
        .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))?;
    let mut initializer = ObjectInitializer::new(context);
    for method in KEY_MOUSE_HOOK_METHODS {
        initializer.function(
            key_mouse_hook_function(state.clone(), method.name),
            JsString::from(method.name),
            method.length,
        );
        if let Some(alias) = pascal_case_alias(method.name) {
            initializer.function(
                key_mouse_hook_function(state.clone(), method.name),
                JsString::from(alias.as_str()),
                method.length,
            );
        }
    }
    let object = initializer.build();
    context
        .register_global_property(js_string!("KeyMouseHook"), object, Attribute::all())
        .map_err(|err| ScriptEngineError::JavaScript(js_error_to_string(err, context)))
}

pub(super) fn host_function(
    state: EngineState,
    target: ScriptHostTarget,
    method: &'static str,
) -> NativeFunction {
    native_closure(move |_, args, context| call_host(&state, target, method, args, context))
}

pub(super) fn key_mouse_hook_function(state: EngineState, method: &'static str) -> NativeFunction {
    native_closure(move |_, args, context| call_key_mouse_hook_host(&state, method, args, context))
}

pub(super) fn native_closure<F>(closure: F) -> NativeFunction
where
    F: Fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue> + 'static,
{
    // The closures capture only Rust-side state behind Rc/RefCell and never store JsValue/JsObject.
    unsafe { NativeFunction::from_closure(closure) }
}
