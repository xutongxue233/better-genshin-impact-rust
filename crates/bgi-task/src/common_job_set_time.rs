use super::{
    image_locator, task_vision_result, CommonJobStep, CommonJobStepAction, CommonJobStepCondition,
    CommonJobStepPhase, RETURN_MAIN_UI_TASK_KEY,
};
use crate::{Result, TaskPortState};
use bgi_input::{InputEvent, InputSequence, MouseButton};
use bgi_vision::{BvLocatorOperation, BvPage, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SET_TIME_TASK_KEY: &str = "SetTime";
pub const SET_TIME_PAGE_CLOSE_WHITE: &str = "Common/Element:page_close_white.png";
pub const SET_TIME_CENTER_X_1080P: f64 = 1441.0;
pub const SET_TIME_CENTER_Y_1080P: f64 = 501.6;
pub const SET_TIME_DEFAULT_STEP_DURATION_MS: u64 = 50;

const TIME_PANEL_X_1080P: f64 = 50.0;
const TIME_PANEL_Y_1080P: f64 = 700.0;
const CONFIRM_MOVE_X_1080P: f64 = 1500.0;
const CONFIRM_MOVE_Y_1080P: f64 = 1000.0;
const SKIP_CLICK_X_1080P: f64 = 45.0;
const SKIP_CLICK_Y_1080P: f64 = 715.0;
const CANCEL_MOVE_X_1080P: f64 = 200.0;
const CANCEL_MOVE_Y_1080P: f64 = 200.0;
const HOUR_CLICK_RADIUS_1: f64 = 30.0;
const HOUR_CLICK_RADIUS_2: f64 = 150.0;
const HOUR_DRAG_RADIUS: f64 = 300.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetTimeExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub requested_hour: i32,
    pub requested_minute: i32,
    pub target_hour: u8,
    pub target_minute: u8,
    pub skip_time_adjustment_animation: bool,
    pub dial_end_index: i32,
    pub dial_click_points: Vec<SetTimeDialPoint>,
    pub dial_drag: SetTimeDialDrag,
    pub steps: Vec<CommonJobStep>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SetTimeDialPoint {
    pub index: f64,
    pub radius: f64,
    pub x_1080p: f64,
    pub y_1080p: f64,
    pub screen_x: i32,
    pub screen_y: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetTimeDialDrag {
    pub from: SetTimeDialPoint,
    pub to: SetTimeDialPoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SetTimeExecutionConfig {
    #[serde(alias = "Hour")]
    pub hour: i32,
    #[serde(alias = "Minute")]
    pub minute: i32,
    #[serde(alias = "skipTimeAdjustmentAnimation")]
    pub skip: bool,
    pub capture_size: Size,
}

impl Default for SetTimeExecutionConfig {
    fn default() -> Self {
        Self {
            hour: 0,
            minute: 0,
            skip: false,
            capture_size: Size::new(1920, 1080),
        }
    }
}

impl SetTimeExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

pub fn plan_set_time(
    capture_size: Size,
    hour: i32,
    minute: i32,
    skip_time_adjustment_animation: bool,
) -> Result<SetTimeExecutionPlan> {
    let (target_hour, target_minute) = normalize_time(hour, minute);
    let dial_end_index = (target_hour as i32 + 6) * 60 + target_minute as i32 - 20;
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let dial_click_points = (-2..=0)
        .map(|offset| {
            dial_point(
                capture_size,
                HOUR_CLICK_RADIUS_1,
                dial_end_index as f64 + offset as f64 * 1440.0 / 3.0,
            )
        })
        .collect::<Vec<_>>();
    let dial_drag = SetTimeDialDrag {
        from: dial_point(
            capture_size,
            HOUR_CLICK_RADIUS_2,
            dial_end_index as f64 + 5.0,
        ),
        to: dial_point(capture_size, HOUR_DRAG_RADIUS, dial_end_index as f64 + 20.5),
    };

    let mut steps = Vec::new();
    steps.push(CommonJobStep::new(
        CommonJobStepPhase::Setup,
        "log set time start",
        CommonJobStepAction::Log {
            message: format!("set time to {target_hour:02}:{target_minute:02}"),
        },
    ));
    steps.push(CommonJobStep::new(
        CommonJobStepPhase::Setup,
        "return to main UI before opening clock",
        CommonJobStepAction::CommonJob {
            task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
            config: None,
        },
    ));
    steps.push(CommonJobStep::new(
        CommonJobStepPhase::TimePanel,
        "press Escape to open menu",
        CommonJobStepAction::Input {
            events: InputSequence::new().key_press(0x1B).events().to_vec(),
        },
    ));
    steps.push(CommonJobStep::new(
        CommonJobStepPhase::TimePanel,
        "wait after opening menu",
        CommonJobStepAction::Page {
            command: task_vision_result(page.wait(800))?,
        },
    ));
    steps.push(CommonJobStep::new(
        CommonJobStepPhase::TimePanel,
        "click clock menu entry",
        CommonJobStepAction::Page {
            command: page.click_1080p(TIME_PANEL_X_1080P, TIME_PANEL_Y_1080P),
        },
    ));
    steps.push(CommonJobStep::new(
        CommonJobStepPhase::TimePanel,
        "wait after opening clock",
        CommonJobStepAction::Page {
            command: task_vision_result(page.wait(900))?,
        },
    ));

    for point in &dial_click_points {
        steps.push(CommonJobStep::new(
            CommonJobStepPhase::TimeDial,
            "click clock dial point",
            CommonJobStepAction::Input {
                events: mouse_click_events(*point, SET_TIME_DEFAULT_STEP_DURATION_MS),
            },
        ));
    }
    steps.push(CommonJobStep::new(
        CommonJobStepPhase::TimeDial,
        "drag clock dial hand",
        CommonJobStepAction::Input {
            events: mouse_drag_events(
                dial_drag.from,
                dial_drag.to,
                SET_TIME_DEFAULT_STEP_DURATION_MS,
            ),
        },
    ));
    steps.push(CommonJobStep::new(
        CommonJobStepPhase::TimeDial,
        "wait after setting clock",
        CommonJobStepAction::Page {
            command: task_vision_result(page.wait(100))?,
        },
    ));
    steps.push(CommonJobStep::new(
        CommonJobStepPhase::TimeDial,
        "move cursor away from clock",
        CommonJobStepAction::Input {
            events: move_events(capture_size, CONFIRM_MOVE_X_1080P, CONFIRM_MOVE_Y_1080P),
        },
    ));
    steps.push(CommonJobStep::new(
        CommonJobStepPhase::TimeDial,
        "wait before confirming time",
        CommonJobStepAction::Page {
            command: task_vision_result(page.wait(300))?,
        },
    ));
    steps.push(CommonJobStep::new(
        CommonJobStepPhase::TimeDial,
        "left click confirm time",
        CommonJobStepAction::Input {
            events: InputSequence::new()
                .mouse_click(MouseButton::Left)
                .events()
                .to_vec(),
        },
    ));
    steps.push(CommonJobStep::new(
        CommonJobStepPhase::TimeDial,
        "wait after confirming time",
        CommonJobStepAction::Page {
            command: task_vision_result(page.wait(7))?,
        },
    ));

    if skip_time_adjustment_animation {
        push_skip_animation_steps(&mut steps, &page, capture_size)?;
    }

    let completion_condition = if skip_time_adjustment_animation {
        CommonJobStepCondition::WhenSkipAnimationNotResolved
    } else {
        CommonJobStepCondition::AfterTimeAdjustment
    };
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Cleanup,
        completion_condition,
        "wait for time adjustment animation",
        CommonJobStepAction::Page {
            command: task_vision_result(page.wait(3_000))?,
        },
    ));
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Cleanup,
        completion_condition,
        "wait for page close button",
        CommonJobStepAction::Locator {
            locator: image_locator(
                &page,
                SET_TIME_PAGE_CLOSE_WHITE,
                Some(right_top_eighth_rect(capture_size)?),
                0.8,
                BvLocatorOperation::WaitFor,
                Some(25_000),
            )?,
        },
    ));
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Cleanup,
        completion_condition,
        "return to main UI after setting time",
        CommonJobStepAction::CommonJob {
            task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
            config: None,
        },
    ));

    Ok(SetTimeExecutionPlan {
        task_key: SET_TIME_TASK_KEY.to_string(),
        display_name: "Set Time".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        requested_hour: hour,
        requested_minute: minute,
        target_hour,
        target_minute,
        skip_time_adjustment_animation,
        dial_end_index,
        dial_click_points,
        dial_drag,
        steps,
        notes: "Legacy SetTime geometry and animation-skip flow are represented as a Rust input/locator/common-job plan with a live template/input executor boundary.".to_string(),
    })
}

