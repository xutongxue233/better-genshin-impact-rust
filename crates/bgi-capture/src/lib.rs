use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("capture mode {0} is not implemented yet")]
    ModeNotImplemented(CaptureMode),
    #[error("unknown capture mode: {0}")]
    UnknownCaptureMode(String),
    #[error("screen capture is only implemented on Windows")]
    UnsupportedPlatform,
    #[error("window handle must not be zero")]
    InvalidWindowHandle,
    #[error("window client area is empty")]
    EmptyClientArea,
    #[error("frame dimensions must be positive")]
    InvalidFrameDimensions,
    #[error("frame data length {actual} does not match expected length {expected}")]
    InvalidFrameDataLength { expected: usize, actual: usize },
    #[error("frame stride {stride} is smaller than one row of {row_bytes} bytes")]
    InvalidFrameStride { stride: usize, row_bytes: usize },
    #[error("Win32 capture call failed: {0}")]
    Win32(String),
}

pub type Result<T> = std::result::Result<T, CaptureError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureMode {
    BitBlt,
    DwmGetDxSharedSurface,
    WindowsGraphicsCapture,
    WindowsGraphicsCaptureHdr,
}

impl CaptureMode {
    pub const ALL: [Self; 4] = [
        Self::BitBlt,
        Self::DwmGetDxSharedSurface,
        Self::WindowsGraphicsCapture,
        Self::WindowsGraphicsCaptureHdr,
    ];

    pub fn legacy_value(self) -> u8 {
        match self {
            Self::BitBlt => 0,
            Self::DwmGetDxSharedSurface => 2,
            Self::WindowsGraphicsCapture => 1,
            Self::WindowsGraphicsCaptureHdr => 3,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::BitBlt => "BitBlt",
            Self::DwmGetDxSharedSurface => "DwmGetDxSharedSurface",
            Self::WindowsGraphicsCapture => "WindowsGraphicsCapture",
            Self::WindowsGraphicsCaptureHdr => "WindowsGraphicsCapture(HDR)",
        }
    }

    pub fn is_hdr(self) -> bool {
        matches!(self, Self::WindowsGraphicsCaptureHdr)
    }
}

impl fmt::Display for CaptureMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BitBlt => "BitBlt",
            Self::DwmGetDxSharedSurface => "DwmGetDxSharedSurface",
            Self::WindowsGraphicsCapture => "WindowsGraphicsCapture",
            Self::WindowsGraphicsCaptureHdr => "WindowsGraphicsCaptureHdr",
        })
    }
}

