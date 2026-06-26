use bgi_script::ScriptHostTarget;

use super::super::{HostObjectBinding, MethodBinding};

const KEY_MOUSE_SCRIPT_METHODS: &[MethodBinding] = &[
    MethodBinding {
        name: "run",
        length: 1,
    },
    MethodBinding {
        name: "runFile",
        length: 1,
    },
    MethodBinding {
        name: "plan",
        length: 1,
    },
    MethodBinding {
        name: "planFile",
        length: 1,
    },
];

const POST_MESSAGE_METHODS: &[MethodBinding] = &[
    MethodBinding {
        name: "keyDown",
        length: 1,
    },
    MethodBinding {
        name: "keyUp",
        length: 1,
    },
    MethodBinding {
        name: "keyPress",
        length: 1,
    },
    MethodBinding {
        name: "click",
        length: 2,
    },
];

pub(super) const KEY_MOUSE_SCRIPT_HOST_OBJECT: HostObjectBinding = HostObjectBinding {
    global_name: "keyMouseScript",
    target: ScriptHostTarget::KeyMouseScript,
    methods: KEY_MOUSE_SCRIPT_METHODS,
};

pub(super) const POST_MESSAGE_HOST_OBJECT: HostObjectBinding = HostObjectBinding {
    global_name: "PostMessage",
    target: ScriptHostTarget::PostMessage,
    methods: POST_MESSAGE_METHODS,
};
