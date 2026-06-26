use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct JsonObjectConfig {
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NotificationConfig {
    pub js_notification_enabled: bool,
    pub bark_action: String,
    pub bark_api_endpoint: String,
    pub bark_auto_copy: String,
    pub bark_badge: i64,
    pub bark_call: String,
    pub bark_ciphertext: String,
    pub bark_copy: String,
    pub bark_device_keys: String,
    pub bark_group: String,
    pub bark_icon: String,
    pub bark_is_archive: String,
    pub bark_level: String,
    pub bark_notification_enabled: bool,
    pub bark_sound: String,
    pub bark_subtitle: String,
    pub bark_url: String,
    pub bark_volume: i64,
    pub ding_ding_secret: String,
    pub ding_dingwebhook_notification_enabled: bool,
    pub dingding_webhook_url: String,
    pub email_notification_enabled: bool,
    pub from_email: String,
    pub from_name: String,
    pub include_screen_shot: bool,
    pub notification_event_subscribe: String,
    pub smtp_password: String,
    pub smtp_port: u16,
    pub smtp_server: String,
    pub smtp_username: String,
    pub feishu_notification_enabled: bool,
    pub feishu_webhook_url: String,
    pub feishu_app_id: String,
    pub feishu_app_secret: String,
    pub one_bot_notification_enabled: bool,
    pub one_bot_endpoint: String,
    pub one_bot_user_id: String,
    pub one_bot_group_id: String,
    pub one_bot_token: String,
    pub telegram_api_base_url: String,
    pub telegram_proxy_url: String,
    pub telegram_proxy_enabled: bool,
    pub telegram_bot_token: String,
    pub telegram_chat_id: String,
    pub telegram_notification_enabled: bool,
    pub to_email: String,
    pub webhook_enabled: bool,
    pub webhook_endpoint: String,
    pub webhook_send_to: String,
    pub web_socket_endpoint: String,
    pub web_socket_notification_enabled: bool,
    pub windows_uwp_notification_enabled: bool,
    pub workweixin_notification_enabled: bool,
    pub workweixin_webhook_url: String,
    pub xxtui_api_key: String,
    pub xxtui_channels: String,
    pub xxtui_from: String,
    pub xxtui_notification_enabled: bool,
    pub discord_webhook_notification_enabled: bool,
    pub discord_webhook_url: String,
    pub discord_webhook_username: String,
    pub discord_webhook_avatar_url: String,
    pub discord_webhook_image_encoder: String,
    pub server_chan_notification_enabled: bool,
    pub server_chan_send_key: String,
    pub meow_notification_enabled: bool,
    pub meow_nickname: String,
    pub meow_title: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            js_notification_enabled: false,
            bark_action: String::new(),
            bark_api_endpoint: String::new(),
            bark_auto_copy: String::new(),
            bark_badge: 1,
            bark_call: String::new(),
            bark_ciphertext: String::new(),
            bark_copy: String::new(),
            bark_device_keys: String::new(),
            bark_group: "default".to_string(),
            bark_icon: String::new(),
            bark_is_archive: "1".to_string(),
            bark_level: "active".to_string(),
            bark_notification_enabled: false,
            bark_sound: "minuet".to_string(),
            bark_subtitle: String::new(),
            bark_url: String::new(),
            bark_volume: 5,
            ding_ding_secret: String::new(),
            ding_dingwebhook_notification_enabled: false,
            dingding_webhook_url: String::new(),
            email_notification_enabled: false,
            from_email: String::new(),
            from_name: String::new(),
            include_screen_shot: true,
            notification_event_subscribe: String::new(),
            smtp_password: String::new(),
            smtp_port: 0,
            smtp_server: String::new(),
            smtp_username: String::new(),
            feishu_notification_enabled: false,
            feishu_webhook_url: String::new(),
            feishu_app_id: String::new(),
            feishu_app_secret: String::new(),
            one_bot_notification_enabled: false,
            one_bot_endpoint: String::new(),
            one_bot_user_id: String::new(),
            one_bot_group_id: String::new(),
            one_bot_token: String::new(),
            telegram_api_base_url: String::new(),
            telegram_proxy_url: "http://127.0.0.1:10809".to_string(),
            telegram_proxy_enabled: false,
            telegram_bot_token: String::new(),
            telegram_chat_id: String::new(),
            telegram_notification_enabled: false,
            to_email: String::new(),
            webhook_enabled: false,
            webhook_endpoint: String::new(),
            webhook_send_to: String::new(),
            web_socket_endpoint: String::new(),
            web_socket_notification_enabled: false,
            windows_uwp_notification_enabled: false,
            workweixin_notification_enabled: false,
            workweixin_webhook_url: String::new(),
            xxtui_api_key: String::new(),
            xxtui_channels: "WX_MP".to_string(),
            xxtui_from: "Better原神".to_string(),
            xxtui_notification_enabled: false,
            discord_webhook_notification_enabled: false,
            discord_webhook_url: String::new(),
            discord_webhook_username: "BetterGI·更好的原神".to_string(),
            discord_webhook_avatar_url:
                "https://img.alicdn.com/imgextra/i2/2042484851/O1CN01LQfLIG1lhoEZwz1Gt_!!2042484851.png"
                    .to_string(),
            discord_webhook_image_encoder: "Jpeg".to_string(),
            server_chan_notification_enabled: false,
            server_chan_send_key: String::new(),
            meow_notification_enabled: false,
            meow_nickname: String::new(),
            meow_title: String::new(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OtherConfig {
    pub restore_focus_on_lost_enabled: bool,
    pub auto_fetch_dispatch_adventurers_guild_country: String,
    pub server_time_zone_offset: String,
    pub auto_restart_config: AutoRestartConfig,
    pub farming_plan_config: FarmingPlanConfig,
    pub miyoushe_config: MiyousheConfig,
    pub ocr_config: OcrConfig,
    pub game_culture_info_name: String,
    pub ui_culture_info_name: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for OtherConfig {
    fn default() -> Self {
        Self {
            restore_focus_on_lost_enabled: false,
            auto_fetch_dispatch_adventurers_guild_country: "无".to_string(),
            server_time_zone_offset: "08:00:00".to_string(),
            auto_restart_config: AutoRestartConfig::default(),
            farming_plan_config: FarmingPlanConfig::default(),
            miyoushe_config: MiyousheConfig::default(),
            ocr_config: OcrConfig::default(),
            game_culture_info_name: "zh-Hans".to_string(),
            ui_culture_info_name: "zh-Hans".to_string(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TpConfig {
    pub map_zoom_enabled: bool,
    pub map_zoom_out_distance: u64,
    pub map_zoom_in_distance: u64,
    pub step_interval_milliseconds: u64,
    pub max_zoom_level: f64,
    pub min_zoom_level: f64,
    pub revive_statue_of_the_seven_point_x: f64,
    pub revive_statue_of_the_seven_point_y: f64,
    pub revive_statue_of_the_seven_area: String,
    pub revive_statue_of_the_seven_country: String,
    pub revive_statue_of_the_seven: Option<Value>,
    pub hp_restore_duration: f64,
    pub tolerance: f64,
    pub max_iterations: u64,
    pub max_mouse_move: u64,
    pub map_scale_factor: f64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for TpConfig {
    fn default() -> Self {
        Self {
            map_zoom_enabled: true,
            map_zoom_out_distance: 1000,
            map_zoom_in_distance: 400,
            step_interval_milliseconds: 20,
            max_zoom_level: 5.0,
            min_zoom_level: 2.0,
            revive_statue_of_the_seven_point_x: 2296.4,
            revive_statue_of_the_seven_point_y: -824.4,
            revive_statue_of_the_seven_area: "道成林".to_string(),
            revive_statue_of_the_seven_country: "须弥".to_string(),
            revive_statue_of_the_seven: None,
            hp_restore_duration: 5.0,
            tolerance: 200.0,
            max_iterations: 30,
            max_mouse_move: 300,
            map_scale_factor: 2.361,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoRestartConfig {
    pub enabled: bool,
    pub failure_count: u64,
    pub restart_game_together: bool,
    pub is_fight_failure_exceptional: bool,
    pub is_pathing_failure_exceptional: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoRestartConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            failure_count: 5,
            restart_game_together: false,
            is_fight_failure_exceptional: false,
            is_pathing_failure_exceptional: false,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FarmingPlanConfig {
    pub miyoushe_data_config: MiyousheDataSupportConfig,
    pub enabled: bool,
    pub daily_elite_cap: u64,
    pub daily_mob_cap: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for FarmingPlanConfig {
    fn default() -> Self {
        Self {
            miyoushe_data_config: MiyousheDataSupportConfig::default(),
            enabled: false,
            daily_elite_cap: 400,
            daily_mob_cap: 2000,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MiyousheDataSupportConfig {
    pub enabled: bool,
    pub daily_elite_cap: u64,
    pub daily_mob_cap: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for MiyousheDataSupportConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            daily_elite_cap: 400,
            daily_mob_cap: 2000,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MiyousheConfig {
    pub cookie: String,
    pub log_sync_cookie: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for MiyousheConfig {
    fn default() -> Self {
        Self {
            cookie: String::new(),
            log_sync_cookie: true,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OcrConfig {
    pub paddle_ocr_model_config: Value,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            paddle_ocr_model_config: Value::Number(2.into()),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DevConfig {
    pub record_map_name: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            record_map_name: "Teyvat".to_string(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HardwareAccelerationConfig {
    pub inference_device: Value,
    pub cpu_ocr: bool,
    pub gpu_device: u64,
    pub additional_path: String,
    pub optimized_model: bool,
    pub cuda_device: u64,
    pub auto_append_cuda_path: bool,
    pub enable_tensor_rt_cache: bool,
    pub embed_tensor_rt_cache: bool,
    pub open_vino_device: String,
    pub enable_open_vino_cache: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for HardwareAccelerationConfig {
    fn default() -> Self {
        Self {
            inference_device: Value::Number(0.into()),
            cpu_ocr: true,
            gpu_device: 0,
            additional_path: String::new(),
            optimized_model: false,
            cuda_device: 0,
            auto_append_cuda_path: false,
            enable_tensor_rt_cache: true,
            embed_tensor_rt_cache: true,
            open_vino_device: "AUTO:GPU,CPU".to_string(),
            enable_open_vino_cache: false,
            extra: Map::new(),
        }
    }
}
