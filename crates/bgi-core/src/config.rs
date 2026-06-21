use crate::error::{BgiError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_CONFIG_RELATIVE_PATH: &str = "User/config.json";

pub fn config_path(base_dir: impl AsRef<Path>) -> PathBuf {
    base_dir.as_ref().join(DEFAULT_CONFIG_RELATIVE_PATH)
}

pub fn read_config(path: impl AsRef<Path>) -> Result<AppConfig> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(|source| BgiError::io(path, source))?;
    json5::from_str(&text).map_err(|err| BgiError::json(Some(path), err.to_string()))
}

pub fn write_config(path: impl AsRef<Path>, config: &AppConfig) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| BgiError::io(parent, source))?;
    }

    let text = serde_json::to_string_pretty(config)
        .map_err(|err| BgiError::json(Some(path), err.to_string()))?;
    fs::write(path, text).map_err(|source| BgiError::io(path, source))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureMode {
    BitBlt,
    DwmGetDxSharedSurface,
    WindowsGraphicsCapture,
    WindowsGraphicsCaptureHdr,
}

impl Default for CaptureMode {
    fn default() -> Self {
        Self::BitBlt
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeType {
    DarkNone,
    DarkMica,
    DarkAcrylic,
    LightNone,
    LightMica,
    LightAcrylic,
}

impl Default for ThemeType {
    fn default() -> Self {
        Self::DarkNone
    }
}

impl ThemeType {
    fn from_i64(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::DarkNone),
            1 => Some(Self::DarkMica),
            2 => Some(Self::DarkAcrylic),
            3 => Some(Self::LightNone),
            4 => Some(Self::LightMica),
            5 => Some(Self::LightAcrylic),
            _ => None,
        }
    }

    fn as_i64(self) -> i64 {
        match self {
            Self::DarkNone => 0,
            Self::DarkMica => 1,
            Self::DarkAcrylic => 2,
            Self::LightNone => 3,
            Self::LightMica => 4,
            Self::LightAcrylic => 5,
        }
    }
}

impl Serialize for ThemeType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.as_i64())
    }
}

impl<'de> Deserialize<'de> for ThemeType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::Number(number) => {
                let number = number
                    .as_i64()
                    .ok_or_else(|| serde::de::Error::custom("theme type must be an integer"))?;
                Self::from_i64(number)
                    .ok_or_else(|| serde::de::Error::custom(format!("unknown theme type {number}")))
            }
            Value::String(name) => match name.as_str() {
                "DarkNone" => Ok(Self::DarkNone),
                "DarkMica" => Ok(Self::DarkMica),
                "DarkAcrylic" => Ok(Self::DarkAcrylic),
                "LightNone" => Ok(Self::LightNone),
                "LightMica" => Ok(Self::LightMica),
                "LightAcrylic" => Ok(Self::LightAcrylic),
                _ => Err(serde::de::Error::custom(format!(
                    "unknown theme type {name}"
                ))),
            },
            other => Err(serde::de::Error::custom(format!(
                "theme type must be a number or string, got {other:?}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub capture_mode: CaptureMode,
    pub detailed_error_logs: bool,
    pub not_show_new_version_notice_end_version: String,
    pub trigger_interval: u64,
    pub auto_fix_win11_bit_blt: bool,
    pub next_scheduled_task: Vec<Value>,
    pub selected_one_dragon_flow_config_name: String,
    pub mask_window_config: MaskWindowConfig,
    pub common_config: CommonConfig,
    pub genshin_start_config: GenshinStartConfig,
    pub auto_pick_config: AutoPickConfig,
    pub auto_skip_config: AutoSkipConfig,
    pub auto_fishing_config: AutoFishingConfig,
    pub quick_teleport_config: QuickTeleportConfig,
    pub auto_genius_invokation_config: AutoGeniusInvokationConfig,
    pub auto_wood_config: AutoWoodConfig,
    pub auto_fight_config: AutoFightConfig,
    pub auto_music_game_config: AutoMusicGameConfig,
    pub auto_domain_config: AutoDomainConfig,
    pub auto_boss_config: AutoBossConfig,
    pub auto_stygian_onslaught_config: AutoStygianOnslaughtConfig,
    pub auto_artifact_salvage_config: AutoArtifactSalvageConfig,
    pub auto_eat_config: AutoEatConfig,
    pub auto_ley_line_outcrop_config: AutoLeyLineOutcropConfig,
    pub auto_cook_config: AutoCookConfig,
    pub map_mask_config: MapMaskConfig,
    pub skill_cd_config: SkillCdConfig,
    pub auto_redeem_code_config: AutoRedeemCodeConfig,
    pub get_grid_icons_config: GetGridIconsConfig,
    pub macro_config: MacroConfig,
    pub record_config: RecordConfig,
    pub script_config: ScriptConfig,
    pub pathing_condition_config: PathingConditionConfig,
    pub hot_key_config: HotKeyConfig,
    pub notification_config: NotificationConfig,
    pub key_bindings_config: KeyBindingsConfig,
    pub other_config: OtherConfig,
    pub tp_config: TpConfig,
    pub dev_config: DevConfig,
    pub hardware_acceleration_config: HardwareAccelerationConfig,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            capture_mode: CaptureMode::default(),
            detailed_error_logs: false,
            not_show_new_version_notice_end_version: String::new(),
            trigger_interval: 50,
            auto_fix_win11_bit_blt: true,
            next_scheduled_task: Vec::new(),
            selected_one_dragon_flow_config_name: String::new(),
            mask_window_config: MaskWindowConfig::default(),
            common_config: CommonConfig::default(),
            genshin_start_config: GenshinStartConfig::default(),
            auto_pick_config: AutoPickConfig::default(),
            auto_skip_config: AutoSkipConfig::default(),
            auto_fishing_config: AutoFishingConfig::default(),
            quick_teleport_config: QuickTeleportConfig::default(),
            auto_genius_invokation_config: AutoGeniusInvokationConfig::default(),
            auto_wood_config: AutoWoodConfig::default(),
            auto_fight_config: AutoFightConfig::default(),
            auto_music_game_config: AutoMusicGameConfig::default(),
            auto_domain_config: AutoDomainConfig::default(),
            auto_boss_config: AutoBossConfig::default(),
            auto_stygian_onslaught_config: AutoStygianOnslaughtConfig::default(),
            auto_artifact_salvage_config: AutoArtifactSalvageConfig::default(),
            auto_eat_config: AutoEatConfig::default(),
            auto_ley_line_outcrop_config: AutoLeyLineOutcropConfig::default(),
            auto_cook_config: AutoCookConfig::default(),
            map_mask_config: MapMaskConfig::default(),
            skill_cd_config: SkillCdConfig::default(),
            auto_redeem_code_config: AutoRedeemCodeConfig::default(),
            get_grid_icons_config: GetGridIconsConfig::default(),
            macro_config: MacroConfig::default(),
            record_config: RecordConfig::default(),
            script_config: ScriptConfig::default(),
            pathing_condition_config: PathingConditionConfig::default(),
            hot_key_config: HotKeyConfig::default(),
            notification_config: NotificationConfig::default(),
            key_bindings_config: KeyBindingsConfig::default(),
            other_config: OtherConfig::default(),
            tp_config: TpConfig::default(),
            dev_config: DevConfig::default(),
            hardware_acceleration_config: HardwareAccelerationConfig::default(),
            extra: Map::new(),
        }
    }
}

impl AppConfig {
    pub fn coverage(&self) -> ConfigCoverage {
        ConfigCoverage {
            modeled_top_level_fields: CONFIG_TOP_LEVEL_FIELDS.len(),
            modeled_config_sections: CONFIG_SECTION_NAMES.len(),
            strongly_typed_sections: CONFIG_STRONGLY_TYPED_SECTION_NAMES.len(),
            compatibility_sections: CONFIG_SECTION_NAMES.len()
                - CONFIG_STRONGLY_TYPED_SECTION_NAMES.len(),
            preserved_unknown_top_level_fields: self.extra.len(),
            config_sections: CONFIG_SECTION_NAMES,
            strongly_typed_config_sections: CONFIG_STRONGLY_TYPED_SECTION_NAMES,
            compatibility_config_sections: CONFIG_COMPATIBILITY_SECTION_NAMES,
            notes: "Compatibility sections preserve their JSON object contents while their concrete task models are ported.",
        }
    }
}

