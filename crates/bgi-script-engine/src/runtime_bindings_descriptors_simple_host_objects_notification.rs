use bgi_script::ScriptHostTarget;

use super::super::{HostObjectBinding, MethodBinding};

const NOTIFICATION_METHODS: &[MethodBinding] = &[
    MethodBinding {
        name: "send",
        length: 1,
    },
    MethodBinding {
        name: "success",
        length: 1,
    },
    MethodBinding {
        name: "error",
        length: 1,
    },
    MethodBinding {
        name: "records",
        length: 0,
    },
];

pub(super) const NOTIFICATION_HOST_OBJECT: HostObjectBinding = HostObjectBinding {
    global_name: "notification",
    target: ScriptHostTarget::Notification,
    methods: NOTIFICATION_METHODS,
};
