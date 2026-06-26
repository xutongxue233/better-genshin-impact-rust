use super::*;

pub(crate) fn script_module_load(
    root: PathBuf,
    specifier: String,
    referrer: Option<PathBuf>,
    json: bool,
) -> Result<()> {
    let mut loader =
        ScriptModuleLoader::new(&root, vec![PathBuf::from("."), PathBuf::from("./packages")])?;
    let first = loader
        .load_js_module(&specifier, referrer.as_deref())
        .with_context(|| {
            format!(
                "failed to load module {specifier:?} from root {}",
                root.display()
            )
        })?;
    let second = loader.load_js_module(&specifier, referrer.as_deref())?;
    let payload = serde_json::json!({
        "resolution": first.resolution,
        "code_bytes": first.code.len(),
        "original_code_bytes": first.original_code.len(),
        "import_rewrites": first.import_rewrites,
        "cache_hit_on_first_load": first.cache_hit,
        "cache_hit_on_second_load": second.cache_hit,
        "cache_len": loader.cache_len(),
    });
    if json {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("root: {}", root.display());
        println!("specifier: {specifier}");
        println!("resolved: {}", first.resolution.resolved_path.display());
        println!("kind: {:?}", first.resolution.kind);
        println!("code_bytes: {}", first.code.len());
        println!("import_rewrites: {}", first.import_rewrites.len());
        println!("cache_hit_on_second_load: {}", second.cache_hit);
    }
    Ok(())
}

pub(crate) fn script_prepare_js(scripts_root: PathBuf, folder: String, json: bool) -> Result<()> {
    let project = ScriptGroupProject {
        name: folder.clone(),
        folder_name: folder.clone(),
        project_type: ScriptProjectType::Javascript,
        ..ScriptGroupProject::default()
    };
    let manifest =
        bgi_script::Manifest::read_from(scripts_root.join(&folder).join("manifest.json"))
            .with_context(|| {
                format!(
                    "failed to read manifest for script project {}",
                    scripts_root.join(&folder).display()
                )
            })?;
    let step = bgi_script::ScriptExecutionStep::from_group_project(
        &project,
        Some(&manifest),
        &scripts_root,
    )?;
    let prepared = PreparedScriptExecution::prepare_javascript(&step, &scripts_root)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&prepared)?);
    } else {
        println!("project: {}", prepared.step.folder_name);
        println!("mode: {:?}", prepared.execution_mode);
        println!(
            "main: {}",
            prepared.main_module.resolution.resolved_path.display()
        );
        println!("code_bytes: {}", prepared.main_module.code.len());
        println!("imports: {}", prepared.main_module.import_rewrites.len());
        println!(
            "host_root: {}",
            prepared.host_runtime_config.script_root.display()
        );
    }
    Ok(())
}

pub(crate) fn script_execute_js(
    scripts_root: PathBuf,
    folder: String,
    settings_json: Option<String>,
    json: bool,
) -> Result<()> {
    let settings = settings_json
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .context("failed to parse --settings-json as JSON")?;
    let outcome = bgi_script_engine::execute_javascript_project(scripts_root, folder, settings)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
    } else {
        println!("project: {}", outcome.folder_name);
        println!("runtime: {:?}", outcome.runtime);
        println!("mode: {:?}", outcome.execution_mode);
        println!("main: {}", outcome.main_script_path.display());
        println!("result: {}", outcome.result_display);
        println!("console_lines: {}", outcome.console.len());
        println!("log_records: {}", outcome.logs.len());
        println!("host_calls: {}", outcome.host_calls.len());
    }
    Ok(())
}

pub(crate) fn script_execute_group(app_root: PathBuf, group: String, json: bool) -> Result<()> {
    let group_path = script_group_file_path(app_root.join("User").join("ScriptGroup"), &group);
    let group = read_script_group_file(&group_path)
        .with_context(|| format!("failed to read script group {}", group_path.display()))?;
    let outcome = bgi_script_engine::execute_script_group(&app_root, &group);
    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
    } else {
        println!("group: {}", outcome.group_name);
        println!("projects: {}", outcome.requested_projects);
        println!("attempted_steps: {}", outcome.attempted_steps);
        println!("completed_steps: {}", outcome.completed_steps);
        println!("planned_steps: {}", outcome.planned_steps);
        println!("failed_steps: {}", outcome.failed_steps);
        println!("skipped_steps: {}", outcome.skipped_steps);
        for step in outcome.steps {
            println!(
                "{:?} {:<10} {:<10} {}/{} {}",
                step.status,
                format!("{:?}", step.project_type),
                step.folder_name,
                step.run_iteration,
                step.run_count,
                step.error.unwrap_or(step.name)
            );
        }
    }
    Ok(())
}
