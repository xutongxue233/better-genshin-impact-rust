use super::*;
use bgi_core::config::ScriptConfig;
use std::fs;
use std::io::Write;

fn test_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    fs::remove_dir_all(&root).unwrap_or(());
    fs::create_dir_all(&root).unwrap();
    root
}
fn script_config() -> ScriptConfig {
    ScriptConfig::default()
}

#[derive(Debug, Default)]
struct RecordingGitRunner {
    commands: Vec<(Option<PathBuf>, Vec<String>)>,
    repo_url: String,
    remote_sha: String,
    current_sha: String,
    origin_url: String,
    repo_json: String,
    objects: BTreeMap<String, (String, String)>,
    binary_objects: BTreeMap<String, (String, Vec<u8>)>,
    trees: BTreeMap<String, String>,
}

impl RecordingGitRunner {
    fn new(repo_url: &str, repo_json: &str) -> Self {
        Self {
            commands: Vec::new(),
            repo_url: repo_url.to_string(),
            remote_sha: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            current_sha: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            origin_url: repo_url.to_string(),
            repo_json: repo_json.to_string(),
            objects: BTreeMap::new(),
            binary_objects: BTreeMap::new(),
            trees: BTreeMap::new(),
        }
    }
}

impl ScriptRepoGitRunner for RecordingGitRunner {
    fn run_git(
        &mut self,
        cwd: Option<&Path>,
        args: &[String],
    ) -> Result<ScriptRepoGitCommandOutput> {
        self.commands
            .push((cwd.map(Path::to_path_buf), args.to_vec()));
        let stdout_bytes = match args
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["rev-parse", "--is-inside-work-tree"] => b"true".to_vec(),
            ["remote", "get-url", "origin"] => self.origin_url.clone().into_bytes(),
            ["ls-remote", "--heads", url, branch] => {
                assert_eq!(*url, self.repo_url);
                format!("{}\trefs/heads/{branch}", self.remote_sha).into_bytes()
            }
            ["rev-parse", "release"] => self.current_sha.clone().into_bytes(),
            ["show", "release:repo.json"] => self.repo_json.clone().into_bytes(),
            ["cat-file", "-t", object] => self
                .binary_objects
                .get(*object)
                .map(|(kind, _)| kind.clone())
                .or_else(|| self.objects.get(*object).map(|(kind, _)| kind.clone()))
                .ok_or_else(|| ScriptRepoError::MissingSource(PathBuf::from(object)))?
                .into_bytes(),
            ["ls-tree", object] => self
                .trees
                .get(*object)
                .cloned()
                .ok_or_else(|| ScriptRepoError::MissingSource(PathBuf::from(object)))?
                .into_bytes(),
            ["show", object] => self
                .binary_objects
                .get(*object)
                .map(|(_, content)| content.clone())
                .or_else(|| {
                    self.objects
                        .get(*object)
                        .map(|(_, content)| content.clone().into_bytes())
                })
                .ok_or_else(|| ScriptRepoError::MissingSource(PathBuf::from(object)))?,
            _ => Vec::new(),
        };
        Ok(ScriptRepoGitCommandOutput {
            stdout: String::from_utf8_lossy(&stdout_bytes).trim().to_string(),
            stderr: String::new(),
            stdout_bytes,
        })
    }
}
#[test]
fn repo_url_and_folder_name_follow_legacy_channel_rules() {
    let config = script_config();
    assert_eq!(
        resolve_repo_url(&config).as_deref(),
        Some("https://cnb.cool/bettergi/bettergi-scripts-list")
    );

    let mut custom = script_config();
    custom.selected_channel_name = "自定义".to_string();
    custom.custom_repo_url = "https://example.com/custom-repo".to_string();
    assert_eq!(resolve_repo_url(&custom), None);
    custom.custom_repo_url = "https://host/owner/custom.git".to_string();
    assert_eq!(
        resolve_repo_url(&custom).as_deref(),
        Some("https://host/owner/custom.git")
    );

    let mut mapping = BTreeMap::new();
    mapping.insert(
        "https://host/owner/custom.git".to_string(),
        "mapped-folder".to_string(),
    );
    assert_eq!(
        repo_folder_name(Some("https://host/owner/custom.git"), &mapping),
        "mapped-folder"
    );
    assert_eq!(
        repo_folder_name(
            Some("https://github.com/babalae/bettergi-scripts-list.git"),
            &BTreeMap::new()
        ),
        "bettergi-scripts-list"
    );
}

