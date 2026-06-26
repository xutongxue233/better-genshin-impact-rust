use super::{
    InputCancellationToken, InputDispatchReport, InputError, InputEvent, PostMessageEvent, Result,
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
