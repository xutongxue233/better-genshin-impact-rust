use super::{invalid_arg_for_method, LimitedFileHost, Result};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

#[path = "script_host_html_mask_helpers.rs"]
mod helpers;
#[path = "script_host_html_mask_model.rs"]
mod model;

use helpers::{
    is_http_url, parse_html_mask_data, serialize_html_mask_message, serialize_html_mask_messages,
};
pub use model::{
    HtmlMaskCommand, HtmlMaskInitialState, HtmlMaskMessage, HtmlMaskSnapshot, HtmlMaskWindowPlan,
};

#[derive(Debug, Clone)]
pub struct HtmlMaskHost {
    file_host: LimitedFileHost,
    windows: HashMap<String, HtmlMaskWindowPlan>,
    opened_windows: Vec<String>,
    to_html: HashMap<String, VecDeque<HtmlMaskMessage>>,
    from_html: HashMap<String, VecDeque<HtmlMaskMessage>>,
    commands: Vec<HtmlMaskCommand>,
    next_window_id: u64,
    next_request_id: u64,
}

impl HtmlMaskHost {
    pub fn new(work_dir: impl Into<PathBuf>) -> Self {
        Self {
            file_host: LimitedFileHost::new(work_dir),
            windows: HashMap::new(),
            opened_windows: Vec::new(),
            to_html: HashMap::new(),
            from_html: HashMap::new(),
            commands: Vec::new(),
            next_window_id: 1,
            next_request_id: 1,
        }
    }

    pub fn with_initial_state(
        work_dir: impl Into<PathBuf>,
        initial_state: HtmlMaskInitialState,
    ) -> Self {
        let mut host = Self::new(work_dir);
        for window in initial_state.windows {
            let window_id = window.window_id.clone();
            host.windows.insert(window_id.clone(), window);
            if !host.opened_windows.iter().any(|id| id == &window_id) {
                host.opened_windows.push(window_id.clone());
            }
            host.to_html.entry(window_id.clone()).or_default();
            host.from_html.entry(window_id).or_default();
        }
        for (window_id, message) in initial_state.from_html {
            if !host.windows.contains_key(&window_id) {
                continue;
            }
            host.from_html
                .entry(window_id)
                .or_default()
                .push_back(message);
        }
        host
    }

    pub fn commands(&self) -> &[HtmlMaskCommand] {
        &self.commands
    }

    pub fn remaining_from_html_messages(&self) -> Vec<(String, HtmlMaskMessage)> {
        let mut messages = self
            .from_html
            .iter()
            .flat_map(|(window_id, queue)| {
                queue
                    .iter()
                    .cloned()
                    .map(|message| (window_id.clone(), message))
            })
            .collect::<Vec<_>>();
        messages.sort_by(|left, right| left.0.cmp(&right.0));
        messages
    }

    pub fn snapshot(&self) -> HtmlMaskSnapshot {
        let mut windows = self.windows.values().cloned().collect::<Vec<_>>();
        windows.sort_by(|left, right| left.window_id.cmp(&right.window_id));
        HtmlMaskSnapshot {
            windows,
            commands: self.commands.clone(),
            to_html_queue_count: self.to_html.values().map(VecDeque::len).sum(),
            from_html_queue_count: self.from_html.values().map(VecDeque::len).sum(),
        }
    }

    pub fn show(&mut self, url: &str, id: Option<&str>) -> Result<HtmlMaskCommand> {
        if url.trim().is_empty() {
            return Err(invalid_arg_for_method("htmlMask.show", 0, "non-empty URL"));
        }

        let (final_url, normalized_path) = if is_http_url(url) {
            (url.to_string(), None)
        } else {
            let normalized = self.file_host.normalize_path(url)?;
            (path_to_file_url(&normalized), Some(normalized))
        };
        let window_id = id
            .filter(|id| !id.trim().is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| self.next_window_id());
        let plan = HtmlMaskWindowPlan {
            window_id: window_id.clone(),
            final_url,
            requested_url: url.to_string(),
            normalized_path,
            click_through: false,
        };

        self.windows.insert(window_id.clone(), plan.clone());
        if !self.opened_windows.iter().any(|id| id == &window_id) {
            self.opened_windows.push(window_id.clone());
        }
        self.to_html.entry(window_id.clone()).or_default();
        self.from_html.entry(window_id).or_default();
        Ok(self.push_html_mask_command(HtmlMaskCommand::Show(plan)))
    }

    pub fn close(&mut self, window_id: &str) -> HtmlMaskCommand {
        self.opened_windows.retain(|id| id != window_id);
        self.windows.remove(window_id);
        self.to_html.remove(window_id);
        self.from_html.remove(window_id);
        self.push_html_mask_command(HtmlMaskCommand::Close {
            window_id: window_id.to_string(),
        })
    }

    pub fn close_all(&mut self) -> HtmlMaskCommand {
        let window_ids = self.opened_windows.clone();
        self.opened_windows.clear();
        self.windows.clear();
        self.to_html.clear();
        self.from_html.clear();
        self.push_html_mask_command(HtmlMaskCommand::CloseAll { window_ids })
    }

    pub fn window_ids(&self) -> Vec<String> {
        let mut ids = self.windows.keys().cloned().collect::<Vec<_>>();
        ids.sort();
        ids
    }