#[test]
fn script_repo_layout_preserves_legacy_paths() {
    let layout = script_repo_layout("C:/BetterGI", &script_config(), &BTreeMap::new());
    assert!(layout.repos_path.ends_with("Repos"));
    assert!(layout.repos_temp_path.ends_with("Repos/Temp"));
    assert!(layout.center_repo_path.ends_with(DEFAULT_REPO_FOLDER_NAME));
    assert!(layout
        .old_center_repo_path
        .ends_with(OLD_CENTER_REPO_FOLDER_NAME));
    assert!(layout
        .subscription_file_path
        .ends_with("bettergi-scripts-list.json"));
    assert!(layout
        .path_mapper
        .get(&ScriptRepoPathKind::Js)
        .unwrap()
        .ends_with("User/JsScript"));
}

#[test]
fn import_uri_decodes_base64_url_encoded_path_json() {
    let path_json = serde_json::to_string(&vec!["js/demo", "pathing/route"]).unwrap();
    let encoded = "WyJqcy9kZW1vIiwicGF0aGluZy9yb3V0ZSJd";
    let plan = parse_import_uri(&format!("bettergi://script?import={encoded}"), true)
        .unwrap()
        .unwrap();

    assert_eq!(plan.path_json, path_json);
    assert_eq!(plan.paths, vec!["js/demo", "pathing/route"]);
    assert!(plan.clear_clipboard_after_import);
    assert!(parse_import_uri("https://example.com", false)
        .unwrap()
        .is_none());
}

#[test]
fn import_plan_maps_repo_prefixes_to_user_destinations_and_subscriptions() {
    let mut children = BTreeMap::new();
    children.insert(
        "js".to_string(),
        vec!["alpha".to_string(), "beta".to_string()],
    );

    let plan = script_import_plan(
        "C:/BetterGI",
        "C:/BetterGI/Repos/bettergi-scripts-list/repo",
        &script_config(),
        &BTreeMap::new(),
        ["js", "combat/team", "unknown/path"],
        ["pathing/old", "js/alpha"],
        &children,
    );

    assert_eq!(
        plan.expanded_paths,
        vec!["combat/team", "js/alpha", "js/beta", "unknown/path"]
    );
    assert_eq!(plan.unknown_paths, vec!["unknown/path"]);
    assert_eq!(plan.targets.len(), 3);
    let js_target = plan
        .targets
        .iter()
        .find(|target| target.source_path == "js/alpha")
        .unwrap();
    assert!(js_target.destination_path.ends_with("User/JsScript/alpha"));
    assert!(js_target.preserves_saved_files);
    assert!(js_target.resolves_js_dependencies);
    assert_eq!(
        plan.merged_subscriptions,
        vec![
            "combat/team",
            "js",
            "js/alpha",
            "pathing/old",
            "unknown/path"
        ]
    );
}

#[test]
fn update_plan_respects_auto_update_switch_custom_repo_and_subscriptions() {
    let mut config = script_config();
    config.selected_channel_name = "自定义".to_string();
    let plan =
        script_repo_update_plan("C:/BetterGI", &config, &BTreeMap::new(), ["js/demo"], false);
    assert!(!plan.enabled);
    assert_eq!(plan.reason, Some("repo_url_unresolved"));

    config.custom_repo_url = "https://host/repo.git".to_string();
    let plan =
        script_repo_update_plan("C:/BetterGI", &config, &BTreeMap::new(), ["js/demo"], false);
    assert!(!plan.enabled);
    assert_eq!(plan.reason, Some("auto_update_disabled"));

    config.auto_update_subscribed_scripts = true;
    let plan =
        script_repo_update_plan("C:/BetterGI", &config, &BTreeMap::new(), ["js/demo"], false);
    assert!(plan.enabled);
    assert_eq!(plan.repo_folder_name, "repo");
    assert_eq!(plan.subscribed_paths, vec!["js/demo"]);

    let manual = script_repo_update_plan(
        "C:/BetterGI",
        &config,
        &BTreeMap::new(),
        Vec::<String>::new(),
        true,
    );
    assert!(!manual.enabled);
    assert_eq!(manual.reason, Some("no_subscribed_paths"));
}

