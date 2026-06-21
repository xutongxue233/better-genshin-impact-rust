use crate::group::{ScriptGroup, ScriptGroupProject, ScriptProjectStatus, ScriptProjectType};
use crate::host::{
    host_binding_count_by_kind, host_bindings, host_member_count, host_permissions,
    HostBindingDescriptor, HostBindingKind, HostPermission,
};
use crate::manifest::Manifest;
use crate::project::{
    LoadedScriptModule, ModuleLoaderPlan, ScriptCodeExecutionMode, ScriptProject,
    ScriptProjectError, ScriptProjectLayout, ScriptProjectLoaderSummary,
};
use crate::script_host::ScriptHostRuntimeConfig;
use crate::settings::{script_settings_summary, ScriptSettingsSummary};
use crate::{
    GameCaptureArea, MacroPlaybackContext, ScriptHttpPolicy, ScriptNotificationPolicy,
    ServerTimeHost,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ScriptRuntimeError {
    #[error("script project is disabled: {0}")]
    ProjectDisabled(String),
    #[error("script project {0} has no runnable target")]
    MissingRunnableTarget(String),
    #[error("javascript project {project} is missing manifest metadata")]
    MissingManifest { project: String },
    #[error("manifest main script is required for project {project}")]
    MissingManifestMain { project: String },
    #[error("script project load failed: {0}")]
    Project(#[from] ScriptProjectError),
    #[error("only javascript execution steps can be prepared for the JS runtime: {0:?}")]
    UnsupportedPreparedProjectType(ScriptProjectType),
}

pub type Result<T> = std::result::Result<T, ScriptRuntimeError>;

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
            notes: "Current C# engine reference; not part of the final Rust runtime.",
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
pub struct PreparedScriptExecution {
    pub step: ScriptExecutionStep,
    pub project_layout: ScriptProjectLayout,
    pub execution_mode: ScriptCodeExecutionMode,
    pub loader_plan: ModuleLoaderPlan,
    pub main_module: LoadedScriptModule,
    pub host_runtime_config: ScriptHostRuntimeConfig,
    pub settings: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptHostExecutionRoots {
    pub strategy_root: PathBuf,
    pub user_auto_pathing_root: PathBuf,
}

impl ScriptHostExecutionRoots {
    pub fn new(
        strategy_root: impl Into<PathBuf>,
        user_auto_pathing_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            strategy_root: strategy_root.into(),
            user_auto_pathing_root: user_auto_pathing_root.into(),
        }
    }

    pub fn from_scripts_root(scripts_root: impl AsRef<Path>) -> Self {
        Self {
            strategy_root: scripts_root.as_ref().join("AutoFight"),
            user_auto_pathing_root: scripts_root.as_ref().join("User").join("AutoPathing"),
        }
    }
}

impl PreparedScriptExecution {
    pub fn prepare_javascript(
        step: &ScriptExecutionStep,
        scripts_root: impl AsRef<Path>,
    ) -> Result<Self> {
        Self::prepare_javascript_with_host_roots(step, scripts_root, None, None)
    }

    pub fn prepare_javascript_with_host_roots(
        step: &ScriptExecutionStep,
        scripts_root: impl AsRef<Path>,
        host_roots: Option<&ScriptHostExecutionRoots>,
        pathing_party_config: Option<Value>,
    ) -> Result<Self> {
        if step.project_type != ScriptProjectType::Javascript {
            return Err(ScriptRuntimeError::UnsupportedPreparedProjectType(
                step.project_type.clone(),
            ));
        }

        let scripts_root = scripts_root.as_ref();
        let default_host_roots;
        let host_roots = match host_roots {
            Some(host_roots) => host_roots,
            None => {
                default_host_roots = ScriptHostExecutionRoots::from_scripts_root(scripts_root);
                &default_host_roots
            }
        };
        let project = ScriptProject::load(scripts_root, &step.folder_name)?;
        let main_code = project.read_main_code()?;
        let execution_mode = project.execution_mode_for_code(&main_code);
        let loader_plan = project.loader_plan_for_code(&main_code);
        let mut loader = project.module_loader()?;
        let main_module = loader.load_js_module(&project.manifest.main, None)?;
        let host_runtime_config = ScriptHostRuntimeConfig {
            script_root: project.layout.project_path.clone(),
            strategy_root: host_roots.strategy_root.clone(),
            user_auto_pathing_root: host_roots.user_auto_pathing_root.clone(),
            pathing_party_config,
            capture_area: GameCaptureArea {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            initial_game_metrics: None,
            capture_frame_source: None,
            runtime_dpi: 1.0,
            input_window_handle: None,
            global_input_dispatch_mode: crate::script_host::GlobalInputDispatchMode::PlanOnly,
            key_mouse_dispatch_mode: crate::script_host::KeyMouseScriptDispatchMode::PlanOnly,
            macro_playback_context: MacroPlaybackContext::default(),
            cancellation: None,
            http_dispatch_mode: crate::script_host::HttpDispatchMode::PlanOnly,
            http_policy: ScriptHttpPolicy::enabled_by_project_hash(
                &project.manifest,
                step.allow_http_hash.as_deref(),
            ),
            notification_policy: ScriptNotificationPolicy::new(true, step.allow_notification),
            notification_dispatch_mode: crate::script_host::NotificationDispatchMode::RecordOnly,
            server_time_zone_offset_milliseconds: ServerTimeHost::default()
                .server_time_zone_offset_milliseconds(),
            html_mask_initial_state: crate::script_host::HtmlMaskInitialState::default(),
        };

        Ok(Self {
            step: step.clone(),
            project_layout: project.layout,
            execution_mode,
            loader_plan,
            main_module,
            host_runtime_config,
            settings: step.settings.clone(),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptRuntimeSummary {
    pub state: ScriptRuntimeState,
    pub engines: Vec<ScriptEngineDescriptor>,
    pub host_binding_count: usize,
    pub host_member_count: usize,
    pub host_object_count: usize,
    pub host_type_count: usize,
    pub permissions: Vec<HostPermission>,
    pub default_cancellation_policy: ScriptCancellationPolicy,
    pub supported_project_types: Vec<ScriptProjectType>,
    pub schedule_kinds: Vec<ScriptScheduleKind>,
    pub project_loader: ScriptProjectLoaderSummary,
    pub settings: ScriptSettingsSummary,
}

pub fn script_runtime_summary() -> ScriptRuntimeSummary {
    let bindings = host_bindings();
    ScriptRuntimeSummary {
        state: ScriptRuntimeState::Stopped,
        engines: script_engines(),
        host_binding_count: bindings.len(),
        host_member_count: host_member_count(&bindings),
        host_object_count: host_binding_count_by_kind(&bindings, HostBindingKind::Object),
        host_type_count: host_binding_count_by_kind(&bindings, HostBindingKind::Type),
        permissions: host_permissions(&bindings),
        default_cancellation_policy: ScriptCancellationPolicy::default(),
        supported_project_types: vec![
            ScriptProjectType::Javascript,
            ScriptProjectType::KeyMouse,
            ScriptProjectType::Pathing,
            ScriptProjectType::Shell,
        ],
        schedule_kinds: vec![
            ScriptScheduleKind::Daily,
            ScriptScheduleKind::EveryTwoDays,
            ScriptScheduleKind::Weekday,
            ScriptScheduleKind::Cron,
            ScriptScheduleKind::Manual,
        ],
        project_loader: ScriptProjectLoaderSummary::default(),
        settings: script_settings_summary(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_codes_preserve_legacy_meaning() {
        assert_eq!(
            ScriptSchedule::parse("Daily").kind,
            ScriptScheduleKind::Daily
        );
        assert_eq!(
            ScriptSchedule::parse("Friday").kind,
            ScriptScheduleKind::Weekday
        );
        assert_eq!(
            ScriptSchedule::parse("0 0 * * *").kind,
            ScriptScheduleKind::Cron
        );
    }

    #[test]
    fn group_plan_sorts_steps_by_index() {
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![
                ScriptGroupProject {
                    index: 2,
                    name: "second.json".to_string(),
                    folder_name: "routes".to_string(),
                    project_type: ScriptProjectType::Pathing,
                    ..ScriptGroupProject::default()
                },
                ScriptGroupProject {
                    index: 1,
                    name: "first.json".to_string(),
                    folder_name: "routes".to_string(),
                    project_type: ScriptProjectType::Pathing,
                    ..ScriptGroupProject::default()
                },
            ],
            ..ScriptGroup::default()
        };

        let plan = ScriptExecutionPlan::for_group(&group, &BTreeMap::new(), ".").unwrap();

        assert_eq!(plan.steps[0].name, "first.json");
        assert_eq!(plan.steps[1].name, "second.json");
    }

    #[test]
    fn javascript_step_requires_manifest() {
        let project = ScriptGroupProject {
            name: "sample".to_string(),
            folder_name: "sample".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        };

        let err = ScriptExecutionPlan::single_project(&project, None, ".").unwrap_err();

        assert!(matches!(
            err,
            ScriptRuntimeError::MissingManifest { project } if project == "sample"
        ));
    }

    #[test]
    fn runtime_summary_counts_host_surface() {
        let summary = script_runtime_summary();

        assert!(summary.host_binding_count >= 10);
        assert!(summary.host_member_count > summary.host_binding_count);
        assert!(summary
            .supported_project_types
            .contains(&ScriptProjectType::Javascript));
    }

    #[test]
    fn prepared_javascript_execution_loads_main_module_and_host_context() {
        let root = test_root("bgi-prepared-js");
        let project_dir = root.join("sample");
        std::fs::create_dir_all(project_dir.join("packages")).unwrap();
        std::fs::write(
            project_dir.join("manifest.json"),
            r#"{
                "manifestVersion": 1,
                "name": "sample",
                "version": "1.0.0",
                "main": "main.js",
                "httpAllowedUrls": ["https://example.com/*"]
            }"#,
        )
        .unwrap();
        std::fs::write(
            project_dir.join("main.js"),
            "import text from './notes.txt'\nexport default text;",
        )
        .unwrap();
        std::fs::write(project_dir.join("notes.txt"), "hello").unwrap();

        let manifest = Manifest::read_from(project_dir.join("manifest.json")).unwrap();
        let project = ScriptGroupProject {
            name: "sample".to_string(),
            folder_name: "sample".to_string(),
            project_type: ScriptProjectType::Javascript,
            js_script_settings_object: Some(serde_json::json!({"level": 2})),
            allow_js_notification: Some(false),
            allow_js_http_hash: Some("https://example.com/*".to_string()),
            ..ScriptGroupProject::default()
        };
        let step =
            ScriptExecutionStep::from_group_project(&project, Some(&manifest), &root).unwrap();

        let prepared = PreparedScriptExecution::prepare_javascript(&step, &root).unwrap();

        assert_eq!(
            prepared.execution_mode,
            ScriptCodeExecutionMode::StandardModule
        );
        assert_eq!(prepared.main_module.import_rewrites.len(), 1);
        assert!(prepared
            .main_module
            .code
            .contains("file.ReadTextSync('notes.txt')"));
        assert_eq!(prepared.settings, Some(serde_json::json!({"level": 2})));
        assert_eq!(prepared.host_runtime_config.script_root, project_dir);
        assert!(
            !prepared
                .host_runtime_config
                .notification_policy
                .project_enabled
        );
        assert!(prepared
            .host_runtime_config
            .http_policy
            .check_url("https://example.com/api")
            .is_ok());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn prepared_javascript_execution_rejects_non_js_steps() {
        let step = ScriptExecutionStep {
            index: 0,
            name: "macro.json".to_string(),
            folder_name: "macro.json".to_string(),
            project_type: ScriptProjectType::KeyMouse,
            engine: ScriptEngineKind::KeyMouseMacro,
            schedule: ScriptSchedule::parse("Daily"),
            run_count: 1,
            settings: None,
            allow_notification: true,
            allow_http_hash: None,
            target_path: None,
            manifest_main: None,
            skipped: false,
        };

        assert!(matches!(
            PreparedScriptExecution::prepare_javascript(&step, ".").unwrap_err(),
            ScriptRuntimeError::UnsupportedPreparedProjectType(ScriptProjectType::KeyMouse)
        ));
    }

    fn test_root(prefix: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "{prefix}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        root
    }
}