    pub fn exists(&self, window_id: &str) -> bool {
        self.windows.contains_key(window_id)
    }

    pub fn set_click_through(&mut self, window_id: &str, enabled: bool) -> Result<HtmlMaskCommand> {
        let Some(window) = self.windows.get_mut(window_id) else {
            return Err(invalid_arg_for_method(
                "htmlMask.setClickThrough",
                0,
                "existing window id",
            ));
        };
        window.click_through = enabled;
        Ok(
            self.push_html_mask_command(HtmlMaskCommand::SetClickThrough {
                window_id: window_id.to_string(),
                enabled,
            }),
        )
    }

    pub fn get_click_through(&self, window_id: &str) -> Result<bool> {
        self.windows
            .get(window_id)
            .map(|window| window.click_through)
            .ok_or_else(|| {
                invalid_arg_for_method("htmlMask.getClickThrough", 0, "existing window id")
            })
    }

    pub fn toggle_click_through(&mut self, window_id: &str) -> Result<HtmlMaskCommand> {
        let enabled = !self.get_click_through(window_id)?;
        self.set_click_through(window_id, enabled)
    }

    pub fn send(&mut self, window_id: &str, url: &str, json_data: &str) -> Result<HtmlMaskCommand> {
        self.ensure_window(window_id, "htmlMask.send")?;
        let message = HtmlMaskMessage {
            url: url.to_string(),
            data: parse_html_mask_data(json_data)?,
            request_id: None,
        };
        self.to_html
            .entry(window_id.to_string())
            .or_default()
            .push_back(message.clone());
        Ok(self.push_html_mask_command(HtmlMaskCommand::Send {
            window_id: window_id.to_string(),
            message,
        }))
    }

    pub fn request(
        &mut self,
        window_id: &str,
        url: &str,
        json_data: &str,
        timeout_ms: u64,
    ) -> Result<HtmlMaskCommand> {
        self.ensure_window(window_id, "htmlMask.request")?;
        let message = HtmlMaskMessage {
            url: url.to_string(),
            data: parse_html_mask_data(json_data)?,
            request_id: Some(self.next_request_id()),
        };
        self.to_html
            .entry(window_id.to_string())
            .or_default()
            .push_back(message.clone());
        Ok(self.push_html_mask_command(HtmlMaskCommand::Request {
            window_id: window_id.to_string(),
            message,
            timeout_ms,
        }))
    }

    pub fn receive(&mut self, window_id: &str, _timeout_ms: u64) -> Result<Option<String>> {
        self.ensure_window(window_id, "htmlMask.receive")?;
        self.poll(window_id)
    }

    pub fn poll(&mut self, window_id: &str) -> Result<Option<String>> {
        self.ensure_window(window_id, "htmlMask.poll")?;
        let Some(queue) = self.from_html.get_mut(window_id) else {
            return Ok(None);
        };
        queue
            .pop_front()
            .map(|message| serialize_html_mask_message(&message))
            .transpose()
    }

    pub fn poll_all(&mut self, window_id: &str) -> Result<String> {
        self.ensure_window(window_id, "htmlMask.pollAll")?;
        let Some(queue) = self.from_html.get_mut(window_id) else {
            return Ok("[]".to_string());
        };
        let messages = queue.drain(..).collect::<Vec<_>>();
        serialize_html_mask_messages(&messages)
    }

    pub fn flush_pending_messages(&mut self, window_id: &str) -> Result<Vec<String>> {
        self.ensure_window(window_id, "htmlMask.flushPendingMessages")?;
        let Some(queue) = self.to_html.get_mut(window_id) else {
            return Ok(Vec::new());
        };
        queue
            .drain(..)
            .map(|message| serialize_html_mask_message(&message))
            .collect()
    }

    pub fn send_from_html(
        &mut self,
        window_id: &str,
        url: &str,
        json_data: &str,
        request_id: Option<&str>,
    ) -> Result<()> {
        self.ensure_window(window_id, "htmlMask.sendFromHtml")?;
        let message = HtmlMaskMessage {
            url: url.to_string(),
            data: parse_html_mask_data(json_data)?,
            request_id: request_id.map(ToOwned::to_owned),
        };
        self.from_html
            .entry(window_id.to_string())
            .or_default()
            .push_back(message);
        Ok(())
    }

    fn ensure_window(&self, window_id: &str, method: &'static str) -> Result<()> {
        if self.windows.contains_key(window_id) {
            Ok(())
        } else {
            Err(invalid_arg_for_method(method, 0, "existing window id"))
        }
    }

    fn next_window_id(&mut self) -> String {
        let id = format!("html-mask-{}", self.next_window_id);
        self.next_window_id = self.next_window_id.saturating_add(1);
        id
    }

    fn next_request_id(&mut self) -> String {
        let id = format!("request-{}", self.next_request_id);
        self.next_request_id = self.next_request_id.saturating_add(1);
        id
    }

    fn push_html_mask_command(&mut self, command: HtmlMaskCommand) -> HtmlMaskCommand {
        self.commands.push(command.clone());
        command
    }
}

pub(crate) fn path_to_file_url(path: &Path) -> String {
    helpers::path_to_file_url(path)
}
