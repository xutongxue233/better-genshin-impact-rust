#[path = "runtime_bindings_descriptors_simple_host_objects_dispatcher.rs"]
mod dispatcher;
#[path = "runtime_bindings_descriptors_simple_host_objects_genshin.rs"]
mod genshin;
#[path = "runtime_bindings_descriptors_simple_host_objects_html_mask.rs"]
mod html_mask;
#[path = "runtime_bindings_descriptors_simple_host_objects_input.rs"]
mod input;
#[path = "runtime_bindings_descriptors_simple_host_objects_misc.rs"]
mod misc;
#[path = "runtime_bindings_descriptors_simple_host_objects_notification.rs"]
mod notification;
#[path = "runtime_bindings_descriptors_simple_host_objects_pathing.rs"]
mod pathing;
#[path = "runtime_bindings_descriptors_simple_host_objects_vision.rs"]
mod vision;

use self::dispatcher::DISPATCHER_HOST_OBJECT;
use self::genshin::GENSHIN_HOST_OBJECT;
use self::html_mask::HTML_MASK_HOST_OBJECT;
use self::input::{KEY_MOUSE_SCRIPT_HOST_OBJECT, POST_MESSAGE_HOST_OBJECT};
use self::misc::{CUSTOM_HOST_FUNCTIONS_HOST_OBJECT, HTTP_HOST_OBJECT, SERVER_TIME_HOST_OBJECT};
use self::notification::NOTIFICATION_HOST_OBJECT;
use self::pathing::{PATHING_SCRIPT_HOST_OBJECT, STRATEGY_FILE_HOST_OBJECT};
use self::vision::VISION_HOST_OBJECT;
use super::HostObjectBinding;

pub(in crate::runtime_bindings) const SIMPLE_HOST_OBJECTS_BEFORE_KEY_MOUSE_HOOK:
    &[HostObjectBinding] = &[
    VISION_HOST_OBJECT,
    KEY_MOUSE_SCRIPT_HOST_OBJECT,
    PATHING_SCRIPT_HOST_OBJECT,
    HTTP_HOST_OBJECT,
    NOTIFICATION_HOST_OBJECT,
    DISPATCHER_HOST_OBJECT,
    POST_MESSAGE_HOST_OBJECT,
    STRATEGY_FILE_HOST_OBJECT,
    SERVER_TIME_HOST_OBJECT,
    HTML_MASK_HOST_OBJECT,
];

pub(in crate::runtime_bindings) const SIMPLE_HOST_OBJECTS_AFTER_KEY_MOUSE_HOOK:
    &[HostObjectBinding] = &[CUSTOM_HOST_FUNCTIONS_HOST_OBJECT, GENSHIN_HOST_OBJECT];
