use crate::CaptureError;

pub(super) fn win32_error(call: &'static str) -> CaptureError {
    CaptureError::Win32(call.to_string())
}
