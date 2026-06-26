use super::{arg_value, i32_like, invalid_arg, u64_like, Result, ScriptHostCall};
use bgi_input::MouseButton;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum KeyMouseHookEventKind {
    KeyDown,
    KeyUp,
    MouseDown,
    MouseUp,
    MouseMove,
    MouseWheel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyMouseHookListener {
    pub id: String,
    pub event: KeyMouseHookEventKind,
    pub use_code_only: bool,
    pub interval_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum KeyMouseHookCommand {
    AddListener(KeyMouseHookListener),
    RemoveAllListeners,
    Dispose,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum KeyMouseHookEvent {
    Key {
        event: KeyMouseHookEventKind,
        key_data: String,
        key_code: String,
    },
    MouseButton {
        event: KeyMouseHookEventKind,
        button: MouseButton,
        x: i32,
        y: i32,
    },
    MouseMove {
        x: i32,
        y: i32,
        timestamp_ms: u64,
    },
    MouseWheel {
        delta: i32,
        x: i32,
        y: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KeyMouseHookDispatch {
    pub listener_id: String,
    pub event: KeyMouseHookEventKind,
    pub args: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyMouseHookSnapshot {
    pub listeners: Vec<KeyMouseHookListener>,
    pub commands: Vec<KeyMouseHookCommand>,
    pub disposed: bool,
}

#[derive(Debug, Clone, Default)]
pub struct KeyMouseHookHost {
    listeners: Vec<KeyMouseHookListener>,
    commands: Vec<KeyMouseHookCommand>,
    last_global_mouse_move_ms: Option<u64>,
    last_listener_mouse_move_ms: HashMap<String, u64>,
    next_listener_id: u64,
    disposed: bool,
}

impl KeyMouseHookHost {
    pub fn listeners(&self) -> &[KeyMouseHookListener] {
        &self.listeners
    }

    pub fn commands(&self) -> &[KeyMouseHookCommand] {
        &self.commands
    }

    pub fn snapshot(&self) -> KeyMouseHookSnapshot {
        KeyMouseHookSnapshot {
            listeners: self.listeners.clone(),
            commands: self.commands.clone(),
            disposed: self.disposed,
        }
    }

    pub fn on_key_down(
        &mut self,
        callback_id: Option<&str>,
        use_code_only: bool,
    ) -> KeyMouseHookCommand {
        self.add_listener(
            KeyMouseHookEventKind::KeyDown,
            callback_id,
            use_code_only,
            None,
        )
    }

    pub fn on_key_up(
        &mut self,
        callback_id: Option<&str>,
        use_code_only: bool,
    ) -> KeyMouseHookCommand {
        self.add_listener(
            KeyMouseHookEventKind::KeyUp,
            callback_id,
            use_code_only,
            None,
        )
    }

    pub fn on_mouse_down(&mut self, callback_id: Option<&str>) -> KeyMouseHookCommand {
        self.add_listener(KeyMouseHookEventKind::MouseDown, callback_id, true, None)
    }

    pub fn on_mouse_up(&mut self, callback_id: Option<&str>) -> KeyMouseHookCommand {
        self.add_listener(KeyMouseHookEventKind::MouseUp, callback_id, true, None)
    }

    pub fn on_mouse_move(
        &mut self,
        callback_id: Option<&str>,
        interval_ms: u64,
    ) -> KeyMouseHookCommand {
        self.add_listener(
            KeyMouseHookEventKind::MouseMove,
            callback_id,
            true,
            Some(interval_ms),
        )
    }

    pub fn on_mouse_wheel(&mut self, callback_id: Option<&str>) -> KeyMouseHookCommand {
        self.add_listener(KeyMouseHookEventKind::MouseWheel, callback_id, true, None)
    }

    pub fn remove_all_listeners(&mut self) -> KeyMouseHookCommand {
        self.listeners.clear();
        self.last_listener_mouse_move_ms.clear();
        let command = KeyMouseHookCommand::RemoveAllListeners;
        self.commands.push(command.clone());
        command
    }

    pub fn dispose(&mut self) -> KeyMouseHookCommand {
        self.remove_all_listeners();
        self.disposed = true;
        let command = KeyMouseHookCommand::Dispose;
        self.commands.push(command.clone());
        command
    }

    pub fn dispatch_event(&mut self, event: KeyMouseHookEvent) -> Vec<KeyMouseHookDispatch> {
        if self.disposed {
            return Vec::new();
        }

        let event_kind = event.kind();
        if let KeyMouseHookEvent::MouseMove { timestamp_ms, .. } = event {
            if let Some(last) = self.last_global_mouse_move_ms {
                if timestamp_ms.saturating_sub(last) < 10 {
                    return Vec::new();
                }
            }
            self.last_global_mouse_move_ms = Some(timestamp_ms);
        }

        let listeners = self
            .listeners
            .iter()
            .filter(|listener| listener.event == event_kind)
            .cloned()
            .collect::<Vec<_>>();
        listeners
            .into_iter()
            .filter_map(|listener| self.dispatch_to_listener(&listener, &event))
            .collect()
    }

    fn add_listener(
        &mut self,
        event: KeyMouseHookEventKind,
        callback_id: Option<&str>,
        use_code_only: bool,
        interval_ms: Option<u64>,
    ) -> KeyMouseHookCommand {
        self.disposed = false;
        let listener = KeyMouseHookListener {
            id: callback_id
                .filter(|id| !id.trim().is_empty())
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| self.next_listener_id()),
            event,
            use_code_only,
            interval_ms,
        };
        let command = KeyMouseHookCommand::AddListener(listener.clone());
        self.listeners.push(listener);
        self.commands.push(command.clone());
        command
    }

    fn dispatch_to_listener(
        &mut self,
        listener: &KeyMouseHookListener,
        event: &KeyMouseHookEvent,
    ) -> Option<KeyMouseHookDispatch> {
        let args = match event {
            KeyMouseHookEvent::Key {
                key_data, key_code, ..
            } => vec![Value::String(if listener.use_code_only {
                key_code.clone()
            } else {
                key_data.clone()
            })],
            KeyMouseHookEvent::MouseButton { button, x, y, .. } => vec![
                Value::String(mouse_button_name(*button).to_string()),
                serde_json::json!(x),
                serde_json::json!(y),
            ],
            KeyMouseHookEvent::MouseMove { x, y, timestamp_ms } => {
                let interval = listener.interval_ms.unwrap_or(200);
                let last = self.last_listener_mouse_move_ms.get(&listener.id).copied();
                if last
                    .map(|last| timestamp_ms.saturating_sub(last) < interval)
                    .unwrap_or(false)
                {
                    return None;
                }
                self.last_listener_mouse_move_ms
                    .insert(listener.id.clone(), *timestamp_ms);
                vec![serde_json::json!(x), serde_json::json!(y)]
            }
            KeyMouseHookEvent::MouseWheel { delta, x, y } => {
                vec![
                    serde_json::json!(delta),
                    serde_json::json!(x),
                    serde_json::json!(y),
                ]
            }
        };
        Some(KeyMouseHookDispatch {
            listener_id: listener.id.clone(),
            event: listener.event,
            args,
        })
    }

    fn next_listener_id(&mut self) -> String {
        let id = format!("listener-{}", self.next_listener_id);
        self.next_listener_id = self.next_listener_id.saturating_add(1);
        id
    }
}

impl KeyMouseHookEvent {
    fn kind(&self) -> KeyMouseHookEventKind {
        match self {
            Self::Key { event, .. } => *event,
            Self::MouseButton { event, .. } => *event,
            Self::MouseMove { .. } => KeyMouseHookEventKind::MouseMove,
            Self::MouseWheel { .. } => KeyMouseHookEventKind::MouseWheel,
        }
    }
}

pub(super) fn key_mouse_hook_event_from_arg(
    call: &ScriptHostCall,
    index: usize,
) -> Result<KeyMouseHookEvent> {
    let value = arg_value(call, index, "key/mouse hook event object")?;
    let Value::Object(map) = value else {
        return Err(invalid_arg(call, index, "key/mouse hook event object"));
    };
    let event_type = map
        .get("type")
        .or_else(|| map.get("Type"))
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_arg(call, index, "event.type"))?;

    match event_type {
        "keyDown" | "KeyDown" => Ok(KeyMouseHookEvent::Key {
            event: KeyMouseHookEventKind::KeyDown,
            key_data: hook_event_string(&map, "keyData", "KeyData", "key_data", "Unknown"),
            key_code: hook_event_string(&map, "keyCode", "KeyCode", "key_code", "Unknown"),
        }),
        "keyUp" | "KeyUp" => Ok(KeyMouseHookEvent::Key {
            event: KeyMouseHookEventKind::KeyUp,
            key_data: hook_event_string(&map, "keyData", "KeyData", "key_data", "Unknown"),
            key_code: hook_event_string(&map, "keyCode", "KeyCode", "key_code", "Unknown"),
        }),
        "mouseDown" | "MouseDown" => Ok(KeyMouseHookEvent::MouseButton {
            event: KeyMouseHookEventKind::MouseDown,
            button: hook_event_mouse_button(&map),
            x: hook_event_i32(&map, "x", "X", "localX", 0),
            y: hook_event_i32(&map, "y", "Y", "localY", 0),
        }),
        "mouseUp" | "MouseUp" => Ok(KeyMouseHookEvent::MouseButton {
            event: KeyMouseHookEventKind::MouseUp,
            button: hook_event_mouse_button(&map),
            x: hook_event_i32(&map, "x", "X", "localX", 0),
            y: hook_event_i32(&map, "y", "Y", "localY", 0),
        }),
        "mouseMove" | "MouseMove" => Ok(KeyMouseHookEvent::MouseMove {
            x: hook_event_i32(&map, "x", "X", "localX", 0),
            y: hook_event_i32(&map, "y", "Y", "localY", 0),
            timestamp_ms: hook_event_u64(&map, "timestampMs", "TimestampMs", "timestamp_ms", 0),
        }),
        "mouseWheel" | "MouseWheel" => Ok(KeyMouseHookEvent::MouseWheel {
            delta: hook_event_i32(&map, "delta", "Delta", "wheelDelta", 0),
            x: hook_event_i32(&map, "x", "X", "localX", 0),
            y: hook_event_i32(&map, "y", "Y", "localY", 0),
        }),
        _ => Err(invalid_arg(call, index, "known key/mouse hook event type")),
    }
}

fn hook_event_string(
    map: &serde_json::Map<String, Value>,
    primary: &str,
    secondary: &str,
    tertiary: &str,
    default: &str,
) -> String {
    map.get(primary)
        .or_else(|| map.get(secondary))
        .or_else(|| map.get(tertiary))
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_string()
}

fn hook_event_i32(
    map: &serde_json::Map<String, Value>,
    primary: &str,
    secondary: &str,
    tertiary: &str,
    default: i32,
) -> i32 {
    map.get(primary)
        .or_else(|| map.get(secondary))
        .or_else(|| map.get(tertiary))
        .and_then(i32_like)
        .unwrap_or(default)
}

fn hook_event_u64(
    map: &serde_json::Map<String, Value>,
    primary: &str,
    secondary: &str,
    tertiary: &str,
    default: u64,
) -> u64 {
    map.get(primary)
        .or_else(|| map.get(secondary))
        .or_else(|| map.get(tertiary))
        .and_then(u64_like)
        .unwrap_or(default)
}

fn hook_event_mouse_button(map: &serde_json::Map<String, Value>) -> MouseButton {
    let button = map
        .get("button")
        .or_else(|| map.get("Button"))
        .and_then(Value::as_str)
        .unwrap_or("Left");
    parse_mouse_button_name(button)
}

fn mouse_button_name(button: MouseButton) -> &'static str {
    match button {
        MouseButton::Left => "Left",
        MouseButton::Middle => "Middle",
        MouseButton::Right => "Right",
        MouseButton::X(1) => "XButton1",
        MouseButton::X(2) => "XButton2",
        MouseButton::X(_) => "XButton",
    }
}

fn parse_mouse_button_name(button: &str) -> MouseButton {
    match button {
        "Middle" | "middle" => MouseButton::Middle,
        "Right" | "right" => MouseButton::Right,
        "XButton1" | "xButton1" | "X1" | "x1" => MouseButton::X(1),
        "XButton2" | "xButton2" | "X2" | "x2" => MouseButton::X(2),
        _ => MouseButton::Left,
    }
}
