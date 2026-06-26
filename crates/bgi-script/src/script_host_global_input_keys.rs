use super::{Result, ScriptHostRuntimeError};
use bgi_input::{InputSequence, MouseButton};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyInputAction {
    Down,
    Up,
    Press,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParsedVirtualKey {
    Keyboard(u16),
    Mouse(MouseButton),
}

pub(super) fn key_down_sequence(key: &str) -> Result<InputSequence> {
    let key = parse_virtual_key(key)?;
    Ok(key_input_sequence(key, KeyInputAction::Down))
}

pub(super) fn key_up_sequence(key: &str) -> Result<InputSequence> {
    let key = parse_virtual_key(key)?;
    Ok(key_input_sequence(key, KeyInputAction::Up))
}

pub(super) fn key_press_sequence(key: &str) -> Result<InputSequence> {
    let key = parse_virtual_key(key)?;
    Ok(key_input_sequence(key, KeyInputAction::Press))
}

fn key_input_sequence(key: ParsedVirtualKey, action: KeyInputAction) -> InputSequence {
    match (key, action) {
        (ParsedVirtualKey::Keyboard(vk), KeyInputAction::Down) => InputSequence::new().key_down(vk),
        (ParsedVirtualKey::Keyboard(vk), KeyInputAction::Up) => InputSequence::new().key_up(vk),
        (ParsedVirtualKey::Keyboard(vk), KeyInputAction::Press) => {
            InputSequence::new().key_press(vk)
        }
        (ParsedVirtualKey::Mouse(button), KeyInputAction::Down) => {
            InputSequence::new().mouse_down(button)
        }
        (ParsedVirtualKey::Mouse(button), KeyInputAction::Up) => {
            InputSequence::new().mouse_up(button)
        }
        (ParsedVirtualKey::Mouse(button), KeyInputAction::Press) => {
            InputSequence::new().mouse_click(button)
        }
    }
}

fn parse_virtual_key(key: &str) -> Result<ParsedVirtualKey> {
    match key {
        "VK_LBUTTON" => Ok(ParsedVirtualKey::Mouse(MouseButton::Left)),
        "VK_RBUTTON" => Ok(ParsedVirtualKey::Mouse(MouseButton::Right)),
        "VK_MBUTTON" => Ok(ParsedVirtualKey::Mouse(MouseButton::Middle)),
        "VK_XBUTTON1" => Ok(ParsedVirtualKey::Mouse(MouseButton::X(1))),
        "VK_XBUTTON2" => Ok(ParsedVirtualKey::Mouse(MouseButton::X(2))),
        _ => virtual_key_code(key)
            .map(ParsedVirtualKey::Keyboard)
            .ok_or_else(|| ScriptHostRuntimeError::UnsupportedVirtualKey(key.to_string())),
    }
}

pub fn virtual_key_code_for_script(key: &str) -> Result<u16> {
    match parse_virtual_key(key)? {
        ParsedVirtualKey::Keyboard(vk) => Ok(vk),
        ParsedVirtualKey::Mouse(_) => Err(ScriptHostRuntimeError::UnsupportedVirtualKey(
            key.to_string(),
        )),
    }
}

fn virtual_key_code(key: &str) -> Option<u16> {
    let key = key.strip_prefix("VK_").unwrap_or(key);
    match key {
        "BACK" | "BACKSPACE" => Some(0x08),
        "TAB" => Some(0x09),
        "RETURN" | "ENTER" => Some(0x0D),
        "SHIFT" => Some(0x10),
        "CONTROL" | "CTRL" => Some(0x11),
        "MENU" | "ALT" => Some(0x12),
        "ESCAPE" | "ESC" => Some(0x1B),
        "SPACE" => Some(0x20),
        "PRIOR" | "PAGE_UP" => Some(0x21),
        "NEXT" | "PAGE_DOWN" => Some(0x22),
        "END" => Some(0x23),
        "HOME" => Some(0x24),
        "LEFT" => Some(0x25),
        "UP" => Some(0x26),
        "RIGHT" => Some(0x27),
        "DOWN" => Some(0x28),
        "INSERT" => Some(0x2D),
        "DELETE" => Some(0x2E),
        "LWIN" => Some(0x5B),
        "RWIN" => Some(0x5C),
        "NUMPAD0" => Some(0x60),
        "NUMPAD1" => Some(0x61),
        "NUMPAD2" => Some(0x62),
        "NUMPAD3" => Some(0x63),
        "NUMPAD4" => Some(0x64),
        "NUMPAD5" => Some(0x65),
        "NUMPAD6" => Some(0x66),
        "NUMPAD7" => Some(0x67),
        "NUMPAD8" => Some(0x68),
        "NUMPAD9" => Some(0x69),
        "MULTIPLY" => Some(0x6A),
        "ADD" => Some(0x6B),
        "SUBTRACT" => Some(0x6D),
        "DECIMAL" => Some(0x6E),
        "DIVIDE" => Some(0x6F),
        "F1" => Some(0x70),
        "F2" => Some(0x71),
        "F3" => Some(0x72),
        "F4" => Some(0x73),
        "F5" => Some(0x74),
        "F6" => Some(0x75),
        "F7" => Some(0x76),
        "F8" => Some(0x77),
        "F9" => Some(0x78),
        "F10" => Some(0x79),
        "F11" => Some(0x7A),
        "F12" => Some(0x7B),
        "LSHIFT" => Some(0xA0),
        "RSHIFT" => Some(0xA1),
        "LCONTROL" | "LCTRL" => Some(0xA2),
        "RCONTROL" | "RCTRL" => Some(0xA3),
        "LMENU" | "LALT" => Some(0xA4),
        "RMENU" | "RALT" => Some(0xA5),
        _ if key.len() == 1 => {
            let ch = key.as_bytes()[0];
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_uppercase() as u16)
            } else {
                None
            }
        }
        _ => key.parse::<u16>().ok(),
    }
}
