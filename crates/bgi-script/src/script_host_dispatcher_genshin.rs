use super::super::Result;
use super::commands::GenshinCommand;

#[derive(Debug, Clone, Default)]
pub struct GenshinHost {
    commands: Vec<GenshinCommand>,
}

impl GenshinHost {
    pub fn commands(&self) -> &[GenshinCommand] {
        &self.commands
    }

    pub fn task_invocation_plans(&self) -> Result<Vec<bgi_task::TaskInvocationPlan>> {
        self.commands
            .iter()
            .filter_map(genshin_command_to_task_input)
            .map(|command| {
                bgi_task::TaskInvocationPlan::from_script_dispatcher_command(command)
                    .map_err(Into::into)
            })
            .collect()
    }

    pub fn push(&mut self, command: GenshinCommand) -> GenshinCommand {
        self.commands.push(command.clone());
        command
    }
}

pub fn genshin_command_to_task_input(
    command: &GenshinCommand,
) -> Option<bgi_task::ScriptDispatcherCommandInput> {
    let (name, config) = match command {
        GenshinCommand::Teleport {
            x,
            y,
            map_name,
            force,
        } => (
            "Teleport",
            serde_json::json!({ "x": x, "y": y, "mapName": map_name, "force": force }),
        ),
        GenshinCommand::MoveMapTo {
            x,
            y,
            map_name,
            force_country,
        } => (
            "Teleport",
            serde_json::json!({
                "kind": "moveMapTo",
                "x": x,
                "y": y,
                "mapName": map_name,
                "forceCountry": force_country
            }),
        ),
        GenshinCommand::TpToStatueOfTheSeven => (
            "Teleport",
            serde_json::json!({ "kind": "statueOfTheSeven" }),
        ),
        GenshinCommand::SwitchParty { party_name } => (
            "SwitchParty",
            serde_json::json!({ "partyName": party_name }),
        ),
        GenshinCommand::BlessingOfTheWelkinMoon => {
            ("BlessingOfTheWelkinMoon", serde_json::json!({}))
        }
        GenshinCommand::ChooseTalkOption {
            option,
            skip_times,
            is_orange,
        } => (
            "ChooseTalkOption",
            serde_json::json!({
                "option": option,
                "skipTimes": skip_times,
                "isOrange": is_orange
            }),
        ),
        GenshinCommand::ClaimBattlePassRewards => ("ClaimBattlePassRewards", serde_json::json!({})),
        GenshinCommand::ClaimEncounterPointsRewards => {
            ("ClaimEncounterPointsRewards", serde_json::json!({}))
        }
        GenshinCommand::ReturnMainUi => ("ReturnMainUi", serde_json::json!({})),
        GenshinCommand::SetTime { hour, minute, skip } => (
            "SetTime",
            serde_json::json!({ "hour": hour, "minute": minute, "skip": skip }),
        ),
        GenshinCommand::AutoFishing {
            fishing_time_policy,
        } => (
            "AutoFishing",
            serde_json::json!({ "fishingTimePolicy": fishing_time_policy }),
        ),
        GenshinCommand::GoToAdventurersGuild { country } => (
            "GoToAdventurersGuild",
            serde_json::json!({ "country": country }),
        ),
        GenshinCommand::GoToCraftingBench { country } => (
            "GoToCraftingBench",
            serde_json::json!({ "country": country }),
        ),
        GenshinCommand::Relogin => ("Relogin", serde_json::json!({})),
        GenshinCommand::WonderlandCycle => ("WonderlandCycle", serde_json::json!({})),
        _ => return None,
    };

    Some(bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
        name: name.to_string(),
        config,
        uses_linked_cancellation: true,
    })
}
