use super::super::{
    CombatMouseButtonPlan, CombatVirtualKeyPlan, COMBAT_DEFAULT_CHARGE_MILLISECONDS,
};
use super::combat_action_events;
use crate::{Result, TaskError};
use bgi_core::{GenshinAction, KeyBindingsConfig, KeyId};
use bgi_input::{InputEvent, KeyActionType, MouseButton};

pub(crate) fn combat_mouse_button_plan(value: Option<&str>) -> Result<CombatMouseButtonPlan> {
    let raw = value.unwrap_or("left").trim();
    let button = match raw.to_ascii_lowercase().as_str() {
        "left" => MouseButton::Left,
        "right" => MouseButton::Right,
        "middle" => MouseButton::Middle,
        other => {
            return Err(TaskError::CombatStrategy(format!(
                "unsupported mouse button: {other}"
            )));
        }
    };
    Ok(CombatMouseButtonPlan {
        raw: raw.to_string(),
        button,
    })
}

pub(crate) fn combat_virtual_key_plan(value: &str) -> Result<CombatVirtualKeyPlan> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(TaskError::CombatStrategy(
            "virtual key argument must not be empty".to_string(),
        ));
    }
    let raw_normalized = raw.to_ascii_uppercase().replace(' ', "_");
    let normalized = raw_normalized
        .strip_prefix("VK_")
        .unwrap_or(&raw_normalized)
        .to_string();
    let (vk, mouse_button) = virtual_key_code_and_mouse_button(&normalized).ok_or_else(|| {
        TaskError::CombatStrategy(format!("invalid virtual key argument: {value}"))
    })?;
    Ok(CombatVirtualKeyPlan {
        raw: raw.to_string(),
        vk,
        mouse_button,
        mapped_action: mapped_genshin_action_for_virtual_key(&normalized),
    })
}

pub(crate) fn combat_virtual_key_events(
    key: &CombatVirtualKeyPlan,
    action_type: KeyActionType,
) -> Result<Vec<InputEvent>> {
    if let Some(action) = key.mapped_action {
        return combat_action_events(&KeyBindingsConfig::default(), action, action_type);
    }
    if let Some(button) = key.mouse_button {
        let events = match action_type {
            KeyActionType::KeyDown => vec![InputEvent::MouseButtonDown { button }],
            KeyActionType::KeyUp => vec![InputEvent::MouseButtonUp { button }],
            KeyActionType::KeyPress => vec![
                InputEvent::MouseButtonDown { button },
                InputEvent::MouseButtonUp { button },
            ],
            KeyActionType::Hold => vec![
                InputEvent::MouseButtonDown { button },
                InputEvent::Delay {
                    milliseconds: COMBAT_DEFAULT_CHARGE_MILLISECONDS,
                },
                InputEvent::MouseButtonUp { button },
            ],
        };
        return Ok(events);
    }
    let events = match action_type {
        KeyActionType::KeyDown => vec![InputEvent::KeyDown {
            vk: key.vk,
            extended: None,
        }],
        KeyActionType::KeyUp => vec![InputEvent::KeyUp {
            vk: key.vk,
            extended: None,
        }],
        KeyActionType::KeyPress => vec![
            InputEvent::KeyDown {
                vk: key.vk,
                extended: None,
            },
            InputEvent::KeyUp {
                vk: key.vk,
                extended: None,
            },
        ],
        KeyActionType::Hold => vec![
            InputEvent::KeyDown {
                vk: key.vk,
                extended: None,
            },
            InputEvent::Delay {
                milliseconds: COMBAT_DEFAULT_CHARGE_MILLISECONDS,
            },
            InputEvent::KeyUp {
                vk: key.vk,
                extended: None,
            },
        ],
    };
    Ok(events)
}

