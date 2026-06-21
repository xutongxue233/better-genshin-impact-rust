use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NavigationItem {
    pub key: &'static str,
    pub label: &'static str,
    pub route: &'static str,
    pub children: Vec<NavigationItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UiShellDecision {
    pub shell: &'static str,
    pub frontend: &'static str,
    pub reason: &'static str,
    pub fallback: &'static str,
}

pub fn ui_shell_decision() -> UiShellDecision {
    UiShellDecision {
        shell: "Tauri 2",
        frontend: "Svelte + TypeScript",
        reason: "BetterGI is a Windows desktop automation tool with dense settings, map editors, script management, tray integration, WebView workflows, and native Rust backends. Tauri keeps Rust as the application boundary while letting the UI use a mature web component stack.",
        fallback: "Slint can replace the web frontend only if a pure-Rust UI becomes a hard requirement; it is not the primary choice because script/map editing benefits from web UI primitives.",
    }
}

pub fn default_navigation() -> Vec<NavigationItem> {
    vec![
        NavigationItem {
            key: "home",
            label: "Startup",
            route: "/",
            children: Vec::new(),
        },
        NavigationItem {
            key: "realtime",
            label: "Realtime Triggers",
            route: "/triggers",
            children: Vec::new(),
        },
        NavigationItem {
            key: "tasks",
            label: "Independent Tasks",
            route: "/tasks",
            children: Vec::new(),
        },
        NavigationItem {
            key: "one-dragon",
            label: "One Dragon",
            route: "/one-dragon",
            children: Vec::new(),
        },
        NavigationItem {
            key: "automation",
            label: "Automation",
            route: "/automation",
            children: vec![
                NavigationItem {
                    key: "scheduler",
                    label: "Scheduler",
                    route: "/automation/scheduler",
                    children: Vec::new(),
                },
                NavigationItem {
                    key: "scripts",
                    label: "JS Scripts",
                    route: "/automation/scripts",
                    children: Vec::new(),
                },
                NavigationItem {
                    key: "pathing",
                    label: "Map Pathing",
                    route: "/automation/pathing",
                    children: Vec::new(),
                },
                NavigationItem {
                    key: "recorder",
                    label: "Record/Replay",
                    route: "/automation/recorder",
                    children: Vec::new(),
                },
            ],
        },
        NavigationItem {
            key: "macro",
            label: "Control Assist",
            route: "/macro",
            children: Vec::new(),
        },
        NavigationItem {
            key: "hotkeys",
            label: "Hotkeys",
            route: "/hotkeys",
            children: Vec::new(),
        },
        NavigationItem {
            key: "notifications",
            label: "Notifications",
            route: "/notifications",
            children: Vec::new(),
        },
        NavigationItem {
            key: "settings",
            label: "Settings",
            route: "/settings",
            children: Vec::new(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_covers_legacy_main_window_sections() {
        let keys: Vec<_> = default_navigation()
            .into_iter()
            .map(|item| item.key)
            .collect();
        assert!(keys.contains(&"home"));
        assert!(keys.contains(&"realtime"));
        assert!(keys.contains(&"tasks"));
        assert!(keys.contains(&"automation"));
        assert!(keys.contains(&"settings"));
    }
}
