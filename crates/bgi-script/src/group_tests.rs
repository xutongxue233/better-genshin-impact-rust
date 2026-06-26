use super::*;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn pathing_project_matches_legacy_defaults() {
    let project = ScriptGroupProject::pathing("route.json", "folder");
    assert_eq!(project.project_type, ScriptProjectType::Pathing);
    assert_eq!(project.status, ScriptProjectStatus::Enabled);
    assert_eq!(project.schedule, "Daily");
}

#[test]
fn script_group_crud_preserves_legacy_json_shape_and_indexes() {
    let root = temp_root("group-crud");
    let created = create_script_group(&root, "Daily").unwrap();
    assert_eq!(created.group.name, "Daily");
    assert_eq!(created.group.index, 1);
    assert!(created.path.ends_with("Daily.json"));

    let with_project = add_script_group_project(
        &root,
        "Daily",
        ScriptGroupProject::javascript("Demo", "demo-folder"),
    )
    .unwrap();
    assert_eq!(with_project.group.projects[0].index, 1);
    assert_eq!(with_project.group.projects[0].schedule, "Daily");

    let updated = update_script_group_project(
        &root,
        "Daily",
        0,
        ScriptGroupProjectPatch {
            status: Some(ScriptProjectStatus::Disabled),
            run_num: Some(0),
            allow_js_notification: Some(false),
            ..ScriptGroupProjectPatch::default()
        },
    )
    .unwrap();
    assert_eq!(
        updated.group.projects[0].status,
        ScriptProjectStatus::Disabled
    );
    assert_eq!(updated.group.projects[0].run_num, 1);
    assert_eq!(updated.group.projects[0].allow_js_notification, Some(false));

    let renamed = rename_script_group(&root, "Daily", "Nightly").unwrap();
    assert_eq!(renamed.group.name, "Nightly");
    assert!(!root.join("Daily.json").exists());
    assert!(root.join("Nightly.json").exists());

    let empty = remove_script_group_project(&root, "Nightly", 0).unwrap();
    assert!(empty.group.projects.is_empty());
    assert!(delete_script_group(&root, "Nightly").unwrap());
    assert!(!delete_script_group(&root, "Nightly").unwrap());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn script_group_project_move_reorders_and_normalizes_indexes() {
    let root = temp_root("group-move");
    create_script_group(&root, "Daily").unwrap();
    add_script_group_project(&root, "Daily", ScriptGroupProject::javascript("One", "one")).unwrap();
    add_script_group_project(&root, "Daily", ScriptGroupProject::key_mouse("Two")).unwrap();
    add_script_group_project(&root, "Daily", ScriptGroupProject::shell("echo three")).unwrap();

    let moved = move_script_group_project(&root, "Daily", 2, 0).unwrap();

    let names = moved
        .group
        .projects
        .iter()
        .map(|project| project.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["echo three", "One", "Two"]);
    assert_eq!(
        moved
            .group
            .projects
            .iter()
            .map(|project| project.index)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );

    let unchanged = move_script_group_project(&root, "Daily", 1, 1).unwrap();
    assert_eq!(unchanged.group.projects[1].name, "One");

    let error = move_script_group_project(&root, "Daily", 0, 3)
        .expect_err("target beyond the end should fail");
    assert!(error.to_string().contains("target index 3"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn resume_group_selection_starts_from_marked_group() {
    let groups = vec![
        ScriptGroup {
            name: "A".to_string(),
            ..ScriptGroup::default()
        },
        ScriptGroup {
            name: "B".to_string(),
            ..ScriptGroup::default()
        },
        ScriptGroup {
            name: "C".to_string(),
            ..ScriptGroup::default()
        },
    ];

    let (selected, consumed) = select_script_groups_from_resume(&groups, Some("B"));

    assert!(consumed);
    assert_eq!(
        selected
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>(),
        ["B", "C"]
    );

    let (fallback, consumed) = select_script_groups_from_resume(&groups, Some("Missing"));
    assert!(!consumed);
    assert_eq!(fallback.len(), 3);
}

#[test]
fn resume_project_selection_marks_previous_projects_skipped() {
    let group = ScriptGroup {
        name: "Daily".to_string(),
        projects: vec![
            ScriptGroupProject {
                index: 1,
                name: "First".to_string(),
                folder_name: "first".to_string(),
                ..ScriptGroupProject::default()
            },
            ScriptGroupProject {
                index: 2,
                name: "Second".to_string(),
                folder_name: "second".to_string(),
                ..ScriptGroupProject::default()
            },
            ScriptGroupProject {
                index: 3,
                name: "Third".to_string(),
                folder_name: "third".to_string(),
                ..ScriptGroupProject::default()
            },
        ],
        ..ScriptGroup::default()
    };
    let pointer = ScriptGroupResumePointer {
        group_name: "Daily".to_string(),
        project_index: 2,
        folder_name: "second".to_string(),
        project_name: "Second".to_string(),
    };

    let (projects, consumed) = select_script_group_projects_from_resume(&group, Some(&pointer));

    assert!(consumed);
    assert_eq!(projects.len(), 3);
    assert_eq!(projects[0].skip_flag, Some(true));
    assert_eq!(projects[0].next_flag, Some(false));
    assert_eq!(projects[1].skip_flag, Some(false));
    assert_eq!(projects[1].next_flag, Some(true));
    assert_eq!(projects[2].skip_flag, Some(false));
    assert_eq!(projects[2].next_flag, Some(false));
}

#[test]
fn resume_project_selection_falls_back_when_pointer_does_not_match() {
    let group = ScriptGroup {
        name: "Daily".to_string(),
        projects: vec![ScriptGroupProject::javascript("Demo", "demo")],
        ..ScriptGroup::default()
    };
    let pointer = ScriptGroupResumePointer {
        group_name: "Other".to_string(),
        project_index: 1,
        folder_name: "demo".to_string(),
        project_name: "Demo".to_string(),
    };

    let (projects, consumed) = select_script_group_projects_from_resume(&group, Some(&pointer));

    assert!(!consumed);
    assert_eq!(projects, group.projects);
}

#[test]
fn available_js_script_projects_reads_valid_manifest_projects() {
    let root = temp_root("available-js");
    let scripts_root = root.join("User/JsScript");
    let project_root = scripts_root.join("demo");
    fs::create_dir_all(&project_root).unwrap();
    fs::write(
        project_root.join("manifest.json"),
        r#"{
            "name": "Demo",
            "version": "1.0.0",
            "description": "line1\nline2",
            "main": "main.js",
            "settingsUi": "settings.json"
        }"#,
    )
    .unwrap();
    fs::write(project_root.join("main.js"), "export default 1;").unwrap();
    fs::write(project_root.join("settings.json"), "[]").unwrap();

    let projects = available_js_script_projects(&scripts_root).unwrap();

    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].folder_name, "demo");
    assert_eq!(projects[0].name, "Demo");
    assert!(projects[0].has_settings_ui);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn key_mouse_pathing_and_shell_projects_match_legacy_add_defaults() {
    let root = temp_root("group-add-kinds");
    create_script_group(&root, "Daily").unwrap();

    let group = add_key_mouse_script_project(&root, "Daily", "macro.json").unwrap();
    assert_eq!(group.group.projects[0].index, 1);
    assert_eq!(
        group.group.projects[0].project_type,
        ScriptProjectType::KeyMouse
    );
    assert_eq!(group.group.projects[0].name, "macro.json");
    assert_eq!(group.group.projects[0].folder_name, "macro.json");

    let group = add_pathing_script_project(&root, "Daily", "route.json", "liyue/mining").unwrap();
    assert_eq!(
        group.group.projects[1].project_type,
        ScriptProjectType::Pathing
    );
    assert_eq!(group.group.projects[1].name, "route.json");
    assert_eq!(group.group.projects[1].folder_name, "liyue/mining");

    let group = add_shell_script_project(&root, "Daily", "echo ok").unwrap();
    assert_eq!(
        group.group.projects[2].project_type,
        ScriptProjectType::Shell
    );
    assert_eq!(group.group.projects[2].name, "echo ok");
    assert_eq!(group.group.projects[2].folder_name, "");
    assert_eq!(
        group
            .group
            .projects
            .iter()
            .map(|project| project.index)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn key_mouse_and_pathing_scans_preserve_relative_paths() {
    let root = temp_root("available-non-js");
    let key_mouse_root = root.join("User/KeyMouseScript");
    let pathing_root = root.join("User/AutoPathing");
    fs::create_dir_all(key_mouse_root.join("combat")).unwrap();
    fs::create_dir_all(pathing_root.join("liyue/mining")).unwrap();
    fs::write(key_mouse_root.join("combat/macro.json"), "{}").unwrap();
    fs::write(pathing_root.join("root.json"), "{}").unwrap();
    fs::write(pathing_root.join("liyue/mining/route.json"), "{}").unwrap();
    fs::write(pathing_root.join("liyue/mining/readme.txt"), "skip").unwrap();

    let key_mouse = available_key_mouse_scripts(&key_mouse_root).unwrap();
    let pathing = available_pathing_scripts(&pathing_root).unwrap();
    let pathing_tree = available_pathing_tree(&pathing_root).unwrap();

    assert_eq!(
        key_mouse,
        vec![AvailableKeyMouseScript {
            name: "macro.json".to_string(),
            relative_path: "combat/macro.json".to_string(),
        }]
    );
    assert_eq!(
        pathing,
        vec![
            AvailablePathingScript {
                name: "route.json".to_string(),
                folder_name: "liyue/mining".to_string(),
                relative_path: "liyue/mining/route.json".to_string(),
            },
            AvailablePathingScript {
                name: "root.json".to_string(),
                folder_name: "".to_string(),
                relative_path: "root.json".to_string(),
            },
        ]
    );
    assert_eq!(pathing_tree.name, "AutoPathing");
    assert_eq!(pathing_tree.children.len(), 2);
    assert_eq!(pathing_tree.children[0].name, "liyue");
    assert!(pathing_tree.children[0].route.is_none());
    assert_eq!(pathing_tree.children[0].children[0].name, "mining");
    let nested_route = &pathing_tree.children[0].children[0].children[0];
    assert_eq!(nested_route.relative_path, "liyue/mining/route.json");
    assert_eq!(
        nested_route.route.as_ref().unwrap().folder_name,
        "liyue/mining"
    );
    assert_eq!(pathing_tree.children[1].relative_path, "root.json");
    assert!(pathing_tree.children[1].route.is_some());

    fs::remove_dir_all(root).unwrap();
}

fn temp_root(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("bgi-{name}-{suffix}"))
}