fn virtual_key_code_and_mouse_button(key: &str) -> Option<(u16, Option<MouseButton>)> {
    if key.len() == 1 {
        let ch = key.chars().next()?;
        if ch.is_ascii_alphabetic() || ch.is_ascii_digit() {
            return Some((ch as u16, None));
        }
    }
    if let Some(index) = key
        .strip_prefix('F')
        .and_then(|value| value.parse::<u16>().ok())
    {
        if (1..=24).contains(&index) {
            return Some((0x70 + index - 1, None));
        }
    }
    if let Some(index) = key
        .strip_prefix("NUMPAD")
        .and_then(|value| value.parse::<u16>().ok())
    {
        if index <= 9 {
            return Some((0x60 + index, None));
        }
    }
    let result = match key {
        "LBUTTON" => (KeyId::MOUSE_LEFT_BUTTON.vk(), Some(MouseButton::Left)),
        "RBUTTON" => (KeyId::MOUSE_RIGHT_BUTTON.vk(), Some(MouseButton::Right)),
        "MBUTTON" => (KeyId::MOUSE_MIDDLE_BUTTON.vk(), Some(MouseButton::Middle)),
        "XBUTTON1" => (KeyId::MOUSE_SIDE_BUTTON1.vk(), Some(MouseButton::X(1))),
        "XBUTTON2" => (KeyId::MOUSE_SIDE_BUTTON2.vk(), Some(MouseButton::X(2))),
        "SHIFT" | "LSHIFT" | "LEFT_SHIFT" => (KeyId::LEFT_SHIFT.vk(), None),
        "RSHIFT" | "RIGHT_SHIFT" => (KeyId::RIGHT_SHIFT.vk(), None),
        "CONTROL" | "CTRL" | "LCONTROL" | "LCTRL" | "LEFT_CONTROL" | "LEFT_CTRL" => {
            (KeyId::LEFT_CTRL.vk(), None)
        }
        "RCONTROL" | "RCTRL" | "RIGHT_CONTROL" | "RIGHT_CTRL" => (KeyId::RIGHT_CTRL.vk(), None),
        "MENU" | "ALT" | "LMENU" | "LALT" | "LEFT_ALT" => (KeyId::LEFT_ALT.vk(), None),
        "RMENU" | "RALT" | "RIGHT_ALT" => (KeyId::RIGHT_ALT.vk(), None),
        "LWIN" | "LEFT_WIN" => (KeyId::LEFT_WIN.vk(), None),
        "RWIN" | "RIGHT_WIN" => (KeyId::RIGHT_WIN.vk(), None),
        "SPACE" => (KeyId::SPACE.vk(), None),
        "RETURN" | "ENTER" => (KeyId::ENTER.vk(), None),
        "ESCAPE" | "ESC" => (KeyId::ESCAPE.vk(), None),
        "TAB" => (KeyId::TAB.vk(), None),
        "BACK" | "BACKSPACE" => (KeyId::BACKSPACE.vk(), None),
        "INSERT" => (KeyId::INSERT.vk(), None),
        "DELETE" | "DEL" => (KeyId::DELETE.vk(), None),
        "HOME" => (KeyId::HOME.vk(), None),
        "END" => (KeyId::END.vk(), None),
        "PRIOR" | "PAGE_UP" | "PAGEUP" => (KeyId::PAGE_UP.vk(), None),
        "NEXT" | "PAGE_DOWN" | "PAGEDOWN" => (KeyId::PAGE_DOWN.vk(), None),
        "LEFT" => (KeyId::LEFT.vk(), None),
        "UP" => (KeyId::UP.vk(), None),
        "RIGHT" => (KeyId::RIGHT.vk(), None),
        "DOWN" => (KeyId::DOWN.vk(), None),
        "CAPITAL" | "CAPSLOCK" | "CAPS_LOCK" => (KeyId::CAPS_LOCK.vk(), None),
        "SCROLL" | "SCROLLLOCK" | "SCROLL_LOCK" => (KeyId::SCROLL_LOCK.vk(), None),
        "PAUSE" => (KeyId::PAUSE.vk(), None),
        "PRINT" | "SNAPSHOT" | "PRINT_SCREEN" => (KeyId::PRINT_SCREEN.vk(), None),
        "APPS" => (KeyId::APPS.vk(), None),
        "DECIMAL" => (KeyId::DECIMAL.vk(), None),
        "DIVIDE" => (KeyId::DIVIDE.vk(), None),
        "MULTIPLY" => (KeyId::MULTIPLY.vk(), None),
        "SUBTRACT" => (KeyId::SUBTRACT.vk(), None),
        "ADD" => (KeyId::ADD.vk(), None),
        "OEM_PLUS" | "PLUS" | "EQUAL" => (KeyId::EQUAL.vk(), None),
        "OEM_MINUS" | "MINUS" => (KeyId::MINUS.vk(), None),
        "OEM_COMMA" | "COMMA" => (KeyId::COMMA.vk(), None),
        "OEM_PERIOD" | "PERIOD" => (KeyId::PERIOD.vk(), None),
        "OEM_1" | "SEMICOLON" => (KeyId::SEMICOLON.vk(), None),
        "OEM_2" | "SLASH" => (KeyId::SLASH.vk(), None),
        "OEM_3" | "TILDE" => (KeyId::TILDE.vk(), None),
        "OEM_4" | "LEFT_SQUARE_BRACKET" => (KeyId::LEFT_SQUARE_BRACKET.vk(), None),
        "OEM_6" | "RIGHT_SQUARE_BRACKET" => (KeyId::RIGHT_SQUARE_BRACKET.vk(), None),
        "OEM_7" | "APOSTROPHE" => (KeyId::APOSTROPHE.vk(), None),
        "OEM_102" | "BACKSLASH" => (KeyId::BACKSLASH.vk(), None),
        _ => return None,
    };
    Some(result)
}

