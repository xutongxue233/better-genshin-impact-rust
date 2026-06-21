use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum HostPermission {
    BackgroundInput,
    Capture,
    Filesystem,
    GameAutomation,
    GameState,
    Input,
    Logging,
    Network,
    Notification,
    Overlay,
    Scheduler,
    Settings,
    Time,
    Vision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HostBindingKind {
    GlobalFunctionSet,
    Object,
    Type,
    Namespace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HostBindingPortState {
    MetadataOnly,
    RustModelReady,
    NativePending,
    SecurityReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HostBindingDescriptor {
    pub name: &'static str,
    pub kind: HostBindingKind,
    pub legacy_type: &'static str,
    pub members: &'static [&'static str],
    pub permissions: &'static [HostPermission],
    pub port_state: HostBindingPortState,
    pub notes: &'static str,
}

const GLOBAL_METHOD_MEMBERS: &[&str] = &[
    "sleep",
    "getVersion",
    "keyDown",
    "keyUp",
    "keyPress",
    "setGameMetrics",
    "getGameMetrics",
    "moveMouseBy",
    "moveMouseTo",
    "click",
    "leftButtonClick",
    "leftButtonDown",
    "leftButtonUp",
    "rightButtonClick",
    "rightButtonDown",
    "rightButtonUp",
    "middleButtonClick",
    "middleButtonDown",
    "middleButtonUp",
    "verticalScroll",
    "captureGameRegion",
    "getAvatars",
    "inputText",
];

const KEY_MOUSE_SCRIPT_MEMBERS: &[&str] = &["run", "runFile", "plan", "planFile"];
const PATHING_SCRIPT_MEMBERS: &[&str] = &[
    "run",
    "runFile",
    "runFileFromUser",
    "plan",
    "planFile",
    "planFileFromUser",
    "isExists",
    "isFile",
    "isFolder",
    "readPathSync",
];
const GENSHIN_MEMBERS: &[&str] = &[
    "uid",
    "tp",
    "moveMapTo",
    "moveIndependentMapTo",
    "getBigMapZoomLevel",
    "setBigMapZoomLevel",
    "tpToStatueOfTheSeven",
    "getPositionFromBigMap",
    "getPositionFromMap",
    "getPositionFromMapWithMatchingMethod",
    "getCameraOrientation",
    "switchParty",
    "clearPartyCache",
    "blessingOfTheWelkinMoon",
    "chooseTalkOption",
    "claimBattlePassRewards",
    "claimEncounterPointsRewards",
    "goToAdventurersGuild",
    "goToCraftingBench",
    "returnMainUi",
    "autoFishing",
    "relogin",
    "wonderlandCycle",
    "setTime",
    "commands",
];
const LOG_MEMBERS: &[&str] = &["debug", "info", "warn", "error"];
const FILE_MEMBERS: &[&str] = &[
    "readPathSync",
    "createDirectory",
    "isFolder",
    "isFile",
    "isExists",
    "readTextSync",
    "readText",
    "readImageMatSync",
    "readImageMatWithResizeSync",
    "writeTextSync",
    "writeText",
    "writeImageSync",
    "renamePathSync",
];
const VISION_MEMBERS: &[&str] = &["findTemplate", "findColor", "crop", "to1080p"];
const HTTP_MEMBERS: &[&str] = &["request"];
const NOTIFICATION_MEMBERS: &[&str] = &["send", "success", "error", "records"];
const DISPATCHER_MEMBERS: &[&str] = &[
    "addTimer",
    "clearAllTriggers",
    "addTrigger",
    "runTask",
    "getLinkedCancellationTokenSource",
    "getLinkedCancellationToken",
    "runAutoDomainTask",
    "runAutoBossTask",
    "runAutoFightTask",
    "runAutoLeyLineOutcropTask",
    "runAutoStygianOnslaughtTask",
    "commands",
];
const POST_MESSAGE_MEMBERS: &[&str] = &["keyDown", "keyUp", "keyPress", "click"];
const STRATEGY_FILE_MEMBERS: &[&str] = &["isFolder", "isFile", "isExists", "readPathSync"];
const SERVER_TIME_MEMBERS: &[&str] = &[
    "getServerTimeZoneOffset",
    "serverTimeZoneOffsetMilliseconds",
];
const HTML_MASK_MEMBERS: &[&str] = &[
    "show",
    "close",
    "closeAll",
    "getWindowIds",
    "exists",
    "setClickThrough",
    "getClickThrough",
    "toggleClickThrough",
    "send",
    "request",
    "receive",
    "poll",
    "pollAll",
    "flushPendingMessages",
    "sendFromHtml",
    "snapshot",
];
const KEY_MOUSE_HOOK_MEMBERS: &[&str] = &[
    "onKeyDown",
    "onKeyUp",
    "onMouseDown",
    "onMouseUp",
    "onMouseMove",
    "onMouseWheel",
    "removeAllListeners",
    "dispose",
    "dispatchEvent",
    "snapshot",
];
const CUSTOM_HOST_MEMBERS: &[&str] = &["newObj", "delObj", "type", "toIterator", "newVarOfArr"];
const OPENCV_NAMESPACE_MEMBERS: &[&str] = &["Mat", "Point2f", "Cv2"];
const VISION_TYPE_MEMBERS: &[&str] = &[
    "RecognitionObject",
    "DesktopRegion",
    "GameCaptureRegion",
    "ImageRegion",
    "Region",
    "BvPage",
    "BvLocator",
    "BvImage",
    "CombatScenes",
    "Avatar",
];
const PARAM_TYPE_MEMBERS: &[&str] = &[
    "AutoSkipConfig",
    "AutoDomainParam",
    "AutoBossParam",
    "AutoFightParam",
    "AutoLeyLineOutcropParam",
    "AutoStygianOnslaughtParam",
];
const CANCELLATION_MEMBERS: &[&str] = &["CancellationTokenSource", "CancellationToken"];
const TASK_TYPE_MEMBERS: &[&str] = &["Task"];
const REALTIME_TIMER_MEMBERS: &[&str] = &["RealtimeTimer", "name", "interval", "config"];
const SOLO_TASK_MEMBERS: &[&str] = &["SoloTask", "name", "config"];

const INPUT_PERMISSIONS: &[HostPermission] = &[HostPermission::Input];
const BACKGROUND_INPUT_PERMISSIONS: &[HostPermission] =
    &[HostPermission::BackgroundInput, HostPermission::Input];
const PATHING_PERMISSIONS: &[HostPermission] = &[
    HostPermission::Filesystem,
    HostPermission::GameAutomation,
    HostPermission::Scheduler,
];
const GENSHIN_PERMISSIONS: &[HostPermission] = &[
    HostPermission::Capture,
    HostPermission::GameAutomation,
    HostPermission::GameState,
    HostPermission::Input,
    HostPermission::Vision,
];
const FILE_PERMISSIONS: &[HostPermission] = &[HostPermission::Filesystem];
const HTTP_PERMISSIONS: &[HostPermission] = &[HostPermission::Network];
const NOTIFICATION_PERMISSIONS: &[HostPermission] = &[HostPermission::Notification];
const SCHEDULER_PERMISSIONS: &[HostPermission] = &[
    HostPermission::GameAutomation,
    HostPermission::Scheduler,
    HostPermission::Settings,
];
const OVERLAY_PERMISSIONS: &[HostPermission] = &[HostPermission::Overlay, HostPermission::Network];
const VISION_PERMISSIONS: &[HostPermission] = &[HostPermission::Capture, HostPermission::Vision];
const TIME_PERMISSIONS: &[HostPermission] = &[HostPermission::Time];
const LOG_PERMISSIONS: &[HostPermission] = &[HostPermission::Logging];
const SETTINGS_PERMISSIONS: &[HostPermission] = &[HostPermission::Settings];
const NO_PERMISSIONS: &[HostPermission] = &[];

pub fn host_bindings() -> Vec<HostBindingDescriptor> {
    vec![
        HostBindingDescriptor {
            name: "global",
            kind: HostBindingKind::GlobalFunctionSet,
            legacy_type: "GlobalMethod",
            members: GLOBAL_METHOD_MEMBERS,
            permissions: &[
                HostPermission::Capture,
                HostPermission::GameState,
                HostPermission::Input,
                HostPermission::Vision,
            ],
            port_state: HostBindingPortState::RustModelReady,
            notes: "Input, mouse, text, game-metrics, captureGameRegion, and getAvatars helpers have Rust host plans; captureGameRegion can execute against an injected BGR24 frame source, and desktop JavaScript runs can inject a real BitBlt game-window source plus SendInput window activation when the window is found. Real-window validation and avatar vision execution remain pending.",
        },
        HostBindingDescriptor {
            name: "keyMouseScript",
            kind: HostBindingKind::Object,
            legacy_type: "KeyMouseScript",
            members: KEY_MOUSE_SCRIPT_MEMBERS,
            permissions: INPUT_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Rust host boundary can run inline JSON or rooted files into input-event plans, and Boa host calls can dispatch through SendInput in desktop/group runs with optional game-window activation and shared Stop-token cancellation.",
        },
        HostBindingDescriptor {
            name: "pathingScript",
            kind: HostBindingKind::Object,
            legacy_type: "AutoPathingScript",
            members: PATHING_SCRIPT_MEMBERS,
            permissions: PATHING_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Rust can prepare inline, script-root file, and User/AutoPathing route execution with rooted path checks; native movement executor parity is still pending.",
        },
        HostBindingDescriptor {
            name: "genshin",
            kind: HostBindingKind::Object,
            legacy_type: "Genshin",
            members: GENSHIN_MEMBERS,
            permissions: GENSHIN_PERMISSIONS,
            port_state: HostBindingPortState::NativePending,
            notes: "High-level game automation helpers that depend on capture, vision, and input.",
        },
        HostBindingDescriptor {
            name: "log",
            kind: HostBindingKind::Object,
            legacy_type: "Log",
            members: LOG_MEMBERS,
            permissions: LOG_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Maps script logging levels to the application log sink.",
        },
        HostBindingDescriptor {
            name: "file",
            kind: HostBindingKind::Object,
            legacy_type: "LimitedFile",
            members: FILE_MEMBERS,
            permissions: FILE_PERMISSIONS,
            port_state: HostBindingPortState::SecurityReview,
            notes: "Rooted path, existence, listing, text read/write, directory creation, rename, and BGR24 Mat image read/resize/write execution are ported with extension limits; full OpenCV Mat API parity remains pending.",
        },
        HostBindingDescriptor {
            name: "vision",
            kind: HostBindingKind::Object,
            legacy_type: "Pure Rust recognition host",
            members: VISION_MEMBERS,
            permissions: VISION_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Executes BGR24 Mat payload crop/1080p derivation, template matching, and color matching through the Rust ImageRegion/PureRustVisionBackend path; OCR, detection, and full OpenCV interop remain pending.",
        },
        HostBindingDescriptor {
            name: "http",
            kind: HostBindingKind::Object,
            legacy_type: "Http",
            members: HTTP_MEMBERS,
            permissions: HTTP_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Request permission, manifest URL allow-list matching, headers normalization, request planning, legacy response shape, and pluggable native HTTP dispatch are ported.",
        },
        HostBindingDescriptor {
            name: "notification",
            kind: HostBindingKind::Object,
            legacy_type: "Notification",
            members: NOTIFICATION_MEMBERS,
            permissions: NOTIFICATION_PERMISSIONS,
            port_state: HostBindingPortState::SecurityReview,
            notes: "Policy validation, rate limiting, JS event mapping, pluggable notification sink delivery, and desktop script-to-app provider forwarding are ported; background notifier lifecycle wiring remains pending.",
        },
        HostBindingDescriptor {
            name: "dispatcher",
            kind: HostBindingKind::Object,
            legacy_type: "Dispatcher",
            members: DISPATCHER_MEMBERS,
            permissions: SCHEDULER_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Models realtime timers, trigger clearing, solo task calls, built-in dispatcher task calls, and linked cancellation handles as Rust command plans.",
        },
        HostBindingDescriptor {
            name: "RealtimeTimer",
            kind: HostBindingKind::Type,
            legacy_type: "RealtimeTimer",
            members: REALTIME_TIMER_MEMBERS,
            permissions: SCHEDULER_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Timer name and config are passed to the realtime trigger dispatcher.",
        },
        HostBindingDescriptor {
            name: "SoloTask",
            kind: HostBindingKind::Type,
            legacy_type: "SoloTask",
            members: SOLO_TASK_MEMBERS,
            permissions: SCHEDULER_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Script-created independent task invocation model.",
        },
        HostBindingDescriptor {
            name: "PostMessage",
            kind: HostBindingKind::Type,
            legacy_type: "Simulator.PostMessage",
            members: POST_MESSAGE_MEMBERS,
            permissions: BACKGROUND_INPUT_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Background input message planning, Windows PostMessageW dispatch boundary, and unified script-host runtime routing are ported.",
        },
        HostBindingDescriptor {
            name: "strategyFile",
            kind: HostBindingKind::Object,
            legacy_type: "StrategyFile",
            members: STRATEGY_FILE_MEMBERS,
            permissions: FILE_PERMISSIONS,
            port_state: HostBindingPortState::SecurityReview,
            notes: "Restricted rooted view over User/AutoFight strategy files is ported for existence checks and listing.",
        },
        HostBindingDescriptor {
            name: "ServerTime",
            kind: HostBindingKind::Type,
            legacy_type: "ServerTime",
            members: SERVER_TIME_MEMBERS,
            permissions: TIME_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Exposes the configured server timezone offset in milliseconds through the unified script-host runtime.",
        },
        HostBindingDescriptor {
            name: "htmlMask",
            kind: HostBindingKind::Object,
            legacy_type: "HtmlMask",
            members: HTML_MASK_MEMBERS,
            permissions: OVERLAY_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Window state, rooted relative URL planning, click-through state, and message queues are modeled; Tauri WebView overlay dispatch remains pending.",
        },
        HostBindingDescriptor {
            name: "KeyMouseHook",
            kind: HostBindingKind::Type,
            legacy_type: "KeyMouseHook",
            members: KEY_MOUSE_HOOK_MEMBERS,
            permissions: INPUT_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Listener registration, removal, event payloads, and mouse-move throttling are modeled; native Windows hook dispatch remains pending.",
        },
        HostBindingDescriptor {
            name: "host",
            kind: HostBindingKind::Object,
            legacy_type: "CustomHostFunctions",
            members: CUSTOM_HOST_MEMBERS,
            permissions: NO_PERMISSIONS,
            port_state: HostBindingPortState::SecurityReview,
            notes: "ClearScript host helpers are modeled as explicit Rust commands, including jagged array variable creation for NewVarOfArr.",
        },
        HostBindingDescriptor {
            name: "OpenCvSharp",
            kind: HostBindingKind::Namespace,
            legacy_type: "HostTypeCollection(OpenCvSharp)",
            members: OPENCV_NAMESPACE_MEMBERS,
            permissions: VISION_PERMISSIONS,
            port_state: HostBindingPortState::NativePending,
            notes: "Namespace exposure for OpenCV objects used by legacy scripts.",
        },
        HostBindingDescriptor {
            name: "visionTypes",
            kind: HostBindingKind::Type,
            legacy_type: "Recognition and Bv* host types",
            members: VISION_TYPE_MEMBERS,
            permissions: VISION_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Rust has value models; native OpenCV/OCR operations remain pending.",
        },
        HostBindingDescriptor {
            name: "taskParams",
            kind: HostBindingKind::Type,
            legacy_type: "Auto*Param host types",
            members: PARAM_TYPE_MEMBERS,
            permissions: SETTINGS_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "AutoSkipConfig and AutoDomain/AutoBoss/AutoFight/AutoLeyLineOutcrop/AutoStygianOnslaught parameter shapes have Rust models; task executors still need native ports.",
        },
        HostBindingDescriptor {
            name: "cancellation",
            kind: HostBindingKind::Type,
            legacy_type: "CancellationTokenSource and CancellationToken",
            members: CANCELLATION_MEMBERS,
            permissions: SCHEDULER_PERMISSIONS,
            port_state: HostBindingPortState::RustModelReady,
            notes: "Linked cancellation token semantics are modeled for script tasks.",
        },
        HostBindingDescriptor {
            name: "Task",
            kind: HostBindingKind::Type,
            legacy_type: "System.Threading.Tasks.Task",
            members: TASK_TYPE_MEMBERS,
            permissions: NO_PERMISSIONS,
            port_state: HostBindingPortState::MetadataOnly,
            notes: "Async interop marker for the eventual JS engine bridge.",
        },
    ]
}

pub fn host_permissions(bindings: &[HostBindingDescriptor]) -> Vec<HostPermission> {
    bindings
        .iter()
        .flat_map(|binding| binding.permissions.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn host_binding_count_by_kind(
    bindings: &[HostBindingDescriptor],
    kind: HostBindingKind,
) -> usize {
    bindings
        .iter()
        .filter(|binding| binding.kind == kind)
        .count()
}

pub fn host_member_count(bindings: &[HostBindingDescriptor]) -> usize {
    bindings.iter().map(|binding| binding.members.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn host_binding_names_are_unique() {
        let bindings = host_bindings();
        let names = bindings
            .iter()
            .map(|binding| binding.name)
            .collect::<BTreeSet<_>>();

        assert_eq!(names.len(), bindings.len());
    }

    #[test]
    fn global_methods_include_legacy_input_surface() {
        let global = host_bindings()
            .into_iter()
            .find(|binding| binding.name == "global")
            .unwrap();

        assert!(global.members.contains(&"keyDown"));
        assert!(global.members.contains(&"captureGameRegion"));
        assert!(global.permissions.contains(&HostPermission::Input));
    }
}
