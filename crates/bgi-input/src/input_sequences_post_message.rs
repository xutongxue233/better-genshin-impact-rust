use bgi_core::{GenshinAction, KeyBindingsConfig};

use crate::{
    post_message_events_for_action, send_post_messages, KeyActionType, PostMessageEvent,
    PostMessageMode, Result, DEFAULT_HOLD_MILLISECONDS, POST_MESSAGE_CLICK_DELAY_MILLISECONDS,
    POST_MESSAGE_KEYDOWN_LPARAM, POST_MESSAGE_KEYUP_LPARAM, WM_ACTIVATE, WM_CHAR, WM_KEYDOWN,
    WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP,
};

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