fn push_skip_animation_steps(
    steps: &mut Vec<CommonJobStep>,
    page: &BvPage,
    capture_size: Size,
) -> Result<()> {
    let condition = CommonJobStepCondition::WhenSkipAnimationRequested;
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Animation,
        condition,
        "wait before animation cancel",
        CommonJobStepAction::Page {
            command: task_vision_result(page.wait(10))?,
        },
    ));
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Animation,
        condition,
        "cancel time adjustment animation",
        CommonJobStepAction::Input {
            events: cancel_animation_events(capture_size),
        },
    ));
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Animation,
        condition,
        "wait after animation cancel",
        CommonJobStepAction::Page {
            command: task_vision_result(page.wait(1_010))?,
        },
    ));
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Animation,
        condition,
        "click skip entry first time",
        CommonJobStepAction::Page {
            command: page.click_1080p(SKIP_CLICK_X_1080P, SKIP_CLICK_Y_1080P),
        },
    ));
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Animation,
        condition,
        "wait between skip clicks",
        CommonJobStepAction::Page {
            command: task_vision_result(page.wait(100))?,
        },
    ));
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Animation,
        condition,
        "click skip entry second time",
        CommonJobStepAction::Page {
            command: page.click_1080p(SKIP_CLICK_X_1080P, SKIP_CLICK_Y_1080P),
        },
    ));
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Animation,
        condition,
        "wait after skip clicks",
        CommonJobStepAction::Page {
            command: task_vision_result(page.wait(200))?,
        },
    ));
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Animation,
        condition,
        "return to main UI after skip attempt",
        CommonJobStepAction::CommonJob {
            task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
            config: None,
        },
    ));
    Ok(())
}

