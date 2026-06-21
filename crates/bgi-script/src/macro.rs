use bgi_input::{InputEvent, InputSequence, MouseButton};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

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

    pub fn to_input_sequence(&self, context: MacroPlaybackContext) -> Result<InputSequence> {
        context.validate()?;
        if context.target_capture_area.is_some() {
            self.info
                .as_ref()
                .ok_or(KeyMouseMacroError::MissingScriptInfo)?
                .validate()?;
        }

        let mut sequence = InputSequence::new();
        let mut previous_time = 0.0_f64;

        for (index, event) in self.macro_events.iter().enumerate() {
            let event_time = event.time.max(0.0);
            let delay_ms = (event_time - previous_time).max(0.0).trunc() as u64;
            if delay_ms > 0 {
                sequence = sequence.delay(delay_ms);
            }
            previous_time = previous_time.max(event_time);

            let adapted = self.adapt_event(event, context)?;
            sequence = append_macro_event(sequence, &adapted, index, context)?;
        }

        Ok(sequence)
    }

    pub fn to_input_events(&self, context: MacroPlaybackContext) -> Result<Vec<InputEvent>> {
        Ok(self.to_input_sequence(context)?.events().to_vec())
    }

    fn adapt_event(&self, event: &MacroEvent, context: MacroPlaybackContext) -> Result<MacroEvent> {
        let Some(area) = context.target_capture_area else {
            return Ok(event.clone());
        };
        area.validate()?;
        let info = self
            .info
            .as_ref()
            .ok_or(KeyMouseMacroError::MissingScriptInfo)?;
        info.validate()?;

        let mut adapted = event.clone();
        match adapted.event_type {
            MacroEventType::MouseMoveTo | MacroEventType::MouseDown | MacroEventType::MouseUp => {
                adapted.mouse_x = trunc_to_i32(
                    area.x as f64
                        + (event.mouse_x - info.x) as f64 * area.width as f64 / info.width as f64,
                );
                adapted.mouse_y = trunc_to_i32(
                    area.y as f64
                        + (event.mouse_y - info.y) as f64 * area.height as f64 / info.height as f64,
                );
            }
            MacroEventType::MouseMoveBy => {
                adapted.mouse_x = round_midpoint_to_even(
                    event.mouse_x as f64 / info.record_dpi * context.target_dpi_scale,
                );
                adapted.mouse_y = round_midpoint_to_even(
                    event.mouse_y as f64 / info.record_dpi * context.target_dpi_scale,
                );
            }
            MacroEventType::KeyDown | MacroEventType::KeyUp | MacroEventType::MouseWheel => {}
        }

        Ok(adapted)
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

fn append_macro_event(
    sequence: InputSequence,
    event: &MacroEvent,
    index: usize,
    context: MacroPlaybackContext,
) -> Result<InputSequence> {
    match event.event_type {
        MacroEventType::KeyDown => {
            let vk = required_key_code(event, index)?;
            Ok(sequence.key_down(vk))
        }
        MacroEventType::KeyUp => {
            let vk = required_key_code(event, index)?;
            Ok(sequence.key_up(vk))
        }
        MacroEventType::MouseMoveTo => Ok(sequence.move_mouse_to(
            normalized_absolute_x(event.mouse_x, context),
            normalized_absolute_y(event.mouse_y, context),
        )),
        MacroEventType::MouseMoveBy => Ok(sequence.move_mouse_by(event.mouse_x, event.mouse_y)),
        MacroEventType::MouseDown => {
            let Some(button) = required_mouse_button(event, index)?.to_input_button() else {
                return Ok(sequence);
            };
            Ok(sequence
                .move_mouse_to(
                    normalized_absolute_x(event.mouse_x, context),
                    normalized_absolute_y(event.mouse_y, context),
                )
                .mouse_down(button))
        }
        MacroEventType::MouseUp => {
            let Some(button) = required_mouse_button(event, index)?.to_input_button() else {
                return Ok(sequence);
            };
            Ok(sequence
                .move_mouse_to(
                    normalized_absolute_x(event.mouse_x, context),
                    normalized_absolute_y(event.mouse_y, context),
                )
                .mouse_up(button))
        }
        MacroEventType::MouseWheel => {
            let clicks = event.mouse_y / 120;
            if clicks == 0 {
                Ok(sequence)
            } else {
                Ok(sequence.vertical_scroll(clicks))
            }
        }
    }
}

fn required_key_code(event: &MacroEvent, index: usize) -> Result<u16> {
    event.key_code.ok_or(KeyMouseMacroError::MissingKeyCode {
        index,
        event_type: event.event_type,
    })
}

fn required_mouse_button(event: &MacroEvent, index: usize) -> Result<MacroMouseButton> {
    let value = event
        .mouse_button
        .as_ref()
        .ok_or(KeyMouseMacroError::MissingMouseButton {
            index,
            event_type: event.event_type,
        })?;

    value
        .parse::<MacroMouseButton>()
        .map_err(|_| KeyMouseMacroError::InvalidMouseButton {
            index,
            value: value.clone(),
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MacroMouseButton {
    None,
    Left,
    Right,
    Middle,
    XButton1,
    XButton2,
}

impl MacroMouseButton {
    fn to_input_button(self) -> Option<MouseButton> {
        match self {
            Self::Left => Some(MouseButton::Left),
            Self::Right => Some(MouseButton::Right),
            Self::Middle => Some(MouseButton::Middle),
            Self::None | Self::XButton1 | Self::XButton2 => None,
        }
    }
}

impl FromStr for MacroMouseButton {
    type Err = ();

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "None" => Ok(Self::None),
            "Left" => Ok(Self::Left),
            "Right" => Ok(Self::Right),
            "Middle" => Ok(Self::Middle),
            "XButton1" => Ok(Self::XButton1),
            "XButton2" => Ok(Self::XButton2),
            _ => Err(()),
        }
    }
}

fn normalized_absolute_x(x: i32, context: MacroPlaybackContext) -> i32 {
    trunc_to_i32(x as f64 * 65_535.0 / context.working_area_width as f64)
}

fn normalized_absolute_y(y: i32, context: MacroPlaybackContext) -> i32 {
    trunc_to_i32(y as f64 * 65_535.0 / context.working_area_height as f64)
}

fn trunc_to_i32(value: f64) -> i32 {
    value.trunc().clamp(i32::MIN as f64, i32::MAX as f64) as i32
}

fn round_midpoint_to_even(value: f64) -> i32 {
    let floor = value.floor();
    let ceil = value.ceil();
    let distance_to_floor = (value - floor).abs();
    let distance_to_ceil = (ceil - value).abs();

    if (distance_to_floor - 0.5).abs() < f64::EPSILON
        && (distance_to_ceil - 0.5).abs() < f64::EPSILON
    {
        let floor_i = floor as i64;
        let ceil_i = ceil as i64;
        let rounded = if floor_i % 2 == 0 { floor_i } else { ceil_i };
        return rounded.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    }

    trunc_to_i32(value.round())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_legacy_numeric_event_types() {
        let script = KeyMouseScript::from_json(
            r#"{
              "macroEvents": [
                { "type": 0, "keyCode": 87, "time": 100 },
                { "type": 1, "keyCode": 87, "time": 150 }
              ],
              "info": { "x": 557, "y": 57, "width": 1920, "height": 1080, "recordDpi": 1 }
            }"#,
        )
        .unwrap();

        assert_eq!(script.macro_events[0].event_type, MacroEventType::KeyDown);
        assert_eq!(script.macro_events[1].event_type, MacroEventType::KeyUp);
        assert_eq!(script.info.unwrap().width, 1920);
    }

    #[test]
    fn accepts_string_event_types_for_forward_compatibility() {
        let event: MacroEvent =
            serde_json::from_str(r#"{ "type": "MouseWheel", "mouseY": -120, "time": 0 }"#).unwrap();

        assert_eq!(event.event_type, MacroEventType::MouseWheel);
    }

    #[test]
    fn converts_key_macro_to_timed_input_sequence() {
        let script = KeyMouseScript::from_json(
            r#"{
              "macroEvents": [
                { "type": 0, "keyCode": 87, "time": 100 },
                { "type": 1, "keyCode": 87, "time": 175 }
              ]
            }"#,
        )
        .unwrap();

        let events = script
            .to_input_events(MacroPlaybackContext::default())
            .unwrap();

        assert_eq!(
            events,
            vec![
                InputEvent::Delay { milliseconds: 100 },
                InputEvent::KeyDown {
                    vk: 87,
                    extended: None
                },
                InputEvent::Delay { milliseconds: 75 },
                InputEvent::KeyUp {
                    vk: 87,
                    extended: None
                }
            ]
        );
    }

    #[test]
    fn adapts_absolute_mouse_events_to_target_capture_area() {
        let script = KeyMouseScript::from_json(
            r#"{
              "macroEvents": [
                { "type": 4, "mouseButton": "Left", "mouseX": 150, "mouseY": 100, "time": 0 }
              ],
              "info": { "x": 100, "y": 50, "width": 200, "height": 100, "recordDpi": 1 }
            }"#,
        )
        .unwrap();
        let context = MacroPlaybackContext::for_current_capture_area(
            MacroCaptureArea {
                x: 0,
                y: 0,
                width: 400,
                height: 200,
            },
            1.0,
            800,
            600,
        );

        let events = script.to_input_events(context).unwrap();

        assert_eq!(
            events,
            vec![
                InputEvent::MouseMoveAbsolute {
                    x: 8191,
                    y: 10922,
                    virtual_desktop: false
                },
                InputEvent::MouseButtonDown {
                    button: MouseButton::Left
                }
            ]
        );
    }

    #[test]
    fn adapts_relative_mouse_events_with_legacy_rounding() {
        let script = KeyMouseScript::from_json(
            r#"{
              "macroEvents": [
                { "type": 3, "mouseX": 13, "mouseY": -5, "time": 0 }
              ],
              "info": { "x": 0, "y": 0, "width": 1920, "height": 1080, "recordDpi": 2 }
            }"#,
        )
        .unwrap();
        let context = MacroPlaybackContext::for_current_capture_area(
            MacroCaptureArea {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            1.0,
            1920,
            1080,
        );

        let events = script.to_input_events(context).unwrap();

        assert_eq!(
            events,
            vec![InputEvent::MouseMoveRelative { dx: 6, dy: -2 }]
        );
    }

    #[test]
    fn mouse_wheel_uses_legacy_click_delta_conversion() {
        let script = KeyMouseScript::from_json(
            r#"{
              "macroEvents": [
                { "type": 6, "mouseY": -240, "time": 0 }
              ]
            }"#,
        )
        .unwrap();

        let events = script
            .to_input_events(MacroPlaybackContext::default())
            .unwrap();

        assert_eq!(
            events,
            vec![InputEvent::MouseWheel {
                amount: -240,
                horizontal: false
            }]
        );
    }
}