fn mapped_genshin_action_for_virtual_key(key: &str) -> Option<GenshinAction> {
    match key {
        "W" => Some(GenshinAction::MoveForward),
        "S" => Some(GenshinAction::MoveBackward),
        "A" => Some(GenshinAction::MoveLeft),
        "D" => Some(GenshinAction::MoveRight),
        "LCONTROL" | "LCTRL" | "LEFT_CONTROL" | "LEFT_CTRL" => {
            Some(GenshinAction::SwitchToWalkOrRun)
        }
        "E" => Some(GenshinAction::ElementalSkill),
        "Q" => Some(GenshinAction::ElementalBurst),
        "LSHIFT" | "LEFT_SHIFT" => Some(GenshinAction::SprintKeyboard),
        "R" => Some(GenshinAction::SwitchAimingMode),
        "SPACE" => Some(GenshinAction::Jump),
        "X" => Some(GenshinAction::Drop),
        "F" => Some(GenshinAction::PickUpOrInteract),
        "Z" => Some(GenshinAction::QuickUseGadget),
        "T" => Some(GenshinAction::InteractionInSomeMode),
        "V" => Some(GenshinAction::QuestNavigation),
        "P" => Some(GenshinAction::AbandonChallenge),
        "1" => Some(GenshinAction::SwitchMember1),
        "2" => Some(GenshinAction::SwitchMember2),
        "3" => Some(GenshinAction::SwitchMember3),
        "4" => Some(GenshinAction::SwitchMember4),
        "5" => Some(GenshinAction::SwitchMember5),
        "TAB" => Some(GenshinAction::ShortcutWheel),
        "B" => Some(GenshinAction::OpenInventory),
        "C" => Some(GenshinAction::OpenCharacterScreen),
        "M" => Some(GenshinAction::OpenMap),
        "ESCAPE" | "ESC" => Some(GenshinAction::OpenPaimonMenu),
        "F1" => Some(GenshinAction::OpenAdventurerHandbook),
        "F2" => Some(GenshinAction::OpenCoOpScreen),
        "F3" => Some(GenshinAction::OpenWishScreen),
        "F4" => Some(GenshinAction::OpenBattlePassScreen),
        "F5" => Some(GenshinAction::OpenTheEventsMenu),
        "F6" => Some(GenshinAction::OpenTheSettingsMenu),
        "F7" => Some(GenshinAction::OpenTheFurnishingScreen),
        "F8" => Some(GenshinAction::OpenStellarReunion),
        "J" => Some(GenshinAction::OpenQuestMenu),
        "Y" => Some(GenshinAction::OpenNotificationDetails),
        "RETURN" | "ENTER" => Some(GenshinAction::OpenChatScreen),
        "U" => Some(GenshinAction::OpenSpecialEnvironmentInformation),
        "G" => Some(GenshinAction::CheckTutorialDetails),
        "LMENU" | "LALT" | "LEFT_ALT" => Some(GenshinAction::ShowCursor),
        "L" => Some(GenshinAction::OpenPartySetupScreen),
        "O" => Some(GenshinAction::OpenFriendsScreen),
        "OEM_2" | "SLASH" => Some(GenshinAction::HideUi),
        _ => None,
    }
}
