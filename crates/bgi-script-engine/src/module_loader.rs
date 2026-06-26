use super::Result;
use bgi_script::{LoadedScriptModule, ScriptModuleLoader};
use boa_engine::{
    js_string,
    module::{ModuleLoader, Referrer},
    object::JsObject,
    Context, JsError, JsNativeError, JsResult, JsString, Module, Source,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct BetterGiModuleLoader {
    loader: RefCell<ScriptModuleLoader>,
    modules: RefCell<HashMap<PathBuf, Module>>,
}

impl BetterGiModuleLoader {
    pub(crate) fn new(script_root: PathBuf, search_paths: Vec<PathBuf>) -> Result<Self> {
        Ok(Self {
            loader: RefCell::new(ScriptModuleLoader::new(script_root, search_paths)?),
            modules: RefCell::new(HashMap::new()),
        })
    }

    pub(crate) fn parse_loaded_module(
        &self,
        module: &LoadedScriptModule,
        context: &mut Context,
    ) -> JsResult<Module> {
        let path = module.resolution.resolved_path.clone();
        if let Some(module) = self.modules.borrow().get(&path).cloned() {
            return Ok(module);
        }

        let parsed = Module::parse(
            Source::from_reader(module.code.as_bytes(), Some(path.as_path())),
            None,
            context,
        )
        .map_err(|err| {
            JsError::from(
                JsNativeError::syntax()
                    .with_message(format!("could not parse module `{}`", path.display()))
                    .with_cause(err),
            )
        })?;
        self.modules.borrow_mut().insert(path, parsed.clone());
        Ok(parsed)
    }
}

impl ModuleLoader for BetterGiModuleLoader {
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        let specifier_text = specifier.to_std_string_escaped();
        let referrer_path = referrer.path().map(Path::to_path_buf);
        let result: JsResult<Module> = (|| {
            let module = self
                .loader
                .borrow_mut()
                .load_js_module(&specifier_text, referrer_path.as_deref())
                .map_err(|err| {
                    JsError::from(
                        JsNativeError::typ()
                            .with_message(err.to_string())
                            .with_cause(JsError::from_opaque(
                                js_string!(specifier_text.clone()).into(),
                            )),
                    )
                })?;
            self.parse_loaded_module(&module, context)
        })();

        finish_load(result, context);
    }

    fn init_import_meta(&self, import_meta: &JsObject, module: &Module, context: &mut Context) {
        let Some(path) = module.path() else {
            return;
        };
        let loader = self.loader.borrow();
        let (url, dirname) = import_meta_paths(loader.script_root(), path);
        let _ = import_meta.set(
            js_string!("url"),
            JsString::from(url.as_str()),
            true,
            context,
        );
        let _ = import_meta.set(
            js_string!("dirname"),
            JsString::from(dirname.as_str()),
            true,
            context,
        );
    }
}

fn import_meta_paths(script_root: &Path, module_path: &Path) -> (String, String) {
    let module_path = path_relative_to_script_root(script_root, module_path)
        .unwrap_or_else(|| path_to_slash_string(module_path));
    let dirname = module_path
        .rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default();
    (module_path, dirname)
}

fn path_relative_to_script_root(script_root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(script_root)
        .ok()
        .map(path_to_slash_string)
}

fn path_to_slash_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
