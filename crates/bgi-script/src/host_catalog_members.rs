use crate::host::HostPermission;

pub(super) const GLOBAL_METHOD_MEMBERS: &[&str] = &[
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

pub(super) const KEY_MOUSE_SCRIPT_MEMBERS: &[&str] = &["run", "runFile", "plan", "planFile"];
pub(super) const PATHING_SCRIPT_MEMBERS: &[&str] = &[
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
pub(super) const GENSHIN_MEMBERS: &[&str] = &[
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
pub(super) const LOG_MEMBERS: &[&str] = &["debug", "info", "warn", "error"];
pub(super) const FILE_MEMBERS: &[&str] = &[
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
pub(super) const VISION_MEMBERS: &[&str] = &["findTemplate", "findColor", "crop", "to1080p"];
pub(super) const HTTP_MEMBERS: &[&str] = &["request"];
pub(super) const NOTIFICATION_MEMBERS: &[&str] = &["send", "success", "error", "records"];
pub(super) const DISPATCHER_MEMBERS: &[&str] = &[
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
pub(super) const POST_MESSAGE_MEMBERS: &[&str] = &["keyDown", "keyUp", "keyPress", "click"];
pub(super) const STRATEGY_FILE_MEMBERS: &[&str] =
    &["isFolder", "isFile", "isExists", "readPathSync"];
pub(super) const SERVER_TIME_MEMBERS: &[&str] = &[
    "getServerTimeZoneOffset",
    "serverTimeZoneOffsetMilliseconds",
];
pub(super) const HTML_MASK_MEMBERS: &[&str] = &[
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
pub(super) const KEY_MOUSE_HOOK_MEMBERS: &[&str] = &[
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
pub(super) const CUSTOM_HOST_MEMBERS: &[&str] =
    &["newObj", "delObj", "type", "toIterator", "newVarOfArr"];
pub(super) const OPENCV_NAMESPACE_MEMBERS: &[&str] = &["Mat", "Point2f", "Cv2"];
pub(super) const VISION_TYPE_MEMBERS: &[&str] = &[
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
pub(super) const PARAM_TYPE_MEMBERS: &[&str] = &[
    "AutoSkipConfig",
    "AutoDomainParam",
    "AutoBossParam",
    "AutoFightParam",
    "AutoLeyLineOutcropParam",
    "AutoStygianOnslaughtParam",
];
pub(super) const CANCELLATION_MEMBERS: &[&str] = &["CancellationTokenSource", "CancellationToken"];
pub(super) const TASK_TYPE_MEMBERS: &[&str] = &["Task"];
pub(super) const REALTIME_TIMER_MEMBERS: &[&str] = &["RealtimeTimer", "name", "interval", "config"];
pub(super) const SOLO_TASK_MEMBERS: &[&str] = &["SoloTask", "name", "config"];

pub(super) const INPUT_PERMISSIONS: &[HostPermission] = &[HostPermission::Input];
pub(super) const BACKGROUND_INPUT_PERMISSIONS: &[HostPermission] =
    &[HostPermission::BackgroundInput, HostPermission::Input];
pub(super) const PATHING_PERMISSIONS: &[HostPermission] = &[
    HostPermission::Filesystem,
    HostPermission::GameAutomation,
    HostPermission::Scheduler,
];
pub(super) const GENSHIN_PERMISSIONS: &[HostPermission] = &[
    HostPermission::Capture,
    HostPermission::GameAutomation,
    HostPermission::GameState,
    HostPermission::Input,
    HostPermission::Vision,
];
pub(super) const FILE_PERMISSIONS: &[HostPermission] = &[HostPermission::Filesystem];
pub(super) const HTTP_PERMISSIONS: &[HostPermission] = &[HostPermission::Network];
pub(super) const NOTIFICATION_PERMISSIONS: &[HostPermission] = &[HostPermission::Notification];
pub(super) const SCHEDULER_PERMISSIONS: &[HostPermission] = &[
    HostPermission::GameAutomation,
    HostPermission::Scheduler,
    HostPermission::Settings,
];
pub(super) const OVERLAY_PERMISSIONS: &[HostPermission] =
    &[HostPermission::Overlay, HostPermission::Network];
pub(super) const VISION_PERMISSIONS: &[HostPermission] =
    &[HostPermission::Capture, HostPermission::Vision];
pub(super) const TIME_PERMISSIONS: &[HostPermission] = &[HostPermission::Time];
pub(super) const LOG_PERMISSIONS: &[HostPermission] = &[HostPermission::Logging];
pub(super) const SETTINGS_PERMISSIONS: &[HostPermission] = &[HostPermission::Settings];
pub(super) const NO_PERMISSIONS: &[HostPermission] = &[];
