use bgi_core::{GenshinAction, KeyBindingsConfig, KeyId};
use serde::{Deserialize, Serialize};

pub const DEFAULT_HOLD_MILLISECONDS: u64 = 1_000;
pub const DEFAULT_RELEASE_KEY_RANGE: std::ops::RangeInclusive<u16> = 0x01..=0xFE;
pub const POST_MESSAGE_CLICK_DELAY_MILLISECONDS: u64 = 100;
pub const POST_MESSAGE_KEYDOWN_LPARAM: isize = 0x001E0001;
pub const POST_MESSAGE_KEYUP_LPARAM: isize = 0xC01E0001u32 as isize;
pub const WM_ACTIVATE: u32 = 0x0006;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_KEYUP: u32 = 0x0101;
pub const WM_CHAR: u32 = 0x0102;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_RBUTTONDOWN: u32 = 0x0204;
pub const WM_RBUTTONUP: u32 = 0x0205;
pub const ABSOLUTE_MOUSE_COORDINATE_MAX: i32 = 65_535;

#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("input simulation is only implemented on Windows")]
    UnsupportedPlatform,
    #[error("empty input event list")]
    EmptyInput,
    #[error("window handle must not be zero")]
    InvalidWindowHandle,
    #[error("key binding {key:?} does not dispatch an input event")]
    UnboundKeyBinding { key: KeyId },
    #[error(
        "genshin action {action:?} is bound to {key:?}, which does not dispatch an input event"
    )]
    UnboundAction { action: GenshinAction, key: KeyId },
    #[error("SendInput dispatched {sent} of {expected} events")]
    PartialDispatch { sent: u32, expected: u32 },
    #[error("PostMessageW failed for message {message:#x}: {details}")]
    PostMessageDispatch { message: u32, details: String },
    #[error("input dispatch was cancelled after {dispatched_events} of {total_events} events")]
    Cancelled {
        dispatched_events: usize,
        total_events: usize,
    },
}

