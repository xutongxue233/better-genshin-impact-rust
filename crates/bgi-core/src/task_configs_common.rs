use super::super::ThemeType;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CommonConfig {
    pub screenshot_enabled: bool,
    pub screenshot_uid_cover_enabled: bool,
    pub reward_recognition_screenshot_enabled: bool,
    pub exit_to_tray: bool,
    pub current_theme_type: ThemeType,
    pub current_backdrop_type: Value,
    pub is_first_run: bool,
    pub run_for_version: String,
    pub once_had_run_device_id_list: Vec<String>,
    pub redeem_code_feeds_update_version: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self {
            screenshot_enabled: false,
            screenshot_uid_cover_enabled: true,
            reward_recognition_screenshot_enabled: false,
            exit_to_tray: false,
            current_theme_type: ThemeType::default(),
            current_backdrop_type: Value::String("Mica".to_string()),
            is_first_run: true,
            run_for_version: String::new(),
            once_had_run_device_id_list: Vec::new(),
            redeem_code_feeds_update_version: "20251013".to_string(),
            extra: Map::new(),
        }
    }
}
