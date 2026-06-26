use super::{Result, ScriptExecutionStep, ScriptRuntimeError};
use crate::group::ScriptProjectType;
use crate::project::{
    LoadedScriptModule, ModuleLoaderPlan, ScriptCodeExecutionMode, ScriptProject,
    ScriptProjectLayout,
};
use crate::script_host::{
    GlobalInputDispatchMode, HtmlMaskInitialState, HttpDispatchMode, KeyMouseScriptDispatchMode,
    NotificationDispatchMode, ScriptHostRuntimeConfig,
};
use crate::{
    GameCaptureArea, MacroPlaybackContext, ScriptHttpPolicy, ScriptNotificationPolicy,
    ServerTimeHost,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

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
            global_input_dispatch_mode: GlobalInputDispatchMode::PlanOnly,
            key_mouse_dispatch_mode: KeyMouseScriptDispatchMode::PlanOnly,
            macro_playback_context: MacroPlaybackContext::default(),
            cancellation: None,
            http_dispatch_mode: HttpDispatchMode::PlanOnly,
            http_policy: ScriptHttpPolicy::enabled_by_project_hash(
                &project.manifest,
                step.allow_http_hash.as_deref(),
            ),
            notification_policy: ScriptNotificationPolicy::new(true, step.allow_notification),
            notification_dispatch_mode: NotificationDispatchMode::RecordOnly,
            server_time_zone_offset_milliseconds: ServerTimeHost::default()
                .server_time_zone_offset_milliseconds(),
            html_mask_initial_state: HtmlMaskInitialState::default(),
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
