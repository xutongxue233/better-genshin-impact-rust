use super::super::Result;
use super::commands::DispatcherCommand;
use super::plans::{RealtimeTimerHostPlan, SoloTaskHostPlan};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct ScriptDispatcherHost {
    commands: Vec<DispatcherCommand>,
}

impl ScriptDispatcherHost {
    pub fn commands(&self) -> &[DispatcherCommand] {
        &self.commands
    }

    pub fn task_invocation_plans(&self) -> Result<Vec<bgi_task::TaskInvocationPlan>> {
        self.commands
            .iter()
            .cloned()
            .map(|command| {
                bgi_task::TaskInvocationPlan::from_script_dispatcher_command(command.into())
                    .map_err(Into::into)
            })
            .collect()
    }

    pub fn add_timer(&mut self, mut timer: RealtimeTimerHostPlan) -> DispatcherCommand {
        timer.clears_existing_triggers = true;
        self.commands.push(DispatcherCommand::ClearAllTriggers);
        let command = DispatcherCommand::AddRealtimeTimer(timer);
        self.commands.push(command.clone());
        command
    }

    pub fn add_trigger(&mut self, mut timer: RealtimeTimerHostPlan) -> DispatcherCommand {
        timer.clears_existing_triggers = false;
        let command = DispatcherCommand::AddRealtimeTimer(timer);
        self.commands.push(command.clone());
        command
    }

    pub fn clear_all_triggers(&mut self) -> DispatcherCommand {
        let command = DispatcherCommand::ClearAllTriggers;
        self.commands.push(command.clone());
        command
    }

    pub fn run_current_task(&mut self) -> DispatcherCommand {
        let command = DispatcherCommand::RunCurrentTask;
        self.commands.push(command.clone());
        command
    }

    pub fn run_solo_task(&mut self, task: SoloTaskHostPlan) -> DispatcherCommand {
        let command = DispatcherCommand::RunSoloTask(task);
        self.commands.push(command.clone());
        command
    }

    pub fn get_linked_cancellation_token_source(&mut self) -> DispatcherCommand {
        let command = DispatcherCommand::LinkedCancellationTokenSource;
        self.commands.push(command.clone());
        command
    }

    pub fn get_linked_cancellation_token(&mut self) -> DispatcherCommand {
        let command = DispatcherCommand::LinkedCancellationToken;
        self.commands.push(command.clone());
        command
    }

    pub fn run_builtin_task(&mut self, name: &str, config: Value) -> DispatcherCommand {
        let command = DispatcherCommand::RunBuiltinTask {
            name: name.to_string(),
            config,
            uses_linked_cancellation: true,
        };
        self.commands.push(command.clone());
        command
    }
}
