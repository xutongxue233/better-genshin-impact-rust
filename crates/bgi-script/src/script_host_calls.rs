use super::*;
use bgi_input::InputSequence;
use serde_json::Value;

impl ScriptHostRuntime {
    pub(super) fn call_global(&mut self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "sleep" | "Sleep" => {
                self.global_input_result(InputSequence::new().delay(arg_u64(&call, 0)?))
            }
            "getVersion" | "GetVersion" => Ok(ScriptHostCallResult::String(
                env!("CARGO_PKG_VERSION").to_string(),
            )),
            "keyDown" | "KeyDown" => {
                self.global_input_result(self.global_input.key_down(arg_str(&call, 0)?)?)
            }
            "keyUp" | "KeyUp" => {
                self.global_input_result(self.global_input.key_up(arg_str(&call, 0)?)?)
            }
            "keyPress" | "KeyPress" => {
                self.global_input_result(self.global_input.key_press(arg_str(&call, 0)?)?)
            }
            "setGameMetrics" | "SetGameMetrics" => {
                let dpi = optional_f64(&call, 2)?.unwrap_or(1.0);
                self.global_input
                    .set_game_metrics(arg_u32(&call, 0)?, arg_u32(&call, 1)?, dpi)?;
                Ok(ScriptHostCallResult::None)
            }
            "getGameMetrics" | "GetGameMetrics" => Ok(ScriptHostCallResult::GameMetrics(
                self.global_input.game_metrics(),
            )),
            "moveMouseBy" | "MoveMouseBy" => self.global_input_result(
                self.global_input
                    .move_mouse_by(arg_i32(&call, 0)?, arg_i32(&call, 1)?),
            ),
            "moveMouseTo" | "MoveMouseTo" => self.global_input_result(
                self.global_input
                    .move_mouse_to(arg_i32(&call, 0)?, arg_i32(&call, 1)?)?,
            ),
            "click" | "Click" => self.global_input_result(
                self.global_input
                    .click(arg_i32(&call, 0)?, arg_i32(&call, 1)?)?,
            ),
            "leftButtonClick" | "LeftButtonClick" => {
                self.global_input_result(self.global_input.left_button_click())
            }
            "leftButtonDown" | "LeftButtonDown" => {
                self.global_input_result(self.global_input.left_button_down())
            }
            "leftButtonUp" | "LeftButtonUp" => {
                self.global_input_result(self.global_input.left_button_up())
            }
            "rightButtonClick" | "RightButtonClick" => {
                self.global_input_result(self.global_input.right_button_click())
            }
            "rightButtonDown" | "RightButtonDown" => {
                self.global_input_result(self.global_input.right_button_down())
            }
            "rightButtonUp" | "RightButtonUp" => {
                self.global_input_result(self.global_input.right_button_up())
            }
            "middleButtonClick" | "MiddleButtonClick" => {
                self.global_input_result(self.global_input.middle_button_click())
            }
            "middleButtonDown" | "MiddleButtonDown" => {
                self.global_input_result(self.global_input.middle_button_down())
            }
            "middleButtonUp" | "MiddleButtonUp" => {
                self.global_input_result(self.global_input.middle_button_up())
            }
            "verticalScroll" | "VerticalScroll" => {
                self.global_input_result(self.global_input.vertical_scroll(arg_i32(&call, 0)?))
            }
            "captureGameRegion" | "CaptureGameRegion" => {
                if let Some(execution) = self.global_input.capture_game_region_execution()? {
                    Ok(ScriptHostCallResult::CaptureGameRegionExecution(execution))
                } else {
                    Ok(ScriptHostCallResult::CaptureGameRegionPlan(
                        self.global_input.capture_game_region(),
                    ))
                }
            }
            "getAvatars" | "GetAvatars" => Ok(ScriptHostCallResult::AvatarRecognitionPlan(
                self.global_input.get_avatars(),
            )),
            "inputText" | "InputText" => {
                self.global_input_result(self.global_input.input_text(arg_str(&call, 0)?))
            }
            _ => Err(unknown_method(&call)),
        }
    }

    pub(super) fn global_input_result(
        &self,
        sequence: InputSequence,
    ) -> Result<ScriptHostCallResult> {
        Ok(ScriptHostCallResult::InputExecution(
            GlobalInputExecution::execute(
                sequence,
                self.global_input_dispatch_mode,
                self.input_window_handle,
            )?,
        ))
    }

    pub(super) fn call_genshin(&mut self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        let command = match call.method.as_str() {
            "uid" | "Uid" => GenshinCommand::Uid,
            "tp" | "Tp" => {
                let (map_name, force) = match call.args.get(2) {
                    None | Some(Value::Null) => (None, false),
                    Some(Value::String(map_name)) => (
                        Some(map_name.clone()),
                        optional_bool(&call, 3)?.unwrap_or(false),
                    ),
                    Some(Value::Bool(force)) => (None, *force),
                    Some(_) => return Err(invalid_arg(&call, 2, "string or bool")),
                };
                GenshinCommand::Teleport {
                    x: arg_f64_like(&call, 0)?,
                    y: arg_f64_like(&call, 1)?,
                    map_name,
                    force,
                }
            }
            "moveMapTo" | "MoveMapTo" => GenshinCommand::MoveMapTo {
                x: arg_f64_like(&call, 0)?,
                y: arg_f64_like(&call, 1)?,
                map_name: None,
                force_country: optional_str(&call, 2)?.map(ToOwned::to_owned),
            },
            "moveIndependentMapTo" | "MoveIndependentMapTo" => GenshinCommand::MoveMapTo {
                x: arg_f64_like(&call, 0)?,
                y: arg_f64_like(&call, 1)?,
                map_name: Some(arg_str(&call, 2)?.to_string()),
                force_country: optional_str(&call, 3)?.map(ToOwned::to_owned),
            },
            "getBigMapZoomLevel" | "GetBigMapZoomLevel" => GenshinCommand::GetBigMapZoomLevel,
            "setBigMapZoomLevel" | "SetBigMapZoomLevel" => GenshinCommand::SetBigMapZoomLevel {
                zoom_level: arg_f64_like(&call, 0)?,
            },
            "tpToStatueOfTheSeven" | "TpToStatueOfTheSeven" => GenshinCommand::TpToStatueOfTheSeven,
            "getPositionFromBigMap" | "GetPositionFromBigMap" => {
                GenshinCommand::GetPositionFromBigMap {
                    map_name: optional_str(&call, 0)?.map(ToOwned::to_owned),
                }
            }
            "getPositionFromMap" | "GetPositionFromMap" => {
                let nearby = if call.args.len() >= 3 {
                    Some((arg_f64_like(&call, 1)?, arg_f64_like(&call, 2)?))
                } else {
                    None
                };
                GenshinCommand::GetPositionFromMap {
                    map_name: optional_str(&call, 0)?.map(ToOwned::to_owned),
                    cache_time_ms: if nearby.is_some() {
                        None
                    } else {
                        optional_u64(&call, 1)?
                    },
                    matching_method: None,
                    nearby,
                }
            }
            "getPositionFromMapWithMatchingMethod" | "GetPositionFromMapWithMatchingMethod" => {
                if call.args.len() == 1 {
                    GenshinCommand::GetPositionFromMap {
                        map_name: None,
                        matching_method: Some(arg_str(&call, 0)?.to_string()),
                        cache_time_ms: None,
                        nearby: None,
                    }
                } else {
                    GenshinCommand::GetPositionFromMap {
                        map_name: optional_str(&call, 0)?.map(ToOwned::to_owned),
                        matching_method: Some(arg_str(&call, 1)?.to_string()),
                        cache_time_ms: optional_u64(&call, 2)?,
                        nearby: None,
                    }
                }
            }
            "getCameraOrientation" | "GetCameraOrientation" => GenshinCommand::GetCameraOrientation,
            "switchParty" | "SwitchParty" => GenshinCommand::SwitchParty {
                party_name: arg_str(&call, 0)?.to_string(),
            },
            "clearPartyCache" | "ClearPartyCache" => GenshinCommand::ClearPartyCache,
            "blessingOfTheWelkinMoon" | "BlessingOfTheWelkinMoon" => {
                GenshinCommand::BlessingOfTheWelkinMoon
            }
            "chooseTalkOption" | "ChooseTalkOption" => GenshinCommand::ChooseTalkOption {
                option: arg_str(&call, 0)?.to_string(),
                skip_times: optional_u32_like(&call, 1)?.unwrap_or(10),
                is_orange: optional_bool(&call, 2)?.unwrap_or(false),
            },
            "claimBattlePassRewards" | "ClaimBattlePassRewards" => {
                GenshinCommand::ClaimBattlePassRewards
            }
            "claimEncounterPointsRewards" | "ClaimEncounterPointsRewards" => {
                GenshinCommand::ClaimEncounterPointsRewards
            }
            "goToAdventurersGuild" | "GoToAdventurersGuild" => {
                GenshinCommand::GoToAdventurersGuild {
                    country: arg_str(&call, 0)?.to_string(),
                }
            }
            "goToCraftingBench" | "GoToCraftingBench" => GenshinCommand::GoToCraftingBench {
                country: arg_str(&call, 0)?.to_string(),
            },
            "returnMainUi" | "ReturnMainUi" => GenshinCommand::ReturnMainUi,
            "autoFishing" | "AutoFishing" => GenshinCommand::AutoFishing {
                fishing_time_policy: optional_i32(&call, 0)?.unwrap_or(0),
            },
            "relogin" | "Relogin" => GenshinCommand::Relogin,
            "wonderlandCycle" | "WonderlandCycle" => GenshinCommand::WonderlandCycle,
            "setTime" | "SetTime" => {
                let hour = arg_u32_like(&call, 0)?;
                let minute = arg_u32_like(&call, 1)?;
                if hour > 23 {
                    return Err(invalid_arg(&call, 0, "hour 0..=23"));
                }
                if minute > 59 {
                    return Err(invalid_arg(&call, 1, "minute 0..=59"));
                }
                GenshinCommand::SetTime {
                    hour,
                    minute,
                    skip: optional_bool(&call, 2)?.unwrap_or(false),
                }
            }
            "commands" | "Commands" => {
                return Ok(ScriptHostCallResult::GenshinCommands(
                    self.genshin.commands().to_vec(),
                ));
            }
            _ => return Err(unknown_method(&call)),
        };
        Ok(ScriptHostCallResult::GenshinCommand(
            self.genshin.push(command),
        ))
    }

    pub(super) fn call_pathing_script(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "run" | "Run" => Ok(ScriptHostCallResult::PathingExecution(
                self.pathing_script.execute(arg_str(&call, 0)?)?,
            )),
            "runFile" | "RunFile" => Ok(ScriptHostCallResult::PathingExecution(
                self.pathing_script.execute_file(arg_str(&call, 0)?)?,
            )),
            "runFileFromUser" | "RunFileFromUser" => Ok(ScriptHostCallResult::PathingExecution(
                self.pathing_script
                    .execute_file_from_user(arg_str(&call, 0)?)?,
            )),
            "plan" | "Plan" => Ok(ScriptHostCallResult::PathingPlan(
                self.pathing_script.run(arg_str(&call, 0)?)?,
            )),
            "planFile" | "PlanFile" => Ok(ScriptHostCallResult::PathingPlan(
                self.pathing_script.run_file(arg_str(&call, 0)?)?,
            )),
            "planFileFromUser" | "PlanFileFromUser" => Ok(ScriptHostCallResult::PathingPlan(
                self.pathing_script.run_file_from_user(arg_str(&call, 0)?)?,
            )),
            "isExists" | "IsExists" => Ok(ScriptHostCallResult::Bool(
                self.pathing_script.is_exists(arg_str(&call, 0)?)?,
            )),
            "isFile" | "IsFile" => Ok(ScriptHostCallResult::Bool(
                self.pathing_script.is_file(arg_str(&call, 0)?)?,
            )),
            "isFolder" | "IsFolder" => Ok(ScriptHostCallResult::Bool(
                self.pathing_script.is_folder(arg_str(&call, 0)?)?,
            )),
            "readPathSync" | "ReadPathSync" => Ok(ScriptHostCallResult::StringList(
                self.pathing_script
                    .read_path_sync(optional_str(&call, 0)?.unwrap_or("."))?,
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    pub(super) fn call_key_mouse_script(
        &self,
        call: ScriptHostCall,
    ) -> Result<ScriptHostCallResult> {
        let cancellation = self.cancellation.as_deref();
        match call.method.as_str() {
            "run" | "Run" => Ok(ScriptHostCallResult::KeyMouseExecution(
                self.key_mouse_script.execute_with_cancellation(
                    arg_str(&call, 0)?,
                    self.key_mouse_dispatch_mode,
                    self.input_window_handle,
                    cancellation,
                )?,
            )),
            "runFile" | "RunFile" => Ok(ScriptHostCallResult::KeyMouseExecution(
                self.key_mouse_script.execute_file_with_cancellation(
                    arg_str(&call, 0)?,
                    self.key_mouse_dispatch_mode,
                    self.input_window_handle,
                    cancellation,
                )?,
            )),
            "plan" | "Plan" => Ok(ScriptHostCallResult::KeyMousePlan(
                self.key_mouse_script.run(arg_str(&call, 0)?)?,
            )),
            "planFile" | "PlanFile" => Ok(ScriptHostCallResult::KeyMousePlan(
                self.key_mouse_script.run_file(arg_str(&call, 0)?)?,
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    pub(super) fn call_file(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "readPathSync" | "ReadPathSync" => Ok(ScriptHostCallResult::StringList(
                self.file
                    .read_path_sync(optional_str(&call, 0)?.unwrap_or("."))?,
            )),
            "createDirectory" | "CreateDirectory" => Ok(ScriptHostCallResult::Bool(
                self.file.create_directory(arg_str(&call, 0)?)?,
            )),
            "isFolder" | "IsFolder" => Ok(ScriptHostCallResult::Bool(
                self.file.is_folder(arg_str(&call, 0)?)?,
            )),
            "isFile" | "IsFile" => Ok(ScriptHostCallResult::Bool(
                self.file.is_file(arg_str(&call, 0)?)?,
            )),
            "isExists" | "IsExists" => Ok(ScriptHostCallResult::Bool(
                self.file.is_exists(arg_str(&call, 0)?)?,
            )),
            "readTextSync" | "ReadTextSync" | "readText" | "ReadText" => Ok(
                ScriptHostCallResult::String(self.file.read_text_sync(arg_str(&call, 0)?)?),
            ),
            "writeTextSync" | "WriteTextSync" | "writeText" | "WriteText" => {
                Ok(ScriptHostCallResult::Bool(self.file.write_text_sync(
                    arg_str(&call, 0)?,
                    arg_str(&call, 1)?,
                    optional_bool(&call, 2)?.unwrap_or(false),
                )?))
            }
            "readImageMatSync" | "ReadImageMatSync" => {
                Ok(ScriptHostCallResult::ImageMatReadExecution(
                    self.file.read_image_mat_sync(arg_str(&call, 0)?)?,
                ))
            }
            "readImageMatWithResizeSync" | "ReadImageMatWithResizeSync" => {
                Ok(ScriptHostCallResult::ImageMatReadExecution(
                    self.file.read_image_mat_with_resize_sync(
                        arg_str(&call, 0)?,
                        arg_f64_like(&call, 1)?,
                        arg_f64_like(&call, 2)?,
                        optional_i32(&call, 3)?.unwrap_or(1),
                    )?,
                ))
            }
            "writeImageSync" | "WriteImageSync" => {
                Ok(ScriptHostCallResult::ImageMatWriteExecution(
                    self.file
                        .write_image_sync(arg_str(&call, 0)?, arg_owned_value(&call, 1)?)?,
                ))
            }
            "renamePathSync" | "RenamePathSync" => Ok(ScriptHostCallResult::Bool(
                self.file
                    .rename_path_sync(arg_str(&call, 0)?, arg_str(&call, 1)?)?,
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    pub(super) fn call_vision(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "findTemplate" | "FindTemplate" => Ok(
                ScriptHostCallResult::VisionRecognitionExecution(self.vision.find_template(
                    arg_owned_value(&call, 0)?,
                    arg_owned_value(&call, 1)?,
                    optional_owned_value(&call, 2),
                )?),
            ),
            "findColor" | "FindColor" => Ok(ScriptHostCallResult::VisionRecognitionExecution(
                self.vision
                    .find_color(arg_owned_value(&call, 0)?, optional_owned_value(&call, 1))?,
            )),
            "crop" | "Crop" => Ok(ScriptHostCallResult::VisionImageMatExecution(
                self.vision
                    .crop(arg_owned_value(&call, 0)?, arg_owned_value(&call, 1)?)?,
            )),
            "to1080p" | "To1080p" => Ok(ScriptHostCallResult::VisionImageMatExecution(
                self.vision.to_1080p(arg_owned_value(&call, 0)?)?,
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    pub(super) fn call_log(&mut self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "debug" | "Debug" => self.log.debug(arg_str(&call, 0)?),
            "info" | "Info" => self.log.info(arg_str(&call, 0)?),
            "warn" | "Warn" => self.log.warn(arg_str(&call, 0)?),
            "error" | "Error" => self.log.error(arg_str(&call, 0)?),
            "records" | "Records" => {
                return Ok(ScriptHostCallResult::LogRecords(
                    self.log.records().to_vec(),
                ));
            }
            _ => return Err(unknown_method(&call)),
        }
        Ok(ScriptHostCallResult::None)
    }
}