pub type Result<T> = std::result::Result<T, InputError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    X(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyActionType {
    KeyPress,
    KeyDown,
    KeyUp,
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputEvent {
    KeyDown {
        vk: u16,
        extended: Option<bool>,
    },
    KeyUp {
        vk: u16,
        extended: Option<bool>,
    },
    UnicodeChar {
        ch: char,
    },
    MouseMoveRelative {
        dx: i32,
        dy: i32,
    },
    MouseMoveAbsolute {
        x: i32,
        y: i32,
        virtual_desktop: bool,
    },
    MouseButtonDown {
        button: MouseButton,
    },
    MouseButtonUp {
        button: MouseButton,
    },
    MouseWheel {
        amount: i32,
        horizontal: bool,
    },
    Delay {
        milliseconds: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbsoluteMouseCoordinateBounds {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

impl AbsoluteMouseCoordinateBounds {
    pub fn new(left: i32, top: i32, width: i32, height: i32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }
}

pub fn normalize_absolute_mouse_coordinate(coordinate: i32, origin: i32, span: i32) -> i32 {
    if span <= 0 {
        return 0;
    }
    let offset = (coordinate - origin).clamp(0, span);
    ((offset as i64 * ABSOLUTE_MOUSE_COORDINATE_MAX as i64) / span as i64) as i32
}

pub fn normalize_absolute_mouse_position(
    x: i32,
    y: i32,
    bounds: AbsoluteMouseCoordinateBounds,
) -> (i32, i32) {
    (
        normalize_absolute_mouse_coordinate(x, bounds.left, bounds.width),
        normalize_absolute_mouse_coordinate(y, bounds.top, bounds.height),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostMessageMode {
    Foreground,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostMessageEvent {
    Message {
        message: u32,
        wparam: isize,
        lparam: isize,
    },
    Delay {
        milliseconds: u64,
    },
}

#[path = "input_sequences.rs"]
mod sequences;

pub use sequences::*;

pub fn input_events_for_action(
    bindings: &KeyBindingsConfig,
    action: GenshinAction,
    action_type: KeyActionType,
) -> Result<Vec<InputEvent>> {
    let key = bindings.action_key(action);
    input_events_for_key(key, action_type).map_err(|error| match error {
        InputError::UnboundKeyBinding { key } => InputError::UnboundAction { action, key },
        other => other,
    })
}

pub fn input_events_for_key(key: KeyId, action_type: KeyActionType) -> Result<Vec<InputEvent>> {
    if matches!(key, KeyId::NONE | KeyId::UNKNOWN) {
        return Err(InputError::UnboundKeyBinding { key });
    }

    let sequence = if let Some(button) = mouse_button_for_key(key) {
        match action_type {
            KeyActionType::KeyPress => InputSequence::new().mouse_click(button),
            KeyActionType::KeyDown => InputSequence::new().mouse_down(button),
            KeyActionType::KeyUp => InputSequence::new().mouse_up(button),
            KeyActionType::Hold => {
                InputSequence::new().mouse_hold(button, DEFAULT_HOLD_MILLISECONDS)
            }
        }
    } else {
        let vk = key.vk();
        match action_type {
            KeyActionType::KeyPress => InputSequence::new().key_press(vk),
            KeyActionType::KeyDown => InputSequence::new().key_down(vk),
            KeyActionType::KeyUp => InputSequence::new().key_up(vk),
            KeyActionType::Hold => InputSequence::new().key_hold(vk, DEFAULT_HOLD_MILLISECONDS),
        }
    };

    Ok(sequence.events().to_vec())
}

pub fn mouse_button_for_key(key: KeyId) -> Option<MouseButton> {
    match key {
        KeyId::MOUSE_LEFT_BUTTON => Some(MouseButton::Left),
        KeyId::MOUSE_RIGHT_BUTTON => Some(MouseButton::Right),
        KeyId::MOUSE_MIDDLE_BUTTON => Some(MouseButton::Middle),
        KeyId::MOUSE_SIDE_BUTTON1 => Some(MouseButton::X(1)),
        KeyId::MOUSE_SIDE_BUTTON2 => Some(MouseButton::X(2)),
        _ => None,
    }
}

pub fn post_message_events_for_action(
    bindings: &KeyBindingsConfig,
    action: GenshinAction,
    action_type: KeyActionType,
    mode: PostMessageMode,
) -> Vec<PostMessageEvent> {
    post_message_events_for_key(bindings.action_key(action), action_type, mode)
}

pub fn post_message_events_for_key(
    key: KeyId,
    action_type: KeyActionType,
    mode: PostMessageMode,
) -> Vec<PostMessageEvent> {
    if matches!(key, KeyId::NONE | KeyId::UNKNOWN) {
        return Vec::new();
    }

    let sequence = match (mode, action_type, key) {
        (PostMessageMode::Foreground, KeyActionType::Hold, _) => {
            PostMessageSequence::new().long_key_press(key.vk())
        }
        (PostMessageMode::Foreground, KeyActionType::KeyPress, KeyId::MOUSE_LEFT_BUTTON) => {
            PostMessageSequence::new().left_button_click()
        }
        (PostMessageMode::Foreground, KeyActionType::KeyPress, KeyId::MOUSE_RIGHT_BUTTON) => {
            PostMessageSequence::new().right_button_click()
        }
        (PostMessageMode::Foreground, KeyActionType::KeyDown, KeyId::MOUSE_LEFT_BUTTON) => {
            PostMessageSequence::new().left_button_down()
        }
        (PostMessageMode::Foreground, KeyActionType::KeyDown, KeyId::MOUSE_RIGHT_BUTTON) => {
            PostMessageSequence::new().right_button_down()
        }
        (PostMessageMode::Foreground, KeyActionType::KeyUp, KeyId::MOUSE_LEFT_BUTTON) => {
            PostMessageSequence::new().left_button_up()
        }
        (PostMessageMode::Foreground, KeyActionType::KeyUp, KeyId::MOUSE_RIGHT_BUTTON) => {
            PostMessageSequence::new().right_button_up()
        }
        (PostMessageMode::Foreground, _, key) if key.is_mouse_button() => {
            PostMessageSequence::new()
        }
        (PostMessageMode::Foreground, KeyActionType::KeyPress, _) => {
            PostMessageSequence::new().key_press(key.vk())
        }
        (PostMessageMode::Foreground, KeyActionType::KeyDown, _) => {
            PostMessageSequence::new().key_down(key.vk())
        }
        (PostMessageMode::Foreground, KeyActionType::KeyUp, _) => {
            PostMessageSequence::new().key_up(key.vk())
        }
        (PostMessageMode::Background, KeyActionType::Hold, _) => PostMessageSequence::new(),
        (PostMessageMode::Background, KeyActionType::KeyPress, KeyId::MOUSE_LEFT_BUTTON) => {
            PostMessageSequence::new().left_button_click_background()
        }
        (PostMessageMode::Background, _, key) if key.is_mouse_button() => {
            PostMessageSequence::new()
        }
        (PostMessageMode::Background, KeyActionType::KeyPress, _) => {
            PostMessageSequence::new().key_press_background(key.vk())
        }
        (PostMessageMode::Background, KeyActionType::KeyDown, _) => {
            PostMessageSequence::new().key_down_background(key.vk())
        }
        (PostMessageMode::Background, KeyActionType::KeyUp, _) => {
            PostMessageSequence::new().key_up_background(key.vk())
        }
    };

    sequence.events().to_vec()
}

pub fn send_events(events: &[InputEvent]) -> Result<()> {
    if events.is_empty() {
        return Err(InputError::EmptyInput);
    }
    platform::send_events(events)
}

pub fn send_events_with_cancellation(
    events: &[InputEvent],
    cancellation: &InputCancellationToken,
) -> Result<InputDispatchReport> {
    if events.is_empty() {
        return Err(InputError::EmptyInput);
    }
    if cancellation.is_cancelled() {
        return Err(InputError::Cancelled {
            dispatched_events: 0,
            total_events: events.len(),
        });
    }
    platform::send_events_with_cancellation(events, cancellation)
}

pub fn activate_window(hwnd: isize) -> Result<()> {
    if hwnd == 0 {
        return Err(InputError::InvalidWindowHandle);
    }
    platform::activate_window(hwnd)
}

pub fn send_events_to_window(hwnd: isize, events: &[InputEvent]) -> Result<()> {
    if hwnd == 0 {
        return Err(InputError::InvalidWindowHandle);
    }
    if events.is_empty() {
        return Err(InputError::EmptyInput);
    }
    platform::activate_window(hwnd)?;
    platform::send_events(events)
}

pub fn send_events_to_window_with_cancellation(
    hwnd: isize,
    events: &[InputEvent],
    cancellation: &InputCancellationToken,
) -> Result<InputDispatchReport> {
    if hwnd == 0 {
        return Err(InputError::InvalidWindowHandle);
    }
    if events.is_empty() {
        return Err(InputError::EmptyInput);
    }
    if cancellation.is_cancelled() {
        return Err(InputError::Cancelled {
            dispatched_events: 0,
            total_events: events.len(),
        });
    }
    platform::activate_window(hwnd)?;
    platform::send_events_with_cancellation(events, cancellation)
}

pub fn send_post_messages(hwnd: isize, events: &[PostMessageEvent]) -> Result<()> {
    if hwnd == 0 {
        return Err(InputError::InvalidWindowHandle);
    }
    if events.is_empty() {
        return Err(InputError::EmptyInput);
    }
    platform::send_post_messages(hwnd, events)
}

pub fn currently_pressed_keys() -> Result<Vec<u16>> {
    platform::currently_pressed_keys()
}

pub fn release_currently_pressed_keys_sequence() -> Result<InputSequence> {
    Ok(release_pressed_keys_sequence(currently_pressed_keys()?))
}

pub fn is_extended_key(vk: u16) -> bool {
    matches!(
        vk,
        0x12 | 0xA4
            | 0xA5
            | 0x11
            | 0xA3
            | 0x2D
            | 0x2E
            | 0x24
            | 0x23
            | 0x21
            | 0x22
            | 0x27
            | 0x26
            | 0x25
            | 0x28
            | 0x90
            | 0x03
            | 0x2C
            | 0x6F
    )
}

#[cfg(windows)]
#[path = "platform_windows.rs"]
mod platform;

#[cfg(not(windows))]
#[path = "platform_stub.rs"]
mod platform;

#[cfg(test)]
mod tests;
