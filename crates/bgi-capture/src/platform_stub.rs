use super::{
    BilibiliLoginWindowMatch, BilibiliLoginWindowSearchConfig, CaptureError, CaptureFrame,
    CaptureMode, CaptureSettings, GameCapture, GameWindowMatch, GameWindowSearchConfig, Result,
    WindowHandle,
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

pub fn find_bilibili_login_window(
    _config: &BilibiliLoginWindowSearchConfig,
) -> Result<Option<BilibiliLoginWindowMatch>> {
    Err(CaptureError::UnsupportedPlatform)
}

pub fn find_process_image_path_by_name(_process_name: &str) -> Result<Option<std::path::PathBuf>> {
    Err(CaptureError::UnsupportedPlatform)
}
