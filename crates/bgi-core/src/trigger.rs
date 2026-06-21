use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameUiCategory {
    Unknown,
    Main,
    BigMap,
    Domain,
    Dialog,
    Inventory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerPortState {
    MetadataOnly,
    CoreReady,
    NativePending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TriggerDescriptor {
    pub key: &'static str,
    pub display_name: &'static str,
    pub priority: i32,
    pub default_enabled: bool,
    pub exclusive: bool,
    pub background: bool,
    pub supported_game_ui_category: GameUiCategory,
    pub port_state: TriggerPortState,
}

pub fn initial_triggers() -> Vec<TriggerDescriptor> {
    let mut triggers = vec![
        TriggerDescriptor {
            key: "RecognitionTest",
            display_name: "Recognition Test",
            priority: 9999,
            default_enabled: false,
            exclusive: false,
            background: false,
            supported_game_ui_category: GameUiCategory::Unknown,
            port_state: TriggerPortState::MetadataOnly,
        },
        TriggerDescriptor {
            key: "GameLoading",
            display_name: "Game Loading",
            priority: 999,
            default_enabled: true,
            exclusive: false,
            background: false,
            supported_game_ui_category: GameUiCategory::Unknown,
            port_state: TriggerPortState::MetadataOnly,
        },
        TriggerDescriptor {
            key: "AutoPick",
            display_name: "Auto Pick",
            priority: 30,
            default_enabled: true,
            exclusive: false,
            background: false,
            supported_game_ui_category: GameUiCategory::Main,
            port_state: TriggerPortState::CoreReady,
        },
        TriggerDescriptor {
            key: "AutoEat",
            display_name: "Auto Eat",
            priority: 25,
            default_enabled: false,
            exclusive: false,
            background: false,
            supported_game_ui_category: GameUiCategory::Main,
            port_state: TriggerPortState::MetadataOnly,
        },
        TriggerDescriptor {
            key: "QuickTeleport",
            display_name: "Quick Teleport",
            priority: 21,
            default_enabled: false,
            exclusive: false,
            background: false,
            supported_game_ui_category: GameUiCategory::BigMap,
            port_state: TriggerPortState::CoreReady,
        },
        TriggerDescriptor {
            key: "AutoSkip",
            display_name: "Auto Skip",
            priority: 20,
            default_enabled: true,
            exclusive: false,
            background: true,
            supported_game_ui_category: GameUiCategory::Dialog,
            port_state: TriggerPortState::CoreReady,
        },
        TriggerDescriptor {
            key: "AutoFish",
            display_name: "Auto Fishing",
            priority: 15,
            default_enabled: false,
            exclusive: false,
            background: false,
            supported_game_ui_category: GameUiCategory::Main,
            port_state: TriggerPortState::MetadataOnly,
        },
        TriggerDescriptor {
            key: "SkillCd",
            display_name: "Skill Cooldown",
            priority: 10,
            default_enabled: false,
            exclusive: false,
            background: false,
            supported_game_ui_category: GameUiCategory::Main,
            port_state: TriggerPortState::MetadataOnly,
        },
        TriggerDescriptor {
            key: "MapMask",
            display_name: "Map Mask",
            priority: 1,
            default_enabled: false,
            exclusive: false,
            background: false,
            supported_game_ui_category: GameUiCategory::BigMap,
            port_state: TriggerPortState::MetadataOnly,
        },
    ];

    triggers.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then(left.key.cmp(right.key))
    });
    triggers
}

pub fn find_trigger(key: &str) -> Option<TriggerDescriptor> {
    initial_triggers()
        .into_iter()
        .find(|trigger| trigger.key.eq_ignore_ascii_case(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_triggers_are_sorted_like_csharp_manager() {
        let triggers = initial_triggers();
        assert_eq!(triggers.first().unwrap().key, "RecognitionTest");
        assert_eq!(triggers[2].key, "AutoPick");
        assert!(triggers
            .windows(2)
            .all(|pair| pair[0].priority >= pair[1].priority));
    }
}
