use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum HotkeyError {
    #[error("invalid hotkey")]
    InvalidHotkey,
    #[error("hotkey registration is only implemented on Windows")]
    UnsupportedPlatform,
    #[error("hotkey is already registered")]
    AlreadyRegistered,
    #[error("hotkey registration failed: {0}")]
    RegisterFailed(String),
}

pub type Result<T> = std::result::Result<T, HotkeyError>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modifiers {
    pub alt: bool,
    pub control: bool,
    pub shift: bool,
    pub windows: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hotkey {
    pub modifiers: Modifiers,
    pub vk: u16,
}

impl Hotkey {
    pub fn new(modifiers: Modifiers, vk: u16) -> Result<Self> {
        if vk == 0 || is_modifier_key(vk) {
            return Err(HotkeyError::InvalidHotkey);
        }
        Ok(Self { modifiers, vk })
    }
}

impl FromStr for Hotkey {
    type Err = HotkeyError;

    fn from_str(value: &str) -> Result<Self> {
        let mut modifiers = Modifiers::default();
        let mut key = None;

        for part in value
            .split('+')
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            match part.to_ascii_lowercase().as_str() {
                "win" | "windows" => modifiers.windows = true,
                "ctrl" | "control" => modifiers.control = true,
                "shift" => modifiers.shift = true,
                "alt" => modifiers.alt = true,
                other => {
                    key = Some(parse_key(other).ok_or(HotkeyError::InvalidHotkey)?);
                }
            }
        }

        Hotkey::new(modifiers, key.ok_or(HotkeyError::InvalidHotkey)?)
    }
}

impl fmt::Display for Hotkey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.modifiers.windows {
            parts.push("Win".to_string());
        }
        if self.modifiers.control {
            parts.push("Ctrl".to_string());
        }
        if self.modifiers.shift {
            parts.push("Shift".to_string());
        }
        if self.modifiers.alt {
            parts.push("Alt".to_string());
        }
        parts.push(format_key(self.vk));
        write!(formatter, "{}", parts.join(" + "))
    }
}

pub trait HotkeyHandler: Send + 'static {
    fn on_hotkey(&mut self, hotkey: Hotkey);
}

impl<F> HotkeyHandler for F
where
    F: FnMut(Hotkey) + Send + 'static,
{
    fn on_hotkey(&mut self, hotkey: Hotkey) {
        self(hotkey);
    }
}

pub struct HotkeyHook {
    inner: platform::HotkeyHook,
}

impl HotkeyHook {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: platform::HotkeyHook::new()?,
        })
    }

    pub fn register(&mut self, hotkey: Hotkey) -> Result<()> {
        self.inner.register(hotkey)
    }

    pub fn unregister_all(&mut self) {
        self.inner.unregister_all();
    }
}

impl Drop for HotkeyHook {
    fn drop(&mut self) {
        self.unregister_all();
    }
}

fn is_modifier_key(vk: u16) -> bool {
    matches!(
        vk,
        0x10 | 0x11 | 0x12 | 0x5B | 0x5C | 0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5
    )
}

