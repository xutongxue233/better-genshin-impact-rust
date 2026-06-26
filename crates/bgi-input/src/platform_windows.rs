use super::{
    is_extended_key, normalize_absolute_mouse_position, AbsoluteMouseCoordinateBounds,
    InputCancellationToken, InputDispatchReport, InputError, InputEvent, MouseButton,
    PostMessageEvent, Result,
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
    GetSystemMetrics, PostMessageW, SetForegroundWindow, ShowWindow, SM_CXSCREEN,
    SM_CXVIRTUALSCREEN, SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
    SW_RESTORE,
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
            let (normalized_x, normalized_y) =
                normalize_absolute_mouse_position(x, y, mouse_absolute_bounds(virtual_desktop));
            vec![mouse_input(normalized_x, normalized_y, 0, flags)]
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

fn mouse_absolute_bounds(virtual_desktop: bool) -> AbsoluteMouseCoordinateBounds {
    if virtual_desktop {
        AbsoluteMouseCoordinateBounds::new(
            unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) },
            unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) },
            unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) },
            unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) },
        )
    } else {
        AbsoluteMouseCoordinateBounds::new(0, 0, unsafe { GetSystemMetrics(SM_CXSCREEN) }, unsafe {
            GetSystemMetrics(SM_CYSCREEN)
        })
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
