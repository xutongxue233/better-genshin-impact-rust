use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HtmlMaskMessage {
    pub url: String,
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HtmlMaskWindowPlan {
    pub window_id: String,
    pub final_url: String,
    pub requested_url: String,
    pub normalized_path: Option<PathBuf>,
    pub click_through: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HtmlMaskCommand {
    Show(HtmlMaskWindowPlan),
    Close {
        window_id: String,
    },
    CloseAll {
        window_ids: Vec<String>,
    },
    SetClickThrough {
        window_id: String,
        enabled: bool,
    },
    Send {
        window_id: String,
        message: HtmlMaskMessage,
    },
    Request {
        window_id: String,
        message: HtmlMaskMessage,
        timeout_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HtmlMaskSnapshot {
    pub windows: Vec<HtmlMaskWindowPlan>,
    pub commands: Vec<HtmlMaskCommand>,
    pub to_html_queue_count: usize,
    pub from_html_queue_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HtmlMaskInitialState {
    pub windows: Vec<HtmlMaskWindowPlan>,
    pub from_html: Vec<(String, HtmlMaskMessage)>,
}
