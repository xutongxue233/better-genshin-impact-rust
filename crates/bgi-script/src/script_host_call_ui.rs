use super::*;
use bgi_input::PostMessageSequence;

impl ScriptHostRuntime {
    pub(super) fn call_post_message(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "keyDown" | "KeyDown" => Ok(post_message_result(
                PostMessageSequence::new()
                    .key_down_background(virtual_key_code_for_script(arg_str(&call, 0)?)?),
            )),
            "keyUp" | "KeyUp" => Ok(post_message_result(
                PostMessageSequence::new()
                    .key_up_background(virtual_key_code_for_script(arg_str(&call, 0)?)?),
            )),
            "keyPress" | "KeyPress" => Ok(post_message_result(
                PostMessageSequence::new()
                    .key_press_background(virtual_key_code_for_script(arg_str(&call, 0)?)?),
            )),
            "click" | "Click" => {
                let sequence = if call.args.len() >= 2 {
                    PostMessageSequence::new()
                        .left_button_click_at(arg_i32(&call, 0)?, arg_i32(&call, 1)?)
                } else {
                    PostMessageSequence::new().left_button_click()
                };
                Ok(post_message_result(sequence))
            }
            _ => Err(unknown_method(&call)),
        }
    }

    pub(super) fn call_strategy_file(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "isFolder" | "IsFolder" => Ok(ScriptHostCallResult::Bool(
                self.strategy_file.is_folder(arg_str(&call, 0)?)?,
            )),
            "isFile" | "IsFile" => Ok(ScriptHostCallResult::Bool(
                self.strategy_file.is_file(arg_str(&call, 0)?)?,
            )),
            "isExists" | "IsExists" => Ok(ScriptHostCallResult::Bool(
                self.strategy_file.is_exists(arg_str(&call, 0)?)?,
            )),
            "readPathSync" | "ReadPathSync" => Ok(ScriptHostCallResult::StringList(
                self.strategy_file
                    .read_path_sync(optional_str(&call, 0)?.unwrap_or("."))?,
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    pub(super) fn call_server_time(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "getServerTimeZoneOffset"
            | "GetServerTimeZoneOffset"
            | "serverTimeZoneOffsetMilliseconds"
            | "ServerTimeZoneOffsetMilliseconds" => Ok(ScriptHostCallResult::Integer(
                self.server_time.server_time_zone_offset_milliseconds() as i64,
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    pub(super) fn call_html_mask(&mut self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "show" | "Show" => Ok(ScriptHostCallResult::HtmlMaskCommand(
                self.html_mask
                    .show(arg_str(&call, 0)?, optional_str(&call, 1)?)?,
            )),
            "close" | "Close" => Ok(ScriptHostCallResult::HtmlMaskCommand(
                self.html_mask.close(arg_str(&call, 0)?),
            )),
            "closeAll" | "CloseAll" => Ok(ScriptHostCallResult::HtmlMaskCommand(
                self.html_mask.close_all(),
            )),
            "getWindowIds" | "GetWindowIds" => Ok(ScriptHostCallResult::StringList(
                self.html_mask.window_ids(),
            )),
            "exists" | "Exists" => Ok(ScriptHostCallResult::Bool(
                self.html_mask.exists(arg_str(&call, 0)?),
            )),
            "setClickThrough" | "SetClickThrough" => Ok(ScriptHostCallResult::HtmlMaskCommand(
                self.html_mask.set_click_through(
                    arg_str(&call, 0)?,
                    optional_bool(&call, 1)?.unwrap_or(true),
                )?,
            )),
            "getClickThrough" | "GetClickThrough" => Ok(ScriptHostCallResult::Bool(
                self.html_mask.get_click_through(arg_str(&call, 0)?)?,
            )),
            "toggleClickThrough" | "ToggleClickThrough" => {
                Ok(ScriptHostCallResult::HtmlMaskCommand(
                    self.html_mask.toggle_click_through(arg_str(&call, 0)?)?,
                ))
            }
            "send" | "Send" => Ok(ScriptHostCallResult::HtmlMaskCommand(self.html_mask.send(
                arg_str(&call, 0)?,
                arg_str(&call, 1)?,
                arg_str(&call, 2)?,
            )?)),
            "request" | "Request" => Ok(ScriptHostCallResult::HtmlMaskCommand(
                self.html_mask.request(
                    arg_str(&call, 0)?,
                    arg_str(&call, 1)?,
                    arg_str(&call, 2)?,
                    optional_u64(&call, 3)?.unwrap_or(0),
                )?,
            )),
            "receive" | "Receive" => Ok(
                match self
                    .html_mask
                    .receive(arg_str(&call, 0)?, optional_u64(&call, 1)?.unwrap_or(0))?
                {
                    Some(message) => ScriptHostCallResult::String(message),
                    None => ScriptHostCallResult::None,
                },
            ),
            "poll" | "Poll" => Ok(match self.html_mask.poll(arg_str(&call, 0)?)? {
                Some(message) => ScriptHostCallResult::String(message),
                None => ScriptHostCallResult::None,
            }),
            "pollAll" | "PollAll" => Ok(ScriptHostCallResult::String(
                self.html_mask.poll_all(arg_str(&call, 0)?)?,
            )),
            "flushPendingMessages" | "FlushPendingMessages" => {
                Ok(ScriptHostCallResult::StringList(
                    self.html_mask.flush_pending_messages(arg_str(&call, 0)?)?,
                ))
            }
            "sendFromHtml" | "SendFromHtml" => {
                self.html_mask.send_from_html(
                    arg_str(&call, 0)?,
                    arg_str(&call, 1)?,
                    arg_str(&call, 2)?,
                    optional_str(&call, 3)?,
                )?;
                Ok(ScriptHostCallResult::None)
            }
            "snapshot" | "Snapshot" => Ok(ScriptHostCallResult::HtmlMaskSnapshot(
                self.html_mask.snapshot(),
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    pub(super) fn call_key_mouse_hook(
        &mut self,
        call: ScriptHostCall,
    ) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "onKeyDown" | "OnKeyDown" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.on_key_down(
                    optional_str(&call, 0)?,
                    optional_bool(&call, 1)?.unwrap_or(true),
                ),
            )),
            "onKeyUp" | "OnKeyUp" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.on_key_up(
                    optional_str(&call, 0)?,
                    optional_bool(&call, 1)?.unwrap_or(true),
                ),
            )),
            "onMouseDown" | "OnMouseDown" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.on_mouse_down(optional_str(&call, 0)?),
            )),
            "onMouseUp" | "OnMouseUp" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.on_mouse_up(optional_str(&call, 0)?),
            )),
            "onMouseMove" | "OnMouseMove" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.on_mouse_move(
                    optional_str(&call, 0)?,
                    optional_u64(&call, 1)?.unwrap_or(200),
                ),
            )),
            "onMouseWheel" | "OnMouseWheel" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.on_mouse_wheel(optional_str(&call, 0)?),
            )),
            "removeAllListeners" | "RemoveAllListeners" => {
                Ok(ScriptHostCallResult::KeyMouseHookCommand(
                    self.key_mouse_hook.remove_all_listeners(),
                ))
            }
            "dispose" | "Dispose" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.dispose(),
            )),
            "dispatchEvent" | "DispatchEvent" => {
                let event = key_mouse_hook::key_mouse_hook_event_from_arg(&call, 0)?;
                Ok(ScriptHostCallResult::KeyMouseHookDispatches(
                    self.key_mouse_hook.dispatch_event(event),
                ))
            }
            "snapshot" | "Snapshot" => Ok(ScriptHostCallResult::KeyMouseHookSnapshot(
                self.key_mouse_hook.snapshot(),
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    pub(super) fn call_custom_host_functions(
        &self,
        call: ScriptHostCall,
    ) -> Result<ScriptHostCallResult> {
        let command = match call.method.as_str() {
            "newVarOfArr" | "NewVarOfArr" => {
                let element_type = arg_str(&call, 0)?.to_string();
                let dimensions = arg_u32_like(&call, 1)?;
                if dimensions == 0 {
                    return Err(invalid_arg(&call, 1, "array dimensions greater than zero"));
                }
                CustomHostFunctionCommand::NewArrayVariable {
                    legacy_jagged_type: legacy_jagged_array_type(&element_type, dimensions),
                    element_type,
                    dimensions,
                }
            }
            "newObj" | "NewObj" => CustomHostFunctionCommand::NewObject {
                type_name: arg_str(&call, 0)?.to_string(),
                args: call.args.iter().skip(1).cloned().collect(),
            },
            "delObj" | "DelObj" => CustomHostFunctionCommand::DeleteObject {
                target: call.args.first().cloned(),
            },
            "type" | "Type" => CustomHostFunctionCommand::TypeLookup {
                type_name: arg_str(&call, 0)?.to_string(),
            },
            "toIterator" | "ToIterator" => CustomHostFunctionCommand::ToIterator {
                source: arg_owned_value(&call, 0)?,
            },
            _ => return Err(unknown_method(&call)),
        };
        Ok(ScriptHostCallResult::CustomHostFunctionCommand(command))
    }
}
