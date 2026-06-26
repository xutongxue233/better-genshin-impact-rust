use bgi_core::{GenshinAction, KeyBindingsConfig, KeyId};

use crate::{
    input_events_for_action, input_events_for_key, send_events, send_events_with_cancellation,
    InputEvent, KeyActionType, MouseButton, Result, DEFAULT_RELEASE_KEY_RANGE,
};

use super::input_sequences_dispatch::{InputCancellationToken, InputDispatchReport};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InputSequence {
    events: Vec<InputEvent>,
}

impl InputSequence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[InputEvent] {
        &self.events
    }

    pub fn key_down(mut self, vk: u16) -> Self {
        self.events.push(InputEvent::KeyDown { vk, extended: None });
        self
    }

    pub fn key_up(mut self, vk: u16) -> Self {
        self.events.push(InputEvent::KeyUp { vk, extended: None });
        self
    }

    pub fn key_press(self, vk: u16) -> Self {
        self.key_down(vk).key_up(vk)
    }

    pub fn key_hold(self, vk: u16, milliseconds: u64) -> Self {
        self.key_down(vk).delay(milliseconds).key_up(vk)
    }

    pub fn modified_key_stroke<I>(mut self, modifiers: I, vk: u16) -> Self
    where
        I: IntoIterator<Item = u16>,
    {
        let modifiers: Vec<u16> = modifiers.into_iter().collect();
        for modifier in &modifiers {
            self.events.push(InputEvent::KeyDown {
                vk: *modifier,
                extended: None,
            });
        }
        self.events.push(InputEvent::KeyDown { vk, extended: None });
        self.events.push(InputEvent::KeyUp { vk, extended: None });
        for modifier in modifiers.into_iter().rev() {
            self.events.push(InputEvent::KeyUp {
                vk: modifier,
                extended: None,
            });
        }
        self
    }

    pub fn text(mut self, text: &str) -> Self {
        for ch in text.chars() {
            self.events.push(InputEvent::UnicodeChar { ch });
        }
        self
    }

    pub fn move_mouse_by(mut self, dx: i32, dy: i32) -> Self {
        self.events.push(InputEvent::MouseMoveRelative { dx, dy });
        self
    }

    pub fn move_mouse_to(mut self, x: i32, y: i32) -> Self {
        self.events.push(InputEvent::MouseMoveAbsolute {
            x,
            y,
            virtual_desktop: false,
        });
        self
    }

    pub fn move_mouse_to_virtual_desktop(mut self, x: i32, y: i32) -> Self {
        self.events.push(InputEvent::MouseMoveAbsolute {
            x,
            y,
            virtual_desktop: true,
        });
        self
    }

    pub fn mouse_click(self, button: MouseButton) -> Self {
        self.mouse_down(button).mouse_up(button)
    }

    pub fn mouse_hold(self, button: MouseButton, milliseconds: u64) -> Self {
        self.mouse_down(button).delay(milliseconds).mouse_up(button)
    }

    pub fn mouse_double_click(self, button: MouseButton) -> Self {
        self.mouse_click(button).mouse_click(button)
    }

    pub fn mouse_down(mut self, button: MouseButton) -> Self {
        self.events.push(InputEvent::MouseButtonDown { button });
        self
    }

    pub fn mouse_up(mut self, button: MouseButton) -> Self {
        self.events.push(InputEvent::MouseButtonUp { button });
        self
    }

    pub fn vertical_scroll(mut self, clicks: i32) -> Self {
        self.events.push(InputEvent::MouseWheel {
            amount: clicks * 120,
            horizontal: false,
        });
        self
    }

    pub fn horizontal_scroll(mut self, clicks: i32) -> Self {
        self.events.push(InputEvent::MouseWheel {
            amount: clicks * 120,
            horizontal: true,
        });
        self
    }

    pub fn release_all_keys(self) -> Self {
        self.release_keyboard_keys(DEFAULT_RELEASE_KEY_RANGE)
            .release_mouse_buttons(DEFAULT_RELEASE_MOUSE_BUTTONS)
    }

    pub fn release_keyboard_keys<I>(mut self, keys: I) -> Self
    where
        I: IntoIterator<Item = u16>,
    {
        for vk in keys {
            self.events.push(InputEvent::KeyUp { vk, extended: None });
        }
        self
    }

    pub fn release_mouse_buttons<I>(mut self, buttons: I) -> Self
    where
        I: IntoIterator<Item = MouseButton>,
    {
        for button in buttons {
            self.events.push(InputEvent::MouseButtonUp { button });
        }
        self
    }

    pub fn delay(mut self, milliseconds: u64) -> Self {
        self.events.push(InputEvent::Delay { milliseconds });
        self
    }

    pub fn key_binding(self, key: KeyId, action_type: KeyActionType) -> Result<Self> {
        Ok(self.with_events(input_events_for_key(key, action_type)?))
    }

    pub fn genshin_action(
        self,
        bindings: &KeyBindingsConfig,
        action: GenshinAction,
        action_type: KeyActionType,
    ) -> Result<Self> {
        Ok(self.with_events(input_events_for_action(bindings, action, action_type)?))
    }

    pub fn send(&self) -> Result<()> {
        send_events(&self.events)
    }

    pub fn send_with_cancellation(
        &self,
        cancellation: &InputCancellationToken,
    ) -> Result<InputDispatchReport> {
        send_events_with_cancellation(&self.events, cancellation)
    }

    fn with_events(mut self, events: Vec<InputEvent>) -> Self {
        self.events.extend(events);
        self
    }
}

pub const DEFAULT_RELEASE_MOUSE_BUTTONS: [MouseButton; 3] =
    [MouseButton::Left, MouseButton::Right, MouseButton::Middle];

pub fn release_all_keys_sequence() -> InputSequence {
    InputSequence::new().release_all_keys()
}

pub fn release_pressed_keys_sequence<I>(keys: I) -> InputSequence
where
    I: IntoIterator<Item = u16>,
{
    InputSequence::new()
        .release_keyboard_keys(keys)
        .release_mouse_buttons(DEFAULT_RELEASE_MOUSE_BUTTONS)
}
