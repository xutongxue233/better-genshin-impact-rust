use super::MethodBinding;

pub(in crate::runtime_bindings) const LOG_METHODS: &[MethodBinding] = &[
    MethodBinding {
        name: "debug",
        length: 1,
    },
    MethodBinding {
        name: "info",
        length: 1,
    },
    MethodBinding {
        name: "warn",
        length: 1,
    },
    MethodBinding {
        name: "error",
        length: 1,
    },
];

pub(in crate::runtime_bindings) const GLOBAL_HOST_FUNCTIONS: &[MethodBinding] = &[
    MethodBinding {
        name: "sleep",
        length: 1,
    },
    MethodBinding {
        name: "getVersion",
        length: 0,
    },
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
        name: "setGameMetrics",
        length: 2,
    },
    MethodBinding {
        name: "getGameMetrics",
        length: 0,
    },
    MethodBinding {
        name: "moveMouseBy",
        length: 2,
    },
    MethodBinding {
        name: "moveMouseTo",
        length: 2,
    },
    MethodBinding {
        name: "click",
        length: 0,
    },
    MethodBinding {
        name: "leftButtonClick",
        length: 0,
    },
    MethodBinding {
        name: "leftButtonDown",
        length: 0,
    },
    MethodBinding {
        name: "leftButtonUp",
        length: 0,
    },
    MethodBinding {
        name: "rightButtonClick",
        length: 0,
    },
    MethodBinding {
        name: "rightButtonDown",
        length: 0,
    },
    MethodBinding {
        name: "rightButtonUp",
        length: 0,
    },
    MethodBinding {
        name: "middleButtonClick",
        length: 0,
    },
    MethodBinding {
        name: "middleButtonDown",
        length: 0,
    },
    MethodBinding {
        name: "middleButtonUp",
        length: 0,
    },
    MethodBinding {
        name: "verticalScroll",
        length: 1,
    },
    MethodBinding {
        name: "captureGameRegion",
        length: 0,
    },
    MethodBinding {
        name: "getAvatars",
        length: 0,
    },
    MethodBinding {
        name: "inputText",
        length: 1,
    },
];
