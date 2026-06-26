use super::*;

#[test]
fn builds_modified_key_sequence_in_legacy_order() {
    let events = InputSequence::new()
        .modified_key_stroke([0x11, 0x10], 0x41)
        .events()
        .to_vec();

    assert_eq!(
        events[0],
        InputEvent::KeyDown {
            vk: 0x11,
            extended: None
        }
    );
    assert_eq!(
        events[1],
        InputEvent::KeyDown {
            vk: 0x10,
            extended: None
        }
    );
    assert_eq!(
        events[2],
        InputEvent::KeyDown {
            vk: 0x41,
            extended: None
        }
    );
    assert_eq!(
        events[3],
        InputEvent::KeyUp {
            vk: 0x41,
            extended: None
        }
    );
    assert_eq!(
        events[4],
        InputEvent::KeyUp {
            vk: 0x10,
            extended: None
        }
    );
    assert_eq!(
        events[5],
        InputEvent::KeyUp {
            vk: 0x11,
            extended: None
        }
    );
}

#[test]
fn wheel_scroll_uses_windows_click_delta() {
    let events = InputSequence::new().vertical_scroll(2).events().to_vec();
    assert_eq!(
        events,
        vec![InputEvent::MouseWheel {
            amount: 240,
            horizontal: false
        }]
    );
}

#[test]
fn absolute_mouse_coordinates_normalize_like_legacy_send_input() {
    let bounds = AbsoluteMouseCoordinateBounds::new(0, 0, 1920, 1080);

    assert_eq!(normalize_absolute_mouse_position(0, 0, bounds), (0, 0));
    assert_eq!(
        normalize_absolute_mouse_position(960, 540, bounds),
        (32_767, 32_767)
    );
    assert_eq!(
        normalize_absolute_mouse_position(1920, 1080, bounds),
        (ABSOLUTE_MOUSE_COORDINATE_MAX, ABSOLUTE_MOUSE_COORDINATE_MAX)
    );
}

#[test]
fn absolute_mouse_coordinates_normalize_virtual_desktop_origin() {
    let bounds = AbsoluteMouseCoordinateBounds::new(-1920, -200, 3840, 1200);

    assert_eq!(
        normalize_absolute_mouse_position(-1920, -200, bounds),
        (0, 0)
    );
    assert_eq!(
        normalize_absolute_mouse_position(0, 400, bounds),
        (32_767, 32_767)
    );
    assert_eq!(
        normalize_absolute_mouse_position(1920, 1000, bounds),
        (ABSOLUTE_MOUSE_COORDINATE_MAX, ABSOLUTE_MOUSE_COORDINATE_MAX)
    );
}

#[test]
fn maps_keyboard_genshin_action_from_default_bindings() {
    let bindings = KeyBindingsConfig::default();
    let events = input_events_for_action(
        &bindings,
        GenshinAction::QuickUseGadget,
        KeyActionType::KeyPress,
    )
    .unwrap();

    assert_eq!(
        events,
        vec![
            InputEvent::KeyDown {
                vk: KeyId::Z.vk(),
                extended: None
            },
            InputEvent::KeyUp {
                vk: KeyId::Z.vk(),
                extended: None
            }
        ]
    );
}

#[test]
fn maps_mouse_genshin_action_from_default_bindings() {
    let bindings = KeyBindingsConfig::default();
    let events = input_events_for_action(
        &bindings,
        GenshinAction::NormalAttack,
        KeyActionType::KeyPress,
    )
    .unwrap();

    assert_eq!(
        events,
        vec![
            InputEvent::MouseButtonDown {
                button: MouseButton::Left
            },
            InputEvent::MouseButtonUp {
                button: MouseButton::Left
            }
        ]
    );
}

#[test]
fn key_down_action_emits_only_down_event() {
    let bindings = KeyBindingsConfig::default();
    let events = input_events_for_action(
        &bindings,
        GenshinAction::MoveForward,
        KeyActionType::KeyDown,
    )
    .unwrap();

    assert_eq!(
        events,
        vec![InputEvent::KeyDown {
            vk: KeyId::W.vk(),
            extended: None
        }]
    );
}

#[test]
fn hold_action_preserves_legacy_one_second_duration() {
    let bindings = KeyBindingsConfig::default();
    let events = input_events_for_action(
        &bindings,
        GenshinAction::ElementalSkill,
        KeyActionType::Hold,
    )
    .unwrap();

    assert_eq!(
        events,
        vec![
            InputEvent::KeyDown {
                vk: KeyId::E.vk(),
                extended: None
            },
            InputEvent::Delay {
                milliseconds: DEFAULT_HOLD_MILLISECONDS
            },
            InputEvent::KeyUp {
                vk: KeyId::E.vk(),
                extended: None
            }
        ]
    );
}

#[test]
fn none_key_binding_is_reported_before_dispatch() {
    let error = input_events_for_key(KeyId::NONE, KeyActionType::KeyPress).unwrap_err();
    assert!(matches!(error, InputError::UnboundKeyBinding { key } if key == KeyId::NONE));
}

#[test]
fn cancelled_input_dispatch_stops_before_platform_dispatch() {
    let cancellation = InputCancellationToken::new();
    cancellation.cancel();
    let events = InputSequence::new()
        .key_press(KeyId::F.vk())
        .events()
        .to_vec();

    let error = send_events_with_cancellation(&events, &cancellation).unwrap_err();

    assert!(matches!(
        error,
        InputError::Cancelled {
            dispatched_events: 0,
            total_events: 2
        }
    ));
}

