use bgi_script::ScriptHostTarget;

#[path = "runtime_bindings_descriptors_file.rs"]
mod file;
#[path = "runtime_bindings_descriptors_global.rs"]
mod global;
#[path = "runtime_bindings_descriptors_key_mouse_hook.rs"]
mod key_mouse_hook;
#[path = "runtime_bindings_descriptors_simple_host_objects.rs"]
mod simple_host_objects;

pub(super) use self::file::FILE_METHODS;
pub(super) use self::global::{GLOBAL_HOST_FUNCTIONS, LOG_METHODS};
pub(super) use self::key_mouse_hook::KEY_MOUSE_HOOK_METHODS;
pub(super) use self::simple_host_objects::{
    SIMPLE_HOST_OBJECTS_AFTER_KEY_MOUSE_HOOK, SIMPLE_HOST_OBJECTS_BEFORE_KEY_MOUSE_HOOK,
};

#[derive(Clone, Copy)]
pub(super) struct MethodBinding {
    pub(super) name: &'static str,
    pub(super) length: usize,
}

#[derive(Clone, Copy)]
pub(super) struct FileMethodBinding {
    pub(super) property_name: &'static str,
    pub(super) host_method: &'static str,
    pub(super) length: usize,
}

#[derive(Clone, Copy)]
pub(super) struct HostObjectBinding {
    pub(super) global_name: &'static str,
    pub(super) target: ScriptHostTarget,
    pub(super) methods: &'static [MethodBinding],
}
