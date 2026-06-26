use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[path = "hot_key_config.rs"]
mod hot_key_config;

pub use hot_key_config::HotKeyConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeyId(pub u16);

impl KeyId {
    pub const NONE: Self = Self(0x00);
    pub const UNKNOWN: Self = Self(0xFF);
    pub const MOUSE_LEFT_BUTTON: Self = Self(0x01);
    pub const MOUSE_RIGHT_BUTTON: Self = Self(0x02);
    pub const MOUSE_MIDDLE_BUTTON: Self = Self(0x04);
    pub const MOUSE_SIDE_BUTTON1: Self = Self(0x05);
    pub const MOUSE_SIDE_BUTTON2: Self = Self(0x06);
    pub const F1: Self = Self(0x70);
    pub const F2: Self = Self(0x71);
    pub const F3: Self = Self(0x72);
    pub const F4: Self = Self(0x73);
    pub const F5: Self = Self(0x74);
    pub const F6: Self = Self(0x75);
    pub const F7: Self = Self(0x76);
    pub const F8: Self = Self(0x77);
    pub const F9: Self = Self(0x78);
    pub const F10: Self = Self(0x79);
    pub const F11: Self = Self(0x7A);
    pub const F12: Self = Self(0x7B);
    pub const ESCAPE: Self = Self(0x1B);
    pub const PRINT_SCREEN: Self = Self(0x2C);
    pub const SCROLL_LOCK: Self = Self(0x91);
    pub const PAUSE: Self = Self(0x13);
    pub const INSERT: Self = Self(0x2D);
    pub const DELETE: Self = Self(0x2E);
    pub const HOME: Self = Self(0x24);
    pub const END: Self = Self(0x23);
    pub const PAGE_UP: Self = Self(0x21);
    pub const PAGE_DOWN: Self = Self(0x22);
    pub const BACKSPACE: Self = Self(0x08);
    pub const TAB: Self = Self(0x09);
    pub const CAPS_LOCK: Self = Self(0x14);
    pub const ENTER: Self = Self(0x0D);
    pub const LEFT_SHIFT: Self = Self(0xA0);
    pub const RIGHT_SHIFT: Self = Self(0xA1);
    pub const LEFT_CTRL: Self = Self(0xA2);
    pub const RIGHT_CTRL: Self = Self(0xA3);
    pub const LEFT_ALT: Self = Self(0xA4);
    pub const RIGHT_ALT: Self = Self(0xA5);
    pub const LEFT_WIN: Self = Self(0x5B);
    pub const RIGHT_WIN: Self = Self(0x5C);
    pub const APPS: Self = Self(0x5D);
    pub const SPACE: Self = Self(0x20);
    pub const LEFT: Self = Self(0x25);
    pub const UP: Self = Self(0x26);
    pub const RIGHT: Self = Self(0x27);
    pub const DOWN: Self = Self(0x28);
    pub const A: Self = Self(0x41);
    pub const B: Self = Self(0x42);
    pub const C: Self = Self(0x43);
    pub const D: Self = Self(0x44);
    pub const E: Self = Self(0x45);
    pub const F: Self = Self(0x46);
    pub const G: Self = Self(0x47);
    pub const H: Self = Self(0x48);
    pub const I: Self = Self(0x49);
    pub const J: Self = Self(0x4A);
    pub const K: Self = Self(0x4B);
    pub const L: Self = Self(0x4C);
    pub const M: Self = Self(0x4D);
    pub const N: Self = Self(0x4E);
    pub const O: Self = Self(0x4F);
    pub const P: Self = Self(0x50);
    pub const Q: Self = Self(0x51);
    pub const R: Self = Self(0x52);
    pub const S: Self = Self(0x53);
    pub const T: Self = Self(0x54);
    pub const U: Self = Self(0x55);
    pub const V: Self = Self(0x56);
    pub const W: Self = Self(0x57);
    pub const X: Self = Self(0x58);
    pub const Y: Self = Self(0x59);
    pub const Z: Self = Self(0x5A);
    pub const D0: Self = Self(0x30);
    pub const D1: Self = Self(0x31);
    pub const D2: Self = Self(0x32);
    pub const D3: Self = Self(0x33);
    pub const D4: Self = Self(0x34);
    pub const D5: Self = Self(0x35);
    pub const D6: Self = Self(0x36);
    pub const D7: Self = Self(0x37);
    pub const D8: Self = Self(0x38);
    pub const D9: Self = Self(0x39);
    pub const APOSTROPHE: Self = Self(0xDE);
    pub const COMMA: Self = Self(0xBC);
    pub const MINUS: Self = Self(0xBD);
    pub const EQUAL: Self = Self(0xBB);
    pub const PERIOD: Self = Self(0xBE);
    pub const SLASH: Self = Self(0xBF);
    pub const BACKSLASH: Self = Self(0xE2);
    pub const SEMICOLON: Self = Self(0xBA);
    pub const LEFT_SQUARE_BRACKET: Self = Self(0xDB);
    pub const RIGHT_SQUARE_BRACKET: Self = Self(0xDD);
    pub const TILDE: Self = Self(0xC0);
    pub const NUM_LOCK: Self = Self(0x90);
    pub const NUM_PAD0: Self = Self(0x60);
    pub const NUM_PAD1: Self = Self(0x61);
    pub const NUM_PAD2: Self = Self(0x62);
    pub const NUM_PAD3: Self = Self(0x63);
    pub const NUM_PAD4: Self = Self(0x64);
    pub const NUM_PAD5: Self = Self(0x65);
    pub const NUM_PAD6: Self = Self(0x66);
    pub const NUM_PAD7: Self = Self(0x67);
    pub const NUM_PAD8: Self = Self(0x68);
    pub const NUM_PAD9: Self = Self(0x69);
    pub const DECIMAL: Self = Self(0x6E);
    pub const DIVIDE: Self = Self(0x6F);
    pub const MULTIPLY: Self = Self(0x6A);
    pub const SUBTRACT: Self = Self(0x6D);
    pub const ADD: Self = Self(0x6B);
    pub const NUM_ENTER: Self = Self(0x0E);

