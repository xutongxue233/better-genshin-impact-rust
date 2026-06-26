use super::super::{invalid_arg_for_method, Result};
use super::model::HtmlMaskMessage;
use serde_json::Value;
use std::path::Path;

pub(super) fn parse_html_mask_data(json_data: &str) -> Result<Option<Value>> {
    if json_data.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(
        serde_json::from_str(json_data).unwrap_or_else(|_| Value::String(json_data.to_string())),
    ))
}

pub(super) fn serialize_html_mask_message(message: &HtmlMaskMessage) -> Result<String> {
    serde_json::to_string(message)
        .map_err(|_| invalid_arg_for_method("htmlMask.message", 0, "serializable message"))
}

pub(super) fn serialize_html_mask_messages(messages: &[HtmlMaskMessage]) -> Result<String> {
    serde_json::to_string(messages)
        .map_err(|_| invalid_arg_for_method("htmlMask.messages", 0, "serializable messages"))
}

pub(super) fn is_http_url(url: &str) -> bool {
    url.get(..4)
        .map(|prefix| prefix.eq_ignore_ascii_case("http"))
        .unwrap_or(false)
}

pub(super) fn path_to_file_url(path: &Path) -> String {
    let mut path = path.to_string_lossy().replace('\\', "/");
    if !path.starts_with('/') {
        path = format!("/{path}");
    }
    format!("file://{path}")
}
