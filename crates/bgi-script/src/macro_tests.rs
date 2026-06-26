use super::*;
use bgi_input::{InputEvent, MouseButton};

#[test]
fn deserializes_legacy_numeric_event_types() {
    let script = KeyMouseScript::from_json(
        r#"{
          "macroEvents": [
            { "type": 0, "keyCode": 87, "time": 100 },
            { "type": 1, "keyCode": 87, "time": 150 }
          ],
          "info": { "x": 557, "y": 57, "width": 1920, "height": 1080, "recordDpi": 1 }
        }"#,
    )
    .unwrap();

    assert_eq!(script.macro_events[0].event_type, MacroEventType::KeyDown);
    assert_eq!(script.macro_events[1].event_type, MacroEventType::KeyUp);
    assert_eq!(script.info.unwrap().width, 1920);
}

#[test]
fn accepts_string_event_types_for_forward_compatibility() {
    let event: MacroEvent =
        serde_json::from_str(r#"{ "type": "MouseWheel", "mouseY": -120, "time": 0 }"#).unwrap();

    assert_eq!(event.event_type, MacroEventType::MouseWheel);
}

#[test]
fn converts_key_macro_to_timed_input_sequence() {
    let script = KeyMouseScript::from_json(
        r#"{
          "macroEvents": [
            { "type": 0, "keyCode": 87, "time": 100 },
            { "type": 1, "keyCode": 87, "time": 175 }
          ]
        }"#,
    )
    .unwrap();

    let events = script
        .to_input_events(MacroPlaybackContext::default())
        .unwrap();

    assert_eq!(
        events,
        vec![
            InputEvent::Delay { milliseconds: 100 },
            InputEvent::KeyDown {
                vk: 87,
                extended: None
            },
            InputEvent::Delay { milliseconds: 75 },
            InputEvent::KeyUp {
                vk: 87,
                extended: None
            }
        ]
    );
}

#[test]
fn adapts_absolute_mouse_events_to_target_capture_area() {
    let script = KeyMouseScript::from_json(
        r#"{
          "macroEvents": [
            { "type": 4, "mouseButton": "Left", "mouseX": 150, "mouseY": 100, "time": 0 }
          ],
          "info": { "x": 100, "y": 50, "width": 200, "height": 100, "recordDpi": 1 }
        }"#,
    )
    .unwrap();
    let context = MacroPlaybackContext::for_current_capture_area(
        MacroCaptureArea {
            x: 0,
            y: 0,
            width: 400,
            height: 200,
        },
        1.0,
        800,
        600,
    );

    let events = script.to_input_events(context).unwrap();

    assert_eq!(
        events,
        vec![
            InputEvent::MouseMoveAbsolute {
                x: 8191,
                y: 10922,
                virtual_desktop: false
            },
            InputEvent::MouseButtonDown {
                button: MouseButton::Left
            }
        ]
    );
}

#[test]
fn adapts_relative_mouse_events_with_legacy_rounding() {
    let script = KeyMouseScript::from_json(
        r#"{
          "macroEvents": [
            { "type": 3, "mouseX": 13, "mouseY": -5, "time": 0 }
          ],
          "info": { "x": 0, "y": 0, "width": 1920, "height": 1080, "recordDpi": 2 }
        }"#,
    )
    .unwrap();
    let context = MacroPlaybackContext::for_current_capture_area(
        MacroCaptureArea {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        },
        1.0,
        1920,
        1080,
    );

    let events = script.to_input_events(context).unwrap();

    assert_eq!(
        events,
        vec![InputEvent::MouseMoveRelative { dx: 6, dy: -2 }]
    );
}

#[test]
fn mouse_wheel_uses_legacy_click_delta_conversion() {
    let script = KeyMouseScript::from_json(
        r#"{
          "macroEvents": [
            { "type": 6, "mouseY": -240, "time": 0 }
          ]
        }"#,
    )
    .unwrap();

    let events = script
        .to_input_events(MacroPlaybackContext::default())
        .unwrap();

    assert_eq!(
        events,
        vec![InputEvent::MouseWheel {
            amount: -240,
            horizontal: false
        }]
    );
}