fn normalize_time(hour: i32, minute: i32) -> (u8, u8) {
    let total_minutes = hour * 60 + minute;
    let normalized_hour = total_minutes.div_euclid(60).rem_euclid(24) as u8;
    let normalized_minute = total_minutes.rem_euclid(60) as u8;
    (normalized_hour, normalized_minute)
}

fn dial_point(capture_size: Size, radius: f64, index: f64) -> SetTimeDialPoint {
    let angle = index * std::f64::consts::PI / 720.0;
    let x_1080p = SET_TIME_CENTER_X_1080P + radius * angle.cos();
    let y_1080p = SET_TIME_CENTER_Y_1080P + radius * angle.sin();
    let (screen_x, screen_y) = screen_point_1080p(capture_size, x_1080p, y_1080p);
    SetTimeDialPoint {
        index,
        radius,
        x_1080p,
        y_1080p,
        screen_x,
        screen_y,
    }
}

fn screen_point_1080p(capture_size: Size, x_1080p: f64, y_1080p: f64) -> (i32, i32) {
    let scale = capture_size.width as f64 / 1920.0;
    (
        (x_1080p * scale).round() as i32,
        (y_1080p * scale).round() as i32,
    )
}

fn mouse_click_events(point: SetTimeDialPoint, step_duration_ms: u64) -> Vec<InputEvent> {
    InputSequence::new()
        .move_mouse_to(point.screen_x, point.screen_y)
        .delay(50)
        .mouse_down(MouseButton::Left)
        .delay(50)
        .mouse_up(MouseButton::Left)
        .delay(step_duration_ms)
        .events()
        .to_vec()
}

fn mouse_drag_events(
    from: SetTimeDialPoint,
    to: SetTimeDialPoint,
    step_duration_ms: u64,
) -> Vec<InputEvent> {
    InputSequence::new()
        .move_mouse_to(from.screen_x, from.screen_y)
        .delay(50)
        .mouse_down(MouseButton::Left)
        .delay(50)
        .move_mouse_to(to.screen_x, to.screen_y)
        .delay(50)
        .mouse_up(MouseButton::Left)
        .delay(step_duration_ms)
        .events()
        .to_vec()
}

fn move_events(capture_size: Size, x_1080p: f64, y_1080p: f64) -> Vec<InputEvent> {
    let (screen_x, screen_y) = screen_point_1080p(capture_size, x_1080p, y_1080p);
    InputSequence::new()
        .move_mouse_to(screen_x, screen_y)
        .events()
        .to_vec()
}

fn cancel_animation_events(capture_size: Size) -> Vec<InputEvent> {
    let (screen_x, screen_y) =
        screen_point_1080p(capture_size, CANCEL_MOVE_X_1080P, CANCEL_MOVE_Y_1080P);
    InputSequence::new()
        .move_mouse_to(screen_x, screen_y)
        .mouse_down(MouseButton::Left)
        .delay(10)
        .mouse_up(MouseButton::Left)
        .events()
        .to_vec()
}

fn right_top_eighth_rect(size: Size) -> Result<Rect> {
    let width = (size.width / 8) as i32;
    task_vision_result(Rect::new(
        size.width as i32 - width,
        0,
        width,
        (size.height / 8) as i32,
    ))
}
