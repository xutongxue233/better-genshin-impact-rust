use super::plans::{RealtimeTimerHostPlan, SoloTaskHostPlan};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum DispatcherCommand {
    ClearAllTriggers,
    AddRealtimeTimer(RealtimeTimerHostPlan),
    RunCurrentTask,
    RunSoloTask(SoloTaskHostPlan),
    LinkedCancellationTokenSource,
    LinkedCancellationToken,
    RunBuiltinTask {
        name: String,
        config: Value,
        uses_linked_cancellation: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum GenshinCommand {
    Uid,
    Teleport {
        x: f64,
        y: f64,
        map_name: Option<String>,
        force: bool,
    },
    MoveMapTo {
        x: f64,
        y: f64,
        map_name: Option<String>,
        force_country: Option<String>,
    },
    GetBigMapZoomLevel,
    SetBigMapZoomLevel {
        zoom_level: f64,
    },
    TpToStatueOfTheSeven,
    GetPositionFromBigMap {
        map_name: Option<String>,
    },
    GetPositionFromMap {
        map_name: Option<String>,
        cache_time_ms: Option<u64>,
        matching_method: Option<String>,
        nearby: Option<(f64, f64)>,
    },
    GetCameraOrientation,
    SwitchParty {
        party_name: String,
    },
    ClearPartyCache,
    BlessingOfTheWelkinMoon,
    ChooseTalkOption {
        option: String,
        skip_times: u32,
        is_orange: bool,
    },
    ClaimBattlePassRewards,
    ClaimEncounterPointsRewards,
    GoToAdventurersGuild {
        country: String,
    },
    GoToCraftingBench {
        country: String,
    },
    ReturnMainUi,
    AutoFishing {
        fishing_time_policy: i32,
    },
    Relogin,
    WonderlandCycle,
    SetTime {
        hour: u32,
        minute: u32,
        skip: bool,
    },
}

impl From<DispatcherCommand> for bgi_task::ScriptDispatcherCommandInput {
    fn from(value: DispatcherCommand) -> Self {
        match value {
            DispatcherCommand::ClearAllTriggers => Self::ClearAllTriggers,
            DispatcherCommand::AddRealtimeTimer(timer) => Self::AddRealtimeTimer(timer.into()),
            DispatcherCommand::RunCurrentTask => Self::RunCurrentTask,
            DispatcherCommand::RunSoloTask(task) => Self::RunSoloTask(task.into()),
            DispatcherCommand::LinkedCancellationTokenSource => Self::LinkedCancellationTokenSource,
            DispatcherCommand::LinkedCancellationToken => Self::LinkedCancellationToken,
            DispatcherCommand::RunBuiltinTask {
                name,
                config,
                uses_linked_cancellation,
            } => Self::RunBuiltinTask {
                name,
                config,
                uses_linked_cancellation,
            },
        }
    }
}
