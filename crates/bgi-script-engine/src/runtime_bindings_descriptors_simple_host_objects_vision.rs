use bgi_script::ScriptHostTarget;

use super::super::{HostObjectBinding, MethodBinding};

const VISION_METHODS: &[MethodBinding] = &[
    MethodBinding {
        name: "findTemplate",
        length: 3,
    },
    MethodBinding {
        name: "findColor",
        length: 2,
    },
    MethodBinding {
        name: "crop",
        length: 2,
    },
    MethodBinding {
        name: "to1080p",
        length: 1,
    },
];

pub(super) const VISION_HOST_OBJECT: HostObjectBinding = HostObjectBinding {
    global_name: "vision",
    target: ScriptHostTarget::Vision,
    methods: VISION_METHODS,
};
