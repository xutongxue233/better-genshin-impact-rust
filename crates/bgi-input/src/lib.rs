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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputDispatchReport {
    pub dispatched_events: usize,
    pub total_events: usize,
    pub cancelled: bool,
}

impl InputDispatchReport {
    pub fn completed(total_events: usize) -> Self {
        Self {
            dispatched_events: total_events,
            total_events,
            cancelled: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InputCancellationToken {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl InputCancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn reset(&self) {
        self.cancelled
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PostMessageSequence {
    events: Vec<PostMessageEvent>,
}

impl PostMessageSequence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[PostMessageEvent] {
        &self.events
    }

    pub fn message(mut self, message: u32, wparam: isize, lparam: isize) -> Self {
        self.events.push(PostMessageEvent::Message {
            message,
            wparam,
            lparam,
        });
        self
    }

    pub fn delay(mut self, milliseconds: u64) -> Self {
        self.events.push(PostMessageEvent::Delay { milliseconds });
        self
    }

    pub fn activate(self) -> Self {
        self.message(WM_ACTIVATE, 1, 0)
    }

    pub fn left_button_click(self) -> Self {
        self.left_button_click_at(16, 16)
    }

    pub fn left_button_click_at(self, x: i32, y: i32) -> Self {
        let lparam = make_lparam(x, y);
        self.message(WM_LBUTTONDOWN, 0, lparam)
            .delay(POST_MESSAGE_CLICK_DELAY_MILLISECONDS)
            .message(WM_LBUTTONUP, 0, lparam)
    }

    pub fn left_button_click_background(self) -> Self {
        self.left_button_click_background_at(16, 16)
    }

    pub fn left_button_click_background_at(self, x: i32, y: i32) -> Self {
        let lparam = make_lparam(x, y);
        self.activate()
            .message(WM_LBUTTONDOWN, 1, lparam)
            .delay(POST_MESSAGE_CLICK_DELAY_MILLISECONDS)
            .message(WM_LBUTTONUP, 0, lparam)
    }

    pub fn left_button_down(self) -> Self {
        self.message(WM_LBUTTONDOWN, 0, 0)
    }

    pub fn left_button_up(self) -> Self {
        self.message(WM_LBUTTONUP, 0, 0)
    }

    pub fn right_button_click(self) -> Self {
        let lparam = make_lparam(16, 16);
        self.message(WM_RBUTTONDOWN, 0, lparam)
            .delay(POST_MESSAGE_CLICK_DELAY_MILLISECONDS)
            .message(WM_RBUTTONUP, 0, lparam)
    }

    pub fn right_button_down(self) -> Self {
        self.message(WM_RBUTTONDOWN, 0, 0)
    }

    pub fn right_button_up(self) -> Self {
        self.message(WM_RBUTTONUP, 0, 0)
    }

    pub fn key_press(self, vk: u16) -> Self {
        self.key_down(vk)
            .message(WM_CHAR, vk as isize, POST_MESSAGE_KEYDOWN_LPARAM)
            .key_up(vk)
    }

    pub fn key_press_with_delay(self, vk: u16, milliseconds: u64) -> Self {
        self.key_down(vk)
            .delay(milliseconds)
            .message(WM_CHAR, vk as isize, POST_MESSAGE_KEYDOWN_LPARAM)
            .key_up(vk)
    }

    pub fn long_key_press(self, vk: u16) -> Self {
        self.key_press_with_delay(vk, DEFAULT_HOLD_MILLISECONDS)
    }

    pub fn key_down(self, vk: u16) -> Self {
        self.message(WM_KEYDOWN, vk as isize, POST_MESSAGE_KEYDOWN_LPARAM)
    }

    pub fn key_up(self, vk: u16) -> Self {
        self.message(WM_KEYUP, vk as isize, POST_MESSAGE_KEYUP_LPARAM)
    }

    pub fn key_press_background(self, vk: u16) -> Self {
        self.activate().key_press(vk)
    }

    pub fn key_down_background(self, vk: u16) -> Self {
        self.activate().key_down(vk)
    }

    pub fn key_up_background(self, vk: u16) -> Self {
        self.activate().key_up(vk)
    }

    pub fn genshin_action(
        self,
        bindings: &KeyBindingsConfig,
        action: GenshinAction,
        action_type: KeyActionType,
        mode: PostMessageMode,
    ) -> Result<Self> {
        Ok(self.with_events(post_message_events_for_action(
            bindings,
            action,
            action_type,
            mode,
        )))
    }

    pub fn send_to(&self, hwnd: isize) -> Result<()> {
        send_post_messages(hwnd, &self.events)
    }

    fn with_events(mut self, events: Vec<PostMessageEvent>) -> Self {
        self.events.extend(events);
        self
    }
}

pub const fn make_lparam(x: i32, y: i32) -> isize {
    (((y as u32) << 16) | ((x as u32) & 0xFFFF)) as isize
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

    Ok(sequence.events)
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

    sequence.events
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
mod platform {
    use super::{
        is_extended_key, InputCancellationToken, InputDispatchReport, InputError, InputEvent,
        MouseButton, PostMessageEvent, Result,
    };
    use std::mem::size_of;
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE,
        KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
        MAPVK_VK_TO_VSC, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN,
        MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE,
        MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK, MOUSEEVENTF_WHEEL,
        MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT, MOUSE_EVENT_FLAGS, VIRTUAL_KEY,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        PostMessageW, SetForegroundWindow, ShowWindow, SW_RESTORE,
    };

    const XBUTTON1_DATA: i32 = 0x0001;
    const XBUTTON2_DATA: i32 = 0x0002;

    pub fn currently_pressed_keys() -> Result<Vec<u16>> {
        Ok(super::DEFAULT_RELEASE_KEY_RANGE
            .filter(|vk| unsafe { GetAsyncKeyState(*vk as i32) } & i16::MIN != 0)
            .collect())
    }

    pub fn activate_window(hwnd: isize) -> Result<()> {
        let hwnd = HWND(hwnd as *mut _);
        unsafe {
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
        }
        Ok(())
    }

    pub fn send_post_messages(hwnd: isize, events: &[PostMessageEvent]) -> Result<()> {
        let hwnd = HWND(hwnd as *mut _);
        for event in events {
            match *event {
                PostMessageEvent::Message {
                    message,
                    wparam,
                    lparam,
                } => {
                    unsafe {
                        PostMessageW(Some(hwnd), message, WPARAM(wparam as usize), LPARAM(lparam))
                    }
                    .map_err(|error| InputError::PostMessageDispatch {
                        message,
                        details: error.to_string(),
                    })?;
                }
                PostMessageEvent::Delay { milliseconds } => {
                    std::thread::sleep(std::time::Duration::from_millis(milliseconds));
                }
            }
        }

        Ok(())
    }

    pub fn send_events(events: &[InputEvent]) -> Result<()> {
        let mut pending = Vec::new();
        for event in events {
            if let InputEvent::Delay { milliseconds } = *event {
                dispatch_inputs(&pending)?;
                pending.clear();
                std::thread::sleep(std::time::Duration::from_millis(milliseconds));
            } else {
                pending.extend(to_inputs(event));
            }
        }

        dispatch_inputs(&pending)
    }

    pub fn send_events_with_cancellation(
        events: &[InputEvent],
        cancellation: &InputCancellationToken,
    ) -> Result<InputDispatchReport> {
        let mut pending = Vec::new();
        let mut processed_events = 0usize;
        for event in events {
            if cancellation.is_cancelled() {
                return Err(InputError::Cancelled {
                    dispatched_events: processed_events,
                    total_events: events.len(),
                });
            }

            if let InputEvent::Delay { milliseconds } = *event {
                dispatch_inputs(&pending)?;
                pending.clear();
                sleep_cancellable(milliseconds, cancellation, processed_events, events.len())?;
            } else {
                pending.extend(to_inputs(event));
            }
            processed_events += 1;
        }

        if cancellation.is_cancelled() {
            return Err(InputError::Cancelled {
                dispatched_events: processed_events,
                total_events: events.len(),
            });
        }
        dispatch_inputs(&pending)?;
        Ok(InputDispatchReport::completed(events.len()))
    }

    fn sleep_cancellable(
        milliseconds: u64,
        cancellation: &InputCancellationToken,
        processed_events: usize,
        total_events: usize,
    ) -> Result<()> {
        let mut remaining = milliseconds;
        while remaining > 0 {
            if cancellation.is_cancelled() {
                return Err(InputError::Cancelled {
                    dispatched_events: processed_events,
                    total_events,
                });
            }
            let chunk = remaining.min(25);
            std::thread::sleep(std::time::Duration::from_millis(chunk));
            remaining -= chunk;
        }
        if cancellation.is_cancelled() {
            return Err(InputError::Cancelled {
                dispatched_events: processed_events,
                total_events,
            });
        }
        Ok(())
    }

    fn dispatch_inputs(inputs: &[INPUT]) -> Result<()> {
        if inputs.is_empty() {
            return Ok(());
        }
        let sent = unsafe { SendInput(inputs, size_of::<INPUT>() as i32) };
        let expected = inputs.len() as u32;
        if sent != expected {
            return Err(InputError::PartialDispatch { sent, expected });
        }
        Ok(())
    }

    fn to_inputs(event: &InputEvent) -> Vec<INPUT> {
        match *event {
            InputEvent::KeyDown { vk, extended } => vec![keyboard_input(vk, false, extended)],
            InputEvent::KeyUp { vk, extended } => vec![keyboard_input(vk, true, extended)],
            InputEvent::UnicodeChar { ch } => {
                vec![unicode_input(ch, false), unicode_input(ch, true)]
            }
            InputEvent::MouseMoveRelative { dx, dy } => {
                vec![mouse_input(dx, dy, 0, MOUSEEVENTF_MOVE)]
            }
            InputEvent::MouseMoveAbsolute {
                x,
                y,
                virtual_desktop,
            } => {
                let mut flags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE;
                if virtual_desktop {
                    flags |= MOUSEEVENTF_VIRTUALDESK;
                }
                vec![mouse_input(x, y, 0, flags)]
            }
            InputEvent::MouseButtonDown { button } => vec![mouse_button(button, true)],
            InputEvent::MouseButtonUp { button } => vec![mouse_button(button, false)],
            InputEvent::MouseWheel { amount, horizontal } => {
                let flags = if horizontal {
                    MOUSEEVENTF_HWHEEL
                } else {
                    MOUSEEVENTF_WHEEL
                };
                vec![mouse_input(0, 0, amount, flags)]
            }
            InputEvent::Delay { .. } => Vec::new(),
        }
    }

    fn keyboard_input(vk: u16, key_up: bool, extended: Option<bool>) -> INPUT {
        let use_extended = extended.unwrap_or_else(|| is_extended_key(vk));
        let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as u16;
        let mut flags = KEYBD_EVENT_FLAGS(0);
        if use_extended {
            flags |= KEYEVENTF_EXTENDEDKEY;
        }
        if key_up {
            flags |= KEYEVENTF_KEYUP;
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk),
                    wScan: scan,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn unicode_input(ch: char, key_up: bool) -> INPUT {
        let mut flags = KEYEVENTF_UNICODE;
        if key_up {
            flags |= KEYEVENTF_KEYUP;
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: ch as u16,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn mouse_input(dx: i32, dy: i32, data: i32, flags: MOUSE_EVENT_FLAGS) -> INPUT {
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx,
                    dy,
                    mouseData: data as u32,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn mouse_button(button: MouseButton, down: bool) -> INPUT {
        let (data, flags) = match (button, down) {
            (MouseButton::Left, true) => (0, MOUSEEVENTF_LEFTDOWN),
            (MouseButton::Left, false) => (0, MOUSEEVENTF_LEFTUP),
            (MouseButton::Middle, true) => (0, MOUSEEVENTF_MIDDLEDOWN),
            (MouseButton::Middle, false) => (0, MOUSEEVENTF_MIDDLEUP),
            (MouseButton::Right, true) => (0, MOUSEEVENTF_RIGHTDOWN),
            (MouseButton::Right, false) => (0, MOUSEEVENTF_RIGHTUP),
            (MouseButton::X(2), true) => (XBUTTON2_DATA, MOUSEEVENTF_XDOWN),
            (MouseButton::X(2), false) => (XBUTTON2_DATA, MOUSEEVENTF_XUP),
            (MouseButton::X(_), true) => (XBUTTON1_DATA, MOUSEEVENTF_XDOWN),
            (MouseButton::X(_), false) => (XBUTTON1_DATA, MOUSEEVENTF_XUP),
        };
        mouse_input(0, 0, data, flags)
    }
}

#[cfg(not(windows))]
mod platform {
    use super::{
        InputCancellationToken, InputDispatchReport, InputError, InputEvent, PostMessageEvent,
        Result,
    };

    pub fn currently_pressed_keys() -> Result<Vec<u16>> {
        Err(InputError::UnsupportedPlatform)
    }

    pub fn activate_window(_hwnd: isize) -> Result<()> {
        Err(InputError::UnsupportedPlatform)
    }

    pub fn send_post_messages(_hwnd: isize, _events: &[PostMessageEvent]) -> Result<()> {
        Err(InputError::UnsupportedPlatform)
    }

    pub fn send_events(_events: &[InputEvent]) -> Result<()> {
        Err(InputError::UnsupportedPlatform)
    }

    pub fn send_events_with_cancellation(
        _events: &[InputEvent],
        _cancellation: &InputCancellationToken,
    ) -> Result<InputDispatchReport> {
        Err(InputError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_modified_key_sequence_in_legacy_order() {
        let events = InputSequence::new()
            .modified_key_stroke([0x11, 0x10], 0x41)
            .events()
            .to_vec();

        assert_eq!(
            events[0],
            InputEvent::KeyDown {
                vk: 0x11,
                extended: None
            }
        );
        assert_eq!(
            events[1],
            InputEvent::KeyDown {
                vk: 0x10,
                extended: None
            }
        );
        assert_eq!(
            events[2],
            InputEvent::KeyDown {
                vk: 0x41,
                extended: None
            }
        );
        assert_eq!(
            events[3],
            InputEvent::KeyUp {
                vk: 0x41,
                extended: None
            }
        );
        assert_eq!(
            events[4],
            InputEvent::KeyUp {
                vk: 0x10,
                extended: None
            }
        );
        assert_eq!(
            events[5],
            InputEvent::KeyUp {
                vk: 0x11,
                extended: None
            }
        );
    }

    #[test]
    fn wheel_scroll_uses_windows_click_delta() {
        let events = InputSequence::new().vertical_scroll(2).events().to_vec();
        assert_eq!(
            events,
            vec![InputEvent::MouseWheel {
                amount: 240,
                horizontal: false
            }]
        );
    }

    #[test]
    fn maps_keyboard_genshin_action_from_default_bindings() {
        let bindings = KeyBindingsConfig::default();
        let events = input_events_for_action(
            &bindings,
            GenshinAction::QuickUseGadget,
            KeyActionType::KeyPress,
        )
        .unwrap();

        assert_eq!(
            events,
            vec![
                InputEvent::KeyDown {
                    vk: KeyId::Z.vk(),
                    extended: None
                },
                InputEvent::KeyUp {
                    vk: KeyId::Z.vk(),
                    extended: None
                }
            ]
        );
    }

    #[test]
    fn maps_mouse_genshin_action_from_default_bindings() {
        let bindings = KeyBindingsConfig::default();
        let events = input_events_for_action(
            &bindings,
            GenshinAction::NormalAttack,
            KeyActionType::KeyPress,
        )
        .unwrap();

        assert_eq!(
            events,
            vec![
                InputEvent::MouseButtonDown {
                    button: MouseButton::Left
                },
                InputEvent::MouseButtonUp {
                    button: MouseButton::Left
                }
            ]
        );
    }

    #[test]
    fn key_down_action_emits_only_down_event() {
        let bindings = KeyBindingsConfig::default();
        let events = input_events_for_action(
            &bindings,
            GenshinAction::MoveForward,
            KeyActionType::KeyDown,
        )
        .unwrap();

        assert_eq!(
            events,
            vec![InputEvent::KeyDown {
                vk: KeyId::W.vk(),
                extended: None
            }]
        );
    }

    #[test]
    fn hold_action_preserves_legacy_one_second_duration() {
        let bindings = KeyBindingsConfig::default();
        let events = input_events_for_action(
            &bindings,
            GenshinAction::ElementalSkill,
            KeyActionType::Hold,
        )
        .unwrap();

        assert_eq!(
            events,
            vec![
                InputEvent::KeyDown {
                    vk: KeyId::E.vk(),
                    extended: None
                },
                InputEvent::Delay {
                    milliseconds: DEFAULT_HOLD_MILLISECONDS
                },
                InputEvent::KeyUp {
                    vk: KeyId::E.vk(),
                    extended: None
                }
            ]
        );
    }

    #[test]
    fn none_key_binding_is_reported_before_dispatch() {
        let error = input_events_for_key(KeyId::NONE, KeyActionType::KeyPress).unwrap_err();
        assert!(matches!(error, InputError::UnboundKeyBinding { key } if key == KeyId::NONE));
    }

    #[test]
    fn cancelled_input_dispatch_stops_before_platform_dispatch() {
        let cancellation = InputCancellationToken::new();
        cancellation.cancel();
        let events = InputSequence::new()
            .key_press(KeyId::F.vk())
            .events()
            .to_vec();

        let error = send_events_with_cancellation(&events, &cancellation).unwrap_err();

        assert!(matches!(
            error,
            InputError::Cancelled {
                dispatched_events: 0,
                total_events: 2
            }
        ));
    }

    #[test]
    fn window_targeted_input_rejects_zero_handles_before_dispatch() {
        assert!(matches!(
            activate_window(0),
            Err(InputError::InvalidWindowHandle)
        ));
        assert!(matches!(
            send_events_to_window(0, InputSequence::new().key_press(0x41).events()),
            Err(InputError::InvalidWindowHandle)
        ));
    }

    #[test]
    fn release_pressed_keys_adds_mouse_button_cleanup() {
        let events = release_pressed_keys_sequence([KeyId::W.vk(), KeyId::LEFT_SHIFT.vk()])
            .events()
            .to_vec();

        assert_eq!(
            events,
            vec![
                InputEvent::KeyUp {
                    vk: KeyId::W.vk(),
                    extended: None
                },
                InputEvent::KeyUp {
                    vk: KeyId::LEFT_SHIFT.vk(),
                    extended: None
                },
                InputEvent::MouseButtonUp {
                    button: MouseButton::Left
                },
                InputEvent::MouseButtonUp {
                    button: MouseButton::Right
                },
                InputEvent::MouseButtonUp {
                    button: MouseButton::Middle
                }
            ]
        );
    }

    #[test]
    fn release_all_keys_matches_legacy_vk_sweep_shape() {
        let sequence = release_all_keys_sequence();
        let events = sequence.events();

        assert_eq!(events.len(), 0xFE + DEFAULT_RELEASE_MOUSE_BUTTONS.len());
        assert_eq!(
            events[0],
            InputEvent::KeyUp {
                vk: 0x01,
                extended: None
            }
        );
        assert_eq!(
            events[0xFD],
            InputEvent::KeyUp {
                vk: 0xFE,
                extended: None
            }
        );
        assert_eq!(
            events.last(),
            Some(&InputEvent::MouseButtonUp {
                button: MouseButton::Middle
            })
        );
    }

    #[test]
    fn make_lparam_matches_legacy_coordinate_packing() {
        assert_eq!(make_lparam(16, 16), 0x0010_0010);
        assert_eq!(make_lparam(0x1_FFFF, 2), 0x0002_FFFF);
    }

    #[test]
    fn builds_legacy_post_message_key_press_sequence() {
        let events = PostMessageSequence::new()
            .key_press(KeyId::F.vk())
            .events()
            .to_vec();

        assert_eq!(
            events,
            vec![
                PostMessageEvent::Message {
                    message: WM_KEYDOWN,
                    wparam: KeyId::F.vk() as isize,
                    lparam: POST_MESSAGE_KEYDOWN_LPARAM
                },
                PostMessageEvent::Message {
                    message: WM_CHAR,
                    wparam: KeyId::F.vk() as isize,
                    lparam: POST_MESSAGE_KEYDOWN_LPARAM
                },
                PostMessageEvent::Message {
                    message: WM_KEYUP,
                    wparam: KeyId::F.vk() as isize,
                    lparam: POST_MESSAGE_KEYUP_LPARAM
                }
            ]
        );
    }

    #[test]
    fn builds_legacy_post_message_click_sequence() {
        let events = PostMessageSequence::new()
            .left_button_click()
            .events()
            .to_vec();

        assert_eq!(
            events,
            vec![
                PostMessageEvent::Message {
                    message: WM_LBUTTONDOWN,
                    wparam: 0,
                    lparam: make_lparam(16, 16)
                },
                PostMessageEvent::Delay {
                    milliseconds: POST_MESSAGE_CLICK_DELAY_MILLISECONDS
                },
                PostMessageEvent::Message {
                    message: WM_LBUTTONUP,
                    wparam: 0,
                    lparam: make_lparam(16, 16)
                }
            ]
        );
    }

    #[test]
    fn background_post_message_key_press_activates_first() {
        let events = PostMessageSequence::new()
            .key_press_background(KeyId::Z.vk())
            .events()
            .to_vec();

        assert_eq!(
            events.first(),
            Some(&PostMessageEvent::Message {
                message: WM_ACTIVATE,
                wparam: 1,
                lparam: 0
            })
        );
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn post_message_action_mapping_uses_configured_key_bindings() {
        let bindings = KeyBindingsConfig::default();
        let events = post_message_events_for_action(
            &bindings,
            GenshinAction::QuickUseGadget,
            KeyActionType::KeyPress,
            PostMessageMode::Background,
        );

        assert_eq!(
            events,
            vec![
                PostMessageEvent::Message {
                    message: WM_ACTIVATE,
                    wparam: 1,
                    lparam: 0
                },
                PostMessageEvent::Message {
                    message: WM_KEYDOWN,
                    wparam: KeyId::Z.vk() as isize,
                    lparam: POST_MESSAGE_KEYDOWN_LPARAM
                },
                PostMessageEvent::Message {
                    message: WM_CHAR,
                    wparam: KeyId::Z.vk() as isize,
                    lparam: POST_MESSAGE_KEYDOWN_LPARAM
                },
                PostMessageEvent::Message {
                    message: WM_KEYUP,
                    wparam: KeyId::Z.vk() as isize,
                    lparam: POST_MESSAGE_KEYUP_LPARAM
                }
            ]
        );
    }

    #[test]
    fn background_post_message_keeps_legacy_mouse_limitations() {
        let bindings = KeyBindingsConfig::default();
        let events = post_message_events_for_action(
            &bindings,
            GenshinAction::SprintMouse,
            KeyActionType::KeyPress,
            PostMessageMode::Background,
        );

        assert!(events.is_empty());
    }
}