    pub const fn vk(self) -> u16 {
        self.0
    }

    pub const fn is_mouse_button(self) -> bool {
        matches!(
            self,
            Self::MOUSE_LEFT_BUTTON
                | Self::MOUSE_RIGHT_BUTTON
                | Self::MOUSE_MIDDLE_BUTTON
                | Self::MOUSE_SIDE_BUTTON1
                | Self::MOUSE_SIDE_BUTTON2
        )
    }
}

impl Default for KeyId {
    fn default() -> Self {
        Self::NONE
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct KeyBindingsConfig {
    pub global_key_mapping_enabled: bool,
    pub move_forward: KeyId,
    pub move_backward: KeyId,
    pub move_left: KeyId,
    pub move_right: KeyId,
    pub switch_to_walk_or_run: KeyId,
    pub normal_attack: KeyId,
    pub elemental_skill: KeyId,
    pub elemental_burst: KeyId,
    pub sprint_keyboard: KeyId,
    pub sprint_mouse: KeyId,
    pub switch_aiming_mode: KeyId,
    pub jump: KeyId,
    pub drop: KeyId,
    pub pick_up_or_interact: KeyId,
    pub quick_use_gadget: KeyId,
    pub interaction_in_some_mode: KeyId,
    pub quest_navigation: KeyId,
    pub abandon_challenge: KeyId,
    pub switch_member1: KeyId,
    pub switch_member2: KeyId,
    pub switch_member3: KeyId,
    pub switch_member4: KeyId,
    pub switch_member5: KeyId,
    pub shortcut_wheel: KeyId,
    pub open_inventory: KeyId,
    pub open_character_screen: KeyId,
    pub open_map: KeyId,
    pub open_paimon_menu: KeyId,
    pub open_adventurer_handbook: KeyId,
    pub open_co_op_screen: KeyId,
    pub open_wish_screen: KeyId,
    pub open_battle_pass_screen: KeyId,
    pub open_the_events_menu: KeyId,
    pub open_the_settings_menu: KeyId,
    pub open_the_furnishing_screen: KeyId,
    pub open_stellar_reunion: KeyId,
    pub open_quest_menu: KeyId,
    pub open_notification_details: KeyId,
    pub open_chat_screen: KeyId,
    pub open_special_environment_information: KeyId,
    pub check_tutorial_details: KeyId,
    pub elemental_sight: KeyId,
    pub show_cursor: KeyId,
    pub open_party_setup_screen: KeyId,
    pub open_friends_screen: KeyId,
    pub hide_ui: KeyId,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for KeyBindingsConfig {
    fn default() -> Self {
        Self {
            global_key_mapping_enabled: false,
            move_forward: KeyId::W,
            move_backward: KeyId::S,
            move_left: KeyId::A,
            move_right: KeyId::D,
            switch_to_walk_or_run: KeyId::LEFT_CTRL,
            normal_attack: KeyId::MOUSE_LEFT_BUTTON,
            elemental_skill: KeyId::E,
            elemental_burst: KeyId::Q,
            sprint_keyboard: KeyId::LEFT_SHIFT,
            sprint_mouse: KeyId::MOUSE_RIGHT_BUTTON,
            switch_aiming_mode: KeyId::R,
            jump: KeyId::SPACE,
            drop: KeyId::X,
            pick_up_or_interact: KeyId::F,
            quick_use_gadget: KeyId::Z,
            interaction_in_some_mode: KeyId::T,
            quest_navigation: KeyId::V,
            abandon_challenge: KeyId::P,
            switch_member1: KeyId::D1,
            switch_member2: KeyId::D2,
            switch_member3: KeyId::D3,
            switch_member4: KeyId::D4,
            switch_member5: KeyId::D5,
            shortcut_wheel: KeyId::TAB,
            open_inventory: KeyId::B,
            open_character_screen: KeyId::C,
            open_map: KeyId::M,
            open_paimon_menu: KeyId::ESCAPE,
            open_adventurer_handbook: KeyId::F1,
            open_co_op_screen: KeyId::F2,
            open_wish_screen: KeyId::F3,
            open_battle_pass_screen: KeyId::F4,
            open_the_events_menu: KeyId::F5,
            open_the_settings_menu: KeyId::F6,
            open_the_furnishing_screen: KeyId::F7,
            open_stellar_reunion: KeyId::F8,
            open_quest_menu: KeyId::J,
            open_notification_details: KeyId::Y,
            open_chat_screen: KeyId::ENTER,
            open_special_environment_information: KeyId::U,
            check_tutorial_details: KeyId::G,
            elemental_sight: KeyId::MOUSE_MIDDLE_BUTTON,
            show_cursor: KeyId::LEFT_ALT,
            open_party_setup_screen: KeyId::L,
            open_friends_screen: KeyId::O,
            hide_ui: KeyId::SLASH,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenshinAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    SwitchToWalkOrRun,
    NormalAttack,
    ElementalSkill,
    ElementalBurst,
    SprintKeyboard,
    SprintMouse,
    SwitchAimingMode,
    Jump,
    Drop,
    PickUpOrInteract,
    QuickUseGadget,
    InteractionInSomeMode,
    QuestNavigation,
    AbandonChallenge,
    SwitchMember1,
    SwitchMember2,
    SwitchMember3,
    SwitchMember4,
    SwitchMember5,
    ShortcutWheel,
    OpenInventory,
    OpenCharacterScreen,
    OpenMap,
    OpenPaimonMenu,
    OpenAdventurerHandbook,
    OpenCoOpScreen,
    OpenWishScreen,
    OpenBattlePassScreen,
    OpenTheEventsMenu,
    OpenTheSettingsMenu,
    OpenTheFurnishingScreen,
    OpenStellarReunion,
    OpenQuestMenu,
    OpenNotificationDetails,
    OpenChatScreen,
    OpenSpecialEnvironmentInformation,
    CheckTutorialDetails,
    ElementalSight,
    ShowCursor,
    OpenPartySetupScreen,
    OpenFriendsScreen,
    HideUi,
}

impl KeyBindingsConfig {
    pub fn action_key(&self, action: GenshinAction) -> KeyId {
        match action {
            GenshinAction::MoveForward => self.move_forward,
            GenshinAction::MoveBackward => self.move_backward,
            GenshinAction::MoveLeft => self.move_left,
            GenshinAction::MoveRight => self.move_right,
            GenshinAction::SwitchToWalkOrRun => self.switch_to_walk_or_run,
            GenshinAction::NormalAttack => self.normal_attack,
            GenshinAction::ElementalSkill => self.elemental_skill,
            GenshinAction::ElementalBurst => self.elemental_burst,
            GenshinAction::SprintKeyboard => self.sprint_keyboard,
            GenshinAction::SprintMouse => self.sprint_mouse,
            GenshinAction::SwitchAimingMode => self.switch_aiming_mode,
            GenshinAction::Jump => self.jump,
            GenshinAction::Drop => self.drop,
            GenshinAction::PickUpOrInteract => self.pick_up_or_interact,
            GenshinAction::QuickUseGadget => self.quick_use_gadget,
            GenshinAction::InteractionInSomeMode => self.interaction_in_some_mode,
            GenshinAction::QuestNavigation => self.quest_navigation,
            GenshinAction::AbandonChallenge => self.abandon_challenge,
            GenshinAction::SwitchMember1 => self.switch_member1,
            GenshinAction::SwitchMember2 => self.switch_member2,
            GenshinAction::SwitchMember3 => self.switch_member3,
            GenshinAction::SwitchMember4 => self.switch_member4,
            GenshinAction::SwitchMember5 => self.switch_member5,
            GenshinAction::ShortcutWheel => self.shortcut_wheel,
            GenshinAction::OpenInventory => self.open_inventory,
            GenshinAction::OpenCharacterScreen => self.open_character_screen,
            GenshinAction::OpenMap => self.open_map,
            GenshinAction::OpenPaimonMenu => self.open_paimon_menu,
            GenshinAction::OpenAdventurerHandbook => self.open_adventurer_handbook,
            GenshinAction::OpenCoOpScreen => self.open_co_op_screen,
            GenshinAction::OpenWishScreen => self.open_wish_screen,
            GenshinAction::OpenBattlePassScreen => self.open_battle_pass_screen,
            GenshinAction::OpenTheEventsMenu => self.open_the_events_menu,
            GenshinAction::OpenTheSettingsMenu => self.open_the_settings_menu,
            GenshinAction::OpenTheFurnishingScreen => self.open_the_furnishing_screen,
            GenshinAction::OpenStellarReunion => self.open_stellar_reunion,
            GenshinAction::OpenQuestMenu => self.open_quest_menu,
            GenshinAction::OpenNotificationDetails => self.open_notification_details,
            GenshinAction::OpenChatScreen => self.open_chat_screen,
            GenshinAction::OpenSpecialEnvironmentInformation => {
                self.open_special_environment_information
            }
            GenshinAction::CheckTutorialDetails => self.check_tutorial_details,
            GenshinAction::ElementalSight => self.elemental_sight,
            GenshinAction::ShowCursor => self.show_cursor,
            GenshinAction::OpenPartySetupScreen => self.open_party_setup_screen,
            GenshinAction::OpenFriendsScreen => self.open_friends_screen,
            GenshinAction::HideUi => self.hide_ui,
        }
    }
}
