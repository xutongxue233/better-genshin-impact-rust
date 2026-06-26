use super::*;

#[test]
fn execution_mode_matches_legacy_module_heuristic() {
    let classic = Manifest {
        library: Vec::new(),
        ..Manifest::default()
    };
    let module_by_library = Manifest {
        library: vec!["./lib".to_string()],
        ..Manifest::default()
    };

    assert_eq!(
        execution_mode_for_code(&classic, "log.info('ok')"),
        ScriptCodeExecutionMode::ClassicScript
    );
    assert_eq!(
        execution_mode_for_code(&classic, "import x from './x.js'"),
        ScriptCodeExecutionMode::StandardModule
    );
    assert_eq!(
        execution_mode_for_code(&module_by_library, "log.info('ok')"),
        ScriptCodeExecutionMode::StandardModule
    );
}

#[test]
fn search_paths_include_manifest_library_dot_and_packages() {
    let manifest = Manifest {
        library: vec!["./lib".to_string(), ".".to_string()],
        ..Manifest::default()
    };
    let paths = normalized_search_paths(Path::new("scripts/sample"), &manifest);

    assert_eq!(paths.len(), 3);
    assert!(paths
        .iter()
        .any(|path| comparable_path_key(path).ends_with("/scripts/sample/lib")));
    assert!(paths
        .iter()
        .any(|path| comparable_path_key(path).ends_with("/scripts/sample/packages")));
}

#[test]
fn import_rewrite_marks_images_and_text_resources() {
    let rewrites = import_rewrites_for_code(
        Path::new("scripts/sample"),
        Path::new("scripts/sample/main.js"),
        "import img from '../../../packages/icon.png'\nimport text from './data.txt'\nimport data, { meta } from './data.json'\nimport { x } from './keep.js'",
    );

    assert_eq!(rewrites.len(), 3);
    assert_eq!(rewrites[0].normalized_specifier, "packages/icon.png");
    assert_eq!(rewrites[0].resource_kind, ImportedResourceKind::Image);
    assert_eq!(rewrites[1].resource_kind, ImportedResourceKind::Text);
    assert_eq!(rewrites[2].import_binding, "data, { meta }");
    assert_eq!(rewrites[2].resource_kind, ImportedResourceKind::Text);
    assert!(rewrites[2]
        .replacement
        .as_ref()
        .unwrap()
        .contains("const data = file.ReadTextSync"));
    assert!(rewrites[0]
        .replacement
        .as_ref()
        .unwrap()
        .contains("file.ReadImageMatSync"));
}

#[test]
fn module_loader_resolves_packages_alias_relative_modules_and_caches_js() {
    let root = test_root("bgi-module-loader");
    fs::create_dir_all(root.join("packages/ui")).unwrap();
    fs::create_dir_all(root.join("lib")).unwrap();
    fs::write(
        root.join("main.js"),
        "import helper from './lib/helper'\nimport icon from '../../../packages/icon.png'\nexport default helper;",
    )
    .unwrap();
    fs::write(root.join("lib/helper.js"), "export const value = 1;").unwrap();
    fs::write(
        root.join("packages/ui/button.js"),
        "export const button = true;",
    )
    .unwrap();
    fs::write(root.join("packages/icon.png"), "not real png").unwrap();

    let mut loader =
        ScriptModuleLoader::new(&root, vec![PathBuf::from("."), PathBuf::from("./packages")])
            .unwrap();
    let package_module = loader.load_js_module("packages/ui/button", None).unwrap();
    assert_eq!(
        package_module.resolution.kind,
        ModuleResolutionKind::PackagesAlias
    );
    assert!(!package_module.cache_hit);

    let cached_package = loader.load_js_module("packages/ui/button", None).unwrap();
    assert!(cached_package.cache_hit);
    assert_eq!(loader.cache_len(), 1);

    let relative_module = loader
        .load_js_module("./lib/helper", Some(&root.join("main.js")))
        .unwrap();
    assert_eq!(
        relative_module.resolution.kind,
        ModuleResolutionKind::RelativeToReferrer
    );
    assert!(relative_module
        .resolution
        .resolved_path
        .ends_with("helper.js"));

    let rewritten_main = loader.load_js_module("main.js", None).unwrap();
    assert!(rewritten_main
        .code
        .contains("file.ReadImageMatSync('packages/icon.png')"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn module_loader_rejects_root_escape_and_non_js_execution() {
    let root = test_root("bgi-module-loader-policy");
    fs::write(root.join("data.txt"), "hello").unwrap();
    fs::write(root.join("main.js"), "export default 1;").unwrap();

    let mut loader = ScriptModuleLoader::new(&root, vec![PathBuf::from(".")]).unwrap();
    assert!(matches!(
        loader.load_js_module("data.txt", None).unwrap_err(),
        ScriptProjectError::UnsupportedModuleExtension(path) if path.ends_with("data.txt")
    ));
    assert!(matches!(
        loader.resolve("../outside.js", None).unwrap_err(),
        ScriptProjectError::Policy(ScriptHostPolicyError::PathTraversal { .. })
            | ScriptProjectError::ModuleNotFound { .. }
    ));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rewrite_script_code_preserves_named_js_imports_and_rewrites_default_resources() {
    let root = Path::new("scripts/sample");
    let current = root.join("main.js");
    let code = "\
import { run } from './lib.js'
import img from './icon.png'
import text from './notes.txt'
import config, { schema } from './config.json'
import * as template from './template.txt'
import { broken } from './broken.txt'";

    let rewritten = rewrite_script_code(root, &current, code);

    assert!(rewritten.contains("import { run } from './lib.js'"));
    assert!(rewritten.contains("const img = file.ReadImageMatSync('icon.png');"));
    assert!(rewritten.contains("const text = file.ReadTextSync('notes.txt');"));
    assert!(rewritten.contains("const config = file.ReadTextSync('config.json');"));
    assert!(rewritten.contains("const template = file.ReadTextSync('template.txt');"));
    assert!(rewritten.contains("import { broken } from './broken.txt'"));
}

#[test]
fn rewrite_script_code_does_not_cross_side_effect_imports() {
    let root = Path::new("scripts/sample");
    let current = root.join("main.js");
    let code = "\
import './setup.js';
import data from './data.json';";

    let rewritten = rewrite_script_code(root, &current, code);

    assert!(rewritten.contains("import './setup.js';"));
    assert!(rewritten.contains("const data = file.ReadTextSync('data.json');"));
}

#[test]
fn rewrite_script_code_replaces_import_meta_paths_with_script_relative_strings() {
    let root = Path::new("scripts/sample");
    let current = root.join("lib").join("main.js");
    let code = "export const here = import.meta.dirname + '/' + import.meta.url;";

    let rewritten = rewrite_script_code(root, &current, code);

    assert!(rewritten.contains(r#""lib" + '/' + "lib/main.js""#));
    assert!(!rewritten.contains("import.meta"));
}

fn test_root(prefix: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "{prefix}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}
