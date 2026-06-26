use super::{task_vision_result, RETURN_MAIN_UI_PAIMON_MENU};
use crate::{Result, TaskPortState};
use bgi_input::{InputEvent, InputSequence};
use bgi_vision::{BvImage, BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const RELOGIN_TASK_KEY: &str = "Relogin";
pub const RELOGIN_MENU_BAG: &str = "AutoWood:menu_bag.png";
pub const RELOGIN_CONFIRM: &str = "AutoWood:confirm.png";
pub const RELOGIN_ENTER_GAME: &str = "AutoWood:exit_welcome.png";
pub const RELOGIN_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const RELOGIN_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;

const VK_ESCAPE: u16 = 0x1B;
const MENU_BAG_ATTEMPTS: u16 = 10;
const MENU_BAG_INTERVAL_MS: u32 = 1_200;
const EXIT_CONFIRM_APPEAR_ATTEMPTS: u16 = 5;
const EXIT_CONFIRM_APPEAR_INTERVAL_MS: u32 = 800;
const EXIT_CONFIRM_DISAPPEAR_ATTEMPTS: u16 = 5;
const EXIT_CONFIRM_DISAPPEAR_INTERVAL_MS: u32 = 1_000;
const AFTER_EXIT_CONFIRM_DELAY_MS: u32 = 1_000;
const THIRD_PARTY_PRE_LOGIN_SLEEP_MS: u32 = 100;
const THIRD_PARTY_MAX_LOGIN_PROBES: u16 = 20;
const THIRD_PARTY_LOGIN_PROBE_INTERVAL_MS: u32 = 500;
const THIRD_PARTY_LOGIN_WINDOW_SLEEP_MS: u32 = 2_000;
const ENTER_GAME_ATTEMPTS: u16 = 120;
const ENTER_GAME_INTERVAL_MS: u32 = 1_000;
const MAIN_UI_ATTEMPTS: u16 = 120;
const MAIN_UI_INTERVAL_MS: u32 = 1_000;
const AFTER_MAIN_UI_DELAY_MS: u32 = 500;
const EXIT_BUTTON_X_1080P: f64 = 50.0;
const EXIT_BUTTON_Y_1080P: f64 = 1030.0;
const ENTER_GAME_X_1080P: f64 = 955.0;
const ENTER_GAME_Y_1080P: f64 = 666.0;
const BILIBILI_AGREEMENT_X_1080P: f64 = 960.0;
const BILIBILI_AGREEMENT_Y_1080P: f64 = 540.0;
const BILIBILI_AGREEMENT_X_DPI_OFFSET: f64 = 70.0;
const BILIBILI_AGREEMENT_Y_DPI_OFFSET: f64 = 75.0;
const BILIBILI_LOGIN_X_1080P: f64 = 960.0;
const BILIBILI_LOGIN_Y_1080P: f64 = 540.0;
const BILIBILI_LOGIN_Y_DPI_OFFSET: f64 = 90.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReloginExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub locators: ReloginLocators,
    pub third_party_rule: ReloginThirdPartyRule,
    pub steps: Vec<ReloginStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReloginLocators {
    pub menu_bag: BvLocatorPlan,
    pub confirm: BvLocatorPlan,
    pub enter_game: BvLocatorPlan,
    pub paimon_menu: BvLocatorPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReloginRetryRule {
    pub max_attempts: u16,
    pub interval_ms: u32,
    pub failure_policy: ReloginFailurePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ReloginFailurePolicy {
    BestEffort,
    HardError { message: String },
    WarningOnly { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReloginThirdPartyRule {
    pub refresh_available_before_login: bool,
    pub bilibili_only: bool,
    pub pre_login_sleep_ms: u32,
    pub max_login_probes: u16,
    pub probe_interval_ms: u32,
    pub agreement_click: ReloginDpiAwarePoint,
    pub login_click: ReloginDpiAwarePoint,
    pub login_window_sleep_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReloginDpiAwarePoint {
    pub x_1080p: f64,
    pub y_1080p: f64,
    pub x_dpi_offset: f64,
    pub y_dpi_offset: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReloginStep {
    pub phase: ReloginStepPhase,
    pub condition: ReloginStepCondition,
    pub label: String,
    pub action: ReloginStepAction,
}

impl ReloginStep {
    fn new(
        phase: ReloginStepPhase,
        condition: ReloginStepCondition,
        label: impl Into<String>,
        action: ReloginStepAction,
    ) -> Self {
        Self {
            phase,
            condition,
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReloginStepPhase {
    ExitToLogin,
    ThirdPartyLogin,
    EnterGame,
    WaitMainUi,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReloginStepCondition {
    Always,
    WhenMenuOpened,
    WhenExitConfirmAppeared,
    WhenLoginScreenVisible,
    WhenEnteredGame,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ReloginStepAction {
    FocusGameWindow,
    RetryUntilAppear {
        locator: BvLocatorPlan,
        rule: ReloginRetryRule,
        retry_action: ReloginRetryAction,
    },
    RetryUntilDisappear {
        locator: BvLocatorPlan,
        rule: ReloginRetryRule,
        retry_action: ReloginRetryAction,
    },
    ThirdPartyLoginProbe {
        rule: ReloginThirdPartyRule,
    },
    Page {
        command: BvPageCommand,
    },
    ReturnResult {
        result: ReloginStepResult,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ReloginRetryAction {
    None,
    Input { events: Vec<InputEvent> },
    Page { command: BvPageCommand },
    Locator { locator: BvLocatorPlan },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReloginStepResult {
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ReloginExecutionConfig {
    pub capture_size: Size,
}

impl Default for ReloginExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                RELOGIN_DEFAULT_CAPTURE_WIDTH,
                RELOGIN_DEFAULT_CAPTURE_HEIGHT,
            ),
        }
    }
}

impl ReloginExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

pub fn plan_relogin(capture_size: Size) -> Result<ReloginExecutionPlan> {
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let locators = ReloginLocators {
        menu_bag: image_locator(
            &page,
            RELOGIN_MENU_BAG,
            Some(menu_bag_roi(capture_size)?),
            0.8,
            BvLocatorOperation::WaitFor,
            Some(MENU_BAG_INTERVAL_MS),
        )?,
        confirm: image_locator(
            &page,
            RELOGIN_CONFIRM,
            None,
            0.8,
            BvLocatorOperation::WaitFor,
            Some(EXIT_CONFIRM_APPEAR_INTERVAL_MS),
        )?,
        enter_game: image_locator(
            &page,
            RELOGIN_ENTER_GAME,
            Some(enter_game_roi(capture_size)?),
            0.8,
            BvLocatorOperation::WaitFor,
            Some(ENTER_GAME_INTERVAL_MS),
        )?,
        paimon_menu: image_locator(
            &page,
            RETURN_MAIN_UI_PAIMON_MENU,
            Some(top_left_quarter_rect(capture_size)?),
            0.8,
            BvLocatorOperation::WaitFor,
            Some(MAIN_UI_INTERVAL_MS),
        )?,
    };
    let confirm_click_locator = with_operation(
        &locators.confirm,
        BvLocatorOperation::Click,
        Some(EXIT_CONFIRM_DISAPPEAR_INTERVAL_MS),
    );
    let enter_game_disappear_locator = with_operation(
        &locators.enter_game,
        BvLocatorOperation::WaitForDisappear,
        Some(ENTER_GAME_INTERVAL_MS),
    );
    let third_party_rule = ReloginThirdPartyRule {
        refresh_available_before_login: true,
        bilibili_only: true,
        pre_login_sleep_ms: THIRD_PARTY_PRE_LOGIN_SLEEP_MS,
        max_login_probes: THIRD_PARTY_MAX_LOGIN_PROBES,
        probe_interval_ms: THIRD_PARTY_LOGIN_PROBE_INTERVAL_MS,
        agreement_click: ReloginDpiAwarePoint {
            x_1080p: BILIBILI_AGREEMENT_X_1080P,
            y_1080p: BILIBILI_AGREEMENT_Y_1080P,
            x_dpi_offset: BILIBILI_AGREEMENT_X_DPI_OFFSET,
            y_dpi_offset: BILIBILI_AGREEMENT_Y_DPI_OFFSET,
        },
        login_click: ReloginDpiAwarePoint {
            x_1080p: BILIBILI_LOGIN_X_1080P,
            y_1080p: BILIBILI_LOGIN_Y_1080P,
            x_dpi_offset: 0.0,
            y_dpi_offset: BILIBILI_LOGIN_Y_DPI_OFFSET,
        },
        login_window_sleep_ms: THIRD_PARTY_LOGIN_WINDOW_SLEEP_MS,
    };
    let steps = vec![
        ReloginStep::new(
            ReloginStepPhase::ExitToLogin,
            ReloginStepCondition::Always,
            "log relogin start",
            ReloginStepAction::Log {
                message: "start Relogin common job plan".to_string(),
            },
        ),
        ReloginStep::new(
            ReloginStepPhase::ExitToLogin,
            ReloginStepCondition::Always,
            "focus game window",
            ReloginStepAction::FocusGameWindow,
        ),
        ReloginStep::new(
            ReloginStepPhase::ExitToLogin,
            ReloginStepCondition::Always,
            "open pause menu with Escape",
            ReloginStepAction::RetryUntilAppear {
                locator: locators.menu_bag.clone(),
                rule: ReloginRetryRule {
                    max_attempts: MENU_BAG_ATTEMPTS,
                    interval_ms: MENU_BAG_INTERVAL_MS,
                    failure_policy: ReloginFailurePolicy::BestEffort,
                },
                retry_action: ReloginRetryAction::Input {
                    events: InputSequence::new().key_press(VK_ESCAPE).events().to_vec(),
                },
            },
        ),
        ReloginStep::new(
            ReloginStepPhase::ExitToLogin,
            ReloginStepCondition::WhenMenuOpened,
            "click exit button until confirm dialog appears",
            ReloginStepAction::RetryUntilAppear {
                locator: locators.confirm.clone(),
                rule: ReloginRetryRule {
                    max_attempts: EXIT_CONFIRM_APPEAR_ATTEMPTS,
                    interval_ms: EXIT_CONFIRM_APPEAR_INTERVAL_MS,
                    failure_policy: ReloginFailurePolicy::BestEffort,
                },
                retry_action: ReloginRetryAction::Page {
                    command: page.click_1080p(EXIT_BUTTON_X_1080P, EXIT_BUTTON_Y_1080P),
                },
            },
        ),
        ReloginStep::new(
            ReloginStepPhase::ExitToLogin,
            ReloginStepCondition::WhenExitConfirmAppeared,
            "click confirm until dialog disappears",
            ReloginStepAction::RetryUntilDisappear {
                locator: with_operation(
                    &locators.confirm,
                    BvLocatorOperation::WaitForDisappear,
                    Some(EXIT_CONFIRM_DISAPPEAR_INTERVAL_MS),
                ),
                rule: ReloginRetryRule {
                    max_attempts: EXIT_CONFIRM_DISAPPEAR_ATTEMPTS,
                    interval_ms: EXIT_CONFIRM_DISAPPEAR_INTERVAL_MS,
                    failure_policy: ReloginFailurePolicy::BestEffort,
                },
                retry_action: ReloginRetryAction::Locator {
                    locator: confirm_click_locator,
                },
            },
        ),
        ReloginStep::new(
            ReloginStepPhase::ExitToLogin,
            ReloginStepCondition::Always,
            "wait after confirming exit",
            ReloginStepAction::Page {
                command: task_vision_result(page.wait(AFTER_EXIT_CONFIRM_DELAY_MS))?,
            },
        ),
        ReloginStep::new(
            ReloginStepPhase::ThirdPartyLogin,
            ReloginStepCondition::Always,
            "probe third-party login window",
            ReloginStepAction::ThirdPartyLoginProbe {
                rule: third_party_rule.clone(),
            },
        ),
        ReloginStep::new(
            ReloginStepPhase::EnterGame,
            ReloginStepCondition::Always,
            "wait for enter-game button",
            ReloginStepAction::RetryUntilAppear {
                locator: locators.enter_game.clone(),
                rule: ReloginRetryRule {
                    max_attempts: ENTER_GAME_ATTEMPTS,
                    interval_ms: ENTER_GAME_INTERVAL_MS,
                    failure_policy: ReloginFailurePolicy::HardError {
                        message: "未检测进入游戏界面".to_string(),
                    },
                },
                retry_action: ReloginRetryAction::None,
            },
        ),
        ReloginStep::new(
            ReloginStepPhase::EnterGame,
            ReloginStepCondition::WhenLoginScreenVisible,
            "click enter-game button until it disappears",
            ReloginStepAction::RetryUntilDisappear {
                locator: enter_game_disappear_locator,
                rule: ReloginRetryRule {
                    max_attempts: ENTER_GAME_ATTEMPTS,
                    interval_ms: ENTER_GAME_INTERVAL_MS,
                    failure_policy: ReloginFailurePolicy::HardError {
                        message: "未检测到进入游戏按钮消失, 可能未点击成功".to_string(),
                    },
                },
                retry_action: ReloginRetryAction::Page {
                    command: page.click_1080p(ENTER_GAME_X_1080P, ENTER_GAME_Y_1080P),
                },
            },
        ),
        ReloginStep::new(
            ReloginStepPhase::WaitMainUi,
            ReloginStepCondition::WhenEnteredGame,
            "wait for main UI after entering game",
            ReloginStepAction::RetryUntilAppear {
                locator: locators.paimon_menu.clone(),
                rule: ReloginRetryRule {
                    max_attempts: MAIN_UI_ATTEMPTS,
                    interval_ms: MAIN_UI_INTERVAL_MS,
                    failure_policy: ReloginFailurePolicy::WarningOnly {
                        message: "未检测到主界面，登录可能未完成".to_string(),
                    },
                },
                retry_action: ReloginRetryAction::None,
            },
        ),
        ReloginStep::new(
            ReloginStepPhase::Cleanup,
            ReloginStepCondition::Always,
            "wait after relogin",
            ReloginStepAction::Page {
                command: task_vision_result(page.wait(AFTER_MAIN_UI_DELAY_MS))?,
            },
        ),
        ReloginStep::new(
            ReloginStepPhase::Cleanup,
            ReloginStepCondition::Always,
            "return completed result",
            ReloginStepAction::ReturnResult {
                result: ReloginStepResult::Completed,
            },
        ),
    ];

    Ok(ReloginExecutionPlan {
        task_key: RELOGIN_TASK_KEY.to_string(),
        display_name: "Relogin".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        locators,
        third_party_rule,
        steps,
        notes: "Legacy ExitAndRelogin UI flow is represented as a Rust common-job plan with an injectable retry/failure-policy executor, template live runtime, game-window focus, and Bilibili third-party login platform hooks.".to_string(),
    })
}

fn image_locator(
    page: &BvPage,
    asset: &str,
    roi: Option<Rect>,
    threshold: f64,
    operation: BvLocatorOperation,
    timeout_ms: Option<u32>,
) -> Result<BvLocatorPlan> {
    let image = task_vision_result(BvImage::new(asset))?;
    let locator = task_vision_result(page.locator_for_image(&image, roi, threshold))?;
    Ok(locator.plan(operation, timeout_ms))
}

fn with_operation(
    locator: &BvLocatorPlan,
    operation: BvLocatorOperation,
    timeout_ms: Option<u32>,
) -> BvLocatorPlan {
    let mut locator = locator.clone();
    locator.operation = operation;
    if let Some(timeout_ms) = timeout_ms {
        locator.timeout_ms = timeout_ms;
    }
    locator
}

fn menu_bag_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(0, 0, (size.width / 2) as i32, size.height as i32))
}

fn enter_game_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        (size.height / 2) as i32,
        size.width as i32,
        (size.height - size.height / 2) as i32,
    ))
}

fn top_left_quarter_rect(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        0,
        (size.width / 4) as i32,
        (size.height / 4) as i32,
    ))
}
