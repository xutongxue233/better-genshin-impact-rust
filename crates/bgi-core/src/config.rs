use crate::error::{BgiError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

#[path = "app_misc_configs.rs"]
mod app_misc_configs;
#[path = "key_bindings.rs"]
mod key_bindings;
#[path = "overlay.rs"]
mod overlay;
#[path = "task_configs.rs"]
mod task_configs;

pub use app_misc_configs::*;
pub use key_bindings::*;
pub use overlay::*;
pub use task_configs::*;

#[cfg(test)]
use overlay::{same_overlay_layout, same_ratio, DEFAULT_METRICS_LAYOUT};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CaptureMode {
    #[default]
    BitBlt,
    DwmGetDxSharedSurface,
    WindowsGraphicsCapture,
    WindowsGraphicsCaptureHdr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeType {
    #[default]
    DarkNone,
    DarkMica,
    DarkAcrylic,
    LightNone,
    LightMica,
    LightAcrylic,
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

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
