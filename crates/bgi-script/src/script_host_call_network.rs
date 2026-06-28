use super::*;

impl ScriptHostRuntime {
    pub(super) fn call_http(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        let mut client = ReqwestScriptHttpClient::new();
        self.call_http_with_client(call, &mut client)
    }

    pub(super) fn call_http_with_client<C: ScriptHttpClient>(
        &self,
        call: ScriptHostCall,
        client: &mut C,
    ) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "request" | "Request" => {
                let request = self.http.request(
                    arg_str(&call, 0)?,
                    arg_str(&call, 1)?,
                    optional_str(&call, 2)?,
                    optional_str(&call, 3)?,
                )?;
                match self.http_dispatch_mode {
                    HttpDispatchMode::PlanOnly => {
                        Ok(ScriptHostCallResult::HttpRequestPlan(request))
                    }
                    HttpDispatchMode::Reqwest => {
                        let response = client.send(request.clone())?;
                        Ok(ScriptHostCallResult::HttpExecution(HttpExecution {
                            mode: self.http_dispatch_mode,
                            request,
                            response: Some(response),
                            dispatched: true,
                        }))
                    }
                }
            }
            _ => Err(unknown_method(&call)),
        }
    }

    pub(super) fn call_dispatcher(&mut self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        let command = match call.method.as_str() {
            "addTimer" | "AddTimer" => self
                .dispatcher
                .add_timer(timer_plan_from_arg(&call, 0, true)?),
            "addTrigger" | "AddTrigger" => self
                .dispatcher
                .add_trigger(timer_plan_from_arg(&call, 0, false)?),
            "clearAllTriggers" | "ClearAllTriggers" => self.dispatcher.clear_all_triggers(),
            "runTask" | "RunTask" if call.args.is_empty() => self.dispatcher.run_current_task(),
            "runTask" | "RunTask" => self
                .dispatcher
                .run_solo_task(solo_task_plan_from_arg(&call, 0)?),
            "getLinkedCancellationTokenSource" | "GetLinkedCancellationTokenSource" => {
                self.dispatcher.get_linked_cancellation_token_source()
            }
            "getLinkedCancellationToken" | "GetLinkedCancellationToken" => {
                self.dispatcher.get_linked_cancellation_token()
            }
            "runAutoDomainTask" | "RunAutoDomainTask" => self
                .dispatcher
                .run_builtin_task("AutoDomain", arg_owned_value(&call, 0)?),
            "runAutoBossTask" | "RunAutoBossTask" => self
                .dispatcher
                .run_builtin_task("AutoBoss", arg_owned_value(&call, 0)?),
            "runAutoFightTask" | "RunAutoFightTask" => self
                .dispatcher
                .run_builtin_task("AutoFight", arg_owned_value(&call, 0)?),
            "runAutoLeyLineOutcropTask" | "RunAutoLeyLineOutcropTask" => self
                .dispatcher
                .run_builtin_task("AutoLeyLineOutcrop", arg_owned_value(&call, 0)?),
            "runAutoStygianOnslaughtTask" | "RunAutoStygianOnslaughtTask" => self
                .dispatcher
                .run_builtin_task("AutoStygianOnslaught", arg_owned_value(&call, 0)?),
            "commands" | "Commands" => {
                return Ok(ScriptHostCallResult::DispatcherCommands(
                    self.dispatcher.commands().to_vec(),
                ));
            }
            _ => return Err(unknown_method(&call)),
        };
        Ok(ScriptHostCallResult::DispatcherCommand(command))
    }

    pub(super) fn call_notification(
        &mut self,
        call: ScriptHostCall,
        now_ms: u64,
    ) -> Result<ScriptHostCallResult> {
        let mut sink = RecordingNotificationSink::default();
        self.call_notification_with_sink(call, now_ms, &mut sink)
    }

    pub(super) fn call_notification_with_sink<S: ScriptNotificationSink>(
        &mut self,
        call: ScriptHostCall,
        now_ms: u64,
        sink: &mut S,
    ) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "send" | "Send" | "success" | "Success" => {
                return self.notification_result(
                    ScriptNotificationKind::Success,
                    arg_str(&call, 0)?,
                    now_ms,
                    sink,
                );
            }
            "error" | "Error" => {
                return self.notification_result(
                    ScriptNotificationKind::Error,
                    arg_str(&call, 0)?,
                    now_ms,
                    sink,
                );
            }
            "records" | "Records" => Ok(ScriptHostCallResult::NotificationRecords(
                self.notification.records().to_vec(),
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    pub(super) fn notification_result<S: ScriptNotificationSink>(
        &mut self,
        kind: ScriptNotificationKind,
        message: &str,
        now_ms: u64,
        sink: &mut S,
    ) -> Result<ScriptHostCallResult> {
        let (record, delivery, dispatched) = match self.notification_dispatch_mode {
            NotificationDispatchMode::RecordOnly => {
                let record = match kind {
                    ScriptNotificationKind::Success => {
                        self.notification.send_at(message, now_ms)?
                    }
                    ScriptNotificationKind::Error => self.notification.error_at(message, now_ms)?,
                };
                (record, None, false)
            }
            NotificationDispatchMode::Sink => {
                let delivery = match kind {
                    ScriptNotificationKind::Success => {
                        self.notification.send_to(message, now_ms, sink)?
                    }
                    ScriptNotificationKind::Error => {
                        self.notification.error_to(message, now_ms, sink)?
                    }
                };
                let record = self
                    .notification
                    .records()
                    .last()
                    .cloned()
                    .expect("notification delivery records before sink dispatch");
                (record, Some(delivery), true)
            }
        };
        Ok(ScriptHostCallResult::NotificationExecution(
            NotificationExecution {
                mode: self.notification_dispatch_mode,
                record,
                delivery,
                dispatched,
            },
        ))
    }
}
