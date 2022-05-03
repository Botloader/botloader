use deno_core::{ModuleLoader, ModuleSource, ModuleType};
use futures::future::ready;
use url::Url;

use crate::{prepend_script_source_header, ScriptLoadState, ScriptsStateStoreHandle};

pub struct ModuleManager {
    pub module_map: Vec<ModuleEntry>,
    pub guild_scripts: ScriptsStateStoreHandle,
}

impl ModuleManager {
    fn try_load_std_module(
        &self,
        module_specifier: &deno_core::ModuleSpecifier,
    ) -> Option<ModuleSource> {
        self.module_map
            .iter()
            .find(|e| e.specifier == *module_specifier)
            .map(|e| ModuleSource {
                code: e.source.to_string(),
                module_url_found: module_specifier.to_string(),
                module_url_specified: module_specifier.to_string(),
                module_type: ModuleType::JavaScript,
            })
    }

    fn try_load_script_module(
        &self,
        module_specifier: &deno_core::ModuleSpecifier,
    ) -> Option<ModuleSource> {
        let name = module_specifier
            .path()
            .strip_prefix('/')
            .unwrap_or_default()
            .strip_suffix(".js")?;

        let mut store = self.guild_scripts.borrow_mut();
        if let Some(script) = store.scripts.iter_mut().find(|v| v.script.name() == name) {
            script.state = ScriptLoadState::Loaded;

            let source = match &script.script {
                crate::VmScript::GuildScript(gs) => {
                    prepend_script_source_header(&script.compiled.output, Some(gs.id))
                }
                crate::VmScript::PackScript(_) => script.compiled.output.clone(),
            };

            return Some(ModuleSource {
                code: source,
                module_url_found: module_specifier.to_string(),
                module_url_specified: module_specifier.to_string(),
                module_type: ModuleType::JavaScript,
            });
        }

        None
    }
}

// TODO: make a formal spec for this behaviour
impl ModuleLoader for ModuleManager {
    fn resolve(
        &self,
        mut specifier: &str,
        referrer: &str,
        _is_main: bool,
    ) -> Result<deno_core::ModuleSpecifier, deno_core::error::AnyError> {
        // info!("resolving module: {} - {}", specifier, referrer);
        if let Ok(u) = Url::parse(specifier) {
            return Ok(u);
        };

        // TODO: remove this hardcoded overload
        if specifier == "botloader" {
            specifier = "/_sdk/index";
        }

        let parsed_referrer = Url::parse(referrer).map_err(|e| {
            anyhow::anyhow!(
                "failed parsing referrer url: {} ({} - {})",
                e,
                specifier,
                referrer
            )
        })?;

        let resolved = parsed_referrer
            .join(format!("{}.js", specifier).as_str())
            .unwrap();

        Ok(resolved)
    }

    fn load(
        &self,
        module_specifier: &deno_core::ModuleSpecifier,
        _maybe_referrer: Option<deno_core::ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> std::pin::Pin<Box<deno_core::ModuleSourceFuture>> {
        // info!("loading module: {}", module_specifier.to_string());

        Box::pin(ready(
            if let Some(l) = self.try_load_std_module(module_specifier) {
                Ok(l)
            } else if let Some(l) = self.try_load_script_module(module_specifier) {
                Ok(l)
            } else {
                Err(anyhow::anyhow!(
                    "failed finding module {:?}",
                    module_specifier
                ))
            },
        ))
    }
}

pub struct ModuleEntry {
    pub specifier: Url,
    pub source: &'static str,
}