#[test]
fn window_targeted_input_rejects_zero_handles_before_dispatch() {
    assert!(matches!(
        activate_window(0),
        Err(InputError::InvalidWindowHandle)
    ));
    assert!(matches!(
        send_events_to_window(0, InputSequence::new().key_press(0x41).events()),
        Err(InputError::InvalidWindowHandle)
    ));
}

#[test]
fn release_pressed_keys_adds_mouse_button_cleanup() {
    let events = release_pressed_keys_sequence([KeyId::W.vk(), KeyId::LEFT_SHIFT.vk()])
        .events()
        .to_vec();

    assert_eq!(
        events,
        vec![
            InputEvent::KeyUp {
                vk: KeyId::W.vk(),
                extended: None
            },
            InputEvent::KeyUp {
                vk: KeyId::LEFT_SHIFT.vk(),
                extended: None
            },
            InputEvent::MouseButtonUp {
                button: MouseButton::Left
            },
            InputEvent::MouseButtonUp {
                button: MouseButton::Right
            },
            InputEvent::MouseButtonUp {
                button: MouseButton::Middle
            }
        ]
    );
}

#[test]
fn release_all_keys_matches_legacy_vk_sweep_shape() {
    let sequence = release_all_keys_sequence();
    let events = sequence.events();

    assert_eq!(events.len(), 0xFE + DEFAULT_RELEASE_MOUSE_BUTTONS.len());
    assert_eq!(
        events[0],
        InputEvent::KeyUp {
            vk: 0x01,
            extended: None
        }
    );
    assert_eq!(
        events[0xFD],
        InputEvent::KeyUp {
            vk: 0xFE,
            extended: None
        }
    );
    assert_eq!(
        events.last(),
        Some(&InputEvent::MouseButtonUp {
            button: MouseButton::Middle
        })
    );
}

#[test]
fn make_lparam_matches_legacy_coordinate_packing() {
    assert_eq!(make_lparam(16, 16), 0x0010_0010);
    assert_eq!(make_lparam(0x1_FFFF, 2), 0x0002_FFFF);
}

#[test]
fn builds_legacy_post_message_key_press_sequence() {
    let events = PostMessageSequence::new()
        .key_press(KeyId::F.vk())
        .events()
        .to_vec();

    assert_eq!(
        events,
        vec![
            PostMessageEvent::Message {
                message: WM_KEYDOWN,
                wparam: KeyId::F.vk() as isize,
                lparam: POST_MESSAGE_KEYDOWN_LPARAM
            },
            PostMessageEvent::Message {
                message: WM_CHAR,
                wparam: KeyId::F.vk() as isize,
                lparam: POST_MESSAGE_KEYDOWN_LPARAM
            },
            PostMessageEvent::Message {
                message: WM_KEYUP,
                wparam: KeyId::F.vk() as isize,
                lparam: POST_MESSAGE_KEYUP_LPARAM
            }
        ]
    );
}

#[test]
fn builds_legacy_post_message_click_sequence() {
    let events = PostMessageSequence::new()
        .left_button_click()
        .events()
        .to_vec();

    assert_eq!(
        events,
        vec![
            PostMessageEvent::Message {
                message: WM_LBUTTONDOWN,
                wparam: 0,
                lparam: make_lparam(16, 16)
            },
            PostMessageEvent::Delay {
                milliseconds: POST_MESSAGE_CLICK_DELAY_MILLISECONDS
            },
            PostMessageEvent::Message {
                message: WM_LBUTTONUP,
                wparam: 0,
                lparam: make_lparam(16, 16)
            }
        ]
    );
}

#[test]
fn background_post_message_key_press_activates_first() {
    let events = PostMessageSequence::new()
        .key_press_background(KeyId::Z.vk())
        .events()
        .to_vec();

    assert_eq!(
        events.first(),
        Some(&PostMessageEvent::Message {
            message: WM_ACTIVATE,
            wparam: 1,
            lparam: 0
        })
    );
    assert_eq!(events.len(), 4);
}

#[test]
fn post_message_action_mapping_uses_configured_key_bindings() {
    let bindings = KeyBindingsConfig::default();
    let events = post_message_events_for_action(
        &bindings,
        GenshinAction::QuickUseGadget,
        KeyActionType::KeyPress,
        PostMessageMode::Background,
    );

    assert_eq!(
        events,
        vec![
            PostMessageEvent::Message {
                message: WM_ACTIVATE,
                wparam: 1,
                lparam: 0
            },
            PostMessageEvent::Message {
                message: WM_KEYDOWN,
                wparam: KeyId::Z.vk() as isize,
                lparam: POST_MESSAGE_KEYDOWN_LPARAM
            },
            PostMessageEvent::Message {
                message: WM_CHAR,
                wparam: KeyId::Z.vk() as isize,
                lparam: POST_MESSAGE_KEYDOWN_LPARAM
            },
            PostMessageEvent::Message {
                message: WM_KEYUP,
                wparam: KeyId::Z.vk() as isize,
                lparam: POST_MESSAGE_KEYUP_LPARAM
            }
        ]
    );
}

#[test]
fn background_post_message_keeps_legacy_mouse_limitations() {
    let bindings = KeyBindingsConfig::default();
    let events = post_message_events_for_action(
        &bindings,
        GenshinAction::SprintMouse,
        KeyActionType::KeyPress,
        PostMessageMode::Background,
    );

    assert!(events.is_empty());
}
