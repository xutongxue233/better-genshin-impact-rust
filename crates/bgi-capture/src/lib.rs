use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BilibiliLoginWindowKind {
    Agreement,
    Login,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BilibiliLoginWindowSearchConfig {
    pub owner_process_name: String,
    pub title_contains: String,
    pub agreement_title_contains: String,
    pub login_title_contains: String,
    pub owner_must_match_process: bool,
}

impl Default for BilibiliLoginWindowSearchConfig {
    fn default() -> Self {
        Self {
            owner_process_name: "YuanShen".to_string(),
            title_contains: "bilibili".to_string(),
            agreement_title_contains: "协议".to_string(),
            login_title_contains: "登录".to_string(),
            owner_must_match_process: true,
        }
    }
}

impl BilibiliLoginWindowSearchConfig {
    pub fn classify_title(&self, title: &str) -> Option<BilibiliLoginWindowKind> {
        if !contains_ignore_case(title, &self.title_contains) {
            return None;
        }
        if contains_ignore_case(title, &self.agreement_title_contains) {
            return Some(BilibiliLoginWindowKind::Agreement);
        }
        if contains_ignore_case(title, &self.login_title_contains) {
            return Some(BilibiliLoginWindowKind::Login);
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BilibiliLoginWindowMatch {
    pub handle: WindowHandle,
    pub title: String,
    pub kind: BilibiliLoginWindowKind,
    pub owner_process_id: Option<u32>,
    pub owner_process_name: Option<String>,
}

pub fn find_genshin_window() -> Result<Option<GameWindowMatch>> {
    find_game_window(&GameWindowSearchConfig::default())
}

pub fn find_game_window(config: &GameWindowSearchConfig) -> Result<Option<GameWindowMatch>> {
    platform::find_game_window(config)
}

pub fn find_bilibili_login_window(
    config: &BilibiliLoginWindowSearchConfig,
) -> Result<Option<BilibiliLoginWindowMatch>> {
    platform::find_bilibili_login_window(config)
}

pub fn find_process_image_path_by_name(process_name: &str) -> Result<Option<PathBuf>> {
    platform::find_process_image_path_by_name(process_name)
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

fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
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
#[path = "platform_windows.rs"]
mod platform;

#[cfg(not(windows))]
#[path = "platform_stub.rs"]
mod platform;

#[cfg(test)]
mod tests;
