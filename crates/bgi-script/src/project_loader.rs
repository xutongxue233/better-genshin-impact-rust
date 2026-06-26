use super::{Result, ScriptProjectError};
use crate::manifest::Manifest;
use crate::policy::ScriptFilePolicy;
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ScriptCodeExecutionMode {
    ClassicScript,
    StandardModule,
}

pub fn execution_mode_for_code(manifest: &Manifest, code: &str) -> ScriptCodeExecutionMode {
    if !manifest.library.is_empty() || code.contains("import ") || code.contains("export ") {
        ScriptCodeExecutionMode::StandardModule
    } else {
        ScriptCodeExecutionMode::ClassicScript
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ImportedResourceKind {
    JavaScript,
    Image,
    Text,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImportRewrite {
    pub import_binding: String,
    pub specifier: String,
    pub normalized_specifier: String,
    pub resource_kind: ImportedResourceKind,
    pub replacement: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModuleLoaderPlan {
    pub script_root: PathBuf,
    pub search_paths: Vec<PathBuf>,
    pub main_script_path: PathBuf,
    pub execution_mode: ScriptCodeExecutionMode,
    pub import_rewrites: Vec<ImportRewrite>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ModuleResolutionKind {
    PackagesAlias,
    RelativeToReferrer,
    SearchPath,
    ScriptRoot,
    StrippedRootFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModuleResolution {
    pub specifier: String,
    pub normalized_specifier: String,
    pub referrer: Option<PathBuf>,
    pub resolved_path: PathBuf,
    pub kind: ModuleResolutionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoadedScriptModule {
    pub resolution: ModuleResolution,
    pub original_code: String,
    pub code: String,
    pub import_rewrites: Vec<ImportRewrite>,
    pub cache_hit: bool,
}

#[derive(Debug, Clone)]
pub struct ScriptModuleLoader {
    script_root: PathBuf,
    search_paths: Vec<PathBuf>,
    file_policy: ScriptFilePolicy,
    cache: HashMap<PathBuf, LoadedScriptModule>,
}

impl ModuleLoaderPlan {
    pub fn from_project(
        script_root: impl Into<PathBuf>,
        search_paths: Vec<PathBuf>,
        main_script_path: impl Into<PathBuf>,
        code: &str,
    ) -> Self {
        let script_root = script_root.into();
        let main_script_path = main_script_path.into();
        let import_rewrites = import_rewrites_for_code(&script_root, &main_script_path, code);
        let execution_mode = if code.contains("import ") || code.contains("export ") {
            ScriptCodeExecutionMode::StandardModule
        } else {
            ScriptCodeExecutionMode::ClassicScript
        };

        Self {
            script_root,
            search_paths,
            main_script_path,
            execution_mode,
            import_rewrites,
        }
    }
}

impl ScriptModuleLoader {
    pub fn new(script_root: impl Into<PathBuf>, search_paths: Vec<PathBuf>) -> Result<Self> {
        let file_policy = ScriptFilePolicy::new(script_root.into());
        let script_root = file_policy.normalize_path(".")?;
        let mut normalized_search_paths = Vec::new();
        for path in search_paths {
            normalized_search_paths.push(file_policy.normalize_path(&path.to_string_lossy())?);
        }

        Ok(Self {
            script_root,
            search_paths: normalized_search_paths,
            file_policy,
            cache: HashMap::new(),
        })
    }

    pub fn script_root(&self) -> &Path {
        &self.script_root
    }

    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }

    pub fn resolve(&self, specifier: &str, referrer: Option<&Path>) -> Result<ModuleResolution> {
        let normalized_specifier = normalize_package_specifier(specifier);
        let normalized_referrer = referrer
            .map(|path| self.file_policy.normalize_path(&path.to_string_lossy()))
            .transpose()?;

        if normalized_specifier.starts_with("packages/") {
            if let Some(path) = self.probe_file(self.script_root.join(&normalized_specifier))? {
                return Ok(ModuleResolution {
                    specifier: specifier.to_string(),
                    normalized_specifier,
                    referrer: normalized_referrer,
                    resolved_path: path,
                    kind: ModuleResolutionKind::PackagesAlias,
                });
            }
        }

        if normalized_specifier.starts_with('.') {
            if let Some(referrer) = &normalized_referrer {
                if let Some(parent) = referrer.parent() {
                    if let Some(path) = self.probe_file(parent.join(&normalized_specifier))? {
                        return Ok(ModuleResolution {
                            specifier: specifier.to_string(),
                            normalized_specifier,
                            referrer: normalized_referrer,
                            resolved_path: path,
                            kind: ModuleResolutionKind::RelativeToReferrer,
                        });
                    }
                }
            }
        }

        for search_path in &self.search_paths {
            if let Some(path) = self.probe_file(search_path.join(&normalized_specifier))? {
                return Ok(ModuleResolution {
                    specifier: specifier.to_string(),
                    normalized_specifier,
                    referrer: normalized_referrer,
                    resolved_path: path,
                    kind: ModuleResolutionKind::SearchPath,
                });
            }
        }

        if let Some(path) = self.probe_file(self.script_root.join(&normalized_specifier))? {
            return Ok(ModuleResolution {
                specifier: specifier.to_string(),
                normalized_specifier,
                referrer: normalized_referrer,
                resolved_path: path,
                kind: ModuleResolutionKind::ScriptRoot,
            });
        }

        if let Some(stripped) = strip_leading_relative_segments(&normalized_specifier) {
            if let Some(path) = self.probe_file(self.script_root.join(&stripped))? {
                return Ok(ModuleResolution {
                    specifier: specifier.to_string(),
                    normalized_specifier: stripped,
                    referrer: normalized_referrer,
                    resolved_path: path,
                    kind: ModuleResolutionKind::StrippedRootFallback,
                });
            }
        }

        Err(ScriptProjectError::ModuleNotFound {
            specifier: specifier.to_string(),
            referrer: normalized_referrer,
        })
    }

    pub fn load_js_module(
        &mut self,
        specifier: &str,
        referrer: Option<&Path>,
    ) -> Result<LoadedScriptModule> {
        let resolution = self.resolve(specifier, referrer)?;
        if !has_js_extension(&resolution.resolved_path) {
            return Err(ScriptProjectError::UnsupportedModuleExtension(
                resolution.resolved_path,
            ));
        }

        if let Some(cached) = self.cache.get(&resolution.resolved_path) {
            let mut cached = cached.clone();
            cached.cache_hit = true;
            return Ok(cached);
        }

        let original_code = fs::read_to_string(&resolution.resolved_path).map_err(|source| {
            ScriptProjectError::Io {
                path: resolution.resolved_path.clone(),
                source,
            }
        })?;
        let code =
            rewrite_script_code(&self.script_root, &resolution.resolved_path, &original_code);
        let import_rewrites =
            import_rewrites_for_code(&self.script_root, &resolution.resolved_path, &original_code);
        let module = LoadedScriptModule {
            resolution,
            original_code,
            code,
            import_rewrites,
            cache_hit: false,
        };
        self.cache
            .insert(module.resolution.resolved_path.clone(), module.clone());
        Ok(module)
    }

    fn probe_file(&self, path: PathBuf) -> Result<Option<PathBuf>> {
        let normalized = self.file_policy.normalize_path(&path.to_string_lossy())?;
        if normalized.is_file() {
            return Ok(Some(normalized));
        }

        let with_js_extension = PathBuf::from(format!("{}.js", normalized.to_string_lossy()));
        let normalized_js = self
            .file_policy
            .normalize_path(&with_js_extension.to_string_lossy())?;
        if normalized_js.is_file() {
            return Ok(Some(normalized_js));
        }

        Ok(None)
    }
}

pub fn normalized_search_paths(project_path: &Path, manifest: &Manifest) -> Vec<PathBuf> {
    let file_policy = ScriptFilePolicy::new(project_path);
    let mut seen = BTreeSet::new();
    manifest
        .library
        .iter()
        .map(String::as_str)
        .chain([".", "./packages"])
        .filter_map(|path| file_policy.normalize_path(path).ok())
        .filter(|path| seen.insert(comparable_path_key(path)))
        .collect()
}

pub fn import_rewrites_for_code(
    script_root: &Path,
    current_file_path: &Path,
    code: &str,
) -> Vec<ImportRewrite> {
    let mut rewrites = Vec::new();

    let normalized_code = normalize_package_specifier(code);
    for captures in import_from_regex().captures_iter(&normalized_code) {
        let import_binding = captures
            .name("binding")
            .map(|value| value.as_str().trim())
            .unwrap_or_default();
        if import_binding.is_empty() || import_binding.starts_with('{') {
            continue;
        }
        let Some(quote) = captures
            .name("quote")
            .and_then(|value| value.as_str().chars().next())
        else {
            continue;
        };
        let Some(specifier) = captures.name("specifier").map(|value| value.as_str()) else {
            continue;
        };
        let replacement_binding = resource_import_binding_name(import_binding);
        let normalized_specifier = normalize_package_specifier(specifier);
        let resource_path =
            resolve_resource_path(script_root, current_file_path, &normalized_specifier);
        let resource_kind = resource_kind_from_path(&resource_path);
        let replacement = match resource_kind {
            ImportedResourceKind::Image => replacement_binding.and_then(|binding| {
                relative_to_script_root(script_root, &resource_path).map(|path| {
                    format!("const {binding} = file.ReadImageMatSync({quote}{path}{quote});")
                })
            }),
            ImportedResourceKind::Text => replacement_binding.and_then(|binding| {
                relative_to_script_root(script_root, &resource_path).map(|path| {
                    format!("const {binding} = file.ReadTextSync({quote}{path}{quote});")
                })
            }),
            ImportedResourceKind::JavaScript | ImportedResourceKind::Unknown => None,
        };

        rewrites.push(ImportRewrite {
            import_binding: import_binding.to_string(),
            specifier: specifier.to_string(),
            normalized_specifier,
            resource_kind,
            replacement,
        });
    }

    rewrites
}

pub fn rewrite_script_code(script_root: &Path, current_file_path: &Path, code: &str) -> String {
    if code.is_empty() {
        return String::new();
    }

    let mut rewritten = String::new();
    let current_module_path = relative_to_script_root(script_root, current_file_path)
        .unwrap_or_else(|| current_file_path.to_string_lossy().replace('\\', "/"));
    let current_module_dir = current_file_path
        .parent()
        .and_then(|parent| relative_to_script_root(script_root, parent))
        .unwrap_or_else(|| ".".to_string());
    let normalized_code = normalize_package_specifier(code);
    for line in rewrite_resource_imports(script_root, current_file_path, &normalized_code).lines() {
        if !line.contains("import.meta") {
            rewritten.push_str(line);
        } else {
            let line = rewrite_import_meta_line(line, &current_module_path, &current_module_dir);
            rewritten.push_str(&line);
        }
        rewritten.push('\n');
    }

    if code.ends_with('\n') {
        rewritten
    } else {
        rewritten.pop();
        rewritten
    }
}

fn import_from_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r#"(?s)import\s+(?P<binding>[\w\d_*$]+|[^;]*?)\s+from\s+(?P<quote>['"])(?P<specifier>[^'"\n]+)['"]"#,
        )
        .expect("resource import rewrite regex must compile")
    })
}

fn rewrite_resource_imports(script_root: &Path, current_file_path: &Path, code: &str) -> String {
    let mut rewritten = String::new();
    let mut cursor = 0;
    for captures in import_from_regex().captures_iter(code) {
        let Some(full_match) = captures.get(0) else {
            continue;
        };
        let Some(replacement) =
            resource_import_replacement(script_root, current_file_path, &captures)
        else {
            continue;
        };
        if full_match.start() < cursor {
            continue;
        }
        rewritten.push_str(&code[cursor..full_match.start()]);
        rewritten.push_str(&replacement);
        cursor = full_match.end();
    }
    rewritten.push_str(&code[cursor..]);
    rewritten
}

fn resource_import_replacement(
    script_root: &Path,
    current_file_path: &Path,
    captures: &regex::Captures<'_>,
) -> Option<String> {
    let import_binding = captures.name("binding")?.as_str().trim();
    let binding = resource_import_binding_name(import_binding)?;
    let quote = captures.name("quote")?.as_str().chars().next()?;
    let specifier = captures.name("specifier")?.as_str();
    let resource_path = resolve_resource_path(script_root, current_file_path, specifier);
    let resource_kind = resource_kind_from_path(&resource_path);
    let relative_path = relative_to_script_root(script_root, &resource_path)?;
    match resource_kind {
        ImportedResourceKind::Image => Some(format!(
            "const {binding} = file.ReadImageMatSync({quote}{relative_path}{quote});"
        )),
        ImportedResourceKind::Text => Some(format!(
            "const {binding} = file.ReadTextSync({quote}{relative_path}{quote});"
        )),
        ImportedResourceKind::JavaScript | ImportedResourceKind::Unknown => None,
    }
}

fn resource_import_binding_name(import_binding: &str) -> Option<&str> {
    let binding = import_binding.trim();
    if binding.is_empty() || binding.starts_with('{') {
        return None;
    }
    if let Some(namespace_binding) = binding.strip_prefix("*") {
        let namespace_binding = namespace_binding.trim_start();
        let namespace_binding = namespace_binding.strip_prefix("as")?.trim_start();
        return is_js_identifier(namespace_binding).then_some(namespace_binding);
    }
    let default_binding = binding
        .split_once(',')
        .map_or(binding, |(default, _)| default)
        .trim();
    is_js_identifier(default_binding).then_some(default_binding)
}

fn is_js_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn rewrite_import_meta_line(line: &str, module_path: &str, module_dir: &str) -> String {
    if !line.contains("import.meta") {
        return line.to_string();
    }

    let module_path = serde_json::to_string(module_path).unwrap_or_else(|_| "\"\"".to_string());
    let module_dir = serde_json::to_string(module_dir).unwrap_or_else(|_| "\".\"".to_string());
    line.replace("import.meta.url", &module_path)
        .replace("import.meta.dirname", &module_dir)
}

pub fn normalize_package_specifier(specifier: &str) -> String {
    specifier.replace("../../../packages", "packages")
}

fn resolve_resource_path(script_root: &Path, current_file_path: &Path, specifier: &str) -> PathBuf {
    let specifier_path = Path::new(specifier);
    if specifier.starts_with("packages/") {
        script_root.join(specifier_path)
    } else if specifier.starts_with('.') {
        current_file_path
            .parent()
            .unwrap_or(script_root)
            .join(specifier_path)
    } else {
        script_root.join(specifier_path)
    }
}

fn resource_kind_from_path(path: &Path) -> ImportedResourceKind {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("js") => ImportedResourceKind::JavaScript,
        Some("png" | "jpg" | "jpeg" | "bmp" | "tiff" | "webp") => ImportedResourceKind::Image,
        Some(_) => ImportedResourceKind::Text,
        None => ImportedResourceKind::Unknown,
    }
}

fn has_js_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("js"))
}

fn strip_leading_relative_segments(specifier: &str) -> Option<String> {
    let mut remaining = specifier;
    let mut stripped = false;
    loop {
        if let Some(next) = remaining.strip_prefix("../") {
            remaining = next;
            stripped = true;
        } else if let Some(next) = remaining.strip_prefix("./") {
            remaining = next;
            stripped = true;
        } else {
            break;
        }
    }
    (stripped && !remaining.is_empty()).then(|| remaining.to_string())
}

fn relative_to_script_root(script_root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(script_root)
        .ok()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

pub(crate) fn comparable_path_key(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    if cfg!(windows) {
        value.to_ascii_lowercase()
    } else {
        value
    }
}
