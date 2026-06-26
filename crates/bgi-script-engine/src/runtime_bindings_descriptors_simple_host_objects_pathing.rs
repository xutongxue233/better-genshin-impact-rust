use bgi_script::ScriptHostTarget;

use super::super::{HostObjectBinding, MethodBinding};

const PATHING_SCRIPT_METHODS: &[MethodBinding] = &[
    MethodBinding {
        name: "run",
        length: 1,
    },
    MethodBinding {
        name: "runFile",
        length: 1,
    },
    MethodBinding {
        name: "runFileFromUser",
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
    MethodBinding {
        name: "planFileFromUser",
        length: 1,
    },
    MethodBinding {
        name: "isExists",
        length: 1,
    },
    MethodBinding {
        name: "isFile",
        length: 1,
    },
    MethodBinding {
        name: "isFolder",
        length: 1,
    },
    MethodBinding {
        name: "readPathSync",
        length: 1,
    },
];

const STRATEGY_FILE_METHODS: &[MethodBinding] = &[
    MethodBinding {
        name: "isFolder",
        length: 1,
    },
    MethodBinding {
        name: "isFile",
        length: 1,
    },
    MethodBinding {
        name: "isExists",
        length: 1,
    },
    MethodBinding {
        name: "readPathSync",
        length: 1,
    },
];

pub(super) const PATHING_SCRIPT_HOST_OBJECT: HostObjectBinding = HostObjectBinding {
    global_name: "pathingScript",
    target: ScriptHostTarget::PathingScript,
    methods: PATHING_SCRIPT_METHODS,
};

pub(super) const STRATEGY_FILE_HOST_OBJECT: HostObjectBinding = HostObjectBinding {
    global_name: "strategyFile",
    target: ScriptHostTarget::StrategyFile,
    methods: STRATEGY_FILE_METHODS,
};
