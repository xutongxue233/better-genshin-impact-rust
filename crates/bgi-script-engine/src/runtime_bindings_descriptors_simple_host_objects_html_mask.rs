use bgi_script::ScriptHostTarget;

use super::super::{HostObjectBinding, MethodBinding};

const HTML_MASK_METHODS: &[MethodBinding] = &[
    MethodBinding {
        name: "show",
        length: 2,
    },
    MethodBinding {
        name: "close",
        length: 1,
    },
    MethodBinding {
        name: "closeAll",
        length: 0,
    },
    MethodBinding {
        name: "getWindowIds",
        length: 0,
    },
    MethodBinding {
        name: "exists",
        length: 1,
    },
    MethodBinding {
        name: "setClickThrough",
        length: 2,
    },
    MethodBinding {
        name: "getClickThrough",
        length: 1,
    },
    MethodBinding {
        name: "toggleClickThrough",
        length: 1,
    },
    MethodBinding {
        name: "send",
        length: 3,
    },
    MethodBinding {
        name: "request",
        length: 4,
    },
    MethodBinding {
        name: "receive",
        length: 2,
    },
    MethodBinding {
        name: "poll",
        length: 1,
    },
    MethodBinding {
        name: "pollAll",
        length: 1,
    },
    MethodBinding {
        name: "flushPendingMessages",
        length: 1,
    },
    MethodBinding {
        name: "sendFromHtml",
        length: 4,
    },
    MethodBinding {
        name: "snapshot",
        length: 0,
    },
];

pub(super) const HTML_MASK_HOST_OBJECT: HostObjectBinding = HostObjectBinding {
    global_name: "htmlMask",
    target: ScriptHostTarget::HtmlMask,
    methods: HTML_MASK_METHODS,
};