fn parse_key(value: &str) -> Option<u16> {
    match value {
        "enter" | "return" => Some(0x0D),
        "escape" | "esc" => Some(0x1B),
        "space" => Some(0x20),
        "tab" => Some(0x09),
        "delete" | "del" => Some(0x2E),
        "insert" | "ins" => Some(0x2D),
        "home" => Some(0x24),
        "end" => Some(0x23),
        "pageup" | "prior" => Some(0x21),
        "pagedown" | "next" => Some(0x22),
        "left" => Some(0x25),
        "up" => Some(0x26),
        "right" => Some(0x27),
        "down" => Some(0x28),
        key if key.len() == 1 => {
            let ch = key.chars().next()?.to_ascii_uppercase();
            if ch.is_ascii_alphanumeric() {
                Some(ch as u16)
            } else {
                None
            }
        }
        key if key.starts_with('f') => {
            let number = key[1..].parse::<u16>().ok()?;
            if (1..=24).contains(&number) {
                Some(0x70 + number - 1)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn format_key(vk: u16) -> String {
    match vk {
        0x0D => "Enter".to_string(),
        0x1B => "Escape".to_string(),
        0x20 => "Space".to_string(),
        0x09 => "Tab".to_string(),
        0x2E => "Delete".to_string(),
        0x2D => "Insert".to_string(),
        0x24 => "Home".to_string(),
        0x23 => "End".to_string(),
        0x21 => "PageUp".to_string(),
        0x22 => "PageDown".to_string(),
        0x25 => "Left".to_string(),
        0x26 => "Up".to_string(),
        0x27 => "Right".to_string(),
        0x28 => "Down".to_string(),
        0x70..=0x87 => format!("F{}", vk - 0x70 + 1),
        0x30..=0x39 | 0x41..=0x5A => char::from_u32(vk as u32).unwrap_or('?').to_string(),
        other => format!("VK_{other:02X}"),
    }
}

#[cfg(windows)]
mod platform {
    use super::{Hotkey, HotkeyError, Result};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
        MOD_SHIFT, MOD_WIN,
    };

    const ERROR_HOTKEY_ALREADY_REGISTERED: u32 = 0x581;

    pub struct HotkeyHook {
        current_id: i32,
        registered: Vec<i32>,
    }

    impl HotkeyHook {
        pub fn new() -> Result<Self> {
            Ok(Self {
                current_id: 0,
                registered: Vec::new(),
            })
        }

        pub fn register(&mut self, hotkey: Hotkey) -> Result<()> {
            self.current_id += 1;
            let modifiers = to_windows_modifiers(hotkey) | MOD_NOREPEAT;
            match unsafe { RegisterHotKey(None, self.current_id, modifiers, hotkey.vk as u32) } {
                Ok(()) => {
                    self.registered.push(self.current_id);
                    Ok(())
                }
                Err(err) if err.code().0 == hresult_from_win32(ERROR_HOTKEY_ALREADY_REGISTERED) => {
                    Err(HotkeyError::AlreadyRegistered)
                }
                Err(err) => Err(HotkeyError::RegisterFailed(format!("Win32 error {err}"))),
            }
        }

        pub fn unregister_all(&mut self) {
            for id in self.registered.drain(..) {
                unsafe {
                    let _ = UnregisterHotKey(None, id);
                }
            }
        }
    }

    fn to_windows_modifiers(hotkey: Hotkey) -> HOT_KEY_MODIFIERS {
        let mut modifiers = HOT_KEY_MODIFIERS(0);
        if hotkey.modifiers.alt {
            modifiers |= MOD_ALT;
        }
        if hotkey.modifiers.control {
            modifiers |= MOD_CONTROL;
        }
        if hotkey.modifiers.shift {
            modifiers |= MOD_SHIFT;
        }
        if hotkey.modifiers.windows {
            modifiers |= MOD_WIN;
        }
        modifiers
    }

    const fn hresult_from_win32(error: u32) -> i32 {
        if error == 0 {
            0
        } else {
            ((error & 0x0000_FFFF) | 0x8007_0000) as i32
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::{Hotkey, HotkeyError, Result};

    pub struct HotkeyHook;

    impl HotkeyHook {
        pub fn new() -> Result<Self> {
            Err(HotkeyError::UnsupportedPlatform)
        }

        pub fn register(&mut self, _hotkey: Hotkey) -> Result<()> {
            Err(HotkeyError::UnsupportedPlatform)
        }

        pub fn unregister_all(&mut self) {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_legacy_hotkey_string() {
        let hotkey: Hotkey = "Ctrl + Shift + F11".parse().unwrap();
        assert!(hotkey.modifiers.control);
        assert!(hotkey.modifiers.shift);
        assert_eq!(hotkey.vk, 0x7A);
        assert_eq!(hotkey.to_string(), "Ctrl + Shift + F11");
    }

    #[test]
    fn rejects_modifier_only_hotkeys() {
        assert!("Ctrl + Shift".parse::<Hotkey>().is_err());
    }
}