pub const CONFIG_TOP_LEVEL_FIELDS: &[&str] = &[
    "captureMode",
    "detailedErrorLogs",
    "notShowNewVersionNoticeEndVersion",
    "triggerInterval",
    "autoFixWin11BitBlt",
    "nextScheduledTask",
    "selectedOneDragonFlowConfigName",
];

pub const CONFIG_SECTION_NAMES: &[&str] = &[
    "maskWindowConfig",
    "commonConfig",
    "genshinStartConfig",
    "autoPickConfig",
    "autoSkipConfig",
    "autoFishingConfig",
    "quickTeleportConfig",
    "autoGeniusInvokationConfig",
    "autoWoodConfig",
    "autoFightConfig",
    "autoMusicGameConfig",
    "autoDomainConfig",
    "autoBossConfig",
    "autoStygianOnslaughtConfig",
    "autoArtifactSalvageConfig",
    "autoEatConfig",
    "autoLeyLineOutcropConfig",
    "autoCookConfig",
    "mapMaskConfig",
    "skillCdConfig",
    "autoRedeemCodeConfig",
    "getGridIconsConfig",
    "macroConfig",
    "recordConfig",
    "scriptConfig",
    "pathingConditionConfig",
    "hotKeyConfig",
    "notificationConfig",
    "keyBindingsConfig",
    "otherConfig",
    "tpConfig",
    "devConfig",
    "hardwareAccelerationConfig",
];

pub const CONFIG_STRONGLY_TYPED_SECTION_NAMES: &[&str] = &[
    "maskWindowConfig",
    "commonConfig",
    "genshinStartConfig",
    "autoPickConfig",
    "autoSkipConfig",
    "autoFishingConfig",
    "quickTeleportConfig",
    "autoGeniusInvokationConfig",
    "autoWoodConfig",
    "autoFightConfig",
    "autoMusicGameConfig",
    "autoDomainConfig",
    "autoBossConfig",
    "autoStygianOnslaughtConfig",
    "autoArtifactSalvageConfig",
    "autoEatConfig",
    "autoLeyLineOutcropConfig",
    "autoCookConfig",
    "mapMaskConfig",
    "skillCdConfig",
    "autoRedeemCodeConfig",
    "getGridIconsConfig",
    "macroConfig",
    "recordConfig",
    "scriptConfig",
    "pathingConditionConfig",
    "hotKeyConfig",
    "notificationConfig",
    "keyBindingsConfig",
    "otherConfig",
    "tpConfig",
    "devConfig",
    "hardwareAccelerationConfig",
];

pub const CONFIG_COMPATIBILITY_SECTION_NAMES: &[&str] = &[];