#[test]
fn zip_import_plan_preserves_legacy_temp_and_marker_paths() {
    let plan = zip_import_plan("C:/BetterGI", "D:/repo.zip", Some("bad:name"));
    assert!(plan.temp_unzip_dir.ends_with("Repos/Temp/importZipFile"));
    assert_eq!(plan.target_folder_name, "bad_name");
    assert!(plan
        .repo_updated_json_path
        .ends_with("bad_name/repo_updated.json"));
    assert_eq!(plan.overlap_threshold, 0.5);
}

#[test]
fn git_update_plan_uses_release_branch_and_folder_mapping() {
    let mut mapping = BTreeMap::new();
    mapping.insert(
        "https://host/owner/custom.git".to_string(),
        "mapped".to_string(),
    );
    let plan = git_update_plan("C:/BetterGI", "https://host/owner/custom.git/", &mapping);

    assert_eq!(plan.repo_url, "https://host/owner/custom.git");
    assert_eq!(plan.branch, "release");
    assert_eq!(plan.repo_folder_name, "mapped");
    assert!(plan.repo_path.ends_with("Repos/mapped"));
    assert!(plan
        .folder_mapping_path
        .ends_with("Repos/repo_folder_mapping.json"));
}

#[test]
fn git_update_exec_clones_missing_repo_and_writes_marker_and_mapping() {
    let root = test_root("bgi-script-repo-git-clone");
    let repo_json = r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[]}]}"#;
    let mut runner = RecordingGitRunner::new("https://host/repo.git", repo_json);
    let plan = git_update_plan(&root, "https://host/repo.git", &BTreeMap::new());

    let result = execute_git_repo_update(&plan, &mut runner).unwrap();

    assert!(result.updated);
    assert!(result.cloned);
    assert!(result.repo_path.join("repo.json").exists());
    assert!(result.repo_updated_json_path.exists());
    assert!(runner
        .commands
        .iter()
        .any(|(_, args)| args.iter().any(|arg| arg == "--depth")
            && args.iter().any(|arg| arg == "--no-tags")
            && args.iter().any(|arg| arg.contains("refs/heads/release"))));
    let mapping = fs::read_to_string(root.join("Repos/repo_folder_mapping.json")).unwrap();
    assert!(mapping.contains("https://host/repo.git"));
    assert!(mapping.contains("repo"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn git_update_exec_skips_clone_when_release_sha_matches() {
    let root = test_root("bgi-script-repo-git-current");
    let repo_path = root.join("Repos/repo");
    fs::create_dir_all(repo_path.join(".git")).unwrap();
    fs::write(
        repo_path.join("repo.json"),
        r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[]}]}"#,
    )
    .unwrap();
    let mut runner = RecordingGitRunner::new(
        "https://host/repo.git",
        r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[]}]}"#,
    );
    runner.current_sha = runner.remote_sha.clone();
    let plan = git_update_plan(&root, "https://host/repo.git", &BTreeMap::new());

    let result = execute_git_repo_update(&plan, &mut runner).unwrap();

    assert!(!result.updated);
    assert!(!result.cloned);
    assert_eq!(result.current_commit, Some(runner.remote_sha.clone()));
    assert!(runner.commands.iter().any(|(_, args)| args
        == &git_args(&["ls-remote", "--heads", "https://host/repo.git", "release"])));
    assert!(!runner
        .commands
        .iter()
        .any(|(_, args)| args.iter().any(|arg| arg == "fetch")));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn git_update_exec_replaces_remote_changed_repo_when_overlap_is_high() {
    let root = test_root("bgi-script-repo-git-remote-changed");
    let repo_path = root.join("Repos/new-repo");
    fs::create_dir_all(repo_path.join(".git")).unwrap();
    fs::write(
        repo_path.join("repo.json"),
        r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[{"name":"demo","type":"directory","lastUpdated":"2024-01-01","children":[]}]}]}"#,
    )
    .unwrap();
    let new_repo_json = r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[{"name":"demo","type":"directory","lastUpdated":"2024-02-01","children":[]}]}]}"#;
    let mut runner = RecordingGitRunner::new("https://host/new-repo.git", new_repo_json);
    runner.origin_url = "https://host/old-repo.git".to_string();
    let plan = git_update_plan(&root, "https://host/new-repo.git", &BTreeMap::new());

    let result = execute_git_repo_update(&plan, &mut runner).unwrap();

    assert!(result.updated);
    assert!(result.remote_changed);
    assert!(!result.created_new_folder);
    assert_eq!(result.repo_folder_name, "new-repo");
    assert!(result.old_repo_overlap_ratio.unwrap() >= 0.5);
    assert!(result.marker_generated);
    let marker = fs::read_to_string(result.repo_updated_json_path).unwrap();
    let marker: serde_json::Value = serde_json::from_str(&marker).unwrap();
    assert_eq!(marker["indexes"][0]["hasUpdate"], true);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn git_checkout_path_exports_repo_subtree_to_destination() {
    let root = test_root("bgi-script-repo-git-checkout");
    let repo_path = root.join("Repos/repo");
    fs::create_dir_all(repo_path.join(".git")).unwrap();
    let mut runner = RecordingGitRunner::new("https://host/repo.git", "{}");
    runner.objects.insert(
        "HEAD:repo/js/demo".to_string(),
        ("tree".to_string(), String::new()),
    );
    runner.trees.insert(
        "HEAD:repo/js/demo".to_string(),
        "100644 blob aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\tmain.js\n040000 tree bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\tlib".to_string(),
    );
    runner.objects.insert(
        "HEAD:repo/js/demo/main.js".to_string(),
        ("blob".to_string(), "import './lib/helper.js';".to_string()),
    );
    runner.trees.insert(
        "HEAD:repo/js/demo/lib".to_string(),
        "100644 blob cccccccccccccccccccccccccccccccccccccccc\thelper.js".to_string(),
    );
    runner.objects.insert(
        "HEAD:repo/js/demo/lib/helper.js".to_string(),
        ("blob".to_string(), "export default 1;".to_string()),
    );

    let destination = root.join("User/JsScript/demo");
    let checkout = checkout_git_repo_path(&mut runner, &repo_path, "js/demo", &destination, true)
        .unwrap()
        .unwrap();

    assert!(checkout.is_directory);
    assert_eq!(checkout.git_tree_path, "repo/js/demo");
    assert_eq!(checkout.files_written.len(), 2);
    assert_eq!(
        fs::read_to_string(destination.join("main.js")).unwrap(),
        "import './lib/helper.js';"
    );
    assert_eq!(
        fs::read_to_string(destination.join("lib/helper.js")).unwrap(),
        "export default 1;"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn git_checkout_path_can_export_root_package_dependency() {
    let root = test_root("bgi-script-repo-git-root-checkout");
    let repo_path = root.join("Repos/repo");
    fs::create_dir_all(repo_path.join(".git")).unwrap();
    let mut runner = RecordingGitRunner::new("https://host/repo.git", "{}");
    runner.objects.insert(
        "HEAD:packages/lib/helper.js".to_string(),
        ("blob".to_string(), "export default 2;".to_string()),
    );

    let destination = root.join("User/JsScript/demo/packages/lib/helper.js");
    let checkout = checkout_git_repo_path(
        &mut runner,
        &repo_path,
        "packages/lib/helper.js",
        &destination,
        false,
    )
    .unwrap()
    .unwrap();

    assert!(!checkout.is_directory);
    assert_eq!(checkout.git_tree_path, "packages/lib/helper.js");
    assert_eq!(
        fs::read_to_string(destination).unwrap(),
        "export default 2;"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn git_checkout_path_preserves_binary_blob_bytes() {
    let root = test_root("bgi-script-repo-git-binary-checkout");
    let repo_path = root.join("Repos/repo");
    fs::create_dir_all(repo_path.join(".git")).unwrap();
    let mut runner = RecordingGitRunner::new("https://host/repo.git", "{}");
    let bytes = vec![0, 159, 146, 150, 255, 10, 13, 0];
    runner.binary_objects.insert(
        "HEAD:repo/js/demo/icon.png".to_string(),
        ("blob".to_string(), bytes.clone()),
    );

    let destination = root.join("User/JsScript/demo/icon.png");
    let checkout = checkout_git_repo_path(
        &mut runner,
        &repo_path,
        "js/demo/icon.png",
        &destination,
        true,
    )
    .unwrap()
    .unwrap();

    assert!(!checkout.is_directory);
    assert_eq!(fs::read(destination).unwrap(), bytes);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn git_repo_import_uses_git_tree_and_preserves_saved_files_and_packages() {
    let root = test_root("bgi-script-repo-git-import");
    let repo_path = root.join("Repos/repo");
    fs::create_dir_all(repo_path.join(".git")).unwrap();
    let mut runner = RecordingGitRunner::new("https://host/repo.git", "{}");
    runner.objects.insert(
        "HEAD:repo/js/demo".to_string(),
        ("tree".to_string(), String::new()),
    );
    runner.trees.insert(
        "HEAD:repo/js/demo".to_string(),
        "100644 blob aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\tmanifest.json\n100644 blob bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\tmain.js".to_string(),
    );
    runner.objects.insert(
        "HEAD:repo/js/demo/manifest.json".to_string(),
        (
            "blob".to_string(),
            r#"{"manifest_version":1,"name":"Demo","version":"1.0.0","main":"main.js","saved_files":["config.json"]}"#.to_string(),
        ),
    );
    runner.objects.insert(
        "HEAD:repo/js/demo/main.js".to_string(),
        (
            "blob".to_string(),
            r#"import helper from "packages/lib/helper.js"; console.log(helper);"#.to_string(),
        ),
    );
    runner.objects.insert(
        "HEAD:packages/lib/helper.js".to_string(),
        ("blob".to_string(), "export default 9;".to_string()),
    );

    let user_script_dir = root.join("User/JsScript/demo");
    fs::create_dir_all(&user_script_dir).unwrap();
    fs::write(user_script_dir.join("config.json"), r#"{"keep":true}"#).unwrap();
    fs::write(user_script_dir.join("old.js"), "old").unwrap();

    let plan = script_import_plan(
        &root,
        &repo_path,
        &script_config(),
        &BTreeMap::new(),
        ["js/demo"],
        Vec::<String>::new(),
        &BTreeMap::new(),
    );
    let result = execute_repo_import_with_git(&plan, Some(&mut runner)).unwrap();

    assert_eq!(result.imported_targets.len(), 1);
    assert_eq!(result.git_checkouts.len(), 1);
    assert_eq!(result.dependency_files_copied.len(), 1);
    assert_eq!(
        fs::read_to_string(user_script_dir.join("config.json")).unwrap(),
        r#"{"keep":true}"#
    );
    assert!(!user_script_dir.join("old.js").exists());
    assert!(user_script_dir.join("main.js").exists());
    assert_eq!(
        fs::read_to_string(user_script_dir.join("packages/lib/helper.js")).unwrap(),
        "export default 9;"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn subscription_path_helpers_normalize_dedupe_and_split() {
    assert_eq!(
        normalize_subscription_paths([" js/demo ", "js\\demo", "./pathing/a", "../bad"]),
        vec!["bad", "js/demo", "pathing/a"]
    );
    let (first, rest) = first_folder_and_remaining_path("js/demo/main.js");
    assert_eq!(first, "js");
    assert_eq!(rest, PathBuf::from("demo/main.js"));
    assert_eq!(
        merge_subscription_paths(["js/a"], ["js/a", "pathing/b"]),
        vec!["js/a", "pathing/b"]
    );
}

#[test]
fn subscription_file_read_write_normalizes_and_deletes_empty_lists() {
    let root = test_root("bgi-script-repo-subscriptions");
    let path = root.join("User/Subscriptions/bettergi-scripts-list.json");

    write_subscription_file(&path, &["js/demo".to_string(), "js\\demo".to_string()]).unwrap();
    assert_eq!(read_subscription_file(&path).unwrap(), vec!["js/demo"]);

    write_subscription_file(&path, &[]).unwrap();
    assert!(!path.exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn file_repo_import_copies_targets_preserves_saved_files_and_copies_packages() {
    let root = test_root("bgi-script-repo-import-exec");
    let repo_root = root.join("Repos/bettergi-scripts-list/repo");
    let script_repo_dir = repo_root.join("js/demo");
    fs::create_dir_all(&script_repo_dir).unwrap();
    fs::create_dir_all(repo_root.join("packages/lib")).unwrap();
    fs::write(
        script_repo_dir.join("manifest.json"),
        r#"{"manifest_version":1,"name":"Demo","version":"1.0.0","main":"main.js","saved_files":["config.json"]}"#,
    )
    .unwrap();
    fs::write(
        script_repo_dir.join("main.js"),
        r#"import helper from "packages/lib/helper.js"; console.log(helper);"#,
    )
    .unwrap();
    fs::write(
        repo_root.join("packages/lib/helper.js"),
        "export default 1;",
    )
    .unwrap();

    let user_script_dir = root.join("User/JsScript/demo");
    fs::create_dir_all(&user_script_dir).unwrap();
    fs::write(user_script_dir.join("config.json"), r#"{"keep":true}"#).unwrap();
    fs::write(user_script_dir.join("old.js"), "old").unwrap();

    let plan = script_import_plan(
        &root,
        &repo_root,
        &script_config(),
        &BTreeMap::new(),
        ["js/demo"],
        Vec::<String>::new(),
        &BTreeMap::new(),
    );
    let result = execute_file_repo_import(&plan).unwrap();

    assert_eq!(result.imported_targets.len(), 1);
    assert_eq!(result.subscriptions, vec!["js/demo"]);
    assert!(root
        .join("User/Subscriptions/bettergi-scripts-list.json")
        .exists());
    assert_eq!(
        fs::read_to_string(user_script_dir.join("config.json")).unwrap(),
        r#"{"keep":true}"#
    );
    assert!(!user_script_dir.join("old.js").exists());
    assert!(user_script_dir.join("main.js").exists());
    assert!(user_script_dir.join("packages/lib/helper.js").exists());
    assert_eq!(result.dependency_files_copied.len(), 1);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn repo_overlap_ratio_uses_directory_overlap_coefficient() {
    let old = r#"{
      "indexes": [
        {"name":"js","type":"directory","children":[
          {"name":"demo","type":"directory","children":[]}
        ]},
        {"name":"pathing","type":"directory","children":[]}
      ]
    }"#;
    let new = r#"{
      "indexes": [
        {"name":"js","type":"directory","children":[
          {"name":"demo","type":"directory","children":[]},
          {"name":"new","type":"directory","children":[]}
        ]},
        {"name":"combat","type":"directory","children":[]}
      ]
    }"#;

    assert_eq!(
        repo_directory_paths(old).unwrap(),
        vec!["js", "js/demo", "pathing"]
    );
    assert!((calculate_repo_overlap_ratio(old, new) - (2.0 / 3.0)).abs() < f64::EPSILON);
    assert_eq!(calculate_repo_overlap_ratio("not-json", new), -1.0);
}

#[test]
fn repo_update_markers_preserve_old_flags_and_mark_newer_or_new_nodes() {
    let old = r#"{
      "indexes": [{
        "name": "js",
        "type": "directory",
        "lastUpdated": "2024-01-01 00:00:00",
        "children": [
          {"name":"demo","type":"directory","lastUpdated":"2024-01-02 00:00:00","children":[]},
          {"name":"flagged","type":"directory","hasUpdate":"true","lastUpdated":"2024-01-01 00:00:00","children":[]}
        ]
      }]
    }"#;
    let new = r#"{
      "indexes": [{
        "name": "js",
        "type": "directory",
        "lastUpdated": "2024-01-01 00:00:00",
        "children": [
          {"name":"demo","type":"directory","lastUpdated":"2024-02-01 00:00:00","children":[]},
          {"name":"flagged","type":"directory","lastUpdated":"2024-01-01 00:00:00","children":[]},
          {"name":"fresh","type":"directory","lastUpdated":"2024-03-01 00:00:00","children":[]}
        ]
      }]
    }"#;

    let marked = add_update_markers_to_new_repo(old, new);
    let value: serde_json::Value = serde_json::from_str(&marked).unwrap();
    let js = &value["indexes"][0];
    assert_eq!(js["hasUpdate"], true);
    assert_eq!(js["lastUpdated"], "2024-03-01 00:00:00");
    assert_eq!(js["children"][0]["hasUpdate"], true);
    assert_eq!(js["children"][1]["hasUpdate"], true);
    assert_eq!(js["children"][2]["hasUpdate"], true);
}

#[test]
fn zip_repo_import_extracts_matches_existing_repo_and_generates_marker() {
    let root = test_root("bgi-script-repo-zip-exec");
    let existing_repo = root.join("Repos/existing");
    fs::create_dir_all(existing_repo.join("repo/js/demo")).unwrap();
    fs::write(
        existing_repo.join("repo.json"),
        r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[{"name":"demo","type":"directory","lastUpdated":"2024-01-01","children":[]}]}]}"#,
    )
    .unwrap();

    let source_repo = root.join("source/repo");
    fs::create_dir_all(source_repo.join("js/demo")).unwrap();
    fs::write(source_repo.join("js/demo/main.js"), "console.log('demo');").unwrap();
    fs::write(
        source_repo.join("repo.json"),
        r#"{"indexes":[{"name":"js","type":"directory","lastUpdated":"2024-01-01","children":[{"name":"demo","type":"directory","lastUpdated":"2024-02-01","children":[]}]}]}"#,
    )
    .unwrap();
    let zip_path = root.join("repo.zip");
    create_test_zip(&zip_path, &source_repo, "packed").unwrap();

    let plan = zip_import_plan(&root, &zip_path, Some("bettergi-scripts-list"));
    let result = execute_zip_repo_import(&plan).unwrap();

    assert_eq!(result.target_folder_name, "existing");
    assert!(result.marker_generated);
    assert!(result
        .best_overlap_ratio
        .map(|ratio| ratio >= 0.5)
        .unwrap_or(false));
    assert!(result.target_path.join("js/demo/main.js").exists());
    assert!(result.repo_updated_json_path.exists());
    assert!(!root.join("Repos/Temp").exists());

    let marker = fs::read_to_string(result.repo_updated_json_path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&marker).unwrap();
    assert_eq!(value["indexes"][0]["hasUpdate"], true);
    assert_eq!(value["indexes"][0]["children"][0]["hasUpdate"], true);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn file_repo_import_reports_missing_sources() {
    let root = test_root("bgi-script-repo-import-missing");
    let repo_root = root.join("Repos/bettergi-scripts-list/repo");
    fs::create_dir_all(&repo_root).unwrap();
    let plan = script_import_plan(
        &root,
        &repo_root,
        &script_config(),
        &BTreeMap::new(),
        ["pathing/missing"],
        Vec::<String>::new(),
        &BTreeMap::new(),
    );
    let error = execute_file_repo_import(&plan).unwrap_err();
    assert!(matches!(error, ScriptRepoError::MissingSource(_)));
    fs::remove_dir_all(root).unwrap();
}

fn create_test_zip(zip_path: &Path, source_root: &Path, archive_root: &str) -> Result<()> {
    let file = File::create(zip_path).map_err(|source| ScriptRepoError::Io {
        path: zip_path.to_path_buf(),
        source,
    })?;
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    add_zip_directory(&mut writer, source_root, source_root, archive_root, options)?;
    writer.finish().map_err(|source| ScriptRepoError::Zip {
        path: zip_path.to_path_buf(),
        source,
    })?;
    Ok(())
}

fn add_zip_directory(
    writer: &mut zip::ZipWriter<File>,
    source_root: &Path,
    current: &Path,
    archive_root: &str,
    options: zip::write::SimpleFileOptions,
) -> Result<()> {
    for entry in fs::read_dir(current).map_err(|source| ScriptRepoError::Io {
        path: current.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| ScriptRepoError::Io {
            path: current.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let relative = path.strip_prefix(source_root).unwrap();
        let archive_name =
            normalize_repo_path(&format!("{archive_root}/{}", relative.to_string_lossy()));
        if path.is_dir() {
            writer
                .add_directory(format!("{archive_name}/"), options)
                .map_err(|source| ScriptRepoError::Zip {
                    path: path.clone(),
                    source,
                })?;
            add_zip_directory(writer, source_root, &path, archive_root, options)?;
        } else {
            writer
                .start_file(&archive_name, options)
                .map_err(|source| ScriptRepoError::Zip {
                    path: path.clone(),
                    source,
                })?;
            let bytes = fs::read(&path).map_err(|source| ScriptRepoError::Io {
                path: path.clone(),
                source,
            })?;
            writer
                .write_all(&bytes)
                .map_err(|source| ScriptRepoError::Io {
                    path: path.clone(),
                    source,
                })?;
        }
    }
    Ok(())
}