impl FromStr for CaptureMode {
    type Err = CaptureError;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "bitblt" => Ok(Self::BitBlt),
            "dwmgetdxsharedsurface" | "dwmsharedsurface" | "dwm" => Ok(Self::DwmGetDxSharedSurface),
            "windowsgraphicscapture" | "wgc" => Ok(Self::WindowsGraphicsCapture),
            "windowsgraphicscapturehdr" | "windowsgraphicscapture(hdr)" | "wgc-hdr" => {
                Ok(Self::WindowsGraphicsCaptureHdr)
            }
            _ => Err(CaptureError::UnknownCaptureMode(value.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelFormat {
    Bgr8,
    Bgra8,
    RgbaF16,
}

impl PixelFormat {
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Bgr8 => 3,
            Self::Bgra8 => 4,
            Self::RgbaF16 => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameSize {
    pub width: u32,
    pub height: u32,
}

impl FrameSize {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(CaptureError::InvalidFrameDimensions);
        }
        Ok(Self { width, height })
    }

    pub fn area(self) -> usize {
        self.width as usize * self.height as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureFrame {
    pub size: FrameSize,
    pub pixel_format: PixelFormat,
    pub stride: usize,
    pub pixels: Vec<u8>,
}

impl CaptureFrame {
    pub fn new(
        width: u32,
        height: u32,
        pixel_format: PixelFormat,
        stride: usize,
        pixels: Vec<u8>,
    ) -> Result<Self> {
        let size = FrameSize::new(width, height)?;
        let row_bytes = width as usize * pixel_format.bytes_per_pixel();
        if stride < row_bytes {
            return Err(CaptureError::InvalidFrameStride { stride, row_bytes });
        }
        let expected = stride * height as usize;
        let actual = pixels.len();
        if actual != expected {
            return Err(CaptureError::InvalidFrameDataLength { expected, actual });
        }
        Ok(Self {
            size,
            pixel_format,
            stride,
            pixels,
        })
    }

    pub fn packed_bgr(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self> {
        Self::new(
            width,
            height,
            PixelFormat::Bgr8,
            width as usize * PixelFormat::Bgr8.bytes_per_pixel(),
            pixels,
        )
    }

    pub fn row_bytes(&self) -> usize {
        self.size.width as usize * self.pixel_format.bytes_per_pixel()
    }

    pub fn is_packed(&self) -> bool {
        self.stride == self.row_bytes()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowHandle(pub isize);

impl WindowHandle {
    pub fn new(raw: isize) -> Result<Self> {
        if raw == 0 {
            return Err(CaptureError::InvalidWindowHandle);
        }
        Ok(Self(raw))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowTitleCandidate {
    pub class_name: String,
    pub title: String,
}

impl WindowTitleCandidate {
    pub fn new(class_name: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            class_name: class_name.into(),
            title: title.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameWindowSearchConfig {
    pub process_names: Vec<String>,
    pub title_candidates: Vec<WindowTitleCandidate>,
}

impl Default for GameWindowSearchConfig {
    fn default() -> Self {
        Self {
            process_names: default_genshin_process_names()
                .into_iter()
                .map(ToOwned::to_owned)
                .collect(),
            title_candidates: default_genshin_window_title_candidates(),
        }
    }
}

impl GameWindowSearchConfig {
    pub fn with_install_path(mut self, install_path: impl AsRef<Path>) -> Self {
        if let Some(name) = install_path
            .as_ref()
            .file_stem()
            .and_then(|name| name.to_str())
        {
            self.push_process_name(name);
        }
        self
    }

    pub fn push_process_name(&mut self, name: impl Into<String>) {
        let name = name.into();
        if !name.trim().is_empty()
            && !self
                .process_names
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&name))
        {
            self.process_names.push(name);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameWindowMatchKind {
    ProcessName,
    WindowClassAndTitle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl WindowRect {
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn width(self) -> i32 {
        self.right - self.left
    }

    pub fn height(self) -> i32 {
        self.bottom - self.top
    }

    pub fn is_empty(self) -> bool {
        self.width() <= 0 || self.height() <= 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameWindowMetrics {
    pub client_width: u32,
    pub client_height: u32,
    pub extended_frame_bounds: WindowRect,
    pub capture_area: WindowRect,
}

impl GameWindowMetrics {
    pub fn from_legacy_capture_rect(
        client_width: u32,
        client_height: u32,
        extended_frame_bounds: WindowRect,
    ) -> Self {
        let capture_area = legacy_capture_rect(client_width, client_height, extended_frame_bounds);
        Self {
            client_width,
            client_height,
            extended_frame_bounds,
            capture_area,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameWindowMatch {
    pub handle: WindowHandle,
    pub process_id: Option<u32>,
    pub process_name: Option<String>,
    pub class_name: Option<String>,
    pub title: Option<String>,
    pub kind: GameWindowMatchKind,
    pub metrics: Option<GameWindowMetrics>,
}

pub fn default_genshin_process_names() -> Vec<&'static str> {
    vec![
        "YuanShen",
        "GenshinImpact",
        "Genshin Impact Cloud Game",
        "Genshin Impact Cloud",
    ]
}

pub fn default_genshin_window_title_candidates() -> Vec<WindowTitleCandidate> {
    vec![
        WindowTitleCandidate::new("UnityWndClass", "原神"),
        WindowTitleCandidate::new("UnityWndClass", "Genshin Impact"),
        WindowTitleCandidate::new("Qt5152QWindowIcon", "云·原神"),
    ]
}

pub fn find_genshin_window() -> Result<Option<GameWindowMatch>> {
    find_game_window(&GameWindowSearchConfig::default())
}

pub fn find_game_window(config: &GameWindowSearchConfig) -> Result<Option<GameWindowMatch>> {
    platform::find_game_window(config)
}

pub fn legacy_capture_rect(
    client_width: u32,
    client_height: u32,
    extended_frame_bounds: WindowRect,
) -> WindowRect {
    let left = extended_frame_bounds.left;
    let top = extended_frame_bounds.bottom - client_height as i32;
    WindowRect::new(
        left,
        top,
        left + client_width as i32,
        top + client_height as i32,
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureSettings {
    pub mode: CaptureMode,
    pub auto_fix_win11_bit_blt: bool,
    pub session_check_interval: Duration,
}

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            mode: CaptureMode::BitBlt,
            auto_fix_win11_bit_blt: true,
            session_check_interval: Duration::from_secs(1),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaptureModeInfo {
    pub mode: CaptureMode,
    pub legacy_value: u8,
    pub description: &'static str,
    pub implemented: bool,
    pub notes: &'static str,
}

pub fn capture_mode_infos() -> Vec<CaptureModeInfo> {
    CaptureMode::ALL
        .into_iter()
        .map(|mode| CaptureModeInfo {
            mode,
            legacy_value: mode.legacy_value(),
            description: mode.description(),
            implemented: matches!(mode, CaptureMode::BitBlt),
            notes: match mode {
                CaptureMode::BitBlt => {
                    "Windows GDI BitBlt boundary returns top-down packed BGR24 frames."
                }
                CaptureMode::DwmGetDxSharedSurface => {
                    "Legacy DWM shared-surface path still needs a Rust D3D11 port."
                }
                CaptureMode::WindowsGraphicsCapture => {
                    "Legacy Windows Graphics Capture path still needs a Rust WinRT/D3D11 port."
                }
                CaptureMode::WindowsGraphicsCaptureHdr => {
                    "Legacy HDR WGC path still needs R16G16B16A16 capture and SDR conversion."
                }
            },
        })
        .collect()
}

pub trait GameCapture {
    fn mode(&self) -> CaptureMode;
    fn is_capturing(&self) -> bool;
    fn start(&mut self, hwnd: WindowHandle, settings: CaptureSettings) -> Result<()>;
    fn capture(&mut self) -> Result<Option<CaptureFrame>>;
    fn stop(&mut self);
}

pub struct CaptureBackend {
    inner: platform::Backend,
}

impl CaptureBackend {
    pub fn new(mode: CaptureMode) -> Result<Self> {
        Ok(Self {
            inner: platform::Backend::new(mode)?,
        })
    }
}

impl GameCapture for CaptureBackend {
    fn mode(&self) -> CaptureMode {
        self.inner.mode()
    }

    fn is_capturing(&self) -> bool {
        self.inner.is_capturing()
    }

    fn start(&mut self, hwnd: WindowHandle, settings: CaptureSettings) -> Result<()> {
        self.inner.start(hwnd, settings)
    }

    fn capture(&mut self) -> Result<Option<CaptureFrame>> {
        self.inner.capture()
    }

    fn stop(&mut self) {
        self.inner.stop();
    }
}

impl Drop for CaptureBackend {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(windows)]
mod platform {
    use super::{
        CaptureError, CaptureFrame, CaptureMode, CaptureSettings, GameCapture, GameWindowMatch,
        GameWindowMatchKind, GameWindowMetrics, GameWindowSearchConfig, PixelFormat, Result,
        WindowHandle, WindowRect, WindowTitleCandidate,
    };
    use std::ffi::c_void;
    use std::mem::{size_of, zeroed};
    use std::ptr::{copy_nonoverlapping, null_mut};
    use std::time::{Duration, Instant};
    use windows::core::{BOOL, PCWSTR, PWSTR};
    use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, RECT};
    use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
    use windows::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush, GetDC,
        GetDeviceCaps, GetObjectW, ReleaseDC, SelectObject, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
        BITSPIXEL, BI_RGB, CLIPCAPS, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ, PLANES, RASTERCAPS,
        RC_BITBLT, SRCCOPY,
    };
    use windows::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE,
        REG_OPEN_CREATE_OPTIONS, REG_SZ,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetClientRect, GetWindowRect, GetWindowTextLengthW,
        GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    };

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

    pub fn find_game_window(config: &GameWindowSearchConfig) -> Result<Option<GameWindowMatch>> {
        let windows = enumerate_top_level_windows()?;

        for process_name in &config.process_names {
            if let Some(candidate) = windows.iter().find(|candidate| {
                candidate
                    .process_name
                    .as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(process_name))
            }) {
                return Ok(Some(candidate.to_match(GameWindowMatchKind::ProcessName)?));
            }
        }

        for title_candidate in &config.title_candidates {
            if let Some(candidate) = windows
                .iter()
                .find(|candidate| candidate.matches_title_candidate(title_candidate))
            {
                return Ok(Some(
                    candidate.to_match(GameWindowMatchKind::WindowClassAndTitle)?,
                ));
            }
        }

        Ok(None)
    }

    #[derive(Debug, Clone)]
    struct WindowCandidate {
        hwnd: isize,
        process_id: Option<u32>,
        process_name: Option<String>,
        class_name: String,
        title: String,
        metrics: Option<GameWindowMetrics>,
    }

    impl WindowCandidate {
        fn to_match(&self, kind: GameWindowMatchKind) -> Result<GameWindowMatch> {
            Ok(GameWindowMatch {
                handle: WindowHandle::new(self.hwnd)?,
                process_id: self.process_id,
                process_name: self.process_name.clone(),
                class_name: Some(self.class_name.clone()),
                title: Some(self.title.clone()),
                kind,
                metrics: self.metrics,
            })
        }

        fn matches_title_candidate(&self, candidate: &WindowTitleCandidate) -> bool {
            self.class_name.eq_ignore_ascii_case(&candidate.class_name)
                && self.title == candidate.title
        }
    }

    fn enumerate_top_level_windows() -> Result<Vec<WindowCandidate>> {
        let mut windows = Vec::<WindowCandidate>::new();
        let lparam = LPARAM((&mut windows as *mut Vec<WindowCandidate>) as isize);
        unsafe {
            EnumWindows(Some(enum_window_proc), lparam)
                .map_err(|err| CaptureError::Win32(format!("EnumWindows: {err}")))?;
        }
        Ok(windows)
    }

    unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if !unsafe { IsWindowVisible(hwnd).as_bool() } {
            return BOOL(1);
        }
        let windows = unsafe { &mut *(lparam.0 as *mut Vec<WindowCandidate>) };
        if let Some(candidate) = unsafe { window_candidate(hwnd) } {
            windows.push(candidate);
        }
        BOOL(1)
    }

    unsafe fn window_candidate(hwnd: HWND) -> Option<WindowCandidate> {
        if hwnd.is_invalid() {
            return None;
        }

        let class_name = window_text_buffer(|buffer| unsafe { GetClassNameW(hwnd, buffer) });
        let title_len = unsafe { GetWindowTextLengthW(hwnd) };
        let title = if title_len > 0 {
            let mut buffer = vec![0u16; title_len as usize + 1];
            let len = unsafe { GetWindowTextW(hwnd, &mut buffer) };
            wide_buffer_to_string(&buffer, len)
        } else {
            String::new()
        };

        let mut process_id = 0u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        }
        let process_id = (process_id != 0).then_some(process_id);
        let process_name = process_id.and_then(process_name_from_id);
        let metrics = unsafe { game_window_metrics(hwnd).ok() };

        Some(WindowCandidate {
            hwnd: hwnd.0 as isize,
            process_id,
            process_name,
            class_name,
            title,
            metrics,
        })
    }

    fn window_text_buffer(read: impl FnOnce(&mut [u16]) -> i32) -> String {
        let mut buffer = vec![0u16; 256];
        let len = read(&mut buffer);
        wide_buffer_to_string(&buffer, len)
    }

    fn process_name_from_id(process_id: u32) -> Option<String> {
        unsafe {
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()?;
            let mut buffer = vec![0u16; 32768];
            let mut len = buffer.len() as u32;
            let result = QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                PWSTR(buffer.as_mut_ptr()),
                &mut len,
            );
            let _ = CloseHandle(process);
            result.ok()?;
            let full_path = String::from_utf16_lossy(&buffer[..len as usize]);
            std::path::Path::new(&full_path)
                .file_stem()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned)
        }
    }

    fn wide_buffer_to_string(buffer: &[u16], len: i32) -> String {
        if len <= 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buffer[..len as usize])
    }

    fn client_size(hwnd: HWND) -> Result<(u32, u32)> {
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

    unsafe fn game_window_metrics(hwnd: HWND) -> Result<GameWindowMetrics> {
        let (client_width, client_height) = client_size(hwnd)?;
        let extended_frame_bounds = extended_frame_bounds(hwnd)?;
        Ok(GameWindowMetrics::from_legacy_capture_rect(
            client_width,
            client_height,
            extended_frame_bounds,
        ))
    }

    unsafe fn extended_frame_bounds(hwnd: HWND) -> Result<WindowRect> {
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

    fn win32_error(call: &'static str) -> CaptureError {
        CaptureError::Win32(call.to_string())
    }

    fn set_directx_user_global_settings() -> Result<()> {
        let key_path = wide_null("Software\\Microsoft\\DirectX\\UserGpuPreferences");
        let value_name = wide_null("DirectXUserGlobalSettings");
        let value_data = wide_null("SwapEffectUpgradeEnable=0;");
        let bytes = unsafe {
            std::slice::from_raw_parts(
                value_data.as_ptr().cast::<u8>(),
                value_data.len() * size_of::<u16>(),
            )
        };

        let mut key = HKEY::default();
        let create_error = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(key_path.as_ptr()),
                None,
                PCWSTR::null(),
                REG_OPEN_CREATE_OPTIONS(0),
                KEY_SET_VALUE,
                None,
                &mut key,
                None,
            )
        };
        if create_error.0 != 0 {
            return Err(CaptureError::Win32(format!(
                "RegCreateKeyExW: {}",
                create_error.0
            )));
        }

        let set_error =
            unsafe { RegSetValueExW(key, PCWSTR(value_name.as_ptr()), None, REG_SZ, Some(bytes)) };
        unsafe {
            let _ = RegCloseKey(key);
        }

        if set_error.0 != 0 {
            return Err(CaptureError::Win32(format!(
                "RegSetValueExW: {}",
                set_error.0
            )));
        }

        Ok(())
    }

    fn wide_null(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(not(windows))]
mod platform {
    use super::{
        CaptureError, CaptureFrame, CaptureMode, CaptureSettings, GameCapture, GameWindowMatch,
        GameWindowSearchConfig, Result, WindowHandle,
    };

    pub struct Backend {
        mode: CaptureMode,
    }

    impl Backend {
        pub fn new(mode: CaptureMode) -> Result<Self> {
            Ok(Self { mode })
        }
    }

    impl GameCapture for Backend {
        fn mode(&self) -> CaptureMode {
            self.mode
        }

        fn is_capturing(&self) -> bool {
            false
        }

        fn start(&mut self, _hwnd: WindowHandle, _settings: CaptureSettings) -> Result<()> {
            Err(CaptureError::UnsupportedPlatform)
        }

        fn capture(&mut self) -> Result<Option<CaptureFrame>> {
            Err(CaptureError::UnsupportedPlatform)
        }

        fn stop(&mut self) {}
    }

    pub fn find_game_window(_config: &GameWindowSearchConfig) -> Result<Option<GameWindowMatch>> {
        Err(CaptureError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_legacy_capture_mode_values() {
        let values: Vec<_> = CaptureMode::ALL
            .into_iter()
            .map(CaptureMode::legacy_value)
            .collect();
        assert_eq!(values, vec![0, 2, 1, 3]);
    }

    #[test]
    fn parses_legacy_capture_mode_names() {
        assert_eq!(
            "BitBlt".parse::<CaptureMode>().unwrap(),
            CaptureMode::BitBlt
        );
        assert_eq!(
            "WindowsGraphicsCaptureHdr".parse::<CaptureMode>().unwrap(),
            CaptureMode::WindowsGraphicsCaptureHdr
        );
        assert_eq!(
            "wgc".parse::<CaptureMode>().unwrap(),
            CaptureMode::WindowsGraphicsCapture
        );
    }

    #[test]
    fn validates_frame_data_length() {
        let frame = CaptureFrame::packed_bgr(2, 2, vec![0; 12]).unwrap();
        assert_eq!(frame.row_bytes(), 6);
        assert!(frame.is_packed());

        let err = CaptureFrame::packed_bgr(2, 2, vec![0; 11]).unwrap_err();
        assert!(matches!(
            err,
            CaptureError::InvalidFrameDataLength {
                expected: 12,
                actual: 11
            }
        ));
    }

    #[test]
    fn rejects_zero_window_handles() {
        assert!(matches!(
            WindowHandle::new(0),
            Err(CaptureError::InvalidWindowHandle)
        ));
    }

    #[test]
    fn game_window_search_defaults_match_legacy_genshin_candidates() {
        let config = GameWindowSearchConfig::default();
        assert_eq!(
            config.process_names,
            vec![
                "YuanShen",
                "GenshinImpact",
                "Genshin Impact Cloud Game",
                "Genshin Impact Cloud"
            ]
        );
        assert_eq!(
            config.title_candidates,
            vec![
                WindowTitleCandidate::new("UnityWndClass", "原神"),
                WindowTitleCandidate::new("UnityWndClass", "Genshin Impact"),
                WindowTitleCandidate::new("Qt5152QWindowIcon", "云·原神"),
            ]
        );
    }

    #[test]
    fn game_window_search_config_adds_custom_install_process_name_once() {
        let config = GameWindowSearchConfig::default()
            .with_install_path(r"D:\Games\Genshin Impact\CustomYuanShen.exe")
            .with_install_path(r"D:\Games\Genshin Impact\CustomYuanShen.exe");
        assert_eq!(
            config
                .process_names
                .iter()
                .filter(|name| name.as_str() == "CustomYuanShen")
                .count(),
            1
        );
    }

    #[test]
    fn legacy_capture_rect_uses_extended_frame_bottom_and_client_size() {
        let bounds = WindowRect::new(120, 80, 2048, 1188);
        let capture = legacy_capture_rect(1920, 1080, bounds);

        assert_eq!(capture, WindowRect::new(120, 108, 2040, 1188));
        assert_eq!(capture.width(), 1920);
        assert_eq!(capture.height(), 1080);

        let metrics = GameWindowMetrics::from_legacy_capture_rect(1920, 1080, bounds);
        assert_eq!(metrics.client_width, 1920);
        assert_eq!(metrics.client_height, 1080);
        assert_eq!(metrics.extended_frame_bounds, bounds);
        assert_eq!(metrics.capture_area, capture);
    }

    #[test]
    fn window_rect_reports_empty_dimensions() {
        assert!(!WindowRect::new(10, 20, 30, 40).is_empty());
        assert!(WindowRect::new(10, 20, 10, 40).is_empty());
        assert!(WindowRect::new(10, 20, 30, 20).is_empty());
        assert!(WindowRect::new(30, 20, 10, 40).is_empty());
    }
}
