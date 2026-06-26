use crate::group::ScriptProjectType;
use crate::project::ScriptProjectError;

#[derive(Debug, thiserror::Error)]
pub enum ScriptRuntimeError {
    #[error("script project is disabled: {0}")]
    ProjectDisabled(String),
    #[error("script project {0} has no runnable target")]
    MissingRunnableTarget(String),
    #[error("javascript project {project} is missing manifest metadata")]
    MissingManifest { project: String },
    #[error("manifest main script is required for project {project}")]
    MissingManifestMain { project: String },
    #[error("script project load failed: {0}")]
    Project(#[from] ScriptProjectError),
    #[error("only javascript execution steps can be prepared for the JS runtime: {0:?}")]
    UnsupportedPreparedProjectType(ScriptProjectType),
}

pub type Result<T> = std::result::Result<T, ScriptRuntimeError>;

#[path = "runtime_model.rs"]
mod runtime_model;
#[path = "runtime_preparation.rs"]
mod runtime_preparation;
#[path = "runtime_summary.rs"]
mod runtime_summary;

pub use runtime_model::*;
pub use runtime_preparation::*;
pub use runtime_summary::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::{ScriptGroup, ScriptGroupProject};
    use crate::manifest::Manifest;
    use crate::project::ScriptCodeExecutionMode;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn schedule_codes_preserve_legacy_meaning() {
        assert_eq!(
            ScriptSchedule::parse("Daily").kind,
            ScriptScheduleKind::Daily
        );
        assert_eq!(
            ScriptSchedule::parse("Friday").kind,
            ScriptScheduleKind::Weekday
        );
        assert_eq!(
            ScriptSchedule::parse("0 0 * * *").kind,
            ScriptScheduleKind::Cron
        );
    }

    #[test]
    fn group_plan_sorts_steps_by_index() {
        let group = ScriptGroup {
            name: "daily".to_string(),
            projects: vec![
                ScriptGroupProject {
                    index: 2,
                    name: "second.json".to_string(),
                    folder_name: "routes".to_string(),
                    project_type: ScriptProjectType::Pathing,
                    ..ScriptGroupProject::default()
                },
                ScriptGroupProject {
                    index: 1,
                    name: "first.json".to_string(),
                    folder_name: "routes".to_string(),
                    project_type: ScriptProjectType::Pathing,
                    ..ScriptGroupProject::default()
                },
            ],
            ..ScriptGroup::default()
        };

        let plan = ScriptExecutionPlan::for_group(&group, &BTreeMap::new(), ".").unwrap();

        assert_eq!(plan.steps[0].name, "first.json");
        assert_eq!(plan.steps[1].name, "second.json");
    }

    #[test]
    fn javascript_step_requires_manifest() {
        let project = ScriptGroupProject {
            name: "sample".to_string(),
            folder_name: "sample".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        };

        let err = ScriptExecutionPlan::single_project(&project, None, ".").unwrap_err();

        assert!(matches!(
            err,
            ScriptRuntimeError::MissingManifest { project } if project == "sample"
        ));
    }

    #[test]
    fn runtime_summary_counts_host_surface() {
        let summary = script_runtime_summary();

        assert!(summary.host_binding_count >= 10);
        assert!(summary.host_member_count > summary.host_binding_count);
        assert!(summary
            .supported_project_types
            .contains(&ScriptProjectType::Javascript));
    }

    #[test]
    fn prepared_javascript_execution_loads_main_module_and_host_context() {
        let root = test_root("bgi-prepared-js");
        let project_dir = root.join("sample");
        std::fs::create_dir_all(project_dir.join("packages")).unwrap();
        std::fs::write(
            project_dir.join("manifest.json"),
            r#"{
                "manifestVersion": 1,
                "name": "sample",
                "version": "1.0.0",
                "main": "main.js",
                "httpAllowedUrls": ["https://example.com/*"]
            }"#,
        )
        .unwrap();
        std::fs::write(
            project_dir.join("main.js"),
            "import text from './notes.txt'\nexport default text;",
        )
        .unwrap();
        std::fs::write(project_dir.join("notes.txt"), "hello").unwrap();

        let manifest = Manifest::read_from(project_dir.join("manifest.json")).unwrap();
        let project = ScriptGroupProject {
            name: "sample".to_string(),
            folder_name: "sample".to_string(),
            project_type: ScriptProjectType::Javascript,
            js_script_settings_object: Some(serde_json::json!({"level": 2})),
            allow_js_notification: Some(false),
            allow_js_http_hash: Some("https://example.com/*".to_string()),
            ..ScriptGroupProject::default()
        };
        let step =
            ScriptExecutionStep::from_group_project(&project, Some(&manifest), &root).unwrap();

        let prepared = PreparedScriptExecution::prepare_javascript(&step, &root).unwrap();

        assert_eq!(
            prepared.execution_mode,
            ScriptCodeExecutionMode::StandardModule
        );
        assert_eq!(prepared.main_module.import_rewrites.len(), 1);
        assert!(prepared
            .main_module
            .code
            .contains("file.ReadTextSync('notes.txt')"));
        assert_eq!(prepared.settings, Some(serde_json::json!({"level": 2})));
        assert_eq!(prepared.host_runtime_config.script_root, project_dir);
        assert!(
            !prepared
                .host_runtime_config
                .notification_policy
                .project_enabled
        );
        assert!(prepared
            .host_runtime_config
            .http_policy
            .check_url("https://example.com/api")
            .is_ok());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn prepared_javascript_execution_rejects_non_js_steps() {
        let step = ScriptExecutionStep {
            index: 0,
            name: "macro.json".to_string(),
            folder_name: "macro.json".to_string(),
            project_type: ScriptProjectType::KeyMouse,
            engine: ScriptEngineKind::KeyMouseMacro,
            schedule: ScriptSchedule::parse("Daily"),
            run_count: 1,
            settings: None,
            allow_notification: true,
            allow_http_hash: None,
            target_path: None,
            manifest_main: None,
            skipped: false,
        };

        assert!(matches!(
            PreparedScriptExecution::prepare_javascript(&step, ".").unwrap_err(),
            ScriptRuntimeError::UnsupportedPreparedProjectType(ScriptProjectType::KeyMouse)
        ));
    }

    fn test_root(prefix: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "{prefix}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        root
    }
}
