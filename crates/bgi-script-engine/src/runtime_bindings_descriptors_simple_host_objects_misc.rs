use bgi_script::ScriptHostTarget;

use super::super::{HostObjectBinding, MethodBinding};

const HTTP_METHODS: &[MethodBinding] = &[MethodBinding {
    name: "request",
    length: 2,
}];

const SERVER_TIME_METHODS: &[MethodBinding] = &[
    MethodBinding {
        name: "getServerTimeZoneOffset",
        length: 0,
    },
    MethodBinding {
        name: "serverTimeZoneOffsetMilliseconds",
        length: 0,
    },
];

const CUSTOM_HOST_FUNCTION_METHODS: &[MethodBinding] = &[
    MethodBinding {
        name: "newObj",
        length: 1,
    },
    MethodBinding {
        name: "delObj",
        length: 1,
    },
    MethodBinding {
        name: "type",
        length: 1,
    },
    MethodBinding {
        name: "toIterator",
        length: 1,
    },
    MethodBinding {
        name: "newVarOfArr",
        length: 2,
    },
];

pub(super) const HTTP_HOST_OBJECT: HostObjectBinding = HostObjectBinding {
    global_name: "http",
    target: ScriptHostTarget::Http,
    methods: HTTP_METHODS,
};

pub(super) const SERVER_TIME_HOST_OBJECT: HostObjectBinding = HostObjectBinding {
    global_name: "ServerTime",
    target: ScriptHostTarget::ServerTime,
    methods: SERVER_TIME_METHODS,
};

pub(super) const CUSTOM_HOST_FUNCTIONS_HOST_OBJECT: HostObjectBinding = HostObjectBinding {
    global_name: "host",
    target: ScriptHostTarget::CustomHostFunctions,
    methods: CUSTOM_HOST_FUNCTION_METHODS,
};
