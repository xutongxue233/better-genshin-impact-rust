#![allow(clippy::field_reassign_with_default)]

use super::*;
use bgi_capture::CaptureFrame;
use bgi_script::{
    ExecutionRecord, ExecutionRecordClock, ExecutionRecordStorage, FarmingPlanConfig,
    GameCaptureFrameSource, ScriptGroupConfig,
};
use bgi_vision::{BgrImage, Size as VisionSize};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use serde_json::json;
use std::fs;
use std::sync::Arc;

struct StaticFrameSource {
    frame: CaptureFrame,
    area: bgi_script::GameCaptureArea,
}

impl GameCaptureFrameSource for StaticFrameSource {
    fn capture_frame(
        &self,
    ) -> std::result::Result<CaptureFrame, bgi_script::ScriptHostRuntimeError> {
        Ok(self.frame.clone())
    }

    fn capture_frame_area(&self, _frame: &CaptureFrame) -> bgi_script::GameCaptureArea {
        self.area
    }
}

#[test]
fn executes_classic_javascript_with_settings_and_log_host() {
    let root = test_root("classic");
    let project = root.join("demo");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            console.log("hello", settings.name);
            log.info("level " + settings.level);
            getGameMetrics().width + settings.level;
        "#,
    )
    .unwrap();

    let outcome =
        execute_javascript_project(&root, "demo", Some(json!({"name": "BetterGI", "level": 2})))
            .unwrap();

    assert_eq!(outcome.runtime, ScriptEngineRuntimeKind::Boa);
    assert_eq!(outcome.result, Some(json!(1922)));
    assert_eq!(outcome.console, vec!["hello BetterGI"]);
    assert_eq!(outcome.logs.len(), 1);
    assert_eq!(outcome.logs[0].message, "level 2");
    assert!(outcome
        .host_calls
        .iter()
        .any(|call| call.target == ScriptHostTarget::Global && call.method == "getGameMetrics"));
}

#[test]
fn classic_javascript_settles_top_level_promise_result() {
    let root = test_root("classic-promise");
    let project = root.join("demo");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            Promise.resolve(40).then((value) => {
                console.log("promise", value);
                return value + 2;
            });
        "#,
    )
    .unwrap();

    let outcome = execute_javascript_project(&root, "demo", None).unwrap();

    assert_eq!(outcome.result, Some(json!(42)));
    assert_eq!(outcome.result_display, "42");
    assert_eq!(outcome.console, vec!["promise 40"]);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn classic_javascript_receives_injected_capture_game_region_execution() {
    let root = test_root("capture-game-region-js");
    let project = root.join("demo");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Capture","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            const capture = CaptureGameRegion();
            `${capture.width}x${capture.height}:${capture.pixels.join(",")}:${capture.image_region.source}`;
        "#,
    )
    .unwrap();

    let step = ScriptExecutionStep {
        index: 1,
        name: "Capture".to_string(),
        folder_name: "demo".to_string(),
        project_type: ScriptProjectType::Javascript,
        engine: bgi_script::ScriptEngineKind::RustJavaScript,
        schedule: bgi_script::ScriptSchedule::parse(""),
        run_count: 1,
        settings: None,
        allow_notification: true,
        allow_http_hash: None,
        target_path: None,
        manifest_main: Some("main.js".to_string()),
        skipped: false,
    };
    let mut prepared = PreparedScriptExecution::prepare_javascript(&step, &root).unwrap();
    prepared.host_runtime_config.capture_area = bgi_script::GameCaptureArea {
        x: 1,
        y: 0,
        width: 2,
        height: 2,
    };
    prepared.host_runtime_config.capture_frame_source = Some(Arc::new(StaticFrameSource {
        frame: CaptureFrame::packed_bgr(
            4,
            2,
            vec![
                1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, //
                5, 5, 5, 6, 6, 6, 7, 7, 7, 8, 8, 8,
            ],
        )
        .unwrap(),
        area: bgi_script::GameCaptureArea {
            x: 1,
            y: 0,
            width: 2,
            height: 2,
        },
    }));
    let host = ScriptHostRuntime::new(prepared.host_runtime_config.clone()).unwrap();
    let outcome = execute_prepared_javascript_with_host(
        &prepared,
        TaskInvocationExecutionMode::PlanOnly,
        &bgi_task::TaskInvocationPlanningContext::default(),
        None,
        None,
        host,
    )
    .unwrap();

    assert_eq!(
        outcome.result,
        Some(json!("2x2:2,2,2,3,3,3,6,6,6,7,7,7:DerivedCrop"))
    );
    let capture_call = outcome
        .host_calls
        .iter()
        .find(|call| call.target == ScriptHostTarget::Global)
        .unwrap();
    assert_eq!(capture_call.result["pixel_format"], json!("BGR24"));
    assert_eq!(capture_call.result["source_width"], json!(4));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn executes_javascript_image_mat_io_with_real_bgr_payload() {
    let root = test_root("image-mat-io");
    let project = root.join("demo");
    fs::create_dir_all(project.join("assets")).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    let source = BgrImage::new(VisionSize::new(2, 1), vec![11, 22, 33, 44, 55, 66]).unwrap();
    source.write_png(project.join("assets/source.png")).unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            const mat = file.readImageMatSync("assets/source.png");
            const write = file.writeImageSync("assets/copy", mat);
            mat.width + mat.height + mat.pixels[0] + write.width;
        "#,
    )
    .unwrap();

    let outcome = execute_javascript_project(&root, "demo", None).unwrap();
    let copied = BgrImage::read(project.join("assets/copy.png")).unwrap();
    let read_call = outcome
        .host_calls
        .iter()
        .find(|call| call.method == "readImageMatSync")
        .unwrap();
    let write_call = outcome
        .host_calls
        .iter()
        .find(|call| call.method == "writeImageSync")
        .unwrap();

    assert_eq!(outcome.result, Some(json!(16)));
    assert_eq!(read_call.result["pixel_format"], json!("BGR24"));
    assert_eq!(read_call.result["width"], json!(2));
    assert_eq!(read_call.result["pixels"], json!([11, 22, 33, 44, 55, 66]));
    assert_eq!(
        write_call.result["normalized_path"],
        serde_json::to_value(project.join("assets").join("copy.png")).unwrap()
    );
    assert_eq!(copied, source);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn executes_javascript_vision_matching_with_mat_payloads() {
    let root = test_root("vision-matching");
    let project = root.join("demo");
    fs::create_dir_all(project.join("assets")).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    let source = BgrImage::new(
        VisionSize::new(3, 3),
        vec![
            1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 40, 40, 40, 50, 50, 50, 6, 6, 6, 70, 70, 70, 80,
            80, 80,
        ],
    )
    .unwrap();
    let template = BgrImage::new(
        VisionSize::new(2, 2),
        vec![40, 40, 40, 50, 50, 50, 70, 70, 70, 80, 80, 80],
    )
    .unwrap();
    source.write_png(project.join("assets/source.png")).unwrap();
    template
        .write_png(project.join("assets/template.png"))
        .unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            const source = file.readImageMatSync("assets/source.png");
            const template = file.readImageMatSync("assets/template.png");
            const cropped = vision.crop(source, { x: 1, y: 1, width: 2, height: 2 });
            const hit = vision.findTemplate(cropped, template, {
                threshold: 0.99,
                use3Channels: true,
                mode: "CCorrNormed",
                maxMatchCount: 1,
                name: "patch"
            });
            const color = vision.findColor(cropped, {
                conversion: "BgrToRgb",
                lowerColor: [40, 40, 40],
                upperColor: [40, 40, 40],
                matchCount: 1,
                name: "gray"
            });
            cropped.width + hit.first.rect.x + hit.first.rect.y + hit.matches.length + color.first.rect.width;
        "#,
    )
    .unwrap();

    let outcome = execute_javascript_project(&root, "demo", None).unwrap();
    let crop_call = outcome
        .host_calls
        .iter()
        .find(|call| call.target == ScriptHostTarget::Vision && call.method == "crop")
        .unwrap();
    let template_call = outcome
        .host_calls
        .iter()
        .find(|call| call.target == ScriptHostTarget::Vision && call.method == "findTemplate")
        .unwrap();
    let color_call = outcome
        .host_calls
        .iter()
        .find(|call| call.target == ScriptHostTarget::Vision && call.method == "findColor")
        .unwrap();

    assert_eq!(outcome.result, Some(json!(4)));
    assert_eq!(crop_call.result["width"], json!(2));
    assert_eq!(crop_call.result["height"], json!(2));
    assert_eq!(crop_call.result["pixel_format"], json!("BGR24"));
    assert_eq!(
        template_call.result["recognition_type"],
        json!("TemplateMatch")
    );
    assert_eq!(template_call.result["first"]["rect"]["x"], json!(0));
    assert_eq!(template_call.result["first"]["rect"]["y"], json!(0));
    assert_eq!(template_call.result["matched_count"], json!(1));
    assert_eq!(color_call.result["recognition_type"], json!("ColorMatch"));
    assert_eq!(color_call.result["first"]["text"], json!("gray"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reports_javascript_errors() {
    let root = test_root("error");
    let project = root.join("demo");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(project.join("main.js"), "throw new Error('boom');").unwrap();

    let error = execute_javascript_project(&root, "demo", None).unwrap_err();
    assert!(error.to_string().contains("boom"));
}

#[test]
fn executes_standard_modules_with_relative_imports() {
    let root = test_root("module");
    let project = root.join("demo");
    let lib = project.join("lib");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&lib).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(lib.join("math.js"), "export const bonus = 40;").unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            import { bonus } from './lib/math.js';
            log.info("module " + settings.name);
            globalThis.__bgiModuleResult = bonus + settings.level;
        "#,
    )
    .unwrap();

    let outcome =
        execute_javascript_project(&root, "demo", Some(json!({"name": "ok", "level": 2}))).unwrap();

    assert_eq!(
        outcome.execution_mode,
        ScriptCodeExecutionMode::StandardModule
    );
    assert_eq!(outcome.result, None);
    assert_eq!(outcome.logs.len(), 1);
    assert_eq!(outcome.logs[0].message, "module ok");
}

