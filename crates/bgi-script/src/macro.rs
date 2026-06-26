use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[path = "macro_playback.rs"]
mod macro_playback;

#[derive(Debug, thiserror::Error)]
pub enum KeyMouseMacroError {
    #[error("failed to parse key/mouse macro JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to read key/mouse macro at {path:?}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("event {index} ({event_type:?}) is missing keyCode")]
    MissingKeyCode {
        index: usize,
        event_type: MacroEventType,
    },
    #[error("event {index} ({event_type:?}) is missing mouseButton")]
    MissingMouseButton {
        index: usize,
        event_type: MacroEventType,
    },
    #[error("event {index} has invalid mouseButton {value:?}")]
    InvalidMouseButton { index: usize, value: String },
    #[error("macro playback context has invalid working area {width}x{height}")]
    InvalidWorkingArea { width: i32, height: i32 },
    #[error("macro playback target capture area is invalid: {area:?}")]
    InvalidCaptureArea { area: MacroCaptureArea },
    #[error("macro info is required for capture-area adaptation")]
    MissingScriptInfo,
    #[error("macro info has invalid dimensions or dpi: {info:?}")]
    InvalidScriptInfo { info: KeyMouseScriptInfo },
}

pub type Result<T> = std::result::Result<T, KeyMouseMacroError>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct KeyMouseScript {
    pub macro_events: Vec<MacroEvent>,
    pub info: Option<KeyMouseScriptInfo>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl KeyMouseScript {
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let json = fs::read_to_string(path).map_err(|source| KeyMouseMacroError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_json(&json)
    }

    pub fn summary(&self) -> KeyMouseMacroSummary {
        let mut key_events = 0;
        let mut absolute_mouse_events = 0;
        let mut relative_mouse_events = 0;
        let mut wheel_events = 0;
        let mut uses_camera_orientation = false;
        let mut duration_ms = 0.0_f64;

        for event in &self.macro_events {
            duration_ms = duration_ms.max(event.time);
            uses_camera_orientation |= event.camera_orientation.is_some();
            match event.event_type {
                MacroEventType::KeyDown | MacroEventType::KeyUp => key_events += 1,
                MacroEventType::MouseMoveTo
                | MacroEventType::MouseDown
                | MacroEventType::MouseUp => absolute_mouse_events += 1,
                MacroEventType::MouseMoveBy => relative_mouse_events += 1,
                MacroEventType::MouseWheel => wheel_events += 1,
            }
        }

        KeyMouseMacroSummary {
            event_count: self.macro_events.len(),
            duration_ms: duration_ms.max(0.0).trunc() as u64,
            has_info: self.info.is_some(),
            key_events,
            absolute_mouse_events,
            relative_mouse_events,
            wheel_events,
            uses_camera_orientation,
        }
    }

    pub fn playback_context_from_info(&self) -> Result<MacroPlaybackContext> {
        let info = self
            .info
            .as_ref()
            .ok_or(KeyMouseMacroError::MissingScriptInfo)?;
        info.validate()?;

        Ok(MacroPlaybackContext {
            target_capture_area: Some(MacroCaptureArea {
                x: info.x,
                y: info.y,
                width: info.width,
                height: info.height,
            }),
            target_dpi_scale: info.record_dpi,
            working_area_width: info.width,
            working_area_height: info.height,
        })
    }
}

impl Default for KeyMouseScript {
    fn default() -> Self {
        Self {
            macro_events: Vec::new(),
            info: None,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyMouseMacroSummary {
    pub event_count: usize,
    pub duration_ms: u64,
    pub has_info: bool,
    pub key_events: usize,
    pub absolute_mouse_events: usize,
    pub relative_mouse_events: usize,
    pub wheel_events: usize,
    pub uses_camera_orientation: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct KeyMouseScriptInfo {
    pub name: String,
    pub description: String,
    pub author: Option<String>,
    pub version: Option<String>,
    pub bgi_version: Option<String>,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub record_dpi: f64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl KeyMouseScriptInfo {
    fn validate(&self) -> Result<()> {
        if self.width <= 0 || self.height <= 0 || self.record_dpi <= 0.0 {
            return Err(KeyMouseMacroError::InvalidScriptInfo { info: self.clone() });
        }
        Ok(())
    }
}

impl Default for KeyMouseScriptInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            author: None,
            version: None,
            bgi_version: None,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            record_dpi: 1.0,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MacroEvent {
    #[serde(rename = "type")]
    pub event_type: MacroEventType,
    pub key_code: Option<u16>,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_button: Option<String>,
    pub time: f64,
    pub camera_orientation: Option<i32>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for MacroEvent {
    fn default() -> Self {
        Self {
            event_type: MacroEventType::KeyDown,
            key_code: None,
            mouse_x: 0,
            mouse_y: 0,
            mouse_button: None,
            time: 0.0,
            camera_orientation: None,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroEventType {
    KeyDown,
    KeyUp,
    MouseMoveTo,
    MouseMoveBy,
    MouseDown,
    MouseUp,
    MouseWheel,
}

impl MacroEventType {
    pub const fn legacy_code(self) -> u8 {
        match self {
            Self::KeyDown => 0,
            Self::KeyUp => 1,
            Self::MouseMoveTo => 2,
            Self::MouseMoveBy => 3,
            Self::MouseDown => 4,
            Self::MouseUp => 5,
            Self::MouseWheel => 6,
        }
    }

    pub const fn from_legacy_code(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::KeyDown),
            1 => Some(Self::KeyUp),
            2 => Some(Self::MouseMoveTo),
            3 => Some(Self::MouseMoveBy),
            4 => Some(Self::MouseDown),
            5 => Some(Self::MouseUp),
            6 => Some(Self::MouseWheel),
            _ => None,
        }
    }
}

impl Default for MacroEventType {
    fn default() -> Self {
        Self::KeyDown
    }
}

impl FromStr for MacroEventType {
    type Err = ();

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "KeyDown" | "keyDown" => Ok(Self::KeyDown),
            "KeyUp" | "keyUp" => Ok(Self::KeyUp),
            "MouseMoveTo" | "mouseMoveTo" => Ok(Self::MouseMoveTo),
            "MouseMoveBy" | "mouseMoveBy" => Ok(Self::MouseMoveBy),
            "MouseDown" | "mouseDown" => Ok(Self::MouseDown),
            "MouseUp" | "mouseUp" => Ok(Self::MouseUp),
            "MouseWheel" | "mouseWheel" => Ok(Self::MouseWheel),
            _ => Err(()),
        }
    }
}

impl Serialize for MacroEventType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.legacy_code())
    }
}

impl<'de> Deserialize<'de> for MacroEventType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MacroEventTypeVisitor;

        impl<'de> Visitor<'de> for MacroEventTypeVisitor {
            type Value = MacroEventType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("legacy macro event type integer or name")
            }

            fn visit_u64<E>(self, value: u64) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                let code = u8::try_from(value).map_err(|_| {
                    E::custom(format!("macro event type {value} is outside u8 range"))
                })?;
                MacroEventType::from_legacy_code(code)
                    .ok_or_else(|| E::custom(format!("unknown macro event type {value}")))
            }

            fn visit_i64<E>(self, value: i64) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value < 0 {
                    return Err(E::custom(format!("unknown macro event type {value}")));
                }
                self.visit_u64(value as u64)
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                value
                    .parse::<MacroEventType>()
                    .map_err(|_| E::custom(format!("unknown macro event type {value:?}")))
            }
        }

        deserializer.deserialize_any(MacroEventTypeVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroCaptureArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl MacroCaptureArea {
    fn validate(self) -> Result<()> {
        if self.width <= 0 || self.height <= 0 {
            return Err(KeyMouseMacroError::InvalidCaptureArea { area: self });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MacroPlaybackContext {
    pub target_capture_area: Option<MacroCaptureArea>,
    pub target_dpi_scale: f64,
    pub working_area_width: i32,
    pub working_area_height: i32,
}

impl MacroPlaybackContext {
    pub fn for_current_capture_area(
        target_capture_area: MacroCaptureArea,
        target_dpi_scale: f64,
        working_area_width: i32,
        working_area_height: i32,
    ) -> Self {
        Self {
            target_capture_area: Some(target_capture_area),
            target_dpi_scale,
            working_area_width,
            working_area_height,
        }
    }

    fn validate(self) -> Result<()> {
        if self.working_area_width <= 0 || self.working_area_height <= 0 {
            return Err(KeyMouseMacroError::InvalidWorkingArea {
                width: self.working_area_width,
                height: self.working_area_height,
            });
        }
        if let Some(area) = self.target_capture_area {
            area.validate()?;
        }
        Ok(())
    }
}

impl Default for MacroPlaybackContext {
    fn default() -> Self {
        Self {
            target_capture_area: None,
            target_dpi_scale: 1.0,
            working_area_width: 1920,
            working_area_height: 1080,
        }
    }
}

#[cfg(test)]
#[path = "macro_tests.rs"]
mod tests;
