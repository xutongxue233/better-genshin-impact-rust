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
fn bilibili_login_window_search_defaults_match_legacy_owner_and_titles() {
    let config = BilibiliLoginWindowSearchConfig::default();

    assert_eq!(config.owner_process_name, "YuanShen");
    assert_eq!(config.title_contains, "bilibili");
    assert_eq!(config.agreement_title_contains, "协议");
    assert_eq!(config.login_title_contains, "登录");
    assert!(config.owner_must_match_process);
    assert_eq!(
        config.classify_title("bilibili 用户协议"),
        Some(BilibiliLoginWindowKind::Agreement)
    );
    assert_eq!(
        config.classify_title("BILIBILI 登录"),
        Some(BilibiliLoginWindowKind::Login)
    );
    assert_eq!(config.classify_title("mihoyo 登录"), None);
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
