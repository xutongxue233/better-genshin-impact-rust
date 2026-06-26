#[path = "platform_windows_capture.rs"]
mod capture;
#[path = "platform_windows_metrics.rs"]
mod metrics;
#[path = "platform_windows_registry.rs"]
mod registry;
#[path = "platform_windows_win32.rs"]
mod win32;
#[path = "platform_windows_window.rs"]
mod window;

pub use capture::Backend;
pub use window::{find_bilibili_login_window, find_game_window, find_process_image_path_by_name};
