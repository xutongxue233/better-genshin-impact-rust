use super::*;
use crate::{ScriptRepoGitCommandOutput, ScriptRepoGitRunner};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
struct BridgeGitRunner {
    objects: BTreeMap<String, (String, Vec<u8>)>,
    trees: BTreeMap<String, String>,
}

impl ScriptRepoGitRunner for BridgeGitRunner {
    fn run_git(
        &mut self,
        _cwd: Option<&Path>,
        args: &[String],
    ) -> Result<ScriptRepoGitCommandOutput, ScriptRepoError> {
        let bytes = match args
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["cat-file", "-t", object] => self
                .objects
                .get(*object)
                .map(|(kind, _)| kind.clone().into_bytes())
                .ok_or_else(|| ScriptRepoError::MissingSource(PathBuf::from(object)))?,
            ["ls-tree", object] => self
                .trees
                .get(*object)
                .cloned()
                .map(String::into_bytes)
                .ok_or_else(|| ScriptRepoError::MissingSource(PathBuf::from(object)))?,
            ["show", object] => self
                .objects
                .get(*object)
                .map(|(_, bytes)| bytes.clone())
                .ok_or_else(|| ScriptRepoError::MissingSource(PathBuf::from(object)))?,
            _ => Vec::new(),
        };
        Ok(ScriptRepoGitCommandOutput {
            stdout: String::from_utf8_lossy(&bytes).trim().to_string(),
            stderr: String::new(),
            stdout_bytes: bytes,
        })
    }
}

fn test_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    fs::remove_dir_all(&root).unwrap_or(());
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn bridge_paths_prefer_repo_updated_json_and_subscription_file() {
    let root = test_root("bgi-repo-bridge-paths");
    let repo = root.join("Repos/repo");
    fs::create_dir_all(&repo).unwrap();
    fs::write(repo.join("repo.json"), "{}").unwrap();
    fs::write(repo.join("repo_updated.json"), "{\"updated\":true}").unwrap();

    let paths = script_repo_bridge_paths(&root, &repo, None).unwrap();

    assert!(paths.repo_json_path.ends_with("repo_updated.json"));
    assert!(paths.subscription_file_path.ends_with("repo.json"));
    assert!(paths.user_config_path.ends_with("User/config.json"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn bridge_file_reads_text_and_image_from_file_repo_with_path_guard() {
    let root = test_root("bgi-repo-bridge-file");
    let repo_root = root.join("Repos/repo/repo/js/demo");
    fs::create_dir_all(&repo_root).unwrap();
    fs::write(repo_root.join("main.js"), "console.log('ok');").unwrap();
    fs::write(repo_root.join("icon.png"), [1_u8, 2, 3, 4]).unwrap();

    let text = read_repo_bridge_file(root.join("Repos/repo"), "js%2Fdemo%2Fmain.js")
        .unwrap()
        .unwrap();
    let image = read_repo_bridge_file(root.join("Repos/repo"), "js/demo/icon.png")
        .unwrap()
        .unwrap();

    assert_eq!(text.kind, ScriptRepoBridgeFileKind::Text);
    assert_eq!(text.content, "console.log('ok');");
    assert_eq!(image.kind, ScriptRepoBridgeFileKind::ImageBase64);
    assert_eq!(image.content, "AQIDBA==");
    assert!(
        read_repo_bridge_file(root.join("Repos/repo"), "../config.json")
            .unwrap()
            .is_none()
    );
    assert!(
        read_repo_bridge_file(root.join("Repos/repo"), "js/demo/main.exe")
            .unwrap()
            .is_none()
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn bridge_file_reads_git_repo_blob() {
    let root = test_root("bgi-repo-bridge-git-file");
    let repo = root.join("Repos/repo");
    fs::create_dir_all(repo.join(".git")).unwrap();
    let mut runner = BridgeGitRunner::default();
    runner.objects.insert(
        "HEAD:repo/js/demo/icon.png".to_string(),
        ("blob".to_string(), vec![0, 255, 1]),
    );

    let response = read_repo_bridge_file_with_git(&repo, "js/demo/icon.png", Some(&mut runner))
        .unwrap()
        .unwrap();

    assert_eq!(response.kind, ScriptRepoBridgeFileKind::ImageBase64);
    assert_eq!(response.content, "AP8B");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn bridge_mark_path_resets_update_flags_recursively() {
    let root = test_root("bgi-repo-bridge-mark");
    let repo_json = root.join("repo_updated.json");
    fs::write(
        &repo_json,
        r#"{"indexes":[{"name":"js","hasUpdate":true,"children":[{"name":"demo","hasUpdate":true,"children":[{"name":"main","hasUpdate":true}]}]},{"name":"pathing","hasUpdate":true}]}"#,
    )
    .unwrap();

    assert!(mark_repo_bridge_path_updated(&repo_json, "js/demo").unwrap());
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&repo_json).unwrap()).unwrap();

    assert_eq!(value["indexes"][0]["hasUpdate"], true);
    assert_eq!(value["indexes"][0]["children"][0]["hasUpdate"], false);
    assert_eq!(
        value["indexes"][0]["children"][0]["children"][0]["hasUpdate"],
        false
    );
    assert_eq!(value["indexes"][1]["hasUpdate"], true);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn bridge_index_nodes_flatten_repo_json_for_import_ui() {
    let nodes = repo_bridge_index_nodes_from_json(
        r#"{"indexes":[{"name":"js","type":"directory","hasUpdate":true,"children":[{"name":"demo","type":"directory","lastUpdated":"2024-01-01","children":[{"name":"main.js","type":"file"}]}]},{"name":"misc","type":"directory","children":[]}]}"#,
    )
    .unwrap();

    assert_eq!(nodes.len(), 4);
    assert_eq!(nodes[0].path, "js");
    assert!(nodes[0].importable);
    assert_eq!(nodes[1].path, "js/demo");
    assert_eq!(nodes[1].depth, 1);
    assert_eq!(nodes[1].last_updated.as_deref(), Some("2024-01-01"));
    assert!(nodes[1].importable);
    assert_eq!(nodes[2].path, "js/demo/main.js");
    assert!(!nodes[2].importable);
    assert_eq!(nodes[3].path, "misc");
    assert!(!nodes[3].importable);
}

#[test]
fn bridge_clear_update_copies_original_repo_json() {
    let root = test_root("bgi-repo-bridge-clear");
    let repo = root.join("Repos/repo");
    fs::create_dir_all(repo.join("nested")).unwrap();
    fs::write(repo.join("nested/repo.json"), "{\"original\":true}").unwrap();
    fs::write(repo.join("repo_updated.json"), "{\"updated\":true}").unwrap();

    let target = clear_repo_bridge_update(&repo).unwrap();

    assert_eq!(target, repo.join("repo_updated.json"));
    assert_eq!(
        fs::read_to_string(repo.join("repo_updated.json")).unwrap(),
        "{\"original\":true}"
    );

    fs::remove_dir_all(root).unwrap();
}
