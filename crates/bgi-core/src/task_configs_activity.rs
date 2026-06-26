use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PathingConditionConfig {
    pub map_matching_method: String,
    pub party_conditions: Vec<Condition>,
    pub avatar_conditions: Vec<Condition>,
    pub only_in_teleport_recover: bool,
    pub use_gadget_interval_ms: u64,
    pub auto_eat_enabled: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for PathingConditionConfig {
    fn default() -> Self {
        Self {
            map_matching_method: "TemplateMatch".to_string(),
            party_conditions: Vec::new(),
            avatar_conditions: vec![
                Condition::new(
                    "队伍中角色",
                    ["绮良良", "莱依拉", "茜特菈莉", "芭芭拉", "七七"],
                    "循环短E",
                ),
                Condition::new("队伍中角色", ["钟离"], "循环长E"),
                Condition::new("队伍中角色", ["迪希雅"], "作为主要行走角色"),
            ],
            only_in_teleport_recover: false,
            use_gadget_interval_ms: 0,
            auto_eat_enabled: false,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Condition {
    pub subject: String,
    pub object: Vec<String>,
    pub result: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Condition {
    fn new<const N: usize>(subject: &str, object: [&str; N], result: &str) -> Self {
        Self {
            subject: subject.to_string(),
            object: object.into_iter().map(str::to_string).collect(),
            result: result.to_string(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GenshinStartConfig {
    pub auto_enter_game_enabled: bool,
    pub genshin_start_args: String,
    pub install_path: String,
    pub linked_start_enabled: bool,
    pub record_game_time_enabled: bool,
    pub start_game_with_cmd: bool,
    pub auto_disable_genshin_hdr_enabled: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for GenshinStartConfig {
    fn default() -> Self {
        Self {
            auto_enter_game_enabled: true,
            genshin_start_args: String::new(),
            install_path: String::new(),
            linked_start_enabled: true,
            record_game_time_enabled: false,
            start_game_with_cmd: false,
            auto_disable_genshin_hdr_enabled: true,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoPickConfig {
    pub enabled: bool,
    pub item_icon_left_offset: i32,
    pub item_text_left_offset: i32,
    pub item_text_right_offset: i32,
    pub ocr_engine: String,
    pub fast_mode_enabled: bool,
    pub pick_key: String,
    pub black_list_enabled: bool,
    pub white_list_enabled: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoPickConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            item_icon_left_offset: 60,
            item_text_left_offset: 115,
            item_text_right_offset: 400,
            ocr_engine: "Paddle".to_string(),
            fast_mode_enabled: false,
            pick_key: "F".to_string(),
            black_list_enabled: true,
            white_list_enabled: false,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoFishingConfig {
    pub enabled: bool,
    pub auto_throw_rod_enabled: bool,
    pub auto_throw_rod_time_out: u64,
    pub whole_process_timeout_seconds: u64,
    pub fishing_time_policy: Value,
    pub torch_dll_full_path: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoFishingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_throw_rod_enabled: false,
            auto_throw_rod_time_out: 15,
            whole_process_timeout_seconds: 300,
            fishing_time_policy: Value::String("All".to_string()),
            torch_dll_full_path: r"C:\torch\lib\torch_cpu.dll".to_string(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoGeniusInvokationConfig {
    pub strategy_name: String,
    pub sleep_delay: u64,
    pub default_character_card_rects: Vec<RectConfig>,
    pub active_character_card_space: i64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoGeniusInvokationConfig {
    fn default() -> Self {
        Self {
            strategy_name: "1.莫娜砂糖琴".to_string(),
            sleep_delay: 0,
            default_character_card_rects: vec![
                RectConfig::new(667, 632, 165, 282),
                RectConfig::new(877, 632, 165, 282),
                RectConfig::new(1088, 632, 165, 282),
            ],
            active_character_card_space: 41,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RectConfig {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl RectConfig {
    const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoSkipConfig {
    pub enabled: bool,
    pub quickly_skip_conversations_enabled: bool,
    pub after_choose_option_sleep_delay: u64,
    pub auto_wait_dialogue_option_voice_enabled: bool,
    pub dialogue_option_voice_max_wait_seconds: u64,
    pub before_click_confirm_delay: u64,
    pub auto_get_daily_rewards_enabled: bool,
    pub auto_re_explore_enabled: bool,
    pub auto_re_explore_character: String,
    pub click_chat_option: String,
    pub custom_priority_options_enabled: bool,
    pub custom_priority_options: String,
    pub auto_hangout_event_enabled: bool,
    pub auto_hangout_end_choose: String,
    pub auto_hangout_choose_option_sleep_delay: u64,
    pub auto_hangout_press_skip_enabled: bool,
    pub run_background_enabled: bool,
    pub bring_game_to_front_after_background_dialog_enabled: bool,
    pub submit_goods_enabled: bool,
    pub picture_in_picture_enabled: bool,
    pub picture_in_picture_source_type: String,
    pub close_popup_paged_enabled: bool,
    pub skip_built_in_click_options: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoSkipConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            quickly_skip_conversations_enabled: true,
            after_choose_option_sleep_delay: 0,
            auto_wait_dialogue_option_voice_enabled: false,
            dialogue_option_voice_max_wait_seconds: 30,
            before_click_confirm_delay: 0,
            auto_get_daily_rewards_enabled: true,
            auto_re_explore_enabled: true,
            auto_re_explore_character: String::new(),
            click_chat_option: "优先选择第一个选项".to_string(),
            custom_priority_options_enabled: false,
            custom_priority_options: String::new(),
            auto_hangout_event_enabled: false,
            auto_hangout_end_choose: String::new(),
            auto_hangout_choose_option_sleep_delay: 0,
            auto_hangout_press_skip_enabled: true,
            run_background_enabled: false,
            bring_game_to_front_after_background_dialog_enabled: false,
            submit_goods_enabled: true,
            picture_in_picture_enabled: false,
            picture_in_picture_source_type: "CaptureLoop".to_string(),
            close_popup_paged_enabled: true,
            skip_built_in_click_options: false,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoMusicGameConfig {
    pub must_canorus_level: bool,
    pub music_level: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoMusicGameConfig {
    fn default() -> Self {
        Self {
            must_canorus_level: false,
            music_level: String::new(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoCookConfig {
    pub check_interval_ms: u64,
    pub stop_task_when_recover_button_detected: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoCookConfig {
    fn default() -> Self {
        Self {
            check_interval_ms: 10,
            stop_task_when_recover_button_detected: true,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoEatConfig {
    pub enabled: bool,
    pub show_notification: bool,
    pub check_interval: u64,
    pub eat_interval: u64,
    pub test_food_name: Option<String>,
    pub default_atk_boosting_dish_name: Option<String>,
    pub default_adventurers_dish_name: Option<String>,
    pub default_def_boosting_dish_name: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoEatConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            show_notification: true,
            check_interval: 150,
            eat_interval: 1000,
            test_food_name: None,
            default_atk_boosting_dish_name: Some("炸萝卜丸子".to_string()),
            default_adventurers_dish_name: None,
            default_def_boosting_dish_name: None,
            extra: Map::new(),
        }
    }
}
