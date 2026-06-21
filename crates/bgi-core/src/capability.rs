use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MigrationState {
    Ported,
    Scaffolded,
    PendingNativeBackend,
    LegacyReference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Capability {
    pub area: &'static str,
    pub state: MigrationState,
    pub rust_module: &'static str,
    pub legacy_reference: &'static str,
    pub notes: &'static str,
}

pub fn migration_capabilities() -> Vec<Capability> {
    vec![
        Capability {
            area: "configuration",
            state: MigrationState::Ported,
            rust_module: "bgi_core::config",
            legacy_reference: "BetterGenshinImpact/Core/Config",
            notes: "Models the full legacy AllConfig top-level surface, strongly types high-value sections, and preserves unknown/compatibility JSON fields while concrete task configs are ported.",
        },
        Capability {
            area: "trigger registry",
            state: MigrationState::Ported,
            rust_module: "bgi_core::trigger",
            legacy_reference: "BetterGenshinImpact/GameTask/GameTaskManager.cs",
            notes: "Provides the initial trigger descriptors and legacy priority ordering.",
        },
        Capability {
            area: "task runtime",
            state: MigrationState::Scaffolded,
            rust_module: "bgi_task",
            legacy_reference: "BetterGenshinImpact/GameTask/TaskTriggerDispatcher.cs and TaskRunner.cs",
            notes: "Dispatcher trigger selection, runner state, independent task descriptors, and progress model are scaffolded; concrete task implementations still need parity ports.",
        },
        Capability {
            area: "script runtime",
            state: MigrationState::Scaffolded,
            rust_module: "bgi_script",
            legacy_reference: "BetterGenshinImpact/Core/Script",
            notes: "Manifest, group, project loading, module search, settings schema, schedule, host API, cancellation, execution-plan, and host security policy models are scaffolded; native JS engine bridge and script host implementations still need parity ports.",
        },
        Capability {
            area: "pathing task model",
            state: MigrationState::Ported,
            rust_module: "bgi_core::pathing",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoPathing/Model",
            notes: "Loads route JSON, applies control.json5 overrides, and summarizes waypoints/actions.",
        },
        Capability {
            area: "asset resolution",
            state: MigrationState::Ported,
            rust_module: "bgi_core::assets",
            legacy_reference: "BetterGenshinImpact/GameTask/GameTaskManager.cs",
            notes: "Resolves feature assets with 1920x1080 fallback matching the legacy loader.",
        },
        Capability {
            area: "screen capture",
            state: MigrationState::Scaffolded,
            rust_module: "bgi_capture",
            legacy_reference: "Fischless.GameCapture",
            notes: "Capture mode model and BitBlt Windows boundary exist; DWM shared surface, WGC, and HDR ports still need D3D/WinRT parity.",
        },
        Capability {
            area: "input and hotkeys",
            state: MigrationState::Scaffolded,
            rust_module: "bgi_input and bgi_hotkey",
            legacy_reference: "Fischless.WindowsInput and Fischless.HotkeyCapture",
            notes: "SendInput event construction, PostMessageW background input planning, GenshinAction key-binding translation, and RegisterHotKey bindings exist; message-loop integration, Wine behavior, and task-level call sites still need parity work.",
        },
        Capability {
            area: "computer vision",
            state: MigrationState::Scaffolded,
            rust_module: "bgi_vision",
            legacy_reference: "BetterGenshinImpact/Core/Recognition",
            notes: "Recognition object, OCR result, template matching semantics, and ONNX model registry are scaffolded; OpenCV, OCR, ONNX, and DirectML providers still need native crates.",
        },
        Capability {
            area: "desktop UI",
            state: MigrationState::LegacyReference,
            rust_module: "not selected",
            legacy_reference: "BetterGenshinImpact/View and ViewModel",
            notes: "The original WPF UI is retained as a reference while a Rust UI toolkit is selected.",
        },
    ]
}
