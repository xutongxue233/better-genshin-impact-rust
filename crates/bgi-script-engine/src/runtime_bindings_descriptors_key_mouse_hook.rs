use super::MethodBinding;

pub(in crate::runtime_bindings) const KEY_MOUSE_HOOK_METHODS: &[MethodBinding] = &[
    MethodBinding {
        name: "onKeyDown",
        length: 2,
    },
    MethodBinding {
        name: "onKeyUp",
        length: 2,
    },
    MethodBinding {
        name: "onMouseDown",
        length: 1,
    },
    MethodBinding {
        name: "onMouseUp",
        length: 1,
    },
    MethodBinding {
        name: "onMouseMove",
        length: 2,
    },
    MethodBinding {
        name: "onMouseWheel",
        length: 1,
    },
    MethodBinding {
        name: "removeAllListeners",
        length: 0,
    },
    MethodBinding {
        name: "dispose",
        length: 0,
    },
    MethodBinding {
        name: "dispatchEvent",
        length: 1,
    },
    MethodBinding {
        name: "snapshot",
        length: 0,
    },
];
