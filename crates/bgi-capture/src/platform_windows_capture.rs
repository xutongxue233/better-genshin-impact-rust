use crate::{
    CaptureError, CaptureFrame, CaptureMode, CaptureSettings, GameCapture, PixelFormat, Result,
    WindowHandle,
};
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{copy_nonoverlapping, null_mut};
use std::time::{Duration, Instant};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush, GetDC,
    GetDeviceCaps, GetObjectW, ReleaseDC, SelectObject, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BITSPIXEL, BI_RGB, CLIPCAPS, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ, PLANES, RASTERCAPS,
    RC_BITBLT, SRCCOPY,
};

use super::metrics::client_size;
use super::registry::set_directx_user_global_settings;
use super::win32::win32_error;

pub enum Backend {
    BitBlt(BitBltCapture),
    Unimplemented(CaptureMode),
}

impl Backend {
    pub fn new(mode: CaptureMode) -> Result<Self> {
        match mode {
            CaptureMode::BitBlt => Ok(Self::BitBlt(BitBltCapture::default())),
            other => Ok(Self::Unimplemented(other)),
        }
    }
}

impl GameCapture for Backend {
    fn mode(&self) -> CaptureMode {
        match self {
            Self::BitBlt(_) => CaptureMode::BitBlt,
            Self::Unimplemented(mode) => *mode,
        }
    }

    fn is_capturing(&self) -> bool {
        match self {
            Self::BitBlt(capture) => capture.is_capturing(),
            Self::Unimplemented(_) => false,
        }
    }

    fn start(&mut self, hwnd: WindowHandle, settings: CaptureSettings) -> Result<()> {
        match self {
            Self::BitBlt(capture) => capture.start(hwnd, settings),
            Self::Unimplemented(mode) => Err(CaptureError::ModeNotImplemented(*mode)),
        }
    }

    fn capture(&mut self) -> Result<Option<CaptureFrame>> {
        match self {
            Self::BitBlt(capture) => capture.capture(),
            Self::Unimplemented(mode) => Err(CaptureError::ModeNotImplemented(*mode)),
        }
    }

    fn stop(&mut self) {
        if let Self::BitBlt(capture) = self {
            capture.stop();
        }
    }
}

#[derive(Default)]
pub struct BitBltCapture {
    hwnd: Option<HWND>,
    session: Option<BitBltSession>,
    is_capturing: bool,
    last_capture_failed: bool,
    last_session_check: Option<Instant>,
    session_check_interval: Duration,
}

impl GameCapture for BitBltCapture {
    fn mode(&self) -> CaptureMode {
        CaptureMode::BitBlt
    }

    fn is_capturing(&self) -> bool {
        self.is_capturing
    }

    fn start(&mut self, hwnd: WindowHandle, settings: CaptureSettings) -> Result<()> {
        let hwnd = HWND(hwnd.0 as *mut c_void);
        if hwnd.is_invalid() {
            return Err(CaptureError::InvalidWindowHandle);
        }
        if settings.auto_fix_win11_bit_blt {
            set_directx_user_global_settings()?;
        }
        self.stop();
        self.hwnd = Some(hwnd);
        self.session_check_interval = settings.session_check_interval;
        self.is_capturing = true;
        self.check_session()
    }

    fn capture(&mut self) -> Result<Option<CaptureFrame>> {
        if self.hwnd.is_none() {
            return Ok(None);
        }

        if self.should_check_session() {
            self.check_session()?;
        }

        match self.capture_once() {
            Ok(frame) => {
                self.last_capture_failed = false;
                Ok(frame)
            }
            Err(_err) if !self.last_capture_failed => {
                self.last_capture_failed = true;
                self.check_session()?;
                self.capture_once()
            }
            Err(err) => Err(err),
        }
    }

    fn stop(&mut self) {
        self.session = None;
        self.hwnd = None;
        self.is_capturing = false;
        self.last_capture_failed = false;
        self.last_session_check = None;
    }
}

impl BitBltCapture {
    fn should_check_session(&self) -> bool {
        self.last_capture_failed
            || self
                .last_session_check
                .map(|checked| checked.elapsed() >= self.session_check_interval)
                .unwrap_or(true)
    }

    fn check_session(&mut self) -> Result<()> {
        let hwnd = self.hwnd.ok_or(CaptureError::InvalidWindowHandle)?;
        let size = client_size(hwnd)?;
        self.last_session_check = Some(Instant::now());

        if let Some(session) = &self.session {
            if session.width == size.0 && session.height == size.1 {
                return Ok(());
            }
        }

        self.session = Some(BitBltSession::new(hwnd, size.0, size.1)?);
        Ok(())
    }

    fn capture_once(&mut self) -> Result<Option<CaptureFrame>> {
        self.session
            .as_ref()
            .map(BitBltSession::capture)
            .transpose()
    }
}

struct BitBltSession {
    hwnd: HWND,
    source_dc: HDC,
    memory_dc: HDC,
    bitmap: HBITMAP,
    old_bitmap: HGDIOBJ,
    bits: *mut c_void,
    width: u32,
    height: u32,
    dib_stride: usize,
}

