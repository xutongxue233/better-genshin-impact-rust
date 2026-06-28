#[path = "script_host_model.rs"]
mod model;

#[path = "script_host_notifications.rs"]
mod notifications;

#[path = "script_host_args.rs"]
mod args;

#[path = "script_host_pathing.rs"]
mod pathing;

#[path = "script_host_dispatcher.rs"]
mod dispatcher;

#[path = "script_host_http.rs"]
mod http;

#[path = "script_host_file.rs"]
mod file;

#[path = "script_host_global_input.rs"]
mod global_input;

#[path = "script_host_html_mask.rs"]
mod html_mask;

#[path = "script_host_key_mouse_hook.rs"]
mod key_mouse_hook;

#[path = "script_host_key_mouse_script.rs"]
mod key_mouse_script;

#[path = "script_host_server_time.rs"]
mod server_time;

#[path = "script_host_call_network.rs"]
mod call_network;
#[path = "script_host_call_ui.rs"]
mod call_ui;
#[path = "script_host_calls.rs"]
mod calls;
#[path = "script_host_vision.rs"]
mod vision;

use args::*;
pub use dispatcher::*;
pub use file::*;
pub use global_input::*;
pub use html_mask::*;
pub use http::*;
pub use key_mouse_hook::*;
pub use key_mouse_script::*;
pub use model::*;
pub use notifications::*;
pub use pathing::*;
pub use server_time::*;
pub(crate) use vision::image_from_mat_value;
pub use vision::{VisionHost, VisionImageMatExecution, VisionRecognitionExecution};

impl ScriptHostRuntime {
    pub fn new(config: ScriptHostRuntimeConfig) -> Result<Self> {
        let mut global_input = GlobalInputHost::new_with_frame_source(
            config.capture_area,
            config.runtime_dpi,
            config.capture_frame_source,
        )?;
        if let Some(metrics) = config.initial_game_metrics {
            global_input.set_game_metrics(metrics.width, metrics.height, metrics.dpi)?;
        }

        Ok(Self {
            global_input,
            global_input_dispatch_mode: config.global_input_dispatch_mode,
            input_window_handle: config.input_window_handle,
            genshin: GenshinHost::default(),
            pathing_script: PathingScriptHost::new(
                config.script_root.clone(),
                config.user_auto_pathing_root,
                config.pathing_party_config,
            ),
            key_mouse_script: KeyMouseScriptHost::new(
                config.script_root.clone(),
                config.macro_playback_context,
            ),
            key_mouse_dispatch_mode: config.key_mouse_dispatch_mode,
            cancellation: config.cancellation,
            file: LimitedFileHost::new(config.script_root.clone()),
            vision: VisionHost,
            log: ScriptLogHost::default(),
            http: HttpHost::new(config.http_policy),
            http_dispatch_mode: config.http_dispatch_mode,
            dispatcher: ScriptDispatcherHost::default(),
            notification: ScriptNotificationHost::new(config.notification_policy),
            notification_dispatch_mode: config.notification_dispatch_mode,
            strategy_file: StrategyFileHost::new(config.strategy_root),
            server_time: ServerTimeHost::from_offset_milliseconds(
                config.server_time_zone_offset_milliseconds,
            ),
            html_mask: HtmlMaskHost::with_initial_state(
                config.script_root,
                config.html_mask_initial_state,
            ),
            key_mouse_hook: KeyMouseHookHost::default(),
            logical_time_ms: 0,
        })
    }

    pub fn call(&mut self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        self.logical_time_ms = self.logical_time_ms.saturating_add(1);
        self.call_at(call, self.logical_time_ms)
    }

    pub fn call_at(&mut self, call: ScriptHostCall, now_ms: u64) -> Result<ScriptHostCallResult> {
        match call.target {
            ScriptHostTarget::Global => self.call_global(call),
            ScriptHostTarget::Genshin => self.call_genshin(call),
            ScriptHostTarget::PathingScript => self.call_pathing_script(call),
            ScriptHostTarget::KeyMouseScript => self.call_key_mouse_script(call),
            ScriptHostTarget::File => self.call_file(call),
            ScriptHostTarget::Vision => self.call_vision(call),
            ScriptHostTarget::Log => self.call_log(call),
            ScriptHostTarget::Http => self.call_http(call),
            ScriptHostTarget::Dispatcher => self.call_dispatcher(call),
            ScriptHostTarget::Notification => self.call_notification(call, now_ms),
            ScriptHostTarget::PostMessage => self.call_post_message(call),
            ScriptHostTarget::StrategyFile => self.call_strategy_file(call),
            ScriptHostTarget::ServerTime => self.call_server_time(call),
            ScriptHostTarget::HtmlMask => self.call_html_mask(call),
            ScriptHostTarget::KeyMouseHook => self.call_key_mouse_hook(call),
            ScriptHostTarget::CustomHostFunctions => self.call_custom_host_functions(call),
        }
    }

    pub fn log_records(&self) -> &[ScriptLogRecord] {
        self.log.records()
    }

    pub fn notification_records(&self) -> &[ScriptNotificationRecord] {
        self.notification.records()
    }

    pub fn dispatcher_commands(&self) -> &[DispatcherCommand] {
        self.dispatcher.commands()
    }

    pub fn dispatcher_task_invocation_plans(&self) -> Result<Vec<bgi_task::TaskInvocationPlan>> {
        self.dispatcher.task_invocation_plans()
    }

    pub fn genshin_commands(&self) -> &[GenshinCommand] {
        self.genshin.commands()
    }

    pub fn genshin_task_invocation_plans(&self) -> Result<Vec<bgi_task::TaskInvocationPlan>> {
        self.genshin.task_invocation_plans()
    }

    pub fn game_metrics(&self) -> GameMetrics {
        self.global_input.game_metrics()
    }

    pub fn html_mask_remaining_from_html_messages(&self) -> Vec<(String, HtmlMaskMessage)> {
        self.html_mask.remaining_from_html_messages()
    }
}

#[cfg(test)]
#[path = "script_host_tests.rs"]
mod tests;
