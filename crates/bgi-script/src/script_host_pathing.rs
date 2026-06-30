use super::{LimitedFileHost, Result};
use bgi_core::{PathingExecutionPlan, PathingSummary, PathingTask};
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PathingScriptSource {
    InlineJson,
    ScriptFile,
    UserAutoPathingFile,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingScriptRunPlan {
    pub source: PathingScriptSource,
    pub normalized_path: Option<PathBuf>,
    pub summary: PathingSummary,
    pub task: PathingTask,
    pub party_config: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingScriptExecution {
    pub plan: PathingScriptRunPlan,
    pub execution_plan: PathingExecutionPlan,
    pub dispatched: bool,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct PathingScriptHost {
    script_file_host: LimitedFileHost,
    user_auto_pathing_file_host: LimitedFileHost,
    party_config: Option<Value>,
}

impl PathingScriptHost {
    pub fn new(
        script_root: impl Into<PathBuf>,
        user_auto_pathing_root: impl Into<PathBuf>,
        party_config: Option<Value>,
    ) -> Self {
        Self {
            script_file_host: LimitedFileHost::new(script_root),
            user_auto_pathing_file_host: LimitedFileHost::new(user_auto_pathing_root),
            party_config,
        }
    }

    pub fn run(&self, json: &str) -> Result<PathingScriptRunPlan> {
        self.plan_from_json(json, PathingScriptSource::InlineJson, None)
    }

    pub fn execute(&self, json: &str) -> Result<PathingScriptExecution> {
        self.run(json)?.execute()
    }

    pub fn run_file(&self, path: &str) -> Result<PathingScriptRunPlan> {
        let json = self.script_file_host.read_text_sync(path)?;
        let normalized_path = self.script_file_host.normalize_path(path)?;
        self.plan_from_json(
            &json,
            PathingScriptSource::ScriptFile,
            Some(normalized_path),
        )
    }

    pub fn execute_file(&self, path: &str) -> Result<PathingScriptExecution> {
        self.run_file(path)?.execute()
    }

    pub fn run_file_from_user(&self, path: &str) -> Result<PathingScriptRunPlan> {
        let json = self.user_auto_pathing_file_host.read_text_sync(path)?;
        let normalized_path = self.user_auto_pathing_file_host.normalize_path(path)?;
        self.plan_from_json(
            &json,
            PathingScriptSource::UserAutoPathingFile,
            Some(normalized_path),
        )
    }

    pub fn execute_file_from_user(&self, path: &str) -> Result<PathingScriptExecution> {
        self.run_file_from_user(path)?.execute()
    }

    pub fn is_exists(&self, path: &str) -> Result<bool> {
        self.user_auto_pathing_file_host.is_exists(path)
    }

    pub fn is_file(&self, path: &str) -> Result<bool> {
        self.user_auto_pathing_file_host.is_file(path)
    }

    pub fn is_folder(&self, path: &str) -> Result<bool> {
        self.user_auto_pathing_file_host.is_folder(path)
    }

    pub fn read_path_sync(&self, path: &str) -> Result<Vec<String>> {
        self.user_auto_pathing_file_host.read_path_sync(path)
    }

    fn plan_from_json(
        &self,
        json: &str,
        source: PathingScriptSource,
        normalized_path: Option<PathBuf>,
    ) -> Result<PathingScriptRunPlan> {
        let mut task = PathingTask::from_json(json)?;
        if let Some(path) = &normalized_path {
            task.file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned);
            task.full_path = Some(path.clone());
        }
        let summary = task.summary();
        Ok(PathingScriptRunPlan {
            source,
            normalized_path,
            summary,
            task,
            party_config: self.party_config.clone(),
        })
    }
}

impl PathingScriptRunPlan {
    pub fn execute(self) -> Result<PathingScriptExecution> {
        Ok(PathingScriptExecution {
            execution_plan: self.task.execution_plan_with_legacy_track_converter(),
            plan: self,
            dispatched: false,
            completed: false,
        })
    }
}
