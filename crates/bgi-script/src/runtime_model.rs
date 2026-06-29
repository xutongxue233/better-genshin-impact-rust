use super::{Result, ScriptRuntimeError};
use crate::group::{ScriptGroup, ScriptGroupProject, ScriptProjectStatus, ScriptProjectType};
use crate::host::{host_bindings, HostBindingDescriptor};
use crate::manifest::Manifest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptRuntimeState {
    Stopped,
    Loading,
    Running,
    Cancelling,
    Cancelled,
    Completed,
    Faulted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptEngineKind {
    ClearScriptV8,
    RustJavaScript,
    KeyMouseMacro,
    PathingExecutor,
    Shell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptEnginePortState {
    LegacyReference,
    Planned,
    MetadataReady,
    NativePending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptEngineDescriptor {
    pub kind: ScriptEngineKind,
    pub port_state: ScriptEnginePortState,
    pub legacy_reference: &'static str,
    pub notes: &'static str,
}

pub fn script_engines() -> Vec<ScriptEngineDescriptor> {
    vec![
        ScriptEngineDescriptor {
            kind: ScriptEngineKind::ClearScriptV8,
            port_state: ScriptEnginePortState::LegacyReference,
            legacy_reference: "Microsoft.ClearScript.V8",
            notes: "Legacy engine reference; not part of the final Rust runtime.",
        },
        ScriptEngineDescriptor {
            kind: ScriptEngineKind::RustJavaScript,
            port_state: ScriptEnginePortState::NativePending,
            legacy_reference: "ScriptProject.ExecuteAsync",
            notes: "Classic/module execution, host runtime routing, and execution context preparation are ported; desktop Stop can interrupt classic-script Boa execution and routed key/mouse macro playback, while long-running async promise conversion and full module/job interruption remain.",
        },
        ScriptEngineDescriptor {
            kind: ScriptEngineKind::KeyMouseMacro,
            port_state: ScriptEnginePortState::MetadataReady,
            legacy_reference: "KeyMouseMacroPlayer.PlayMacro",
            notes: "Recorder JSON parsing, rooted script host planning, input event playback planning, SendInput dispatch, cancellation-aware playback checks, and desktop Stop wiring are ported; live recording and broader task call-site migration remain.",
        },
        ScriptEngineDescriptor {
            kind: ScriptEngineKind::PathingExecutor,
            port_state: ScriptEnginePortState::MetadataReady,
            legacy_reference: "PathExecutor.Pathing",
            notes: "Route models exist; action handlers and movement control are pending.",
        },
        ScriptEngineDescriptor {
            kind: ScriptEngineKind::Shell,
            port_state: ScriptEnginePortState::Planned,
            legacy_reference: "ShellTask",
            notes:
                "Requires command policy, environment handling, output capture, and cancellation.",
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptCancellationKind {
    GlobalToken,
    LinkedToken,
    ManualStop,
    EngineInterrupt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptCancellationPolicy {
    pub reset_before_run: bool,
    pub manual_stop_sets_flag: bool,
    pub clear_disposes_token: bool,
    pub linked_token_for_dispatcher_tasks: bool,
    pub final_engine_interrupt: bool,
}

impl Default for ScriptCancellationPolicy {
    fn default() -> Self {
        Self {
            reset_before_run: true,
            manual_stop_sets_flag: true,
            clear_disposes_token: true,
            linked_token_for_dispatcher_tasks: true,
            final_engine_interrupt: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptScheduleKind {
    Daily,
    EveryTwoDays,
    Weekday,
    Cron,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptSchedule {
    pub raw: String,
    pub kind: ScriptScheduleKind,
    pub description: String,
}

impl ScriptSchedule {
    pub fn parse(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let (kind, description) = match raw.as_str() {
            "" => (ScriptScheduleKind::Manual, "Manual".to_string()),
            "Daily" => (ScriptScheduleKind::Daily, "Daily".to_string()),
            "EveryTwoDays" => (
                ScriptScheduleKind::EveryTwoDays,
                "Every two days".to_string(),
            ),
            "Monday" | "Tuesday" | "Wednesday" | "Thursday" | "Friday" | "Saturday" | "Sunday" => {
                (ScriptScheduleKind::Weekday, raw.clone())
            }
            _ => (ScriptScheduleKind::Cron, format!("Cron: {raw}")),
        };

        Self {
            raw,
            kind,
            description,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealtimeTimerPlan {
    pub name: String,
    pub interval_ms: u64,
    pub config: Option<Value>,
    pub replaces_existing_triggers: bool,
}

impl RealtimeTimerPlan {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            interval_ms: 50,
            config: None,
            replaces_existing_triggers: false,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(ScriptRuntimeError::MissingRunnableTarget(
                "RealtimeTimer".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoloTaskPlan {
    pub name: String,
    pub config: Option<Value>,
    pub uses_linked_cancellation: bool,
}

impl SoloTaskPlan {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            config: None,
            uses_linked_cancellation: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptExecutionStep {
    pub index: i32,
    pub name: String,
    pub folder_name: String,
    pub project_type: ScriptProjectType,
    pub engine: ScriptEngineKind,
    pub schedule: ScriptSchedule,
    pub run_count: u32,
    pub settings: Option<Value>,
    pub allow_notification: bool,
    pub allow_http_hash: Option<String>,
    pub target_path: Option<PathBuf>,
    pub manifest_main: Option<String>,
    pub skipped: bool,
}

impl ScriptExecutionStep {
    pub fn from_group_project(
        project: &ScriptGroupProject,
        manifest: Option<&Manifest>,
        script_root: impl AsRef<Path>,
    ) -> Result<Self> {
        if project.status == ScriptProjectStatus::Disabled {
            return Err(ScriptRuntimeError::ProjectDisabled(project.name.clone()));
        }

        let engine = engine_for_project_type(project.project_type.clone());
        let script_root = script_root.as_ref();
        let target_path = match project.project_type.clone() {
            ScriptProjectType::Javascript => {
                let manifest = manifest.ok_or_else(|| ScriptRuntimeError::MissingManifest {
                    project: project.name.clone(),
                })?;
                if manifest.main.trim().is_empty() {
                    return Err(ScriptRuntimeError::MissingManifestMain {
                        project: project.name.clone(),
                    });
                }
                Some(script_root.join(&project.folder_name).join(&manifest.main))
            }
            ScriptProjectType::KeyMouse => {
                if project.name.trim().is_empty() {
                    return Err(ScriptRuntimeError::MissingRunnableTarget(
                        project.name.clone(),
                    ));
                }
                Some(PathBuf::from(&project.name))
            }
            ScriptProjectType::Pathing => {
                if project.name.trim().is_empty() || project.folder_name.trim().is_empty() {
                    return Err(ScriptRuntimeError::MissingRunnableTarget(
                        project.name.clone(),
                    ));
                }
                Some(PathBuf::from(&project.folder_name).join(&project.name))
            }
            ScriptProjectType::Shell => {
                if project.name.trim().is_empty() {
                    return Err(ScriptRuntimeError::MissingRunnableTarget(
                        "Shell".to_string(),
                    ));
                }
                None
            }
        };

        Ok(Self {
            index: project.index,
            name: project.name.clone(),
            folder_name: project.folder_name.clone(),
            project_type: project.project_type.clone(),
            engine,
            schedule: ScriptSchedule::parse(project.schedule.clone()),
            run_count: project.run_num.max(1) as u32,
            settings: project.js_script_settings_object.clone(),
            allow_notification: project.allow_js_notification.unwrap_or(true),
            allow_http_hash: project.allow_js_http_hash.clone(),
            target_path,
            manifest_main: manifest.map(|manifest| manifest.main.clone()),
            skipped: project.skip_flag.unwrap_or(false),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScriptExecutionPlan {
    pub group_name: Option<String>,
    pub steps: Vec<ScriptExecutionStep>,
    pub cancellation_policy: ScriptCancellationPolicy,
    pub host_bindings: Vec<HostBindingDescriptor>,
}

impl ScriptExecutionPlan {
    pub fn for_group(
        group: &ScriptGroup,
        manifests: &BTreeMap<String, Manifest>,
        script_root: impl AsRef<Path>,
    ) -> Result<Self> {
        let mut projects = group.projects.clone();
        projects.sort_by_key(|project| project.index);

        let steps = projects
            .iter()
            .filter(|project| project.status == ScriptProjectStatus::Enabled)
            .map(|project| {
                ScriptExecutionStep::from_group_project(
                    project,
                    manifests.get(&project.folder_name),
                    script_root.as_ref(),
                )
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            group_name: Some(group.name.clone()),
            steps,
            cancellation_policy: ScriptCancellationPolicy::default(),
            host_bindings: host_bindings(),
        })
    }

    pub fn single_project(
        project: &ScriptGroupProject,
        manifest: Option<&Manifest>,
        script_root: impl AsRef<Path>,
    ) -> Result<Self> {
        Ok(Self {
            group_name: None,
            steps: vec![ScriptExecutionStep::from_group_project(
                project,
                manifest,
                script_root,
            )?],
            cancellation_policy: ScriptCancellationPolicy::default(),
            host_bindings: host_bindings(),
        })
    }

    pub fn runnable_step_count(&self) -> usize {
        self.steps.iter().filter(|step| !step.skipped).count()
    }
}

pub fn engine_for_project_type(project_type: ScriptProjectType) -> ScriptEngineKind {
    match project_type {
        ScriptProjectType::Javascript => ScriptEngineKind::RustJavaScript,
        ScriptProjectType::KeyMouse => ScriptEngineKind::KeyMouseMacro,
        ScriptProjectType::Pathing => ScriptEngineKind::PathingExecutor,
        ScriptProjectType::Shell => ScriptEngineKind::Shell,
    }
}