impl BitBltSession {
    fn new(hwnd: HWND, width: u32, height: u32) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(CaptureError::EmptyClientArea);
        }

        unsafe {
            let source_dc = GetDC(Some(hwnd));
            if source_dc.is_invalid() {
                return Err(win32_error("GetDC"));
            }

            let mut session = Self {
                hwnd,
                source_dc,
                memory_dc: HDC::default(),
                bitmap: HBITMAP::default(),
                old_bitmap: HGDIOBJ::default(),
                bits: null_mut(),
                width,
                height,
                dib_stride: 0,
            };

            if let Err(err) = session.init_dib() {
                session.release();
                return Err(err);
            }

            Ok(session)
        }
    }

    unsafe fn init_dib(&mut self) -> Result<()> {
        let raster_caps = GetDeviceCaps(Some(self.source_dc), RASTERCAPS);
        if raster_caps & RC_BITBLT as i32 == 0 {
            return Err(CaptureError::Win32(
                "source device does not support BitBlt".to_string(),
            ));
        }

        let bits_pixel = GetDeviceCaps(Some(self.source_dc), BITSPIXEL);
        if bits_pixel != 24 && bits_pixel != 32 {
            return Err(CaptureError::Win32(format!(
                "unsupported source color depth {bits_pixel}"
            )));
        }

        let planes = GetDeviceCaps(Some(self.source_dc), PLANES);
        if planes > 1 {
            return Err(CaptureError::Win32(format!(
                "unsupported source plane count {planes}"
            )));
        }

        if GetDeviceCaps(Some(self.source_dc), CLIPCAPS) == 0 {
            return Err(CaptureError::Win32(
                "source device does not support clipping".to_string(),
            ));
        }

        self.memory_dc = CreateCompatibleDC(Some(self.source_dc));
        if self.memory_dc.is_invalid() {
            return Err(win32_error("CreateCompatibleDC"));
        }

        let bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: self.width as i32,
                biHeight: -(self.height as i32),
                biPlanes: 1,
                biBitCount: 24,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: Default::default(),
        };

        self.bitmap = CreateDIBSection(
            Some(self.memory_dc),
            &bitmap_info,
            DIB_RGB_COLORS,
            &mut self.bits,
            None,
            0,
        )
        .map_err(|err| CaptureError::Win32(format!("CreateDIBSection: {err}")))?;
        if self.bitmap.is_invalid() || self.bits.is_null() {
            return Err(win32_error("CreateDIBSection"));
        }

        let mut bitmap: BITMAP = zeroed();
        let read = GetObjectW(
            HGDIOBJ(self.bitmap.0),
            size_of::<BITMAP>() as i32,
            Some((&mut bitmap as *mut BITMAP).cast()),
        );
        if read == 0 {
            return Err(win32_error("GetObjectW"));
        }
        if bitmap.bmPlanes != 1 || bitmap.bmBitsPixel != 24 {
            return Err(CaptureError::Win32(
                "created bitmap is not BGR24".to_string(),
            ));
        }
        self.dib_stride = bitmap.bmWidthBytes as usize;

        self.old_bitmap = SelectObject(self.memory_dc, HGDIOBJ(self.bitmap.0));
        if self.old_bitmap.is_invalid() {
            return Err(win32_error("SelectObject"));
        }

        let _ = GdiFlush();
        Ok(())
    }

    fn capture(&self) -> Result<CaptureFrame> {
        unsafe {
            BitBlt(
                self.memory_dc,
                0,
                0,
                self.width as i32,
                self.height as i32,
                Some(self.source_dc),
                0,
                0,
                SRCCOPY,
            )
            .map_err(|err| CaptureError::Win32(format!("BitBlt: {err}")))?;

            if !GdiFlush().as_bool() {
                return Err(win32_error("GdiFlush"));
            }

            let packed_stride = self.width as usize * PixelFormat::Bgr8.bytes_per_pixel();
            let mut pixels = vec![0u8; packed_stride * self.height as usize];
            let source = self.bits.cast::<u8>();

            for row in 0..self.height as usize {
                copy_nonoverlapping(
                    source.add(self.dib_stride * row),
                    pixels.as_mut_ptr().add(packed_stride * row),
                    packed_stride,
                );
            }

            CaptureFrame::packed_bgr(self.width, self.height, pixels)
        }
    }

    unsafe fn release(&mut self) {
        let _ = GdiFlush();

        if !self.old_bitmap.is_invalid() && !self.memory_dc.is_invalid() {
            let _ = SelectObject(self.memory_dc, self.old_bitmap);
            self.old_bitmap = HGDIOBJ::default();
        }

        if !self.bitmap.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(self.bitmap.0));
            self.bitmap = HBITMAP::default();
        }

        if !self.memory_dc.is_invalid() {
            let _ = DeleteDC(self.memory_dc);
            self.memory_dc = HDC::default();
        }

        if !self.source_dc.is_invalid() {
            let _ = ReleaseDC(Some(self.hwnd), self.source_dc);
            self.source_dc = HDC::default();
        }

        self.bits = null_mut();
    }
}

impl Drop for BitBltSession {
    fn drop(&mut self) {
        unsafe {
            self.release();
        }
    }
}
