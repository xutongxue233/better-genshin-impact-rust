use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QuickTeleportConfig {
    pub enabled: bool,
    pub teleport_list_click_delay: u64,
    pub wait_teleport_panel_delay: u64,
    pub hotkey_tp_enabled: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for QuickTeleportConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            teleport_list_click_delay: 200,
            wait_teleport_panel_delay: 50,
            hotkey_tp_enabled: false,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MapMaskConfig {
    pub enabled: bool,
    pub mini_map_mask_enabled: bool,
    pub path_auto_record_enabled: bool,
    pub map_point_api_provider: String,
    pub ho_yo_lab_language: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for MapMaskConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mini_map_mask_enabled: false,
            path_auto_record_enabled: false,
            map_point_api_provider: "MihoyoMap".to_string(),
            ho_yo_lab_language: "en-us".to_string(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SkillCdConfig {
    pub enabled: bool,
    pub custom_cd_list: Vec<Value>,
    pub trigger_on_skill_use: bool,
    pub hide_when_zero: bool,
    pub p_x: f64,
    pub p_y: f64,
    pub gap: f64,
    pub scale: f64,
    pub background_normal_color: String,
    pub text_normal_color: String,
    pub background_ready_color: String,
    pub text_ready_color: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for SkillCdConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            custom_cd_list: Vec::new(),
            trigger_on_skill_use: false,
            hide_when_zero: false,
            p_x: 1520.0,
            p_y: 245.0,
            gap: 91.2,
            scale: 1.0,
            background_normal_color: "#FFFFFFFF".to_string(),
            text_normal_color: "#DA4A23FF".to_string(),
            background_ready_color: "#FFFFFFFF".to_string(),
            text_ready_color: "#5DCC17FF".to_string(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoRedeemCodeConfig {
    pub clipboard_listener_enabled: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoRedeemCodeConfig {
    fn default() -> Self {
        Self {
            clipboard_listener_enabled: true,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GetGridIconsConfig {
    pub grid_name: Value,
    pub star_as_suffix: bool,
    pub lv_as_suffix: bool,
    pub max_num_to_get: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for GetGridIconsConfig {
    fn default() -> Self {
        Self {
            grid_name: Value::String("Weapons".to_string()),
            star_as_suffix: false,
            lv_as_suffix: false,
            max_num_to_get: i32::MAX as u64,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MacroConfig {
    pub enhance_wait_delay: u64,
    pub f_fire_interval: u64,
    pub f_press_hold_to_continuation_enabled: bool,
    pub runaround_interval: u64,
    pub runaround_mouse_x_interval: i64,
    pub space_fire_interval: u64,
    pub space_press_hold_to_continuation_enabled: bool,
    pub combat_macro_enabled: bool,
    pub combat_macro_hotkey_mode: String,
    pub combat_macro_priority: i64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for MacroConfig {
    fn default() -> Self {
        Self {
            enhance_wait_delay: 0,
            f_fire_interval: 100,
            f_press_hold_to_continuation_enabled: false,
            runaround_interval: 10,
            runaround_mouse_x_interval: 500,
            space_fire_interval: 100,
            space_press_hold_to_continuation_enabled: false,
            combat_macro_enabled: false,
            combat_macro_hotkey_mode: "按住时重复(新)".to_string(),
            combat_macro_priority: 1,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RecordConfig {
    pub angle2_mouse_move_by_x: f64,
    pub angle2_direct_input_x: f64,
    pub is_record_camera_orientation: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for RecordConfig {
    fn default() -> Self {
        Self {
            angle2_mouse_move_by_x: 1.0,
            angle2_direct_input_x: 1.0,
            is_record_camera_orientation: false,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptConfig {
    pub auto_update_script_repo_period: u64,
    pub last_update_script_repo_time: String,
    pub script_repo_hint_dot_visible: bool,
    pub subscribed_script_paths: Vec<String>,
    pub selected_channel_name: String,
    pub custom_repo_url: String,
    pub webview_width: f64,
    pub webview_height: f64,
    pub webview_left: f64,
    pub webview_top: f64,
    pub webview_state: Value,
    pub guide_status: bool,
    pub auto_update_subscribed_scripts: bool,
    pub auto_update_before_command_line_run: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for ScriptConfig {
    fn default() -> Self {
        Self {
            auto_update_script_repo_period: 1,
            last_update_script_repo_time: "0001-01-01T00:00:00".to_string(),
            script_repo_hint_dot_visible: false,
            subscribed_script_paths: Vec::new(),
            selected_channel_name: String::new(),
            custom_repo_url: String::new(),
            webview_width: 0.0,
            webview_height: 0.0,
            webview_left: 0.0,
            webview_top: 0.0,
            webview_state: Value::String("Normal".to_string()),
            guide_status: false,
            auto_update_subscribed_scripts: false,
            auto_update_before_command_line_run: false,
            extra: Map::new(),
        }
    }
}
