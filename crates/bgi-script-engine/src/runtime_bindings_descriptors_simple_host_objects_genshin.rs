use bgi_script::ScriptHostTarget;

use super::super::{HostObjectBinding, MethodBinding};

const GENSHIN_METHODS: &[MethodBinding] = &[
    MethodBinding {
        name: "uid",
        length: 0,
    },
    MethodBinding {
        name: "tp",
        length: 1,
    },
    MethodBinding {
        name: "moveMapTo",
        length: 2,
    },
    MethodBinding {
        name: "moveIndependentMapTo",
        length: 3,
    },
    MethodBinding {
        name: "getBigMapZoomLevel",
        length: 0,
    },
    MethodBinding {
        name: "setBigMapZoomLevel",
        length: 1,
    },
    MethodBinding {
        name: "tpToStatueOfTheSeven",
        length: 0,
    },
    MethodBinding {
        name: "getPositionFromBigMap",
        length: 0,
    },
    MethodBinding {
        name: "getPositionFromMap",
        length: 0,
    },
    MethodBinding {
        name: "getPositionFromMapWithMatchingMethod",
        length: 1,
    },
    MethodBinding {
        name: "getCameraOrientation",
        length: 0,
    },
    MethodBinding {
        name: "switchParty",
        length: 1,
    },
    MethodBinding {
        name: "clearPartyCache",
        length: 0,
    },
    MethodBinding {
        name: "blessingOfTheWelkinMoon",
        length: 0,
    },
    MethodBinding {
        name: "chooseTalkOption",
        length: 1,
    },
    MethodBinding {
        name: "claimBattlePassRewards",
        length: 0,
    },
    MethodBinding {
        name: "claimEncounterPointsRewards",
        length: 0,
    },
    MethodBinding {
        name: "goToAdventurersGuild",
        length: 1,
    },
    MethodBinding {
        name: "goToCraftingBench",
        length: 1,
    },
    MethodBinding {
        name: "returnMainUi",
        length: 0,
    },
    MethodBinding {
        name: "autoFishing",
        length: 0,
    },
    MethodBinding {
        name: "relogin",
        length: 0,
    },
    MethodBinding {
        name: "wonderlandCycle",
        length: 0,
    },
    MethodBinding {
        name: "setTime",
        length: 2,
    },
    MethodBinding {
        name: "commands",
        length: 0,
    },
];

pub(super) const GENSHIN_HOST_OBJECT: HostObjectBinding = HostObjectBinding {
    global_name: "genshin",
    target: ScriptHostTarget::Genshin,
    methods: GENSHIN_METHODS,
};
