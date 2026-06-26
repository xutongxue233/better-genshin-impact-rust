use super::metrics::game_window_metrics;
use crate::{
    BilibiliLoginWindowMatch, BilibiliLoginWindowSearchConfig, CaptureError, GameWindowMatch,
    GameWindowMatchKind, GameWindowMetrics, GameWindowSearchConfig, Result, WindowHandle,
    WindowTitleCandidate,
};
use std::path::PathBuf;
use windows::core::{BOOL, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindow, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsWindowVisible, GW_OWNER,
};

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

pub fn find_bilibili_login_window(
    config: &BilibiliLoginWindowSearchConfig,
) -> Result<Option<BilibiliLoginWindowMatch>> {
    let windows = enumerate_top_level_windows()?;
    for candidate in &windows {
        if let Some(login_match) = candidate.to_bilibili_login_match(config)? {
            return Ok(Some(login_match));
        }
    }
    Ok(None)
}

pub fn find_process_image_path_by_name(process_name: &str) -> Result<Option<PathBuf>> {
    let windows = enumerate_top_level_windows()?;
    Ok(windows
        .iter()
        .find(|candidate| {
            candidate
                .process_name
                .as_deref()
                .is_some_and(|name| name.eq_ignore_ascii_case(process_name))
        })
        .and_then(|candidate| candidate.process_image_path.clone()))
}

#[derive(Debug, Clone)]
struct WindowCandidate {
    hwnd: isize,
    process_id: Option<u32>,
    process_name: Option<String>,
    process_image_path: Option<PathBuf>,
    owner_process_id: Option<u32>,
    owner_process_name: Option<String>,
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
        self.class_name.eq_ignore_ascii_case(&candidate.class_name) && self.title == candidate.title
    }

    fn to_bilibili_login_match(
        &self,
        config: &BilibiliLoginWindowSearchConfig,
    ) -> Result<Option<BilibiliLoginWindowMatch>> {
        let Some(kind) = config.classify_title(&self.title) else {
            return Ok(None);
        };
        if config.owner_must_match_process
            && !self
                .owner_process_name
                .as_deref()
                .is_some_and(|name| name.eq_ignore_ascii_case(&config.owner_process_name))
        {
            return Ok(None);
        }
        Ok(Some(BilibiliLoginWindowMatch {
            handle: WindowHandle::new(self.hwnd)?,
            title: self.title.clone(),
            kind,
            owner_process_id: self.owner_process_id,
            owner_process_name: self.owner_process_name.clone(),
        }))
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

    let process_id = unsafe { process_id_for_window(hwnd) };
    let process_image_path = process_id.and_then(process_image_path_from_id);
    let process_name = process_image_path
        .as_deref()
        .and_then(process_name_from_path);
    let owner = unsafe { GetWindow(hwnd, GW_OWNER).ok() }.filter(|owner| !owner.is_invalid());
    let owner_process_id = owner.and_then(|owner| unsafe { process_id_for_window(owner) });
    let owner_process_name = owner_process_id
        .and_then(process_image_path_from_id)
        .as_deref()
        .and_then(process_name_from_path);
    let metrics = unsafe { game_window_metrics(hwnd).ok() };

    Some(WindowCandidate {
        hwnd: hwnd.0 as isize,
        process_id,
        process_name,
        process_image_path,
        owner_process_id,
        owner_process_name,
        class_name,
        title,
        metrics,
    })
}

unsafe fn process_id_for_window(hwnd: HWND) -> Option<u32> {
    let mut process_id = 0u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
    }
    (process_id != 0).then_some(process_id)
}

fn window_text_buffer(read: impl FnOnce(&mut [u16]) -> i32) -> String {
    let mut buffer = vec![0u16; 256];
    let len = read(&mut buffer);
    wide_buffer_to_string(&buffer, len)
}

fn process_image_path_from_id(process_id: u32) -> Option<PathBuf> {
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
        Some(PathBuf::from(full_path))
    }
}

fn process_name_from_path(path: &std::path::Path) -> Option<String> {
    path.file_stem()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
}

fn wide_buffer_to_string(buffer: &[u16], len: i32) -> String {
    if len <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buffer[..len as usize])
}
