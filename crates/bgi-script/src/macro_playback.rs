use super::{
    KeyMouseMacroError, KeyMouseScript, MacroEvent, MacroEventType, MacroPlaybackContext, Result,
};
use bgi_input::{InputEvent, InputSequence, MouseButton};
use std::str::FromStr;

impl KeyMouseScript {
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