#[test]
fn executes_standard_modules_with_shared_module_record_cache() {
    let root = test_root("module-cache");
    let project = root.join("demo");
    let lib = project.join("lib");
    fs::create_dir_all(&lib).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(
        lib.join("shared.js"),
        r#"
            globalThis.__sharedRuns = (globalThis.__sharedRuns ?? 0) + 1;
            export const value = 9;
        "#,
    )
    .unwrap();
    fs::write(
        lib.join("a.js"),
        "import { value } from './shared.js'; export const a = value + 1;",
    )
    .unwrap();
    fs::write(
        lib.join("b.js"),
        "import { value } from './shared.js'; export const b = value + 2;",
    )
    .unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            import { a } from './lib/a.js';
            import { b } from './lib/b.js';
            log.info(`${a}:${b}:${globalThis.__sharedRuns}`);
        "#,
    )
    .unwrap();

    let outcome = execute_javascript_project(&root, "demo", None).unwrap();

    assert_eq!(outcome.logs[0].message, "10:11:1");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn executes_standard_modules_with_circular_imports() {
    let root = test_root("module-cycle");
    let project = root.join("demo");
    let lib = project.join("lib");
    fs::create_dir_all(&lib).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(
        lib.join("a.js"),
        r#"
            import { fromB } from './b.js';
            export const valueA = 'A';
            export function fromA() {
                return valueA + fromB();
            }
        "#,
    )
    .unwrap();
    fs::write(
        lib.join("b.js"),
        r#"
            import { valueA } from './a.js';
            export function fromB() {
                return valueA + 'B';
            }
        "#,
    )
    .unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            import { fromA } from './lib/a.js';
            log.info(fromA());
        "#,
    )
    .unwrap();

    let outcome = execute_javascript_project(&root, "demo", None).unwrap();

    assert_eq!(outcome.logs[0].message, "AAB");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn executes_standard_modules_with_import_meta_path_rewrite() {
    let root = test_root("module-import-meta");
    let project = root.join("demo");
    let lib = project.join("lib");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&lib).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(
        lib.join("where.js"),
        "export const where = import.meta.dirname + ':' + import.meta.url;",
    )
    .unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            import { where } from './lib/where.js';
            log.info(import.meta.url + "|" + where);
        "#,
    )
    .unwrap();

    let outcome = execute_javascript_project(&root, "demo", None).unwrap();

    assert_eq!(outcome.logs[0].message, "main.js|lib:lib/where.js");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn executes_standard_modules_with_import_meta_object_properties() {
    let root = test_root("module-import-meta-object");
    let project = root.join("demo");
    let lib = project.join("lib");
    fs::create_dir_all(&lib).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(
        lib.join("meta.js"),
        r#"
            const meta = import.meta;
            const { url, dirname } = meta;
            export const where = `${dirname}:${url}`;
        "#,
    )
    .unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            import { where } from './lib/meta.js';
            const meta = import.meta;
            log.info(`${meta.dirname}:${meta.url}|${where}`);
        "#,
    )
    .unwrap();

    let outcome = execute_javascript_project(&root, "demo", None).unwrap();

    assert_eq!(outcome.logs[0].message, ":main.js|lib:lib/meta.js");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn executes_standard_modules_with_nested_referrer_paths_and_resources() {
    let root = test_root("module-nested-referrer");
    let project = root.join("demo");
    let nested = project.join("lib").join("nested");
    fs::create_dir_all(&nested).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(nested.join("note.txt"), "nested note").unwrap();
    fs::write(nested.join("value.js"), "export const value = 7;").unwrap();
    fs::write(
        nested.join("entry.js"),
        r#"
            import { value } from './value.js';
            import note from './note.txt';
            export const combined = `${import.meta.dirname}:${import.meta.url}:${note}:${value}`;
        "#,
    )
    .unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            import { combined } from './lib/nested/entry.js';
            log.info(combined);
        "#,
    )
    .unwrap();

    let outcome = execute_javascript_project(&root, "demo", None).unwrap();

    assert_eq!(
        outcome.logs[0].message,
        "lib/nested:lib/nested/entry.js:nested note:7"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn executes_standard_modules_with_legacy_shaped_resource_imports() {
    let root = test_root("module-resource-imports");
    let project = root.join("demo");
    fs::create_dir_all(project.join("assets")).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(project.join("assets/config.json"), r#"{"enabled":true}"#).unwrap();
    fs::write(project.join("assets/template.txt"), "template body").unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            import config, { unused } from './assets/config.json';
            import * as template from './assets/template.txt';
            log.info(`${config}:${template}`);
        "#,
    )
    .unwrap();

    let outcome = execute_javascript_project(&root, "demo", None).unwrap();

    assert_eq!(outcome.logs[0].message, r#"{"enabled":true}:template body"#);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn executes_script_group_in_order_and_honors_run_count() {
    let app_root = test_root("group");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(
        &scripts_root,
        "first",
        "First",
        r#"
            log.info("first " + settings.level);
            "first";
        "#,
    );
    write_js_project(
        &scripts_root,
        "second",
        "Second",
        r#"
            console.log("second");
            "second";
        "#,
    );

    let group = ScriptGroup {
        name: "daily".to_string(),
        config: ScriptGroupConfig {
            pathing_config: json!({"partyName": "team-a"}),
            ..ScriptGroupConfig::default()
        },
        projects: vec![
            ScriptGroupProject {
                index: 20,
                name: "First".to_string(),
                folder_name: "first".to_string(),
                project_type: ScriptProjectType::Javascript,
                js_script_settings_object: Some(json!({"level": 2})),
                ..ScriptGroupProject::default()
            },
            ScriptGroupProject {
                index: 10,
                name: "Second".to_string(),
                folder_name: "second".to_string(),
                project_type: ScriptProjectType::Javascript,
                run_num: 2,
                ..ScriptGroupProject::default()
            },
        ],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.requested_projects, 2);
    assert_eq!(outcome.attempted_steps, 3);
    assert_eq!(outcome.completed_steps, 3);
    assert_eq!(outcome.failed_steps, 0);
    assert_eq!(outcome.steps.len(), 3);
    assert_eq!(outcome.steps[0].name, "Second");
    assert_eq!(outcome.steps[0].run_iteration, 1);
    assert_eq!(outcome.steps[1].name, "Second");
    assert_eq!(outcome.steps[1].run_iteration, 2);
    assert_eq!(outcome.steps[2].name, "First");
    assert_eq!(
        outcome.steps[2].javascript.as_ref().unwrap().logs[0].message,
        "first 2"
    );
}

#[test]
fn script_group_records_failures_and_continues() {
    let app_root = test_root("group-failure");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(&scripts_root, "bad", "Bad", "throw new Error('boom');");
    write_js_project(&scripts_root, "good", "Good", r#""good";"#);

    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![
            ScriptGroupProject {
                index: 1,
                name: "Bad".to_string(),
                folder_name: "bad".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            },
            ScriptGroupProject {
                index: 2,
                name: "Good".to_string(),
                folder_name: "good".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            },
        ],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.attempted_steps, 2);
    assert_eq!(outcome.failed_steps, 1);
    assert_eq!(outcome.completed_steps, 1);
    assert_eq!(outcome.steps[0].status, ScriptGroupStepStatus::Failed);
    assert!(outcome.steps[0].error.as_deref().unwrap().contains("boom"));
    assert_eq!(outcome.steps[1].status, ScriptGroupStepStatus::Completed);
    assert_eq!(
        outcome.steps[1].javascript.as_ref().unwrap().result,
        Some(json!("good"))
    );
}

#[test]
fn script_group_execution_records_successful_javascript_steps() {
    let app_root = test_root("group-execution-records");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(&scripts_root, "good", "Good", r#""good";"#);
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    let clock = test_execution_record_clock();
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "Good".to_string(),
            folder_name: "good".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
    roots.global_input_dispatch_mode = GlobalInputDispatchMode::PlanOnly;
    let outcome =
        execute_script_group_with_execution_records_and_clock(&roots, &group, &storage, clock)
            .unwrap();

    assert_eq!(outcome.completed_steps, 1);
    let records = storage
        .recent_execution_records_for_today(1, date("2026-06-26"))
        .unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].execution_records.len(), 1);
    let record = &records[0].execution_records[0];
    assert!(record.is_successful);
    assert_eq!(record.group_name, "daily");
    assert_eq!(record.project_name, "Good");
    assert_eq!(record.folder_name, "good");
    assert_eq!(record.project_type, "Javascript");
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_execution_records_run_pre_execution_priority_projects_before_current_group() {
    let app_root = test_root("group-pre-execution-records");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(&scripts_root, "pre", "Pre", r#""pre";"#);
    write_js_project(&scripts_root, "current", "Current", r#""current";"#);
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    let target_group = ScriptGroup {
        name: "Target".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "Pre".to_string(),
            folder_name: "pre".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };
    let current_group = ScriptGroup {
        name: "Current".to_string(),
        config: ScriptGroupConfig {
            pathing_config: json!({
                "enabled": true,
                "preExecutionPriorityConfig": {
                    "enabled": true,
                    "groupNames": "Target"
                }
            }),
            ..ScriptGroupConfig::default()
        },
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "Current".to_string(),
            folder_name: "current".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };
    let all_groups = vec![target_group.clone(), current_group.clone()];
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome = execute_script_group_with_pre_execution_records_hooks_and_cancellation(
        &roots,
        &current_group,
        &all_groups,
        &storage,
        test_execution_record_clock(),
        None,
        None,
        |_| {},
        |_| {},
    )
    .unwrap();

    assert_eq!(outcome.completed_steps, 2);
    assert_eq!(
        outcome
            .steps
            .iter()
            .map(|step| step.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Pre", "Current"]
    );
    let records = storage
        .recent_execution_records_for_today(1, date("2026-06-26"))
        .unwrap();
    let mut record_groups = records[0]
        .execution_records
        .iter()
        .map(|record| {
            (
                record.group_name.as_str(),
                record.project_name.as_str(),
                record.folder_name.as_str(),
            )
        })
        .collect::<Vec<_>>();
    record_groups.sort_unstable();
    assert_eq!(
        record_groups,
        vec![("Current", "Current", "current"), ("Target", "Pre", "pre")]
    );
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_execution_records_skip_pre_execution_priority_candidates_already_completed() {
    let app_root = test_root("group-pre-execution-record-skip");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(&scripts_root, "pre", "Pre", r#""pre";"#);
    write_js_project(&scripts_root, "current", "Current", r#""current";"#);
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    storage
        .save_execution_record(&ExecutionRecord {
            id: "pre-existing-guid".to_string(),
            group_name: "Target".to_string(),
            project_name: "Pre".to_string(),
            folder_name: "pre".to_string(),
            project_type: "Javascript".to_string(),
            server_start_time: Some("2026-06-26T09:00:00+08:00".to_string()),
            start_time: "2026-06-26T09:00:00+08:00".to_string(),
            server_end_time: Some("2026-06-26T09:30:00+08:00".to_string()),
            end_time: "2026-06-26T09:30:00+08:00".to_string(),
            is_successful: true,
        })
        .unwrap();
    let target_group = ScriptGroup {
        name: "Target".to_string(),
        config: ScriptGroupConfig {
            pathing_config: json!({
                "taskCompletionSkipRuleConfig": {
                    "enable": true,
                    "skipPolicy": "SameNameSkipPolicy",
                    "boundaryTime": -1,
                    "lastRunGapSeconds": 3600,
                    "referencePoint": "EndTime"
                }
            }),
            ..ScriptGroupConfig::default()
        },
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "Pre".to_string(),
            folder_name: "pre".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };
    let current_group = ScriptGroup {
        name: "Current".to_string(),
        config: ScriptGroupConfig {
            pathing_config: json!({
                "enabled": true,
                "preExecutionPriorityConfig": {
                    "enabled": true,
                    "groupNames": "Target"
                }
            }),
            ..ScriptGroupConfig::default()
        },
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "Current".to_string(),
            folder_name: "current".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };
    let all_groups = vec![target_group, current_group.clone()];
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome = execute_script_group_with_pre_execution_records_hooks_and_cancellation(
        &roots,
        &current_group,
        &all_groups,
        &storage,
        test_execution_record_clock(),
        None,
        None,
        |_| {},
        |_| {},
    )
    .unwrap();

    assert_eq!(outcome.completed_steps, 1);
    assert_eq!(outcome.steps[0].name, "Current");
    let records = storage
        .recent_execution_records_for_today(1, date("2026-06-26"))
        .unwrap();
    assert_eq!(records[0].execution_records.len(), 2);
    assert_eq!(
        records[0]
            .execution_records
            .iter()
            .filter(|record| record.project_name == "Pre")
            .count(),
        1
    );
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_execution_records_honor_pre_execution_priority_retry_count() {
    let app_root = test_root("group-pre-execution-retry-count");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(&scripts_root, "pre", "Pre", r#""pre";"#);
    write_js_project(&scripts_root, "first", "First", r#""first";"#);
    write_js_project(&scripts_root, "second", "Second", r#""second";"#);
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    let target_group = ScriptGroup {
        name: "Target".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "Pre".to_string(),
            folder_name: "pre".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };
    let current_group = ScriptGroup {
        name: "Current".to_string(),
        config: ScriptGroupConfig {
            pathing_config: json!({
                "enabled": true,
                "preExecutionPriorityConfig": {
                    "enabled": true,
                    "groupNames": "Target",
                    "maxRetryCount": 0
                }
            }),
            ..ScriptGroupConfig::default()
        },
        projects: vec![
            ScriptGroupProject {
                index: 1,
                name: "First".to_string(),
                folder_name: "first".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            },
            ScriptGroupProject {
                index: 2,
                name: "Second".to_string(),
                folder_name: "second".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            },
        ],
        ..ScriptGroup::default()
    };
    let all_groups = vec![target_group, current_group.clone()];
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome = execute_script_group_with_pre_execution_records_hooks_and_cancellation(
        &roots,
        &current_group,
        &all_groups,
        &storage,
        test_execution_record_clock(),
        None,
        None,
        |_| {},
        |_| {},
    )
    .unwrap();

    assert_eq!(
        outcome
            .steps
            .iter()
            .map(|step| step.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Pre", "First", "Second"]
    );
    let records = storage
        .recent_execution_records_for_today(1, date("2026-06-26"))
        .unwrap();
    assert_eq!(
        records[0]
            .execution_records
            .iter()
            .filter(|record| record.project_name == "Pre")
            .count(),
        1
    );
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_execution_records_skip_matching_completed_task() {
    let app_root = test_root("group-execution-record-skip");
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    let existing = ExecutionRecord {
        id: "existing-guid".to_string(),
        group_name: "other".to_string(),
        project_name: "Good".to_string(),
        folder_name: "elsewhere".to_string(),
        project_type: "Javascript".to_string(),
        server_start_time: Some("2026-06-26T09:00:00+08:00".to_string()),
        start_time: "2026-06-26T09:00:00+08:00".to_string(),
        server_end_time: Some("2026-06-26T09:30:00+08:00".to_string()),
        end_time: "2026-06-26T09:30:00+08:00".to_string(),
        is_successful: true,
    };
    storage.save_execution_record(&existing).unwrap();
    let group = ScriptGroup {
        name: "daily".to_string(),
        config: ScriptGroupConfig {
            pathing_config: json!({
                "taskCompletionSkipRuleConfig": {
                    "enable": true,
                    "skipPolicy": "SameNameSkipPolicy",
                    "boundaryTime": -1,
                    "lastRunGapSeconds": 3600,
                    "referencePoint": "EndTime"
                }
            }),
            ..ScriptGroupConfig::default()
        },
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "Good".to_string(),
            folder_name: "good".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome = execute_script_group_with_execution_records_and_clock(
        &roots,
        &group,
        &storage,
        test_execution_record_clock(),
    )
    .unwrap();

    assert_eq!(outcome.skipped_steps, 1);
    assert_eq!(outcome.attempted_steps, 0);
    assert_eq!(
        outcome.steps[0].skip_reason.as_deref(),
        Some("检查出满足跳过条件: 名称相同, 需在 2026-6-26 10:30:00 之后才能开始执行, 匹配记录 GUID=existing-guid")
    );
    let records = storage
        .recent_execution_records_for_today(1, date("2026-06-26"))
        .unwrap();
    assert_eq!(records[0].execution_records.len(), 1);
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_project_execution_records_skip_selected_project_only() {
    let app_root = test_root("group-project-execution-record-skip");
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    storage
        .save_execution_record(&ExecutionRecord {
            id: "selected-guid".to_string(),
            group_name: "daily".to_string(),
            project_name: "Second".to_string(),
            folder_name: "second".to_string(),
            project_type: "Javascript".to_string(),
            server_start_time: Some("2026-06-26T09:00:00+08:00".to_string()),
            start_time: "2026-06-26T09:00:00+08:00".to_string(),
            server_end_time: Some("2026-06-26T09:30:00+08:00".to_string()),
            end_time: "2026-06-26T09:30:00+08:00".to_string(),
            is_successful: true,
        })
        .unwrap();
    let group = execution_record_skip_group(vec![
        ScriptGroupProject {
            index: 1,
            name: "First".to_string(),
            folder_name: "first".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        },
        ScriptGroupProject {
            index: 2,
            name: "Second".to_string(),
            folder_name: "second".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        },
    ]);
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome = execute_script_group_project_with_execution_records_hooks_and_cancellation(
        &roots,
        &group,
        1,
        &storage,
        test_execution_record_clock(),
        false,
        None,
        None,
        |_| {},
        |_| {},
    )
    .unwrap();

    assert_eq!(outcome.requested_projects, 1);
    assert_eq!(outcome.steps.len(), 1);
    assert_eq!(outcome.steps[0].name, "Second");
    assert_eq!(outcome.skipped_steps, 1);
    assert_eq!(
        outcome.steps[0].skip_reason.as_deref(),
        Some("检查出满足跳过条件: 名称相同, 需在 2026-6-26 10:30:00 之后才能开始执行, 匹配记录 GUID=selected-guid")
    );
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_resume_execution_records_preserve_resume_skips_and_completion_skips() {
    let app_root = test_root("group-resume-execution-record-skip");
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    storage
        .save_execution_record(&ExecutionRecord {
            id: "resume-guid".to_string(),
            group_name: "daily".to_string(),
            project_name: "Second".to_string(),
            folder_name: "second".to_string(),
            project_type: "Javascript".to_string(),
            server_start_time: Some("2026-06-26T09:00:00+08:00".to_string()),
            start_time: "2026-06-26T09:00:00+08:00".to_string(),
            server_end_time: Some("2026-06-26T09:30:00+08:00".to_string()),
            end_time: "2026-06-26T09:30:00+08:00".to_string(),
            is_successful: true,
        })
        .unwrap();
    let group = execution_record_skip_group(vec![
        ScriptGroupProject {
            index: 1,
            name: "First".to_string(),
            folder_name: "first".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        },
        ScriptGroupProject {
            index: 2,
            name: "Second".to_string(),
            folder_name: "second".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        },
    ]);
    let resume_pointer = ScriptGroupResumePointer {
        group_name: "daily".to_string(),
        project_index: 2,
        folder_name: "second".to_string(),
        project_name: "Second".to_string(),
    };
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome = execute_script_group_from_resume_with_execution_records_hooks_and_cancellation(
        &roots,
        &group,
        &resume_pointer,
        &storage,
        test_execution_record_clock(),
        None,
        None,
        |_| {},
        |_| {},
    )
    .unwrap();

    assert_eq!(outcome.requested_projects, 2);
    assert_eq!(outcome.skipped_steps, 2);
    assert_eq!(outcome.steps[0].name, "First");
    assert_eq!(outcome.steps[0].skip_reason, None);
    assert_eq!(outcome.steps[1].name, "Second");
    assert_eq!(
        outcome.steps[1].skip_reason.as_deref(),
        Some("检查出满足跳过条件: 名称相同, 需在 2026-6-26 10:30:00 之后才能开始执行, 匹配记录 GUID=resume-guid")
    );
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_project_execution_records_write_selected_project_only() {
    let app_root = test_root("group-project-execution-record-write");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(&scripts_root, "first", "First", r#""first";"#);
    write_js_project(&scripts_root, "second", "Second", r#""second";"#);
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![
            ScriptGroupProject {
                index: 1,
                name: "First".to_string(),
                folder_name: "first".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            },
            ScriptGroupProject {
                index: 2,
                name: "Second".to_string(),
                folder_name: "second".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            },
        ],
        ..ScriptGroup::default()
    };
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome = execute_script_group_project_with_execution_records_hooks_and_cancellation(
        &roots,
        &group,
        1,
        &storage,
        test_execution_record_clock(),
        false,
        None,
        None,
        |_| {},
        |_| {},
    )
    .unwrap();

    assert_eq!(outcome.completed_steps, 1);
    assert_eq!(outcome.steps[0].name, "Second");
    let records = storage
        .recent_execution_records_for_today(1, date("2026-06-26"))
        .unwrap();
    assert_eq!(
        records[0]
            .execution_records
            .iter()
            .map(|record| record.project_name.as_str())
            .collect::<Vec<_>>(),
        vec!["Second"]
    );
    assert!(records[0].execution_records[0].is_successful);
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_project_execution_records_honor_run_count_when_requested() {
    let app_root = test_root("group-project-execution-record-run-count");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(&scripts_root, "repeat", "Repeat", r#""repeat";"#);
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "Repeat".to_string(),
            folder_name: "repeat".to_string(),
            project_type: ScriptProjectType::Javascript,
            run_num: 2,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome = execute_script_group_project_with_execution_records_hooks_and_cancellation(
        &roots,
        &group,
        0,
        &storage,
        test_execution_record_clock(),
        true,
        None,
        None,
        |_| {},
        |_| {},
    )
    .unwrap();

    assert_eq!(outcome.completed_steps, 2);
    assert_eq!(
        outcome
            .steps
            .iter()
            .map(|step| step.run_iteration)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let records = storage
        .recent_execution_records_for_today(1, date("2026-06-26"))
        .unwrap();
    assert_eq!(records[0].execution_records.len(), 2);
    assert!(records[0]
        .execution_records
        .iter()
        .all(|record| record.project_name == "Repeat" && record.is_successful));
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_resume_execution_records_write_from_resume_point_only() {
    let app_root = test_root("group-resume-execution-record-write");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(&scripts_root, "first", "First", r#""first";"#);
    write_js_project(&scripts_root, "second", "Second", r#""second";"#);
    write_js_project(&scripts_root, "third", "Third", r#""third";"#);
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![
            ScriptGroupProject {
                index: 1,
                name: "First".to_string(),
                folder_name: "first".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            },
            ScriptGroupProject {
                index: 2,
                name: "Second".to_string(),
                folder_name: "second".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            },
            ScriptGroupProject {
                index: 3,
                name: "Third".to_string(),
                folder_name: "third".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            },
        ],
        ..ScriptGroup::default()
    };
    let resume_pointer = ScriptGroupResumePointer {
        group_name: "daily".to_string(),
        project_index: 2,
        folder_name: "second".to_string(),
        project_name: "Second".to_string(),
    };
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome = execute_script_group_from_resume_with_execution_records_hooks_and_cancellation(
        &roots,
        &group,
        &resume_pointer,
        &storage,
        test_execution_record_clock(),
        None,
        None,
        |_| {},
        |_| {},
    )
    .unwrap();

    assert_eq!(outcome.skipped_steps, 1);
    assert_eq!(outcome.completed_steps, 2);
    let records = storage
        .recent_execution_records_for_today(1, date("2026-06-26"))
        .unwrap();
    let mut names = records[0]
        .execution_records
        .iter()
        .map(|record| record.project_name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    assert_eq!(names, vec!["Second", "Third"]);
    assert!(records[0]
        .execution_records
        .iter()
        .all(|record| record.is_successful));
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_execution_records_apply_skip_during_before_running_project() {
    let app_root = test_root("group-execution-record-skip-during");
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    let group = ScriptGroup {
        name: "daily".to_string(),
        config: ScriptGroupConfig {
            pathing_config: json!({
                "enabled": true,
                "skipDuring": "10"
            }),
            ..ScriptGroupConfig::default()
        },
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "Missing".to_string(),
            folder_name: "missing".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome = execute_script_group_with_execution_records_and_clock(
        &roots,
        &group,
        &storage,
        test_execution_record_clock(),
    )
    .unwrap();

    assert_eq!(outcome.skipped_steps, 1);
    assert_eq!(
        outcome.steps[0].skip_reason.as_deref(),
        Some("Missing任务已到禁止执行时段，将跳过！")
    );
    assert!(storage
        .recent_execution_records_for_today(1, date("2026-06-26"))
        .unwrap()
        .is_empty());
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_execution_records_apply_task_cycle_before_running_project() {
    let app_root = test_root("group-execution-record-cycle-skip");
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    let group = ScriptGroup {
        name: "daily".to_string(),
        config: ScriptGroupConfig {
            pathing_config: json!({
                "enabled": true,
                "taskCycleConfig": {
                    "enable": true,
                    "boundaryTime": 0,
                    "cycle": 3,
                    "index": 1
                }
            }),
            ..ScriptGroupConfig::default()
        },
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "CycleMissing".to_string(),
            folder_name: "missing".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome = execute_script_group_with_execution_records_and_clock(
        &roots,
        &group,
        &storage,
        test_execution_record_clock(),
    )
    .unwrap();

    assert_eq!(outcome.skipped_steps, 1);
    assert_eq!(
        outcome.steps[0].skip_reason.as_deref(),
        Some("CycleMissing任务已经不在执行周期（当前值$3!=配置值$1），将跳过此任务！")
    );
    assert!(storage
        .recent_execution_records_for_today(1, date("2026-06-26"))
        .unwrap()
        .is_empty());
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_execution_records_apply_farming_plan_skip_before_records() {
    let app_root = test_root("group-execution-record-farming-skip");
    let pathing_root = app_root.join("User").join("AutoPathing");
    let route_dir = pathing_root.join("daily");
    fs::create_dir_all(&route_dir).unwrap();
    fs::write(
        route_dir.join("elite-route.json"),
        r#"{
            "info": {"name": "elite-route"},
            "farming_info": {
                "allow_farming_count": true,
                "primary_target": "elite",
                "normal_mob_count": 2,
                "elite_mob_count": 1
            },
            "positions": []
        }"#,
    )
    .unwrap();
    let farming_log = app_root.join("log").join("FarmingPlan");
    fs::create_dir_all(&farming_log).unwrap();
    fs::write(
        farming_log.join("20260626.json"),
        r#"{
            "total_normal_mob_count": 0,
            "total_elite_mob_count": 1,
            "records": []
        }"#,
    )
    .unwrap();
    let mut farming_config = FarmingPlanConfig::default();
    farming_config.enabled = true;
    farming_config.daily_elite_cap = 1;
    farming_config.daily_mob_cap = 2000;
    let farming_context = FarmingPlanExecutionContext::from_app_root(&app_root, farming_config);
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "elite-route.json".to_string(),
            folder_name: "daily".to_string(),
            project_type: ScriptProjectType::Pathing,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome =
        execute_script_group_with_pre_execution_records_and_farming_plan_hooks_and_cancellation(
            &roots,
            &group,
            std::slice::from_ref(&group),
            &storage,
            test_execution_record_clock(),
            &farming_context,
            None,
            None,
            |_| {},
            |_| {},
        )
        .unwrap();

    assert_eq!(outcome.skipped_steps, 1);
    assert_eq!(
        outcome.steps[0].skip_reason.as_deref(),
        Some("elite-route.json:精英超上限:1/1,脚本主目标为精英,跳过此任务！")
    );
    assert!(storage
        .recent_execution_records_for_today(1, date("2026-06-26"))
        .unwrap()
        .is_empty());
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_pathing_plan_only_does_not_record_farming_session() {
    let app_root = test_root("group-execution-record-farming-plan-only");
    let pathing_root = app_root.join("User").join("AutoPathing");
    let route_dir = pathing_root.join("daily");
    fs::create_dir_all(&route_dir).unwrap();
    fs::write(
        route_dir.join("elite-route.json"),
        r#"{
            "info": {"name": "elite-route"},
            "farming_info": {
                "allow_farming_count": true,
                "primary_target": "elite",
                "normal_mob_count": 2,
                "elite_mob_count": 1
            },
            "positions": [
                {"x": 1.0, "y": 2.0, "type": "path"},
                {"x": 3.0, "y": 4.0, "type": "target", "action": "fight"}
            ]
        }"#,
    )
    .unwrap();
    let mut farming_config = FarmingPlanConfig::default();
    farming_config.enabled = true;
    farming_config.daily_elite_cap = 2000;
    farming_config.daily_mob_cap = 2000;
    let farming_context = FarmingPlanExecutionContext::from_app_root(&app_root, farming_config);
    let storage = ExecutionRecordStorage::from_app_root(&app_root);
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "elite-route.json".to_string(),
            folder_name: "daily".to_string(),
            project_type: ScriptProjectType::Pathing,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome =
        execute_script_group_with_pre_execution_records_and_farming_plan_hooks_and_cancellation(
            &roots,
            &group,
            std::slice::from_ref(&group),
            &storage,
            test_execution_record_clock(),
            &farming_context,
            None,
            None,
            |_| {},
            |_| {},
        )
        .unwrap();

    assert_eq!(outcome.planned_steps, 1);
    let pathing_execution = outcome.steps[0].pathing_execution.as_ref().unwrap();
    assert!(!pathing_execution.completed);
    assert!(!app_root
        .join("log")
        .join("FarmingPlan")
        .join("20260626.json")
        .exists());
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_executes_shell_steps() {
    let app_root = test_root("group-shell");
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: shell_echo_command("group-shell-ok"),
            project_type: ScriptProjectType::Shell,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.attempted_steps, 1);
    assert_eq!(outcome.completed_steps, 1);
    assert_eq!(outcome.failed_steps, 0);
    let shell = outcome.steps[0].shell_result.as_ref().unwrap();
    assert_eq!(shell.status, bgi_task::ShellExecutionStatus::Completed);
    assert!(
        format!("{}\n{}", shell.output_shell, shell.output).contains("group-shell-ok"),
        "unexpected shell output: {:?}",
        shell
    );
}

#[test]
fn script_group_resume_execution_skips_projects_before_pointer() {
    let app_root = test_root("group-resume-shell");
    let resume_command = shell_echo_command("resume-here");
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![
            ScriptGroupProject {
                index: 1,
                name: shell_echo_command("before-resume"),
                project_type: ScriptProjectType::Shell,
                ..ScriptGroupProject::default()
            },
            ScriptGroupProject {
                index: 2,
                name: resume_command.clone(),
                project_type: ScriptProjectType::Shell,
                ..ScriptGroupProject::default()
            },
        ],
        ..ScriptGroup::default()
    };
    let pointer = ScriptGroupResumePointer {
        group_name: "daily".to_string(),
        project_index: 2,
        folder_name: String::new(),
        project_name: resume_command,
    };
    let roots = ScriptGroupExecutionRoots::from_app_root(&app_root);

    let outcome = execute_script_group_from_resume_with_task_dispatcher_hooks_and_cancellation(
        &roots,
        &group,
        &pointer,
        None,
        None,
        |_| {},
        |_| {},
    );

    assert_eq!(outcome.requested_projects, 2);
    assert_eq!(outcome.skipped_steps, 1);
    assert_eq!(outcome.completed_steps, 1);
    assert_eq!(outcome.steps[0].status, ScriptGroupStepStatus::Skipped);
    assert_eq!(outcome.steps[1].status, ScriptGroupStepStatus::Completed);
    assert!(outcome.steps[1]
        .shell_result
        .as_ref()
        .unwrap()
        .output
        .contains("resume-here"));
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_shell_cancellation_stops_group_execution() {
    let app_root = test_root("group-shell-cancel");
    let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
    roots.key_mouse_dispatch_mode = KeyMouseScriptDispatchMode::PlanOnly;
    roots.http_dispatch_mode = HttpDispatchMode::PlanOnly;
    roots.notification_dispatch_mode = NotificationDispatchMode::RecordOnly;
    let cancellation = InputCancellationToken::new();
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![
            ScriptGroupProject {
                index: 1,
                name: shell_sleep_command(2),
                project_type: ScriptProjectType::Shell,
                ..ScriptGroupProject::default()
            },
            ScriptGroupProject {
                index: 2,
                name: shell_echo_command("after-shell-cancel"),
                project_type: ScriptProjectType::Shell,
                ..ScriptGroupProject::default()
            },
        ],
        ..ScriptGroup::default()
    };

    let outcome = std::thread::scope(|scope| {
        scope.spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(50));
            cancellation.cancel();
        });
        execute_script_group_with_task_dispatcher_hooks_and_cancellation(
            &roots,
            &group,
            None,
            Some(&cancellation),
            |_| {},
            |_| {},
        )
    });

    assert_eq!(outcome.attempted_steps, 1);
    assert_eq!(outcome.cancelled_steps, 1);
    assert_eq!(outcome.steps.len(), 1);
    assert_eq!(outcome.steps[0].status, ScriptGroupStepStatus::Cancelled);
    let shell = outcome.steps[0].shell_result.as_ref().unwrap();
    assert_eq!(shell.status, ShellExecutionStatus::Cancelled);
    assert!(shell.waited_for_exit);
}

#[test]
fn script_group_classic_javascript_cancellation_stops_group_execution() {
    let app_root = test_root("group-js-cancel");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(
        &scripts_root,
        "infinite",
        "Infinite",
        r#"
            while (true) {
              Math.random();
            }
        "#,
    );
    let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
    roots.http_dispatch_mode = HttpDispatchMode::PlanOnly;
    roots.notification_dispatch_mode = NotificationDispatchMode::RecordOnly;
    let cancellation = InputCancellationToken::new();
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![
            ScriptGroupProject {
                index: 1,
                name: "Infinite".to_string(),
                folder_name: "infinite".to_string(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            },
            ScriptGroupProject {
                index: 2,
                name: shell_echo_command("after-js-cancel"),
                project_type: ScriptProjectType::Shell,
                ..ScriptGroupProject::default()
            },
        ],
        ..ScriptGroup::default()
    };

    let outcome = std::thread::scope(|scope| {
        scope.spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(50));
            cancellation.cancel();
        });
        execute_script_group_with_task_dispatcher_hooks_and_cancellation(
            &roots,
            &group,
            None,
            Some(&cancellation),
            |_| {},
            |_| {},
        )
    });

    assert_eq!(outcome.attempted_steps, 1);
    assert_eq!(outcome.cancelled_steps, 1);
    assert_eq!(outcome.failed_steps, 0);
    assert_eq!(outcome.steps.len(), 1);
    assert_eq!(outcome.steps[0].status, ScriptGroupStepStatus::Cancelled);
    assert!(outcome.steps[0]
        .error
        .as_deref()
        .unwrap()
        .contains("cancelled"));
    assert!(outcome.steps[0].javascript.is_none());
    assert!(outcome.steps[0].shell_result.is_none());
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_project_execution_runs_only_selected_project_once() {
    let app_root = test_root("group-project-shell");
    let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
    roots.key_mouse_dispatch_mode = KeyMouseScriptDispatchMode::PlanOnly;
    roots.http_dispatch_mode = HttpDispatchMode::PlanOnly;
    roots.notification_dispatch_mode = NotificationDispatchMode::RecordOnly;
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![
            ScriptGroupProject {
                index: 2,
                name: shell_echo_command("selected-project"),
                project_type: ScriptProjectType::Shell,
                run_num: 3,
                ..ScriptGroupProject::default()
            },
            ScriptGroupProject {
                index: 1,
                name: shell_echo_command("not-selected"),
                project_type: ScriptProjectType::Shell,
                ..ScriptGroupProject::default()
            },
        ],
        ..ScriptGroup::default()
    };

    let outcome =
        execute_script_group_project_with_host_hooks(&roots, &group, 0, |_| {}, |_| {}).unwrap();

    assert_eq!(outcome.requested_projects, 1);
    assert_eq!(outcome.attempted_steps, 1);
    assert_eq!(outcome.completed_steps, 1);
    assert_eq!(outcome.steps.len(), 1);
    assert_eq!(outcome.steps[0].project_index, 0);
    assert_eq!(outcome.steps[0].project_order, 2);
    assert_eq!(outcome.steps[0].run_iteration, 1);
    assert_eq!(outcome.steps[0].run_count, 3);
    let shell = outcome.steps[0].shell_result.as_ref().unwrap();
    assert!(!format!("{}\n{}", shell.output_shell, shell.output).contains("not-selected"));
    assert!(format!("{}\n{}", shell.output_shell, shell.output).contains("selected-project"));
}

#[test]
fn script_group_project_repeated_execution_honors_run_count() {
    let app_root = test_root("group-project-repeated-shell");
    let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
    roots.key_mouse_dispatch_mode = KeyMouseScriptDispatchMode::PlanOnly;
    roots.http_dispatch_mode = HttpDispatchMode::PlanOnly;
    roots.notification_dispatch_mode = NotificationDispatchMode::RecordOnly;
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: shell_echo_command("repeat-project"),
            project_type: ScriptProjectType::Shell,
            run_num: 3,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_project_repeated_with_task_dispatcher_hooks(
        &roots,
        &group,
        0,
        None,
        |_| {},
        |_| {},
    )
    .unwrap();

    assert_eq!(outcome.requested_projects, 1);
    assert_eq!(outcome.attempted_steps, 3);
    assert_eq!(outcome.completed_steps, 3);
    assert_eq!(outcome.steps.len(), 3);
    assert_eq!(
        outcome
            .steps
            .iter()
            .map(|step| step.run_iteration)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert!(outcome
        .steps
        .iter()
        .all(|step| step.run_count == 3 && step.shell_result.is_some()));
}

#[test]
fn script_group_key_mouse_steps_can_run_in_plan_only_mode() {
    let app_root = test_root("group-keymouse");
    let macro_root = app_root.join("User").join("KeyMouseScript");
    fs::create_dir_all(&macro_root).unwrap();
    fs::write(
        macro_root.join("macro.json"),
        r#"{
          "macroEvents": [
            { "type": 0, "keyCode": 87, "time": 10 },
            { "type": 1, "keyCode": 87, "time": 30 }
          ]
        }"#,
    )
    .unwrap();
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "macro.json".to_string(),
            folder_name: "macro.json".to_string(),
            project_type: ScriptProjectType::KeyMouse,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.attempted_steps, 1);
    assert_eq!(outcome.planned_steps, 1);
    assert_eq!(outcome.failed_steps, 0);
    let execution = outcome.steps[0].key_mouse_execution.as_ref().unwrap();
    assert_eq!(execution.mode, KeyMouseScriptDispatchMode::PlanOnly);
    assert!(!execution.dispatched);
    assert_eq!(execution.plan.summary.event_count, 2);
    assert_eq!(execution.plan.input_events.len(), 4);
}

#[test]
fn script_group_key_mouse_cancellation_stops_group_execution() {
    let app_root = test_root("group-keymouse-cancel");
    let macro_root = app_root.join("User").join("KeyMouseScript");
    fs::create_dir_all(&macro_root).unwrap();
    fs::write(
        macro_root.join("macro.json"),
        r#"{
          "macroEvents": [
            { "type": 0, "keyCode": 87, "time": 10 },
            { "type": 1, "keyCode": 87, "time": 30 }
          ]
        }"#,
    )
    .unwrap();
    let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
    roots.key_mouse_dispatch_mode = KeyMouseScriptDispatchMode::SendInput;
    roots.http_dispatch_mode = HttpDispatchMode::PlanOnly;
    roots.notification_dispatch_mode = NotificationDispatchMode::RecordOnly;
    let cancellation = InputCancellationToken::new();
    cancellation.cancel();
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![
            ScriptGroupProject {
                index: 1,
                name: "macro.json".to_string(),
                folder_name: "macro.json".to_string(),
                project_type: ScriptProjectType::KeyMouse,
                run_num: 3,
                ..ScriptGroupProject::default()
            },
            ScriptGroupProject {
                index: 2,
                name: shell_echo_command("after-cancel"),
                project_type: ScriptProjectType::Shell,
                ..ScriptGroupProject::default()
            },
        ],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_with_task_dispatcher_hooks_and_cancellation(
        &roots,
        &group,
        None,
        Some(&cancellation),
        |_| {},
        |_| {},
    );

    assert_eq!(outcome.attempted_steps, 1);
    assert_eq!(outcome.cancelled_steps, 1);
    assert_eq!(outcome.completed_steps, 0);
    assert_eq!(outcome.steps.len(), 1);
    assert_eq!(outcome.steps[0].status, ScriptGroupStepStatus::Cancelled);
    assert_eq!(outcome.steps[0].run_iteration, 0);
    assert!(outcome.steps[0].key_mouse_execution.is_none());
    assert!(outcome.steps[0].shell_result.is_none());
}

#[test]
fn script_group_javascript_key_mouse_host_uses_configured_dispatch_mode() {
    let app_root = test_root("group-js-keymouse-host");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(
        &scripts_root,
        "macro-host",
        "MacroHost",
        r#"
            const result = keyMouseScript.run(JSON.stringify({
                macroEvents: [
                    { type: 0, keyCode: 87, time: 10 },
                    { type: 1, keyCode: 87, time: 30 }
                ]
            }));
            log.info(result.mode + ":" + result.dispatched);
            result.dispatched_events;
        "#,
    );
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "MacroHost".to_string(),
            folder_name: "macro-host".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.completed_steps, 1);
    let javascript = outcome.steps[0].javascript.as_ref().unwrap();
    assert_eq!(javascript.result, Some(json!(0)));
    assert_eq!(javascript.logs[0].message, "PlanOnly:false");
    assert!(matches!(
        javascript.host_calls[0].result["mode"],
        serde_json::Value::String(ref mode) if mode == "PlanOnly"
    ));
    assert_eq!(javascript.host_calls[0].result["dispatched"], false);
}

#[test]
fn script_group_javascript_pathing_host_returns_execution_plan() {
    let app_root = test_root("group-js-pathing-host");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(
        &scripts_root,
        "pathing-host",
        "PathingHost",
        r#"
            const route = JSON.stringify({
                info: { name: "route", type: "collect", map_name: "Teyvat" },
                positions: [
                    { x: 1, y: 2, type: "path" },
                    { x: 3, y: 4, type: "teleport" },
                    { x: 5, y: 6, type: "target", action: "fight" }
                ]
            });
            const execution = pathingScript.run(route);
            const plan = pathingScript.plan(route);
            log.info(
                execution.dispatched + ":" +
                execution.execution_plan.segment_count + ":" +
                execution.execution_plan.waypoint_count + ":" +
                plan.summary.waypoint_count
            );
            execution.execution_plan.expected_fight_count;
        "#,
    );
    let group = ScriptGroup {
        name: "daily".to_string(),
        config: ScriptGroupConfig {
            pathing_config: json!({"partyName": "daily"}),
            ..ScriptGroupConfig::default()
        },
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "PathingHost".to_string(),
            folder_name: "pathing-host".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.completed_steps, 1);
    let javascript = outcome.steps[0].javascript.as_ref().unwrap();
    assert_eq!(javascript.result, Some(json!(1)));
    assert_eq!(javascript.logs[0].message, "false:2:3:3");
    assert_eq!(javascript.host_calls.len(), 3);
    assert_eq!(
        javascript.host_calls[0].target,
        ScriptHostTarget::PathingScript
    );
    assert_eq!(javascript.host_calls[0].method, "run");
    assert_eq!(javascript.host_calls[0].result["dispatched"], false);
    assert_eq!(
        javascript.host_calls[0].result["execution_plan"]["segment_count"],
        2
    );
    assert_eq!(javascript.host_calls[1].method, "plan");
    assert_eq!(
        javascript.host_calls[1].result["party_config"],
        json!({"partyName": "daily"})
    );
    assert_eq!(javascript.host_calls[2].target, ScriptHostTarget::Log);
}

#[test]
fn script_group_javascript_http_host_uses_plan_only_in_tests() {
    let app_root = test_root("group-js-http-host");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project_with_manifest(
        &scripts_root,
        "http-host",
        r#"{
          "name": "HttpHost",
          "version": "1.0",
          "main": "main.js",
          "httpAllowedUrls": ["https://example.com/*"]
        }"#,
        r#"
            const request = http.request(
                "POST",
                "https://example.com/status",
                JSON.stringify({ ok: true }),
                JSON.stringify({ "Content-Type": "application/json", "X-Test": "1" })
            );
            log.info(request.method + ":" + request.content_type);
            request.headers.length;
        "#,
    );
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "HttpHost".to_string(),
            folder_name: "http-host".to_string(),
            project_type: ScriptProjectType::Javascript,
            allow_js_http_hash: Some("https://example.com/*".to_string()),
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.completed_steps, 1);
    let javascript = outcome.steps[0].javascript.as_ref().unwrap();
    assert_eq!(javascript.result, Some(json!(1)));
    assert_eq!(javascript.logs[0].message, "POST:application/json");
    assert_eq!(javascript.host_calls[0].target, ScriptHostTarget::Http);
    assert_eq!(javascript.host_calls[0].result["method"], "POST");
    assert_eq!(
        javascript.host_calls[0].result["url"],
        "https://example.com/status"
    );
}

#[test]
fn javascript_can_poll_html_mask_messages_from_initial_state() {
    let root = test_root("html-mask-initial-state");
    write_js_project(
        &root,
        "demo",
        "HtmlMaskBridge",
        r#"
            const message = htmlMask.poll("mask");
            message && message.includes("from-html");
        "#,
    );
    let step = ScriptExecutionStep {
        index: 1,
        name: "HtmlMaskBridge".to_string(),
        folder_name: "demo".to_string(),
        project_type: ScriptProjectType::Javascript,
        engine: bgi_script::ScriptEngineKind::RustJavaScript,
        schedule: bgi_script::ScriptSchedule::parse(""),
        run_count: 1,
        settings: None,
        allow_notification: true,
        allow_http_hash: None,
        target_path: None,
        manifest_main: Some("main.js".to_string()),
        skipped: false,
    };
    let mut prepared = PreparedScriptExecution::prepare_javascript(&step, &root).unwrap();
    prepared.host_runtime_config.html_mask_initial_state = bgi_script::HtmlMaskInitialState {
        windows: vec![bgi_script::HtmlMaskWindowPlan {
            window_id: "mask".to_string(),
            final_url: "https://example.com/mask".to_string(),
            requested_url: "https://example.com/mask".to_string(),
            normalized_path: None,
            click_through: true,
        }],
        from_html: vec![(
            "mask".to_string(),
            bgi_script::HtmlMaskMessage {
                url: "/from-html".to_string(),
                data: Some(json!({ "from-html": true })),
                request_id: Some("req-1".to_string()),
            },
        )],
    };

    let outcome = execute_prepared_javascript(&prepared).unwrap();

    assert_eq!(outcome.result, Some(json!(true)));
    assert!(outcome.html_mask_from_html.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn script_group_javascript_exposes_migrated_host_objects() {
    let app_root = test_root("group-js-host-objects");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(
        &scripts_root,
        "host-objects",
        "HostObjects",
        r#"
            file.writeTextSync("overlay.html", "<html></html>");
            file.writeText("notes.txt", "hello");
            const note = file.readText("notes.txt");
            const macroPlan = keyMouseScript.plan(JSON.stringify({
                macroEvents: [
                    { type: 0, keyCode: 87, time: 0 },
                    { type: 1, keyCode: 87, time: 50 }
                ]
            }));
            const timer = dispatcher.addTimer({ name: "tick", config: { enabled: true } });
            const task = dispatcher.runAutoFightTask({ strategyName: "default" });
            const notify = notification.success("ready");
            const post = PostMessage.keyPress("F");
            const offset = ServerTime.getServerTimeZoneOffset();
            const legacyOffset = ServerTime.serverTimeZoneOffsetMilliseconds();
            const mask = htmlMask.show("overlay.html", "mask");
            htmlMask.sendFromHtml("mask", "/event", JSON.stringify({ ok: true }), "req-1");
            const polled = htmlMask.poll("mask");
            const hook = KeyMouseHook.onKeyDown("key", true);
            const hookDispatch = KeyMouseHook.dispatchEvent({ type: "keyDown", keyCode: "F", keyData: "F" });
            const jagged = host.newVarOfArr("System.String", 2);
            genshin.tp(1, 2, true);
            genshin.chooseTalkOption("hello", 1, false);
            genshin.claimEncounterPointsRewards();
            log.info([
                note,
                macroPlan.summary.event_count,
                timer.AddRealtimeTimer.name,
                task.RunBuiltinTask.name,
                notify.record.kind,
                post.length,
                offset === legacyOffset,
                mask.Show.final_url.length > 0,
                polled.includes("ok"),
                hook.AddListener.id,
                hookDispatch.length,
                jagged.NewArrayVariable.element_type
            ].join(":"));
            dispatcher.commands().length + genshin.commands().length;
        "#,
    );
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "HostObjects".to_string(),
            folder_name: "host-objects".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.completed_steps, 1);
    let javascript = outcome.steps[0].javascript.as_ref().unwrap();
    assert_eq!(javascript.result, Some(json!(6)));
    assert_eq!(
        javascript.logs[0].message,
        "hello:2:tick:AutoFight:Success:4:true:true:true:key:1:System.String"
    );
    assert!(javascript
        .host_calls
        .iter()
        .any(|call| call.target == ScriptHostTarget::HtmlMask && call.method == "show"));
    assert!(javascript
        .host_calls
        .iter()
        .any(|call| call.target == ScriptHostTarget::CustomHostFunctions
            && call.method == "newVarOfArr"));
    assert_eq!(javascript.task_invocations.dispatcher.len(), 2);
    assert_eq!(
        javascript.task_invocations.dispatcher[0].kind,
        bgi_task::TaskInvocationKind::ClearRealtimeTriggers
    );
    assert_eq!(
        javascript.task_invocations.dispatcher[1]
            .task_key
            .as_deref(),
        Some("AutoFight")
    );
    assert_eq!(javascript.task_invocations.genshin.len(), 3);
    assert!(javascript
        .task_invocations
        .genshin
        .iter()
        .any(|plan| plan.task_key.as_deref() == Some("Teleport")));
    assert!(javascript
        .task_invocations
        .genshin
        .iter()
        .any(|plan| plan.task_key.as_deref() == Some("ChooseTalkOption")));
    assert!(javascript
        .task_invocations
        .genshin
        .iter()
        .any(|plan| plan.task_key.as_deref() == Some("ClaimEncounterPointsRewards")));
    assert!(javascript
        .task_invocations
        .errors
        .iter()
        .any(|error| error.contains("dispatcher[1]") && error.contains("tick")));
    assert_eq!(
        javascript.task_execution.mode,
        TaskInvocationExecutionMode::PlanOnly
    );
    assert_eq!(javascript.task_execution.total(), 5);
    assert_eq!(
        javascript.task_execution.dispatcher[0].status,
        bgi_task::TaskInvocationExecutionStatus::Planned
    );
    assert!(javascript
        .task_execution
        .dispatcher
        .iter()
        .chain(javascript.task_execution.genshin.iter())
        .any(|result| matches!(
            result.status,
            bgi_task::TaskInvocationExecutionStatus::RustExecutionPlanReady
                | bgi_task::TaskInvocationExecutionStatus::RustInvocationPlanReady
                | bgi_task::TaskInvocationExecutionStatus::NativePending
        )));
}

#[test]
fn prepared_javascript_can_apply_dispatcher_timer_invocations_to_runtime() {
    let root = test_root("dispatcher-runtime");
    let project = root.join("demo");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            dispatcher.clearAllTriggers();
            dispatcher.addTimer({ name: "AutoPick", interval: 250, config: { source: "script" } });
            dispatcher.commands().length;
        "#,
    )
    .unwrap();
    let prepared = prepare_javascript_project(&root, "demo", None).unwrap();
    let mut dispatcher = DispatcherRuntime {
        frame_index: 7,
        ..DispatcherRuntime::default()
    };

    let outcome =
        execute_prepared_javascript_with_task_dispatcher(&prepared, &mut dispatcher).unwrap();

    assert_eq!(outcome.result, Some(json!(3)));
    assert_eq!(
        outcome.task_execution.mode,
        TaskInvocationExecutionMode::ExecuteReady
    );
    assert_eq!(outcome.task_execution.dispatcher.len(), 3);
    assert!(outcome
        .task_execution
        .dispatcher
        .iter()
        .all(|result| result.executed));
    assert_eq!(dispatcher.registered_realtime_triggers.len(), 1);
    assert_eq!(
        dispatcher.registered_realtime_triggers[0].task_key,
        "AutoPick"
    );
    assert_eq!(dispatcher.registered_realtime_triggers[0].interval_ms, 250);
    assert_eq!(
        dispatcher.registered_realtime_triggers[0].config,
        Some(json!({"ForceInteraction": false, "TextList": []}))
    );
    assert_eq!(
        dispatcher.registered_realtime_triggers[0].registered_at_frame,
        7
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn script_group_javascript_accepts_legacy_pascal_case_host_member_aliases() {
    let app_root = test_root("group-js-host-aliases");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(
        &scripts_root,
        "host-aliases",
        "HostAliases",
        r#"
            file.WriteTextSync("overlay.html", "<html></html>");
            file.WriteText("notes.txt", "hello");
            const note = file.ReadText("notes.txt");
            const macroPlan = keyMouseScript.Plan(JSON.stringify({
                macroEvents: [
                    { type: 0, keyCode: 87, time: 0 },
                    { type: 1, keyCode: 87, time: 50 }
                ]
            }));
            const timer = dispatcher.AddTimer({ name: "tick", config: { enabled: true } });
            const task = dispatcher.RunAutoFightTask({ strategyName: "default" });
            const notify = notification.Success("ready");
            const post = PostMessage.KeyPress("F");
            const offset = ServerTime.GetServerTimeZoneOffset();
            const legacyOffset = ServerTime.ServerTimeZoneOffsetMilliseconds();
            const mask = htmlMask.Show("overlay.html", "mask");
            htmlMask.SendFromHtml("mask", "/event", JSON.stringify({ ok: true }), "req-1");
            const polled = htmlMask.Poll("mask");
            const hook = KeyMouseHook.OnKeyDown("key", true);
            const hookDispatch = KeyMouseHook.DispatchEvent({ type: "KeyDown", keyCode: "F", keyData: "F" });
            const jagged = host.NewVarOfArr("System.String", 2);
            const obj = host.NewObj("System.Text.StringBuilder", "hello");
            const del = host.DelObj(obj);
            const type = host.Type("System.String");
            const iterator = host.ToIterator([1, 2, 3]);
            genshin.Tp(1, 2, true);
            genshin.ChooseTalkOption("hello", 1, false);
            genshin.ClaimEncounterPointsRewards();
            const metrics = GetGameMetrics();
            log.Info([
                metrics.width,
                note,
                macroPlan.summary.event_count,
                timer.AddRealtimeTimer.name,
                task.RunBuiltinTask.name,
                notify.record.kind,
                post.length,
                offset === legacyOffset,
                mask.Show.final_url.length > 0,
                polled.includes("ok"),
                hook.AddListener.id,
                hookDispatch.length,
                jagged.NewArrayVariable.element_type,
                obj.NewObject.type_name,
                del.DeleteObject.target.NewObject.type_name,
                type.TypeLookup.type_name,
                iterator.ToIterator.source.length
            ].join(":"));
            dispatcher.Commands().length + genshin.Commands().length;
        "#,
    );
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "HostAliases".to_string(),
            folder_name: "host-aliases".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.completed_steps, 1);
    let javascript = outcome.steps[0].javascript.as_ref().unwrap();
    assert_eq!(javascript.result, Some(json!(6)));
    assert_eq!(
        javascript.logs[0].message,
        "1920:hello:2:tick:AutoFight:Success:4:true:true:true:key:1:System.String:System.Text.StringBuilder:System.Text.StringBuilder:System.String:3"
    );
    assert!(javascript
        .host_calls
        .iter()
        .any(|call| call.target == ScriptHostTarget::Global && call.method == "getGameMetrics"));
    assert!(javascript
        .host_calls
        .iter()
        .any(|call| call.target == ScriptHostTarget::Log && call.method == "info"));
    assert!(javascript
        .host_calls
        .iter()
        .any(|call| call.target == ScriptHostTarget::KeyMouseHook && call.method == "onKeyDown"));
    assert!(javascript.host_calls.iter().any(|call| call.target
        == ScriptHostTarget::CustomHostFunctions
        && call.method == "newObj"));
    assert!(javascript
        .host_calls
        .iter()
        .any(|call| call.target == ScriptHostTarget::CustomHostFunctions
            && call.method == "toIterator"));
    assert_eq!(javascript.task_invocations.dispatcher.len(), 2);
    assert_eq!(javascript.task_invocations.genshin.len(), 3);
}

#[test]
fn script_group_javascript_exposes_legacy_scheduler_type_constructors() {
    let app_root = test_root("group-js-scheduler-types");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(
        &scripts_root,
        "scheduler-types",
        "SchedulerTypes",
        r#"
            const timer = new RealtimeTimer("AutoPick", {
                textList: ["Crystal Core"],
                forceInteraction: true
            });
            timer.Interval = 125;
            const added = dispatcher.AddTimer(timer);

            const solo = SoloTask("AutoFight", { strategyName: "daily" });
            const task = dispatcher.RunTask(solo);

            log.Info([
                added.AddRealtimeTimer.name,
                added.AddRealtimeTimer.interval_ms,
                added.AddRealtimeTimer.config.TextList[0],
                added.AddRealtimeTimer.config.ForceInteraction,
                task.RunSoloTask.name,
                task.RunSoloTask.config.strategyName
            ].join(":"));
            dispatcher.Commands().length;
        "#,
    );
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "SchedulerTypes".to_string(),
            folder_name: "scheduler-types".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.completed_steps, 1);
    let javascript = outcome.steps[0].javascript.as_ref().unwrap();
    assert_eq!(javascript.result, Some(json!(3)));
    assert_eq!(
        javascript.logs[0].message,
        "AutoPick:125:Crystal Core:true:AutoFight:daily"
    );
    assert_eq!(javascript.task_invocations.dispatcher.len(), 3);
    assert!(javascript.task_invocations.errors.is_empty());
}

#[test]
fn script_group_execute_ready_uses_roots_for_independent_task_invocation_plan() {
    let app_root = test_root("group-js-task-invocation-context");
    let scripts_root = app_root.join("User").join("JsScript");
    fs::create_dir_all(app_root.join("User").join("AutoPathing").join("routes")).unwrap();
    fs::write(
        app_root
            .join("User")
            .join("AutoPathing")
            .join("routes")
            .join("route.json"),
        r#"{
          "info": { "name": "group context route", "type": "collect", "map_name": "Teyvat" },
          "positions": [
            { "x": 1.0, "y": 2.0, "type": "path" },
            { "x": 3.0, "y": 4.0, "type": "target", "action": "fight" }
          ]
        }"#,
    )
    .unwrap();
    write_js_project(
        &scripts_root,
        "task-context",
        "TaskContext",
        r#"
            dispatcher.RunTask(SoloTask("AutoPathing", { route: "routes/route.json" }));
            dispatcher.Commands().length;
        "#,
    );
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "TaskContext".to_string(),
            folder_name: "task-context".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };
    let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
    roots.task_invocation_mode = TaskInvocationExecutionMode::ExecuteReady;
    let mut dispatcher = DispatcherRuntime::default();

    let outcome = execute_script_group_with_task_dispatcher_hooks(
        &roots,
        &group,
        Some(&mut dispatcher),
        |_| {},
        |_| {},
    );

    assert_eq!(outcome.completed_steps, 1);
    let javascript = outcome.steps[0].javascript.as_ref().unwrap();
    assert_eq!(javascript.result, Some(json!(1)));
    assert_eq!(javascript.task_execution.dispatcher.len(), 1);
    let result = &javascript.task_execution.dispatcher[0];
    assert_eq!(
        result.status,
        bgi_task::TaskInvocationExecutionStatus::RustExecutionPlanReady
    );
    let Some(independent_plan) = result.independent_task_execution_plan.as_deref() else {
        panic!("expected AutoPathing independent execution plan");
    };
    let bgi_task::IndependentTaskExecutionPlan::AutoPathingPlan(auto_pathing_plan) =
        independent_plan
    else {
        panic!("expected AutoPathing plan");
    };
    assert_eq!(auto_pathing_plan.summary.name, "group context route");
    assert_eq!(
        auto_pathing_plan.normalized_path,
        PathBuf::from("routes").join("route.json")
    );
    assert_eq!(auto_pathing_plan.execution_plan.waypoint_count, 2);

    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_javascript_exposes_legacy_task_param_constructors() {
    let app_root = test_root("group-js-task-param-types");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(
        &scripts_root,
        "task-param-types",
        "TaskParamTypes",
        r#"
            const skip = new AutoSkipConfig();
            skip.ClickChatOption = "随机选择选项";

            const domain = new AutoDomainParam(0, "domain");
            const boss = AutoBossParam("boss");
            const fight = new AutoFightParam("fight");
            const ley = new AutoLeyLineOutcropParam(3, "蒙德", "启示之花");
            const stygian = AutoStygianOnslaughtParam("stygian");

            const domainRun = dispatcher.RunAutoDomainTask(domain);
            const bossRun = dispatcher.RunAutoBossTask(boss);
            const fightRun = dispatcher.RunAutoFightTask(fight);
            const leyRun = dispatcher.RunAutoLeyLineOutcropTask(ley);
            const stygianRun = dispatcher.RunAutoStygianOnslaughtTask(stygian);

            log.Info([
                skip.ClickChatOption,
                domain.DomainRoundNum,
                domain.CombatStrategyPath,
                boss.CombatStrategyPath,
                fight.CombatStrategyPath,
                ley.Country,
                ley.LeyLineOutcropType,
                stygian.CombatScriptBagPath,
                domainRun.RunBuiltinTask.name,
                bossRun.RunBuiltinTask.name,
                fightRun.RunBuiltinTask.name,
                leyRun.RunBuiltinTask.name,
                stygianRun.RunBuiltinTask.name
            ].join("|"));
            dispatcher.Commands().length;
        "#,
    );
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "TaskParamTypes".to_string(),
            folder_name: "task-param-types".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.completed_steps, 1);
    let javascript = outcome.steps[0].javascript.as_ref().unwrap();
    assert_eq!(javascript.result, Some(json!(5)));
    assert_eq!(
        javascript.logs[0].message,
        "随机选择选项|9999|User/AutoFight/domain.txt|User/AutoFight/boss.txt|User/AutoFight/fight.txt|蒙德|启示之花|stygian|AutoDomain|AutoBoss|AutoFight|AutoLeyLineOutcrop|AutoStygianOnslaught"
    );
    assert_eq!(javascript.task_invocations.dispatcher.len(), 5);
    assert!(javascript.task_invocations.errors.is_empty());
}

#[test]
fn script_group_javascript_exposes_legacy_vision_model_constructors() {
    let app_root = test_root("group-js-vision-types");
    let scripts_root = app_root.join("User").join("JsScript");
    write_js_project(
        &scripts_root,
        "vision-types",
        "VisionTypes",
        r#"
            const roi = new Rect(1, 2, 30, 40);
            const image = new BvImage("AutoPick:F.png", roi, 0.91);
            const locator = new BvLocator(image.RecognitionObject);
            const emptyObject = new RecognitionObject();
            const emptyLocator = new BvLocator(emptyObject);
            const page = new BvPage();

            log.Info([
                image.FeatureName,
                image.AssetName,
                image.RecognitionObject.Name,
                image.RecognitionObject.RegionOfInterest.Width,
                image.RecognitionObject.Template.Threshold,
                locator.RecognitionObject.Template.TemplateAsset.includes("GameTask"),
                emptyLocator.RecognitionObject.RecognitionType,
                page.DefaultTimeoutMs,
                page.CaptureSize.Width
            ].join("|"));
            page.DefaultRetryIntervalMs;
        "#,
    );
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "VisionTypes".to_string(),
            folder_name: "vision-types".to_string(),
            project_type: ScriptProjectType::Javascript,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.completed_steps, 1);
    let javascript = outcome.steps[0].javascript.as_ref().unwrap();
    assert_eq!(javascript.result, Some(json!(1000)));
    assert_eq!(
        javascript.logs[0].message,
        "AutoPick|F.png|AutoPick:F.png|30|0.91|true|None|10000|1920"
    );
}

#[test]
fn key_mouse_hook_dispatch_invokes_registered_javascript_callbacks() {
    let root = test_root("key-mouse-hook-callback");
    let project = root.join("demo");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            const seen = [];
            const key = KeyMouseHook.onKeyDown((value) => seen.push(`key:${value}`), true);
            const move = KeyMouseHook.onMouseMove((x, y) => seen.push(`move:${x},${y}`), 25);
            const first = KeyMouseHook.dispatchEvent({ type: "keyDown", keyCode: "F", keyData: "Control, F" });
            const second = KeyMouseHook.dispatchEvent({ type: "mouseMove", x: 12, y: 34, timestampMs: 100 });
            `${key.AddListener.id}:${move.AddListener.interval_ms}:${first.length}:${second.length}:${seen.join("|")}`;
        "#,
    )
    .unwrap();

    let outcome = execute_javascript_project(&root, "demo", None).unwrap();
    let key_registration = outcome
        .host_calls
        .iter()
        .find(|call| call.target == ScriptHostTarget::KeyMouseHook && call.method == "onKeyDown")
        .unwrap();
    let dispatch_count = outcome
        .host_calls
        .iter()
        .filter(|call| {
            call.target == ScriptHostTarget::KeyMouseHook && call.method == "dispatchEvent"
        })
        .count();

    assert_eq!(
        outcome.result,
        Some(json!("callback-1:25:1:1:key:F|move:12,34"))
    );
    assert_eq!(
        key_registration.args,
        vec![json!("callback-1"), json!(true)]
    );
    assert_eq!(dispatch_count, 2);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn key_mouse_hook_dispatch_settles_async_javascript_callback() {
    let root = test_root("key-mouse-hook-async-callback");
    let project = root.join("demo");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("manifest.json"),
        r#"{"name":"Demo","version":"1.0","main":"main.js"}"#,
    )
    .unwrap();
    fs::write(
        project.join("main.js"),
        r#"
            const seen = [];
            KeyMouseHook.onKeyDown(async (value) => {
                const resolved = await Promise.resolve(value + "!");
                seen.push(resolved);
            }, true);
            const dispatched = KeyMouseHook.dispatchEvent({ type: "keyDown", keyCode: "F", keyData: "F" });
            `${dispatched.length}:${seen.join("|")}`;
        "#,
    )
    .unwrap();

    let outcome = execute_javascript_project(&root, "demo", None).unwrap();

    assert_eq!(outcome.result, Some(json!("1:F!")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn script_group_pathing_steps_return_execution_plan() {
    let app_root = test_root("group-pathing");
    let pathing_root = app_root.join("User").join("AutoPathing").join("routes");
    fs::create_dir_all(&pathing_root).unwrap();
    fs::write(
        pathing_root.join("route.json"),
        r#"{
          "info": { "name": "route", "type": "collect", "map_name": "Teyvat" },
          "positions": [
            { "x": 1.0, "y": 2.0, "type": "path" },
            { "x": 3.0, "y": 4.0, "type": "teleport" },
            { "x": 5.0, "y": 6.0, "type": "target", "action": "fight" }
          ]
        }"#,
    )
    .unwrap();
    let group = ScriptGroup {
        name: "daily".to_string(),
        config: ScriptGroupConfig {
            pathing_config: json!({"partyName": "daily"}),
            ..ScriptGroupConfig::default()
        },
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "route.json".to_string(),
            folder_name: "routes".to_string(),
            project_type: ScriptProjectType::Pathing,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.attempted_steps, 1);
    assert_eq!(outcome.planned_steps, 1);
    assert_eq!(outcome.failed_steps, 0);
    let execution = outcome.steps[0].pathing_execution.as_ref().unwrap();
    assert!(!execution.dispatched);
    assert_eq!(execution.plan.summary.waypoint_count, 3);
    assert_eq!(
        execution.plan.party_config,
        Some(json!({"partyName": "daily"}))
    );
    assert_eq!(execution.execution_plan.segment_count, 2);
    assert_eq!(execution.execution_plan.expected_fight_count, 1);
    assert!(execution.execution_plan.segments[1].starts_with_teleport);
}

#[test]
fn script_group_pathing_applies_control_json_overrides() {
    let app_root = test_root("group-pathing-control-json");
    let pathing_root = app_root.join("User").join("AutoPathing").join("routes");
    fs::create_dir_all(&pathing_root).unwrap();
    fs::write(
        pathing_root.join("route.json"),
        r#"{
          "info": { "name": "route", "type": "collect", "map_name": "Teyvat" },
          "positions": [
            { "x": 1.0, "y": 2.0, "type": "path" }
          ]
        }"#,
    )
    .unwrap();
    fs::write(
        pathing_root.join("control.json5"),
        r#"{
          global_cover: {
            farming_info: {
              allow_farming_count: true,
              primary_target: "elite",
              normal_mob_count: 2,
              elite_mob_count: 3
            }
          },
          json_list: [
            {
              name: "route",
              cover: {
                info: { name: "merged route" },
                positions: [
                  { x: 1.0, y: 2.0, type: "path" },
                  { x: 3.0, y: 4.0, type: "target", action: "fight" }
                ]
              }
            }
          ]
        }"#,
    )
    .unwrap();
    let group = ScriptGroup {
        name: "daily".to_string(),
        config: ScriptGroupConfig {
            pathing_config: json!({"partyName": "daily"}),
            ..ScriptGroupConfig::default()
        },
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "route.json".to_string(),
            folder_name: "routes".to_string(),
            project_type: ScriptProjectType::Pathing,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };

    let outcome = execute_script_group_for_test(&app_root, &group);

    assert_eq!(outcome.planned_steps, 1);
    let execution = outcome.steps[0].pathing_execution.as_ref().unwrap();
    assert_eq!(execution.plan.summary.name, "merged route");
    assert_eq!(execution.execution_plan.expected_fight_count, 1);
    assert!(execution.execution_plan.farming.allow_farming_count);
    assert_eq!(execution.execution_plan.farming.primary_target, "elite");
    assert_eq!(execution.execution_plan.farming.normal_mob_count, 2.0);
    assert_eq!(execution.execution_plan.farming.elite_mob_count, 3.0);
    fs::remove_dir_all(app_root).unwrap();
}

#[test]
fn script_group_pathing_rejects_routes_requiring_newer_bgi_version() {
    let app_root = test_root("group-pathing-bgi-version");
    let pathing_root = app_root.join("User").join("AutoPathing").join("routes");
    fs::create_dir_all(&pathing_root).unwrap();
    fs::write(
        pathing_root.join("route.json"),
        r#"{
          "info": { "name": "route", "type": "collect", "map_name": "Teyvat", "bgi_version": "9.9.9" },
          "positions": []
        }"#,
    )
    .unwrap();
    let group = ScriptGroup {
        name: "daily".to_string(),
        projects: vec![ScriptGroupProject {
            index: 1,
            name: "route.json".to_string(),
            folder_name: "routes".to_string(),
            project_type: ScriptProjectType::Pathing,
            ..ScriptGroupProject::default()
        }],
        ..ScriptGroup::default()
    };
    let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
    roots.app_version = Some("1.0.0".to_string());

    let outcome = execute_script_group_with_roots(&roots, &group);

    assert_eq!(outcome.failed_steps, 1);
    assert!(outcome.steps[0]
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("版本号要求 9.9.9 大于当前 BetterGI 版本号 1.0.0"));
    fs::remove_dir_all(app_root).unwrap();
}

#[cfg(windows)]
fn shell_echo_command(message: &str) -> String {
    format!("echo {message} & exit")
}

#[cfg(not(windows))]
fn shell_echo_command(message: &str) -> String {
    format!("echo {message}; exit")
}

#[cfg(windows)]
fn shell_sleep_command(seconds: u64) -> String {
    format!("ping -n {} 127.0.0.1 > nul & exit", seconds + 1)
}

#[cfg(not(windows))]
fn shell_sleep_command(seconds: u64) -> String {
    format!("sleep {seconds}; exit")
}

fn write_js_project(root: &Path, folder: &str, name: &str, main: &str) {
    write_js_project_with_manifest(
        root,
        folder,
        &format!(r#"{{"name":"{name}","version":"1.0","main":"main.js"}}"#),
        main,
    );
}

fn write_js_project_with_manifest(root: &Path, folder: &str, manifest: &str, main: &str) {
    let project = root.join(folder);
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("manifest.json"), manifest).unwrap();
    fs::write(project.join("main.js"), main).unwrap();
}

fn execute_script_group_for_test(
    app_root: &Path,
    group: &ScriptGroup,
) -> ScriptGroupExecutionOutcome {
    let mut roots = ScriptGroupExecutionRoots::from_app_root(app_root);
    roots.global_input_dispatch_mode = GlobalInputDispatchMode::PlanOnly;
    roots.key_mouse_dispatch_mode = KeyMouseScriptDispatchMode::PlanOnly;
    roots.http_dispatch_mode = HttpDispatchMode::PlanOnly;
    roots.notification_dispatch_mode = NotificationDispatchMode::RecordOnly;
    execute_script_group_with_roots(&roots, group)
}

fn test_execution_record_clock() -> ExecutionRecordClock {
    ExecutionRecordClock::fixed(
        NaiveDateTime::parse_from_str("2026-06-26T10:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
        FixedOffset::east_opt(8 * 3_600).unwrap(),
        DateTime::parse_from_rfc3339("2026-06-26T10:00:00+08:00").unwrap(),
    )
}

fn execution_record_skip_group(projects: Vec<ScriptGroupProject>) -> ScriptGroup {
    ScriptGroup {
        name: "daily".to_string(),
        config: ScriptGroupConfig {
            pathing_config: json!({
                "taskCompletionSkipRuleConfig": {
                    "enable": true,
                    "skipPolicy": "SameNameSkipPolicy",
                    "boundaryTime": -1,
                    "lastRunGapSeconds": 3600,
                    "referencePoint": "EndTime"
                }
            }),
            ..ScriptGroupConfig::default()
        },
        projects,
        ..ScriptGroup::default()
    }
}

fn date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
}

fn test_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("bgi-script-engine-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}