#[derive(Debug, Clone, Serialize)]
pub struct ConfigCoverage {
    pub modeled_top_level_fields: usize,
    pub modeled_config_sections: usize,
    pub strongly_typed_sections: usize,
    pub compatibility_sections: usize,
    pub preserved_unknown_top_level_fields: usize,
    pub config_sections: &'static [&'static str],
    pub strongly_typed_config_sections: &'static [&'static str],
    pub compatibility_config_sections: &'static [&'static str],
    pub notes: &'static str,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct JsonObjectConfig {
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MaskWindowConfig {
    pub directions_enabled: bool,
    pub display_recognition_results_on_mask: bool,
    pub mask_enabled: bool,
    pub show_log_box: bool,
    pub show_status: bool,
    pub uid_cover_enabled: bool,
    pub show_fps: bool,
    pub show_overlay_metrics: bool,
    pub overlay_metric_items: BTreeMap<String, bool>,
    pub text_opacity: f64,
    pub overlay_layout_edit_enabled: bool,
    pub log_text_box_left_ratio: f64,
    pub log_text_box_top_ratio: f64,
    pub log_text_box_width_ratio: f64,
    pub log_text_box_height_ratio: f64,
    pub status_list_left_ratio: f64,
    pub status_list_top_ratio: f64,
    pub status_list_width_ratio: f64,
    pub status_list_height_ratio: f64,
    pub metrics_left_ratio: f64,
    pub metrics_top_ratio: f64,
    pub metrics_width_ratio: f64,
    pub metrics_height_ratio: f64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OverlayMetricItem {
    GameFps,
    ProcessingCost,
    PeakProcessingCost,
    CaptureCost,
    TriggerCost,
    SkippedTicks,
    GpuUsage,
    CpuUsage,
    MemoryUsage,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OverlayMetricDescriptor {
    pub key: &'static str,
    pub display_name: &'static str,
    pub tooltip: &'static str,
    pub enabled_by_default: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OverlayLayoutRect {
    pub left_ratio: f64,
    pub top_ratio: f64,
    pub width_ratio: f64,
    pub height_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MaskWindowState {
    pub mask_enabled: bool,
    pub show_log_box: bool,
    pub show_status: bool,
    pub display_recognition_results_on_mask: bool,
    pub directions_enabled: bool,
    pub uid_cover_enabled: bool,
    pub show_fps: bool,
    pub show_overlay_metrics: bool,
    pub text_opacity: f64,
    pub overlay_layout_edit_enabled: bool,
    pub log_box_layout: OverlayLayoutRect,
    pub status_layout: OverlayLayoutRect,
    pub metrics_layout: OverlayLayoutRect,
    pub metrics: Vec<OverlayMetricDescriptor>,
    pub enabled_metric_keys: Vec<String>,
    pub migrated_legacy_metrics_layout: bool,
}

impl Default for MaskWindowConfig {
    fn default() -> Self {
        Self {
            directions_enabled: false,
            display_recognition_results_on_mask: true,
            mask_enabled: true,
            show_log_box: true,
            show_status: true,
            uid_cover_enabled: false,
            show_fps: false,
            show_overlay_metrics: false,
            overlay_metric_items: overlay_metric_defaults(),
            text_opacity: 1.0,
            overlay_layout_edit_enabled: false,
            log_text_box_left_ratio: 20.0 / 1920.0,
            log_text_box_top_ratio: 822.0 / 1080.0,
            log_text_box_width_ratio: 480.0 / 1920.0,
            log_text_box_height_ratio: 188.0 / 1080.0,
            status_list_left_ratio: 20.0 / 1920.0,
            status_list_top_ratio: 790.0 / 1080.0,
            status_list_width_ratio: 480.0 / 1920.0,
            status_list_height_ratio: 24.0 / 1080.0,
            metrics_left_ratio: 20.0 / 1920.0,
            metrics_top_ratio: 744.0 / 1080.0,
            metrics_width_ratio: 477.0 / 1920.0,
            metrics_height_ratio: 58.0 / 1080.0,
            extra: Map::new(),
        }
    }
}

impl OverlayMetricItem {
    pub fn key(self) -> &'static str {
        match self {
            Self::GameFps => "GameFps",
            Self::ProcessingCost => "ProcessingCost",
            Self::PeakProcessingCost => "PeakProcessingCost",
            Self::CaptureCost => "CaptureCost",
            Self::TriggerCost => "TriggerCost",
            Self::SkippedTicks => "SkippedTicks",
            Self::GpuUsage => "GpuUsage",
            Self::CpuUsage => "CpuUsage",
            Self::MemoryUsage => "MemoryUsage",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::GameFps => "游戏帧率",
            Self::ProcessingCost => "处理耗时",
            Self::PeakProcessingCost => "峰值耗时",
            Self::CaptureCost => "截图耗时",
            Self::TriggerCost => "触发器耗时",
            Self::SkippedTicks => "跳过次数",
            Self::GpuUsage => "显卡占用",
            Self::CpuUsage => "CPU占用",
            Self::MemoryUsage => "内存占用",
        }
    }

    pub fn tooltip(self) -> &'static str {
        match self {
            Self::GameFps => "游戏当前渲染帧率。",
            Self::ProcessingCost => "BetterGI 每轮截图、识别和触发器处理的总耗时。",
            Self::PeakProcessingCost => "最近 5 秒内 BetterGI 单轮处理耗时的峰值。",
            Self::CaptureCost => "单次获取游戏画面的耗时。",
            Self::TriggerCost => "本轮实际执行触发器的总耗时。",
            Self::SkippedTicks => "上一轮未结束导致本秒跳过的调度次数。",
            Self::GpuUsage => "显卡核心占用率，读取不到时自动隐藏。",
            Self::CpuUsage => "CPU 总占用率，读取不到时自动隐藏。",
            Self::MemoryUsage => "系统内存占用率，读取不到时自动隐藏。",
        }
    }

    pub fn enabled_by_default(self) -> bool {
        matches!(
            self,
            Self::GameFps
                | Self::ProcessingCost
                | Self::PeakProcessingCost
                | Self::CaptureCost
                | Self::SkippedTicks
                | Self::MemoryUsage
        )
    }
}

pub const OVERLAY_METRIC_ITEMS: &[OverlayMetricItem] = &[
    OverlayMetricItem::GameFps,
    OverlayMetricItem::ProcessingCost,
    OverlayMetricItem::PeakProcessingCost,
    OverlayMetricItem::CaptureCost,
    OverlayMetricItem::TriggerCost,
    OverlayMetricItem::SkippedTicks,
    OverlayMetricItem::GpuUsage,
    OverlayMetricItem::CpuUsage,
    OverlayMetricItem::MemoryUsage,
];

const DEFAULT_METRICS_LAYOUT: OverlayLayoutRect = OverlayLayoutRect {
    left_ratio: 20.0 / 1920.0,
    top_ratio: 744.0 / 1080.0,
    width_ratio: 477.0 / 1920.0,
    height_ratio: 58.0 / 1080.0,
};

const LEGACY_METRICS_LAYOUTS: &[OverlayLayoutRect] = &[
    OverlayLayoutRect {
        left_ratio: 4.0 / 1920.0,
        top_ratio: 4.0 / 1080.0,
        width_ratio: 720.0 / 1920.0,
        height_ratio: 42.0 / 1080.0,
    },
    OverlayLayoutRect {
        left_ratio: 600.0 / 1920.0,
        top_ratio: 16.0 / 1080.0,
        width_ratio: 720.0 / 1920.0,
        height_ratio: 42.0 / 1080.0,
    },
    OverlayLayoutRect {
        left_ratio: 20.0 / 1920.0,
        top_ratio: 724.0 / 1080.0,
        width_ratio: 760.0 / 1920.0,
        height_ratio: 58.0 / 1080.0,
    },
    OverlayLayoutRect {
        left_ratio: 20.0 / 1920.0,
        top_ratio: 724.0 / 1080.0,
        width_ratio: 760.0 / 1920.0,
        height_ratio: 42.0 / 1080.0,
    },
    OverlayLayoutRect {
        left_ratio: 20.0 / 1920.0,
        top_ratio: 760.0 / 1080.0,
        width_ratio: 477.0 / 1920.0,
        height_ratio: 42.0 / 1080.0,
    },
    OverlayLayoutRect {
        left_ratio: 20.0 / 1920.0,
        top_ratio: 760.0 / 1080.0,
        width_ratio: 477.0 / 1920.0,
        height_ratio: 58.0 / 1080.0,
    },
];

impl MaskWindowConfig {
    pub fn ensure_overlay_metric_items(&mut self) {
        let legacy_trigger_interval = "TriggerInterval";
        if let Some(enabled) = self
            .overlay_metric_items
            .get(legacy_trigger_interval)
            .copied()
        {
            self.overlay_metric_items
                .entry(OverlayMetricItem::PeakProcessingCost.key().to_string())
                .or_insert(enabled);
        }

        for item in OVERLAY_METRIC_ITEMS {
            self.overlay_metric_items
                .entry(item.key().to_string())
                .or_insert_with(|| item.enabled_by_default());
        }

        self.overlay_metric_items
            .retain(|key, _| overlay_metric_item_from_key(key).is_some());
    }

    pub fn migrate_legacy_overlay_metrics_layout(&mut self) -> bool {
        let current = self.metrics_layout();
        if !LEGACY_METRICS_LAYOUTS
            .iter()
            .any(|layout| same_overlay_layout(&current, layout))
        {
            return false;
        }

        self.metrics_left_ratio = DEFAULT_METRICS_LAYOUT.left_ratio;
        self.metrics_top_ratio = DEFAULT_METRICS_LAYOUT.top_ratio;
        self.metrics_width_ratio = DEFAULT_METRICS_LAYOUT.width_ratio;
        self.metrics_height_ratio = DEFAULT_METRICS_LAYOUT.height_ratio;
        true
    }

    pub fn overlay_state(&self) -> MaskWindowState {
        let mut config = self.clone();
        config.ensure_overlay_metric_items();
        let migrated_legacy_metrics_layout = config.migrate_legacy_overlay_metrics_layout();
        let enabled_metric_keys = OVERLAY_METRIC_ITEMS
            .iter()
            .filter(|item| {
                config
                    .overlay_metric_items
                    .get(item.key())
                    .copied()
                    .unwrap_or_else(|| item.enabled_by_default())
            })
            .map(|item| item.key().to_string())
            .collect::<Vec<_>>();

        MaskWindowState {
            mask_enabled: config.mask_enabled,
            show_log_box: config.show_log_box,
            show_status: config.show_status,
            display_recognition_results_on_mask: config.display_recognition_results_on_mask,
            directions_enabled: config.directions_enabled,
            uid_cover_enabled: config.uid_cover_enabled,
            show_fps: config.show_fps,
            show_overlay_metrics: config.show_overlay_metrics,
            text_opacity: config.text_opacity,
            overlay_layout_edit_enabled: config.overlay_layout_edit_enabled,
            log_box_layout: OverlayLayoutRect {
                left_ratio: config.log_text_box_left_ratio,
                top_ratio: config.log_text_box_top_ratio,
                width_ratio: config.log_text_box_width_ratio,
                height_ratio: config.log_text_box_height_ratio,
            },
            status_layout: OverlayLayoutRect {
                left_ratio: config.status_list_left_ratio,
                top_ratio: config.status_list_top_ratio,
                width_ratio: config.status_list_width_ratio,
                height_ratio: config.status_list_height_ratio,
            },
            metrics_layout: config.metrics_layout(),
            metrics: overlay_metric_descriptors(),
            enabled_metric_keys,
            migrated_legacy_metrics_layout,
        }
    }

    fn metrics_layout(&self) -> OverlayLayoutRect {
        OverlayLayoutRect {
            left_ratio: self.metrics_left_ratio,
            top_ratio: self.metrics_top_ratio,
            width_ratio: self.metrics_width_ratio,
            height_ratio: self.metrics_height_ratio,
        }
    }
}

pub fn overlay_metric_descriptors() -> Vec<OverlayMetricDescriptor> {
    OVERLAY_METRIC_ITEMS
        .iter()
        .map(|item| OverlayMetricDescriptor {
            key: item.key(),
            display_name: item.display_name(),
            tooltip: item.tooltip(),
            enabled_by_default: item.enabled_by_default(),
        })
        .collect()
}

pub fn overlay_metric_item_from_key(key: &str) -> Option<OverlayMetricItem> {
    OVERLAY_METRIC_ITEMS
        .iter()
        .copied()
        .find(|item| item.key() == key)
}

fn same_overlay_layout(left: &OverlayLayoutRect, right: &OverlayLayoutRect) -> bool {
    same_ratio(left.left_ratio, right.left_ratio)
        && same_ratio(left.top_ratio, right.top_ratio)
        && same_ratio(left.width_ratio, right.width_ratio)
        && same_ratio(left.height_ratio, right.height_ratio)
}

fn same_ratio(left: f64, right: f64) -> bool {
    (left - right).abs() < 0.0000001
}

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
pub struct AutoWoodConfig {
    pub after_z_sleep_delay: u64,
    pub wood_count_ocr_enabled: bool,
    pub use_wonderland_refresh: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoWoodConfig {
    fn default() -> Self {
        Self {
            after_z_sleep_delay: 0,
            wood_count_ocr_enabled: false,
            use_wonderland_refresh: true,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoFightConfig {
    pub strategy_name: String,
    pub team_names: String,
    pub fight_finish_detect_enabled: bool,
    pub action_scheduler_by_cd: String,
    pub only_pick_elite_drops_mode: String,
    pub finish_detect_config: FightFinishDetectConfig,
    pub pick_drops_after_fight_enabled: bool,
    pub pick_drops_after_fight_seconds: u64,
    pub battle_threshold_for_loot: Option<u64>,
    pub kazuha_pickup_enabled: bool,
    pub qin_double_pick_up: bool,
    pub guardian_avatar: String,
    pub guardian_combat_skip: bool,
    pub skip_model: bool,
    pub guardian_avatar_hold: bool,
    pub burst_enabled: bool,
    pub kazuha_party_name: String,
    pub swimming_enabled: bool,
    pub exp_based_pickup_enabled: bool,
    pub timeout: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoFightConfig {
    fn default() -> Self {
        Self {
            strategy_name: "根据队伍自动选择".to_string(),
            team_names: String::new(),
            fight_finish_detect_enabled: true,
            action_scheduler_by_cd: String::new(),
            only_pick_elite_drops_mode: "Closed".to_string(),
            finish_detect_config: FightFinishDetectConfig::default(),
            pick_drops_after_fight_enabled: false,
            pick_drops_after_fight_seconds: 15,
            battle_threshold_for_loot: None,
            kazuha_pickup_enabled: true,
            qin_double_pick_up: false,
            guardian_avatar: String::new(),
            guardian_combat_skip: false,
            skip_model: false,
            guardian_avatar_hold: false,
            burst_enabled: false,
            kazuha_party_name: String::new(),
            swimming_enabled: true,
            exp_based_pickup_enabled: false,
            timeout: 120,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FightFinishDetectConfig {
    pub battle_end_progress_bar_color: String,
    pub battle_end_progress_bar_color_tolerance: String,
    pub fast_check_enabled: bool,
    pub rotate_find_enemy_enabled: bool,
    pub fast_check_params: String,
    pub check_end_delay: String,
    pub before_detect_delay: String,
    pub rotary_factor: u64,
    pub is_first_check: bool,
    pub check_before_burst: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for FightFinishDetectConfig {
    fn default() -> Self {
        Self {
            battle_end_progress_bar_color: String::new(),
            battle_end_progress_bar_color_tolerance: String::new(),
            fast_check_enabled: false,
            rotate_find_enemy_enabled: false,
            fast_check_params: String::new(),
            check_end_delay: "0.4;钟离,1.4;".to_string(),
            before_detect_delay: "0.4".to_string(),
            rotary_factor: 12,
            is_first_check: false,
            check_before_burst: false,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoDomainConfig {
    pub fight_end_delay: f64,
    pub short_movement: bool,
    pub walk_to_f: bool,
    pub left_right_move_times: u64,
    pub auto_eat: bool,
    pub party_name: String,
    pub domain_name: String,
    pub auto_artifact_salvage: bool,
    pub sunday_selected_value: String,
    pub specify_resin_use: bool,
    pub resin_priority_list: Vec<String>,
    pub original_resin_use_count: u64,
    pub original_resin20_use_count: u64,
    pub original_resin40_use_count: u64,
    pub condensed_resin_use_count: u64,
    pub transient_resin_use_count: u64,
    pub fragile_resin_use_count: u64,
    pub revive_retry_count: u64,
    pub reward_recognition_enabled: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoDomainConfig {
    fn default() -> Self {
        Self {
            fight_end_delay: 5.0,
            short_movement: false,
            walk_to_f: true,
            left_right_move_times: 3,
            auto_eat: false,
            party_name: String::new(),
            domain_name: String::new(),
            auto_artifact_salvage: false,
            sunday_selected_value: String::new(),
            specify_resin_use: false,
            resin_priority_list: vec!["浓缩树脂".to_string(), "原粹树脂".to_string()],
            original_resin_use_count: 0,
            original_resin20_use_count: 0,
            original_resin40_use_count: 0,
            condensed_resin_use_count: 0,
            transient_resin_use_count: 0,
            fragile_resin_use_count: 0,
            revive_retry_count: 3,
            reward_recognition_enabled: false,
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
            click_chat_option:
                "\u{4f18}\u{5148}\u{9009}\u{62e9}\u{7b2c}\u{4e00}\u{4e2a}\u{9009}\u{9879}"
                    .to_string(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoBossConfig {
    pub boss_name: String,
    pub strategy_name: String,
    pub team_name: String,
    pub specify_run_count: bool,
    pub run_count: u64,
    pub use_transient_resin: bool,
    pub use_fragile_resin: bool,
    pub revive_retry_count: u64,
    pub return_to_statue_after_each_round: bool,
    pub reward_recognition_enabled: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoBossConfig {
    fn default() -> Self {
        Self {
            boss_name: String::new(),
            strategy_name: "根据队伍自动选择".to_string(),
            team_name: String::new(),
            specify_run_count: false,
            run_count: 1,
            use_transient_resin: false,
            use_fragile_resin: false,
            revive_retry_count: 3,
            return_to_statue_after_each_round: false,
            reward_recognition_enabled: false,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoStygianOnslaughtConfig {
    pub strategy_name: String,
    pub boss_num: u64,
    pub auto_artifact_salvage: bool,
    pub specify_resin_use: bool,
    pub resin_priority_list: Vec<String>,
    pub original_resin_use_count: u64,
    pub condensed_resin_use_count: u64,
    pub transient_resin_use_count: u64,
    pub fragile_resin_use_count: u64,
    pub fight_team_name: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoStygianOnslaughtConfig {
    fn default() -> Self {
        Self {
            strategy_name: String::new(),
            boss_num: 1,
            auto_artifact_salvage: false,
            specify_resin_use: false,
            resin_priority_list: vec!["浓缩树脂".to_string(), "原粹树脂".to_string()],
            original_resin_use_count: 0,
            condensed_resin_use_count: 0,
            transient_resin_use_count: 0,
            fragile_resin_use_count: 0,
            fight_team_name: String::new(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoArtifactSalvageConfig {
    pub java_script: String,
    pub artifact_set_filter: String,
    pub regular_expression: String,
    pub max_artifact_star: String,
    pub max_num_to_check: u64,
    pub recognition_failure_policy: Value,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoArtifactSalvageConfig {
    fn default() -> Self {
        Self {
            java_script: "var hasATK = Array.from(ArtifactStat.MinorAffixes).some(affix => affix.Type == 'ATK');\nvar hasDEF = Array.from(ArtifactStat.MinorAffixes).some(affix => affix.Type == 'DEF');\nvar hasHP = Array.from(ArtifactStat.MinorAffixes).some(affix => affix.Type == 'HP');\nOutput = (hasATK && hasDEF) || (hasHP && hasDEF);".to_string(),
            artifact_set_filter: String::new(),
            regular_expression: r"(?=[\S\s]*攻击力\+[\d]*\n)(?=[\S\s]*防御力\+[\d]*\n)".to_string(),
            max_artifact_star: "4".to_string(),
            max_num_to_check: 100,
            recognition_failure_policy: Value::String("Skip".to_string()),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoLeyLineOutcropConfig {
    pub ley_line_outcrop_type: String,
    pub country: String,
    pub is_resin_exhaustion_mode: bool,
    pub open_mode_count_min: bool,
    pub count: u64,
    pub use_transient_resin: bool,
    pub use_fragile_resin: bool,
    pub team: String,
    pub friendship_team: String,
    pub timeout: u64,
    pub use_adventurer_handbook: bool,
    pub is_notification: bool,
    pub is_go_to_synthesizer: bool,
    pub scan_drops_after_reward_enabled: bool,
    pub scan_drops_after_reward_seconds: u64,
    pub fight_config: AutoLeyLineOutcropFightConfig,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoLeyLineOutcropConfig {
    fn default() -> Self {
        Self {
            ley_line_outcrop_type: "启示之花".to_string(),
            country: "蒙德".to_string(),
            is_resin_exhaustion_mode: false,
            open_mode_count_min: false,
            count: 6,
            use_transient_resin: false,
            use_fragile_resin: false,
            team: String::new(),
            friendship_team: String::new(),
            timeout: 120,
            use_adventurer_handbook: false,
            is_notification: false,
            is_go_to_synthesizer: false,
            scan_drops_after_reward_enabled: false,
            scan_drops_after_reward_seconds: 12,
            fight_config: AutoLeyLineOutcropFightConfig::default(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoLeyLineOutcropFightConfig {
    pub strategy_name: String,
    pub team_names: String,
    pub fight_finish_detect_enabled: bool,
    pub action_scheduler_by_cd: String,
    pub finish_detect_config: LeyLineFightFinishDetectConfig,
    pub guardian_avatar: String,
    pub guardian_combat_skip: bool,
    pub guardian_avatar_hold: bool,
    pub burst_enabled: bool,
    pub swimming_enabled: bool,
    pub kazuha_pickup_enabled: bool,
    pub qin_double_pick_up: bool,
    pub timeout: u64,
    pub seek_enemy_enabled: bool,
    pub seek_enemy_interval_seconds: u64,
    pub seek_enemy_rotary_factor: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoLeyLineOutcropFightConfig {
    fn default() -> Self {
        Self {
            strategy_name: String::new(),
            team_names: String::new(),
            fight_finish_detect_enabled: true,
            action_scheduler_by_cd: String::new(),
            finish_detect_config: LeyLineFightFinishDetectConfig::default(),
            guardian_avatar: String::new(),
            guardian_combat_skip: false,
            guardian_avatar_hold: false,
            burst_enabled: false,
            swimming_enabled: false,
            kazuha_pickup_enabled: true,
            qin_double_pick_up: false,
            timeout: 120,
            seek_enemy_enabled: false,
            seek_enemy_interval_seconds: 3,
            seek_enemy_rotary_factor: 6,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LeyLineFightFinishDetectConfig {
    pub battle_end_progress_bar_color: String,
    pub battle_end_progress_bar_color_tolerance: String,
    pub fast_check_enabled: bool,
    pub rotate_find_enemy_enabled: bool,
    pub fast_check_params: String,
    pub check_end_delay: String,
    pub before_detect_delay: String,
    pub rotary_factor: u64,
    pub is_first_check: bool,
    pub check_before_burst: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for LeyLineFightFinishDetectConfig {
    fn default() -> Self {
        Self {
            battle_end_progress_bar_color: String::new(),
            battle_end_progress_bar_color_tolerance: String::new(),
            fast_check_enabled: false,
            rotate_find_enemy_enabled: false,
            fast_check_params: String::new(),
            check_end_delay: String::new(),
            before_detect_delay: String::new(),
            rotary_factor: 10,
            is_first_check: false,
            check_before_burst: false,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeyId(pub u16);

impl KeyId {
    pub const NONE: Self = Self(0x00);
    pub const UNKNOWN: Self = Self(0xFF);
    pub const MOUSE_LEFT_BUTTON: Self = Self(0x01);
    pub const MOUSE_RIGHT_BUTTON: Self = Self(0x02);
    pub const MOUSE_MIDDLE_BUTTON: Self = Self(0x04);
    pub const MOUSE_SIDE_BUTTON1: Self = Self(0x05);
    pub const MOUSE_SIDE_BUTTON2: Self = Self(0x06);
    pub const F1: Self = Self(0x70);
    pub const F2: Self = Self(0x71);
    pub const F3: Self = Self(0x72);
    pub const F4: Self = Self(0x73);
    pub const F5: Self = Self(0x74);
    pub const F6: Self = Self(0x75);
    pub const F7: Self = Self(0x76);
    pub const F8: Self = Self(0x77);
    pub const F9: Self = Self(0x78);
    pub const F10: Self = Self(0x79);
    pub const F11: Self = Self(0x7A);
    pub const F12: Self = Self(0x7B);
    pub const ESCAPE: Self = Self(0x1B);
    pub const PRINT_SCREEN: Self = Self(0x2C);
    pub const SCROLL_LOCK: Self = Self(0x91);
    pub const PAUSE: Self = Self(0x13);
    pub const INSERT: Self = Self(0x2D);
    pub const DELETE: Self = Self(0x2E);
    pub const HOME: Self = Self(0x24);
    pub const END: Self = Self(0x23);
    pub const PAGE_UP: Self = Self(0x21);
    pub const PAGE_DOWN: Self = Self(0x22);
    pub const BACKSPACE: Self = Self(0x08);
    pub const TAB: Self = Self(0x09);
    pub const CAPS_LOCK: Self = Self(0x14);
    pub const ENTER: Self = Self(0x0D);
    pub const LEFT_SHIFT: Self = Self(0xA0);
    pub const RIGHT_SHIFT: Self = Self(0xA1);
    pub const LEFT_CTRL: Self = Self(0xA2);
    pub const RIGHT_CTRL: Self = Self(0xA3);
    pub const LEFT_ALT: Self = Self(0xA4);
    pub const RIGHT_ALT: Self = Self(0xA5);
    pub const LEFT_WIN: Self = Self(0x5B);
    pub const RIGHT_WIN: Self = Self(0x5C);
    pub const APPS: Self = Self(0x5D);
    pub const SPACE: Self = Self(0x20);
    pub const LEFT: Self = Self(0x25);
    pub const UP: Self = Self(0x26);
    pub const RIGHT: Self = Self(0x27);
    pub const DOWN: Self = Self(0x28);
    pub const A: Self = Self(0x41);
    pub const B: Self = Self(0x42);
    pub const C: Self = Self(0x43);
    pub const D: Self = Self(0x44);
    pub const E: Self = Self(0x45);
    pub const F: Self = Self(0x46);
    pub const G: Self = Self(0x47);
    pub const H: Self = Self(0x48);
    pub const I: Self = Self(0x49);
    pub const J: Self = Self(0x4A);
    pub const K: Self = Self(0x4B);
    pub const L: Self = Self(0x4C);
    pub const M: Self = Self(0x4D);
    pub const N: Self = Self(0x4E);
    pub const O: Self = Self(0x4F);
    pub const P: Self = Self(0x50);
    pub const Q: Self = Self(0x51);
    pub const R: Self = Self(0x52);
    pub const S: Self = Self(0x53);
    pub const T: Self = Self(0x54);
    pub const U: Self = Self(0x55);
    pub const V: Self = Self(0x56);
    pub const W: Self = Self(0x57);
    pub const X: Self = Self(0x58);
    pub const Y: Self = Self(0x59);
    pub const Z: Self = Self(0x5A);
    pub const D0: Self = Self(0x30);
    pub const D1: Self = Self(0x31);
    pub const D2: Self = Self(0x32);
    pub const D3: Self = Self(0x33);
    pub const D4: Self = Self(0x34);
    pub const D5: Self = Self(0x35);
    pub const D6: Self = Self(0x36);
    pub const D7: Self = Self(0x37);
    pub const D8: Self = Self(0x38);
    pub const D9: Self = Self(0x39);
    pub const APOSTROPHE: Self = Self(0xDE);
    pub const COMMA: Self = Self(0xBC);
    pub const MINUS: Self = Self(0xBD);
    pub const EQUAL: Self = Self(0xBB);
    pub const PERIOD: Self = Self(0xBE);
    pub const SLASH: Self = Self(0xBF);
    pub const BACKSLASH: Self = Self(0xE2);
    pub const SEMICOLON: Self = Self(0xBA);
    pub const LEFT_SQUARE_BRACKET: Self = Self(0xDB);
    pub const RIGHT_SQUARE_BRACKET: Self = Self(0xDD);
    pub const TILDE: Self = Self(0xC0);
    pub const NUM_LOCK: Self = Self(0x90);
    pub const NUM_PAD0: Self = Self(0x60);
    pub const NUM_PAD1: Self = Self(0x61);
    pub const NUM_PAD2: Self = Self(0x62);
    pub const NUM_PAD3: Self = Self(0x63);
    pub const NUM_PAD4: Self = Self(0x64);
    pub const NUM_PAD5: Self = Self(0x65);
    pub const NUM_PAD6: Self = Self(0x66);
    pub const NUM_PAD7: Self = Self(0x67);
    pub const NUM_PAD8: Self = Self(0x68);
    pub const NUM_PAD9: Self = Self(0x69);
    pub const DECIMAL: Self = Self(0x6E);
    pub const DIVIDE: Self = Self(0x6F);
    pub const MULTIPLY: Self = Self(0x6A);
    pub const SUBTRACT: Self = Self(0x6D);
    pub const ADD: Self = Self(0x6B);
    pub const NUM_ENTER: Self = Self(0x0E);

    pub const fn vk(self) -> u16 {
        self.0
    }

    pub const fn is_mouse_button(self) -> bool {
        matches!(
            self,
            Self::MOUSE_LEFT_BUTTON
                | Self::MOUSE_RIGHT_BUTTON
                | Self::MOUSE_MIDDLE_BUTTON
                | Self::MOUSE_SIDE_BUTTON1
                | Self::MOUSE_SIDE_BUTTON2
        )
    }
}

impl Default for KeyId {
    fn default() -> Self {
        Self::NONE
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct KeyBindingsConfig {
    pub global_key_mapping_enabled: bool,
    pub move_forward: KeyId,
    pub move_backward: KeyId,
    pub move_left: KeyId,
    pub move_right: KeyId,
    pub switch_to_walk_or_run: KeyId,
    pub normal_attack: KeyId,
    pub elemental_skill: KeyId,
    pub elemental_burst: KeyId,
    pub sprint_keyboard: KeyId,
    pub sprint_mouse: KeyId,
    pub switch_aiming_mode: KeyId,
    pub jump: KeyId,
    pub drop: KeyId,
    pub pick_up_or_interact: KeyId,
    pub quick_use_gadget: KeyId,
    pub interaction_in_some_mode: KeyId,
    pub quest_navigation: KeyId,
    pub abandon_challenge: KeyId,
    pub switch_member1: KeyId,
    pub switch_member2: KeyId,
    pub switch_member3: KeyId,
    pub switch_member4: KeyId,
    pub switch_member5: KeyId,
    pub shortcut_wheel: KeyId,
    pub open_inventory: KeyId,
    pub open_character_screen: KeyId,
    pub open_map: KeyId,
    pub open_paimon_menu: KeyId,
    pub open_adventurer_handbook: KeyId,
    pub open_co_op_screen: KeyId,
    pub open_wish_screen: KeyId,
    pub open_battle_pass_screen: KeyId,
    pub open_the_events_menu: KeyId,
    pub open_the_settings_menu: KeyId,
    pub open_the_furnishing_screen: KeyId,
    pub open_stellar_reunion: KeyId,
    pub open_quest_menu: KeyId,
    pub open_notification_details: KeyId,
    pub open_chat_screen: KeyId,
    pub open_special_environment_information: KeyId,
    pub check_tutorial_details: KeyId,
    pub elemental_sight: KeyId,
    pub show_cursor: KeyId,
    pub open_party_setup_screen: KeyId,
    pub open_friends_screen: KeyId,
    pub hide_ui: KeyId,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for KeyBindingsConfig {
    fn default() -> Self {
        Self {
            global_key_mapping_enabled: false,
            move_forward: KeyId::W,
            move_backward: KeyId::S,
            move_left: KeyId::A,
            move_right: KeyId::D,
            switch_to_walk_or_run: KeyId::LEFT_CTRL,
            normal_attack: KeyId::MOUSE_LEFT_BUTTON,
            elemental_skill: KeyId::E,
            elemental_burst: KeyId::Q,
            sprint_keyboard: KeyId::LEFT_SHIFT,
            sprint_mouse: KeyId::MOUSE_RIGHT_BUTTON,
            switch_aiming_mode: KeyId::R,
            jump: KeyId::SPACE,
            drop: KeyId::X,
            pick_up_or_interact: KeyId::F,
            quick_use_gadget: KeyId::Z,
            interaction_in_some_mode: KeyId::T,
            quest_navigation: KeyId::V,
            abandon_challenge: KeyId::P,
            switch_member1: KeyId::D1,
            switch_member2: KeyId::D2,
            switch_member3: KeyId::D3,
            switch_member4: KeyId::D4,
            switch_member5: KeyId::D5,
            shortcut_wheel: KeyId::TAB,
            open_inventory: KeyId::B,
            open_character_screen: KeyId::C,
            open_map: KeyId::M,
            open_paimon_menu: KeyId::ESCAPE,
            open_adventurer_handbook: KeyId::F1,
            open_co_op_screen: KeyId::F2,
            open_wish_screen: KeyId::F3,
            open_battle_pass_screen: KeyId::F4,
            open_the_events_menu: KeyId::F5,
            open_the_settings_menu: KeyId::F6,
            open_the_furnishing_screen: KeyId::F7,
            open_stellar_reunion: KeyId::F8,
            open_quest_menu: KeyId::J,
            open_notification_details: KeyId::Y,
            open_chat_screen: KeyId::ENTER,
            open_special_environment_information: KeyId::U,
            check_tutorial_details: KeyId::G,
            elemental_sight: KeyId::MOUSE_MIDDLE_BUTTON,
            show_cursor: KeyId::LEFT_ALT,
            open_party_setup_screen: KeyId::L,
            open_friends_screen: KeyId::O,
            hide_ui: KeyId::SLASH,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenshinAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    SwitchToWalkOrRun,
    NormalAttack,
    ElementalSkill,
    ElementalBurst,
    SprintKeyboard,
    SprintMouse,
    SwitchAimingMode,
    Jump,
    Drop,
    PickUpOrInteract,
    QuickUseGadget,
    InteractionInSomeMode,
    QuestNavigation,
    AbandonChallenge,
    SwitchMember1,
    SwitchMember2,
    SwitchMember3,
    SwitchMember4,
    SwitchMember5,
    ShortcutWheel,
    OpenInventory,
    OpenCharacterScreen,
    OpenMap,
    OpenPaimonMenu,
    OpenAdventurerHandbook,
    OpenCoOpScreen,
    OpenWishScreen,
    OpenBattlePassScreen,
    OpenTheEventsMenu,
    OpenTheSettingsMenu,
    OpenTheFurnishingScreen,
    OpenStellarReunion,
    OpenQuestMenu,
    OpenNotificationDetails,
    OpenChatScreen,
    OpenSpecialEnvironmentInformation,
    CheckTutorialDetails,
    ElementalSight,
    ShowCursor,
    OpenPartySetupScreen,
    OpenFriendsScreen,
    HideUi,
}

impl KeyBindingsConfig {
    pub fn action_key(&self, action: GenshinAction) -> KeyId {
        match action {
            GenshinAction::MoveForward => self.move_forward,
            GenshinAction::MoveBackward => self.move_backward,
            GenshinAction::MoveLeft => self.move_left,
            GenshinAction::MoveRight => self.move_right,
            GenshinAction::SwitchToWalkOrRun => self.switch_to_walk_or_run,
            GenshinAction::NormalAttack => self.normal_attack,
            GenshinAction::ElementalSkill => self.elemental_skill,
            GenshinAction::ElementalBurst => self.elemental_burst,
            GenshinAction::SprintKeyboard => self.sprint_keyboard,
            GenshinAction::SprintMouse => self.sprint_mouse,
            GenshinAction::SwitchAimingMode => self.switch_aiming_mode,
            GenshinAction::Jump => self.jump,
            GenshinAction::Drop => self.drop,
            GenshinAction::PickUpOrInteract => self.pick_up_or_interact,
            GenshinAction::QuickUseGadget => self.quick_use_gadget,
            GenshinAction::InteractionInSomeMode => self.interaction_in_some_mode,
            GenshinAction::QuestNavigation => self.quest_navigation,
            GenshinAction::AbandonChallenge => self.abandon_challenge,
            GenshinAction::SwitchMember1 => self.switch_member1,
            GenshinAction::SwitchMember2 => self.switch_member2,
            GenshinAction::SwitchMember3 => self.switch_member3,
            GenshinAction::SwitchMember4 => self.switch_member4,
            GenshinAction::SwitchMember5 => self.switch_member5,
            GenshinAction::ShortcutWheel => self.shortcut_wheel,
            GenshinAction::OpenInventory => self.open_inventory,
            GenshinAction::OpenCharacterScreen => self.open_character_screen,
            GenshinAction::OpenMap => self.open_map,
            GenshinAction::OpenPaimonMenu => self.open_paimon_menu,
            GenshinAction::OpenAdventurerHandbook => self.open_adventurer_handbook,
            GenshinAction::OpenCoOpScreen => self.open_co_op_screen,
            GenshinAction::OpenWishScreen => self.open_wish_screen,
            GenshinAction::OpenBattlePassScreen => self.open_battle_pass_screen,
            GenshinAction::OpenTheEventsMenu => self.open_the_events_menu,
            GenshinAction::OpenTheSettingsMenu => self.open_the_settings_menu,
            GenshinAction::OpenTheFurnishingScreen => self.open_the_furnishing_screen,
            GenshinAction::OpenStellarReunion => self.open_stellar_reunion,
            GenshinAction::OpenQuestMenu => self.open_quest_menu,
            GenshinAction::OpenNotificationDetails => self.open_notification_details,
            GenshinAction::OpenChatScreen => self.open_chat_screen,
            GenshinAction::OpenSpecialEnvironmentInformation => {
                self.open_special_environment_information
            }
            GenshinAction::CheckTutorialDetails => self.check_tutorial_details,
            GenshinAction::ElementalSight => self.elemental_sight,
            GenshinAction::ShowCursor => self.show_cursor,
            GenshinAction::OpenPartySetupScreen => self.open_party_setup_screen,
            GenshinAction::OpenFriendsScreen => self.open_friends_screen,
            GenshinAction::HideUi => self.hide_ui,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HotKeyConfig {
    pub auto_track_hotkey: String,
    pub auto_track_hotkey_type: String,
    pub auto_domain_hotkey: String,
    pub auto_domain_hotkey_type: String,
    pub auto_fight_hotkey: String,
    pub auto_fight_hotkey_type: String,
    pub auto_fishing_enabled_hotkey: String,
    pub auto_fishing_enabled_hotkey_type: String,
    pub auto_genius_invokation_hotkey: String,
    pub auto_genius_invokation_hotkey_type: String,
    pub bgi_enabled_hotkey: String,
    pub bgi_enabled_hotkey_type: String,
    pub auto_pick_enabled_hotkey: String,
    pub auto_pick_enabled_hotkey_type: String,
    pub auto_skip_enabled_hotkey: String,
    pub auto_skip_enabled_hotkey_type: String,
    pub auto_skip_hangout_enabled_hotkey: String,
    pub auto_skip_hangout_enabled_hotkey_type: String,
    pub auto_wood_hotkey: String,
    pub auto_wood_hotkey_type: String,
    pub enhance_artifact_hotkey: String,
    pub enhance_artifact_hotkey_type: String,
    pub quick_buy_hotkey: String,
    pub quick_buy_hotkey_type: String,
    pub quick_serenitea_pot_hotkey: String,
    pub quick_serenitea_pot_hotkey_type: String,
    pub quick_teleport_enabled_hotkey: String,
    pub quick_teleport_enabled_hotkey_type: String,
    pub quick_teleport_tick_hotkey: String,
    pub quick_teleport_tick_hotkey_type: String,
    pub skill_cd_enabled_hotkey: String,
    pub skill_cd_enabled_hotkey_type: String,
    pub take_screenshot_hotkey: String,
    pub take_screenshot_hotkey_type: String,
    pub turn_around_hotkey: String,
    pub turn_around_hotkey_type: String,
    pub click_genshin_confirm_button_hotkey: String,
    pub click_genshin_confirm_button_hotkey_type: String,
    pub click_genshin_cancel_button_hotkey: String,
    pub click_genshin_cancel_button_hotkey_type: String,
    pub one_key_fight_hotkey: String,
    pub one_key_fight_hotkey_type: String,
    pub map_pos_record_hotkey: String,
    pub map_pos_record_hotkey_type: String,
    pub auto_music_game_hotkey: String,
    pub auto_music_game_hotkey_type: String,
    pub auto_fishing_game_hotkey: String,
    pub auto_fishing_game_hotkey_type: String,
    pub auto_cook_game_hotkey: String,
    pub auto_cook_game_hotkey_type: String,
    pub auto_track_path_hotkey: String,
    pub auto_track_path_hotkey_type: String,
    pub test1_hotkey: String,
    pub test1_hotkey_type: String,
    pub test2_hotkey: String,
    pub test2_hotkey_type: String,
    pub rec_big_map_pos_hotkey: String,
    pub rec_big_map_pos_hotkey_type: String,
    pub path_recorder_hotkey: String,
    pub path_recorder_hotkey_type: String,
    pub add_waypoint_hotkey: String,
    pub add_waypoint_hotkey_type: String,
    pub execute_path_hotkey: String,
    pub execute_path_hotkey_type: String,
    pub log_box_display_hotkey: String,
    pub log_box_display_hotkey_type: String,
    pub key_mouse_macro_record_hotkey: String,
    pub key_mouse_macro_record_hotkey_type: String,
    pub suspend_hotkey: String,
    pub suspend_hotkey_type: String,
    pub cancel_task_hotkey: String,
    pub cancel_task_hotkey_type: String,
    pub onedragon_hotkey: String,
    pub onedragon_hotkey_type: String,
    pub map_mask_enabled_hotkey: String,
    pub map_mask_enabled_hotkey_type: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for HotKeyConfig {
    fn default() -> Self {
        const KEYBOARD_MONITOR: &str = "KeyboardMonitor";
        Self {
            auto_track_hotkey: String::new(),
            auto_track_hotkey_type: KEYBOARD_MONITOR.to_string(),
            auto_domain_hotkey: String::new(),
            auto_domain_hotkey_type: KEYBOARD_MONITOR.to_string(),
            auto_fight_hotkey: String::new(),
            auto_fight_hotkey_type: KEYBOARD_MONITOR.to_string(),
            auto_fishing_enabled_hotkey: String::new(),
            auto_fishing_enabled_hotkey_type: KEYBOARD_MONITOR.to_string(),
            auto_genius_invokation_hotkey: String::new(),
            auto_genius_invokation_hotkey_type: KEYBOARD_MONITOR.to_string(),
            bgi_enabled_hotkey: "F11".to_string(),
            bgi_enabled_hotkey_type: "GlobalRegister".to_string(),
            auto_pick_enabled_hotkey: String::new(),
            auto_pick_enabled_hotkey_type: KEYBOARD_MONITOR.to_string(),
            auto_skip_enabled_hotkey: String::new(),
            auto_skip_enabled_hotkey_type: KEYBOARD_MONITOR.to_string(),
            auto_skip_hangout_enabled_hotkey: String::new(),
            auto_skip_hangout_enabled_hotkey_type: KEYBOARD_MONITOR.to_string(),
            auto_wood_hotkey: String::new(),
            auto_wood_hotkey_type: KEYBOARD_MONITOR.to_string(),
            enhance_artifact_hotkey: String::new(),
            enhance_artifact_hotkey_type: KEYBOARD_MONITOR.to_string(),
            quick_buy_hotkey: String::new(),
            quick_buy_hotkey_type: KEYBOARD_MONITOR.to_string(),
            quick_serenitea_pot_hotkey: String::new(),
            quick_serenitea_pot_hotkey_type: KEYBOARD_MONITOR.to_string(),
            quick_teleport_enabled_hotkey: String::new(),
            quick_teleport_enabled_hotkey_type: KEYBOARD_MONITOR.to_string(),
            quick_teleport_tick_hotkey: String::new(),
            quick_teleport_tick_hotkey_type: KEYBOARD_MONITOR.to_string(),
            skill_cd_enabled_hotkey: String::new(),
            skill_cd_enabled_hotkey_type: KEYBOARD_MONITOR.to_string(),
            take_screenshot_hotkey: String::new(),
            take_screenshot_hotkey_type: KEYBOARD_MONITOR.to_string(),
            turn_around_hotkey: String::new(),
            turn_around_hotkey_type: KEYBOARD_MONITOR.to_string(),
            click_genshin_confirm_button_hotkey: String::new(),
            click_genshin_confirm_button_hotkey_type: KEYBOARD_MONITOR.to_string(),
            click_genshin_cancel_button_hotkey: String::new(),
            click_genshin_cancel_button_hotkey_type: KEYBOARD_MONITOR.to_string(),
            one_key_fight_hotkey: String::new(),
            one_key_fight_hotkey_type: KEYBOARD_MONITOR.to_string(),
            map_pos_record_hotkey: String::new(),
            map_pos_record_hotkey_type: KEYBOARD_MONITOR.to_string(),
            auto_music_game_hotkey: String::new(),
            auto_music_game_hotkey_type: KEYBOARD_MONITOR.to_string(),
            auto_fishing_game_hotkey: String::new(),
            auto_fishing_game_hotkey_type: KEYBOARD_MONITOR.to_string(),
            auto_cook_game_hotkey: String::new(),
            auto_cook_game_hotkey_type: KEYBOARD_MONITOR.to_string(),
            auto_track_path_hotkey: String::new(),
            auto_track_path_hotkey_type: KEYBOARD_MONITOR.to_string(),
            test1_hotkey: String::new(),
            test1_hotkey_type: KEYBOARD_MONITOR.to_string(),
            test2_hotkey: String::new(),
            test2_hotkey_type: KEYBOARD_MONITOR.to_string(),
            rec_big_map_pos_hotkey: String::new(),
            rec_big_map_pos_hotkey_type: KEYBOARD_MONITOR.to_string(),
            path_recorder_hotkey: String::new(),
            path_recorder_hotkey_type: KEYBOARD_MONITOR.to_string(),
            add_waypoint_hotkey: String::new(),
            add_waypoint_hotkey_type: KEYBOARD_MONITOR.to_string(),
            execute_path_hotkey: String::new(),
            execute_path_hotkey_type: KEYBOARD_MONITOR.to_string(),
            log_box_display_hotkey: String::new(),
            log_box_display_hotkey_type: KEYBOARD_MONITOR.to_string(),
            key_mouse_macro_record_hotkey: String::new(),
            key_mouse_macro_record_hotkey_type: KEYBOARD_MONITOR.to_string(),
            suspend_hotkey: String::new(),
            suspend_hotkey_type: KEYBOARD_MONITOR.to_string(),
            cancel_task_hotkey: String::new(),
            cancel_task_hotkey_type: KEYBOARD_MONITOR.to_string(),
            onedragon_hotkey: String::new(),
            onedragon_hotkey_type: KEYBOARD_MONITOR.to_string(),
            map_mask_enabled_hotkey: String::new(),
            map_mask_enabled_hotkey_type: KEYBOARD_MONITOR.to_string(),
            extra: Map::new(),
        }
    }
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

fn overlay_metric_defaults() -> BTreeMap<String, bool> {
    [
        ("GameFps", true),
        ("ProcessingCost", true),
        ("PeakProcessingCost", true),
        ("CaptureCost", true),
        ("TriggerCost", false),
        ("SkippedTicks", true),
        ("GpuUsage", false),
        ("CpuUsage", false),
        ("MemoryUsage", true),
    ]
    .into_iter()
    .map(|(key, enabled)| (key.to_string(), enabled))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_keeps_csharp_defaults_for_first_ported_fields() {
        let config = AppConfig::default();
        assert_eq!(config.trigger_interval, 50);
        assert!(config.auto_fix_win11_bit_blt);
        assert_eq!(config.auto_pick_config.pick_key, "F");
        assert_eq!(config.hot_key_config.bgi_enabled_hotkey, "F11");
        assert_eq!(
            config
                .key_bindings_config
                .action_key(GenshinAction::QuickUseGadget),
            KeyId::Z
        );
        assert_eq!(config.coverage().strongly_typed_sections, 33);
        assert_eq!(config.coverage().compatibility_sections, 0);
    }

    #[test]
    fn overlay_metric_items_follow_legacy_order_defaults_and_cleanup() {
        let descriptors = overlay_metric_descriptors();
        assert_eq!(
            descriptors.iter().map(|item| item.key).collect::<Vec<_>>(),
            vec![
                "GameFps",
                "ProcessingCost",
                "PeakProcessingCost",
                "CaptureCost",
                "TriggerCost",
                "SkippedTicks",
                "GpuUsage",
                "CpuUsage",
                "MemoryUsage",
            ]
        );
        assert_eq!(descriptors[0].display_name, "游戏帧率");
        assert_eq!(descriptors[6].display_name, "显卡占用");
        assert!(OverlayMetricItem::GameFps.enabled_by_default());
        assert!(!OverlayMetricItem::TriggerCost.enabled_by_default());
        assert!(!OverlayMetricItem::GpuUsage.enabled_by_default());

        let mut config = MaskWindowConfig::default();
        config.overlay_metric_items.clear();
        config
            .overlay_metric_items
            .insert("TriggerInterval".to_string(), false);
        config
            .overlay_metric_items
            .insert("UnknownMetric".to_string(), true);
        config.ensure_overlay_metric_items();

        assert!(!config.overlay_metric_items.contains_key("TriggerInterval"));
        assert!(!config.overlay_metric_items.contains_key("UnknownMetric"));
        assert_eq!(
            config.overlay_metric_items.get("PeakProcessingCost"),
            Some(&false)
        );
        assert_eq!(config.overlay_metric_items.get("GameFps"), Some(&true));
        assert_eq!(config.overlay_metric_items.get("GpuUsage"), Some(&false));
    }

    #[test]
    fn overlay_state_migrates_legacy_metrics_layout_without_mutating_source() {
        let mut config = MaskWindowConfig {
            show_overlay_metrics: true,
            metrics_left_ratio: 20.0 / 1920.0,
            metrics_top_ratio: 760.0 / 1080.0,
            metrics_width_ratio: 477.0 / 1920.0,
            metrics_height_ratio: 58.0 / 1080.0,
            ..MaskWindowConfig::default()
        };
        config.overlay_metric_items.clear();
        config
            .overlay_metric_items
            .insert("GameFps".to_string(), true);
        config
            .overlay_metric_items
            .insert("GpuUsage".to_string(), true);

        let state = config.overlay_state();
        assert!(state.show_overlay_metrics);
        assert!(state.migrated_legacy_metrics_layout);
        assert!(same_overlay_layout(
            &state.metrics_layout,
            &DEFAULT_METRICS_LAYOUT
        ));
        assert_eq!(
            state.enabled_metric_keys,
            [
                "GameFps",
                "ProcessingCost",
                "PeakProcessingCost",
                "CaptureCost",
                "SkippedTicks",
                "GpuUsage",
                "MemoryUsage",
            ]
        );
        assert_eq!(state.metrics.len(), OVERLAY_METRIC_ITEMS.len());

        assert!(same_ratio(config.metrics_top_ratio, 760.0 / 1080.0));
    }

    #[test]
    fn deserializes_camel_case_config() {
        let json = r#"{
            "captureMode": "BitBlt",
            "triggerInterval": 75,
            "autoPickConfig": { "enabled": false, "pickKey": "E" }
        }"#;
        let config: AppConfig = json5::from_str(json).unwrap();
        assert_eq!(config.trigger_interval, 75);
        assert!(!config.auto_pick_config.enabled);
        assert_eq!(config.auto_pick_config.pick_key, "E");
    }

    #[test]
    fn deserializes_legacy_numeric_key_bindings() {
        let json = r#"{
            "keyBindingsConfig": {
                "moveForward": 38,
                "normalAttack": 1,
                "quickUseGadget": 90
            }
        }"#;
        let config: AppConfig = json5::from_str(json).unwrap();
        assert_eq!(config.key_bindings_config.move_forward, KeyId::UP);
        assert_eq!(
            config.key_bindings_config.normal_attack,
            KeyId::MOUSE_LEFT_BUTTON
        );
        assert_eq!(
            config
                .key_bindings_config
                .action_key(GenshinAction::QuickUseGadget),
            KeyId::Z
        );
    }
}
