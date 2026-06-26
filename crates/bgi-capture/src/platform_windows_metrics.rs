use crate::{CaptureError, GameWindowMetrics, Result, WindowRect};
use std::mem::size_of;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::UI::WindowsAndMessaging::{GetClientRect, GetWindowRect};

pub(super) fn client_size(hwnd: HWND) -> Result<(u32, u32)> {
    let mut rect = RECT::default();
    unsafe {
        GetClientRect(hwnd, &mut rect)
            .map_err(|err| CaptureError::Win32(format!("GetClientRect: {err}")))?;
    }
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        return Err(CaptureError::EmptyClientArea);
    }
    Ok((width as u32, height as u32))
}

pub(super) unsafe fn game_window_metrics(hwnd: HWND) -> Result<GameWindowMetrics> {
    let (client_width, client_height) = client_size(hwnd)?;
    let extended_frame_bounds = extended_frame_bounds(hwnd)?;
    Ok(GameWindowMetrics::from_legacy_capture_rect(
        client_width,
        client_height,
        extended_frame_bounds,
    ))
}

fn extended_frame_bounds(hwnd: HWND) -> Result<WindowRect> {
    let mut rect = RECT::default();
    let result = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            (&mut rect as *mut RECT).cast(),
            size_of::<RECT>() as u32,
        )
    };

    if result.is_ok() && rect.right > rect.left && rect.bottom > rect.top {
        return Ok(window_rect_from_win32(rect));
    }

    unsafe {
        GetWindowRect(hwnd, &mut rect)
            .map_err(|err| CaptureError::Win32(format!("GetWindowRect: {err}")))?;
    }
    if rect.right <= rect.left || rect.bottom <= rect.top {
        return Err(CaptureError::Win32("window bounds are empty".to_string()));
    }
    Ok(window_rect_from_win32(rect))
}

fn window_rect_from_win32(rect: RECT) -> WindowRect {
    WindowRect::new(rect.left, rect.top, rect.right, rect.bottom)
}
