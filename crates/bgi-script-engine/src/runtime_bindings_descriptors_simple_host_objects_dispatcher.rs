use bgi_script::ScriptHostTarget;

use super::super::{HostObjectBinding, MethodBinding};

const DISPATCHER_METHODS: &[MethodBinding] = &[
    MethodBinding {
        name: "addTimer",
        length: 1,
    },
    MethodBinding {
        name: "addTrigger",
        length: 1,
    },
    MethodBinding {
        name: "clearAllTriggers",
        length: 0,
    },
    MethodBinding {
        name: "runTask",
        length: 1,
    },
    MethodBinding {
        name: "getLinkedCancellationTokenSource",
        length: 0,
    },
    MethodBinding {
        name: "getLinkedCancellationToken",
        length: 0,
    },
    MethodBinding {
        name: "runAutoDomainTask",
        length: 1,
    },
    MethodBinding {
        name: "runAutoBossTask",
        length: 1,
    },
    MethodBinding {
        name: "runAutoFightTask",
        length: 1,
    },
    MethodBinding {
        name: "runAutoLeyLineOutcropTask",
        length: 1,
    },
    MethodBinding {
        name: "runAutoStygianOnslaughtTask",
        length: 1,
    },
    MethodBinding {
        name: "commands",
        length: 0,
    },
];

pub(super) const DISPATCHER_HOST_OBJECT: HostObjectBinding = HostObjectBinding {
    global_name: "dispatcher",
    target: ScriptHostTarget::Dispatcher,
    methods: DISPATCHER_METHODS,
};
