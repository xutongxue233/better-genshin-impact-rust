use super::{Result, ScriptHostRuntimeError};
use crate::policy::ScriptFilePolicy;
use crate::r#macro::{KeyMouseScript, MacroPlaybackContext};
use bgi_input::{
    send_events, send_events_to_window, send_events_to_window_with_cancellation,
    send_events_with_cancellation, InputCancellationToken, InputEvent, InputSequence,
};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyMouseScriptRunPlan {
    pub source: KeyMouseScriptSource,
    pub normalized_path: Option<PathBuf>,
    pub summary: crate::KeyMouseMacroSummary,
    pub input_events: Vec<InputEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum KeyMouseScriptDispatchMode {
    PlanOnly,
    SendInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyMouseScriptExecution {
    pub mode: KeyMouseScriptDispatchMode,
    pub plan: KeyMouseScriptRunPlan,
    pub dispatched: bool,
    pub dispatched_events: usize,
    pub processed_events: usize,
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum KeyMouseScriptSource {
    InlineJson,
    File,
}

impl KeyMouseScriptRunPlan {
    pub fn sequence(&self) -> InputSequence {
        self.input_events
            .iter()
            .copied()
            .fold(InputSequence::new(), append_input_event)
    }

    pub fn send(&self) -> Result<()> {
        Ok(send_events(&self.input_events)?)
    }

    pub fn send_to_window(&self, hwnd: isize) -> Result<()> {
        Ok(send_events_to_window(hwnd, &self.input_events)?)
    }

    pub fn send_with_cancellation(
        &self,
        cancellation: &InputCancellationToken,
    ) -> Result<(usize, bool)> {
        match send_events_with_cancellation(&self.input_events, cancellation) {
            Ok(report) => Ok((report.dispatched_events, report.cancelled)),
            Err(bgi_input::InputError::Cancelled {
                dispatched_events, ..
            }) => Ok((dispatched_events, true)),
            Err(error) => Err(error.into()),
        }
    }

    pub fn send_to_window_with_cancellation(
        &self,
        hwnd: isize,
        cancellation: &InputCancellationToken,
    ) -> Result<(usize, bool)> {
        match send_events_to_window_with_cancellation(hwnd, &self.input_events, cancellation) {
            Ok(report) => Ok((report.dispatched_events, report.cancelled)),
            Err(bgi_input::InputError::Cancelled {
                dispatched_events, ..
            }) => Ok((dispatched_events, true)),
            Err(error) => Err(error.into()),
        }
    }

    pub fn execute(
        &self,
        mode: KeyMouseScriptDispatchMode,
        window_handle: Option<isize>,
    ) -> Result<KeyMouseScriptExecution> {
        self.execute_with_cancellation(mode, window_handle, None)
    }

    pub fn execute_with_cancellation(
        &self,
        mode: KeyMouseScriptDispatchMode,
        window_handle: Option<isize>,
        cancellation: Option<&InputCancellationToken>,
    ) -> Result<KeyMouseScriptExecution> {
        let mut dispatched_events = 0;
        let mut cancelled = false;
        if mode == KeyMouseScriptDispatchMode::SendInput {
            let result = match (window_handle, cancellation) {
                (Some(hwnd), Some(cancellation)) => {
                    self.send_to_window_with_cancellation(hwnd, cancellation)?
                }
                (None, Some(cancellation)) => self.send_with_cancellation(cancellation)?,
                (Some(hwnd), None) => {
                    self.send_to_window(hwnd)?;
                    (self.input_events.len(), false)
                }
                (None, None) => {
                    self.send()?;
                    (self.input_events.len(), false)
                }
            };
            dispatched_events = result.0;
            cancelled = result.1;
        }

        Ok(KeyMouseScriptExecution {
            mode,
            plan: self.clone(),
            dispatched: mode == KeyMouseScriptDispatchMode::SendInput,
            dispatched_events,
            processed_events: if mode == KeyMouseScriptDispatchMode::SendInput {
                dispatched_events
            } else {
                0
            },
            cancelled,
        })
    }
}

#[derive(Debug, Clone)]
pub struct KeyMouseScriptHost {
    file_policy: ScriptFilePolicy,
    playback_context: MacroPlaybackContext,
}

impl KeyMouseScriptHost {
    pub fn new(script_root: impl Into<PathBuf>, playback_context: MacroPlaybackContext) -> Self {
        Self {
            file_policy: ScriptFilePolicy::new(script_root),
            playback_context,
        }
    }

    pub fn with_policy(
        file_policy: ScriptFilePolicy,
        playback_context: MacroPlaybackContext,
    ) -> Self {
        Self {
            file_policy,
            playback_context,
        }
    }

    pub fn file_policy(&self) -> &ScriptFilePolicy {
        &self.file_policy
    }

    pub fn playback_context(&self) -> MacroPlaybackContext {
        self.playback_context
    }

    pub fn run(&self, json: &str) -> Result<KeyMouseScriptRunPlan> {
        let script = KeyMouseScript::from_json(json)?;
        self.plan(script, KeyMouseScriptSource::InlineJson, None)
    }

    pub fn execute(
        &self,
        json: &str,
        mode: KeyMouseScriptDispatchMode,
        window_handle: Option<isize>,
    ) -> Result<KeyMouseScriptExecution> {
        self.run(json)?.execute(mode, window_handle)
    }

    pub fn execute_with_cancellation(
        &self,
        json: &str,
        mode: KeyMouseScriptDispatchMode,
        window_handle: Option<isize>,
        cancellation: Option<&InputCancellationToken>,
    ) -> Result<KeyMouseScriptExecution> {
        self.run(json)?
            .execute_with_cancellation(mode, window_handle, cancellation)
    }

    pub fn run_file(&self, path: &str) -> Result<KeyMouseScriptRunPlan> {
        let normalized = self.file_policy.normalize_path(path)?;
        self.file_policy.validate_write_extension(&normalized)?;
        let json = read_text(&normalized)?;
        let script = KeyMouseScript::from_json(&json)?;
        self.plan(script, KeyMouseScriptSource::File, Some(normalized))
    }

    pub fn execute_file(
        &self,
        path: &str,
        mode: KeyMouseScriptDispatchMode,
        window_handle: Option<isize>,
    ) -> Result<KeyMouseScriptExecution> {
        self.run_file(path)?.execute(mode, window_handle)
    }

    pub fn execute_file_with_cancellation(
        &self,
        path: &str,
        mode: KeyMouseScriptDispatchMode,
        window_handle: Option<isize>,
        cancellation: Option<&InputCancellationToken>,
    ) -> Result<KeyMouseScriptExecution> {
        self.run_file(path)?
            .execute_with_cancellation(mode, window_handle, cancellation)
    }

    pub fn run_path(&self, path: impl AsRef<Path>) -> Result<KeyMouseScriptRunPlan> {
        let path_string = path.as_ref().to_string_lossy();
        self.run_file(&path_string)
    }

    fn plan(
        &self,
        script: KeyMouseScript,
        source: KeyMouseScriptSource,
        normalized_path: Option<PathBuf>,
    ) -> Result<KeyMouseScriptRunPlan> {
        let input_events = script.to_input_events(self.playback_context)?;
        Ok(KeyMouseScriptRunPlan {
            source,
            normalized_path,
            summary: script.summary(),
            input_events,
        })
    }
}

fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|source| ScriptHostRuntimeError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn append_input_event(sequence: InputSequence, event: InputEvent) -> InputSequence {
    match event {
        InputEvent::KeyDown { vk, .. } => sequence.key_down(vk),
        InputEvent::KeyUp { vk, .. } => sequence.key_up(vk),
        InputEvent::UnicodeChar { ch } => sequence.text(&ch.to_string()),
        InputEvent::MouseMoveRelative { dx, dy } => sequence.move_mouse_by(dx, dy),
        InputEvent::MouseMoveAbsolute {
            x,
            y,
            virtual_desktop,
        } => {
            if virtual_desktop {
                sequence.move_mouse_to_virtual_desktop(x, y)
            } else {
                sequence.move_mouse_to(x, y)
            }
        }
        InputEvent::MouseButtonDown { button } => sequence.mouse_down(button),
        InputEvent::MouseButtonUp { button } => sequence.mouse_up(button),
        InputEvent::MouseWheel { amount, horizontal } => {
            let clicks = amount / 120;
            if horizontal {
                sequence.horizontal_scroll(clicks)
            } else {
                sequence.vertical_scroll(clicks)
            }
        }
        InputEvent::Delay { milliseconds } => sequence.delay(milliseconds),
    }
}
