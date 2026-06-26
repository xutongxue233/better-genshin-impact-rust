use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn settings_schema_deserializes_legacy_items() {
    let schema = ScriptSettingsSchema::from_json(
        r#"[
                { "name": "strategy", "type": "input-text", "label": "Strategy", "default": "default.txt" },
                { "name": "enabled", "type": "checkbox", "default": "true" },
                { "name": "items", "type": "multi-checkbox", "options": ["a", "b"], "default": ["a"] }
            ]"#,
    )
    .unwrap();

    assert_eq!(schema.items.len(), 3);
    assert_eq!(schema.items[0].kind(), ScriptSettingKind::InputText);
    assert_eq!(schema.items[2].kind(), ScriptSettingKind::MultiCheckbox);
}

#[test]
fn settings_defaults_match_wpf_binding_initialization() {
    let schema = ScriptSettingsSchema::from_json(
        r#"[
                { "name": "text", "type": "input-text", "default": 3 },
                { "name": "flag", "type": "checkbox", "default": "true" },
                { "name": "choices", "type": "multi-checkbox", "default": ["x", "y"] }
            ]"#,
    )
    .unwrap();
    let mut settings = Map::new();

    schema.apply_defaults(&mut settings);

    assert_eq!(settings["text"], Value::String("3".to_string()));
    assert_eq!(settings["flag"], Value::Bool(true));
    assert_eq!(
        settings["choices"],
        Value::Array(vec![
            Value::String("x".to_string()),
            Value::String("y".to_string())
        ])
    );
}

#[test]
fn multi_checkbox_cleanup_removes_values_not_in_options() {
    let schema = ScriptSettingsSchema::from_json(
        r#"[
                { "name": "choices", "type": "multi-checkbox", "options": ["x", "y"] }
            ]"#,
    )
    .unwrap();
    let mut settings = Map::from_iter([(
        "choices".to_string(),
        Value::Array(vec![
            Value::String("x".to_string()),
            Value::String("z".to_string()),
        ]),
    )]);

    let removed = schema.clean_invalid_values(&mut settings);

    assert_eq!(removed, 1);
    assert_eq!(
        settings["choices"],
        Value::Array(vec![Value::String("x".to_string())])
    );
}

#[test]
fn settings_document_loads_project_schema_defaults_and_cleans_values() {
    let root = temp_root("settings-document");
    let scripts_root = root.join("User/JsScript");
    let project_root = scripts_root.join("demo");
    fs::create_dir_all(&project_root).unwrap();
    fs::write(
        project_root.join("manifest.json"),
        r#"{
                "name": "demo",
                "version": "1.0.0",
                "main": "main.js",
                "settingsUi": "settings.json"
            }"#,
    )
    .unwrap();
    fs::write(project_root.join("main.js"), "export default 1;").unwrap();
    fs::write(
        project_root.join("settings.json"),
        r#"[
                { "name": "strategy", "type": "input-text", "default": "default.txt" },
                { "name": "choices", "type": "multi-checkbox", "options": ["a", "b"], "default": ["a"] }
            ]"#,
    )
    .unwrap();

    let document = read_script_settings_document(
        &scripts_root,
        "demo",
        Some(serde_json::json!({ "choices": ["a", "z"] })),
    )
    .unwrap();

    assert_eq!(document.manifest.name, "demo");
    assert_eq!(document.schema.as_ref().unwrap().items.len(), 2);
    assert_eq!(
        document.values["strategy"],
        Value::String("default.txt".to_string())
    );
    assert_eq!(
        document.values["choices"],
        Value::Array(vec![Value::String("a".to_string())])
    );
    assert_eq!(document.cleaned_invalid_values, 1);
    assert!(document.defaults_applied);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_group_project_settings_updates_js_settings_object() {
    let root = temp_root("settings-save");
    let scripts_root = root.join("User/JsScript");
    let project_root = scripts_root.join("demo");
    let group_root = root.join("User/ScriptGroup");
    fs::create_dir_all(&project_root).unwrap();
    fs::create_dir_all(&group_root).unwrap();
    fs::write(
        project_root.join("manifest.json"),
        r#"{
                "name": "demo",
                "version": "1.0.0",
                "main": "main.js",
                "settingsUi": "settings.json"
            }"#,
    )
    .unwrap();
    fs::write(project_root.join("main.js"), "export default 1;").unwrap();
    fs::write(
        project_root.join("settings.json"),
        r#"[
                { "name": "enabled", "type": "checkbox", "default": "true" },
                { "name": "choices", "type": "multi-checkbox", "options": ["a", "b"] }
            ]"#,
    )
    .unwrap();
    fs::write(
        group_root.join("Daily.json"),
        r#"{
                "index": 0,
                "name": "Daily",
                "projects": [
                    {
                        "index": 0,
                        "name": "demo",
                        "folderName": "demo",
                        "type": "Javascript",
                        "status": "Enabled",
                        "schedule": "Daily",
                        "runNum": 1,
                        "jsScriptSettingsObject": { "choices": ["a", "stale"] }
                    }
                ]
            }"#,
    )
    .unwrap();

    let result = save_script_group_project_settings(
        &group_root,
        "Daily",
        0,
        &scripts_root,
        serde_json::json!({ "choices": ["b", "stale"] }),
    )
    .unwrap();
    let saved = fs::read_to_string(group_root.join("Daily.json")).unwrap();
    let group = parse_script_group_json(&saved, None).unwrap();

    assert_eq!(result.cleaned_invalid_values, 1);
    assert_eq!(
        group.projects[0]
            .js_script_settings_object
            .as_ref()
            .unwrap()["enabled"],
        true
    );
    assert_eq!(
        group.projects[0]
            .js_script_settings_object
            .as_ref()
            .unwrap()["choices"],
        Value::Array(vec![Value::String("b".to_string())])
    );

    fs::remove_dir_all(root).unwrap();
}

fn temp_root(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("bgi-{name}-{suffix}"))
}
