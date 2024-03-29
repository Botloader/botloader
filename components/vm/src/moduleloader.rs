use deno_core::{ModuleLoader, ModuleSource, ModuleType, ResolutionKind};
use futures::future::ready;
use url::Url;

use crate::{ScriptLoadState, ScriptsStateStoreHandle};

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
            .map(|e| {
                ModuleSource::new(
                    ModuleType::JavaScript,
                    deno_core::ModuleSourceCode::Bytes(e.source.as_bytes().into()),
                    &e.specifier,
                )
            })
    }

    fn try_load_script_module(
        &self,
        module_specifier: &deno_core::ModuleSpecifier,
    ) -> Option<ModuleSource> {
        let mut store = self.guild_scripts.borrow_mut();
        if let Some(script) = store
            .scripts
            .iter_mut()
            .find(|v| &v.url == module_specifier)
        {
            if let Some(compiled) = &script.compiled {
                script.state = ScriptLoadState::Loaded;

                return Some(ModuleSource::new(
                    ModuleType::JavaScript,
                    deno_core::ModuleSourceCode::Bytes(compiled.output.as_bytes().into()),
                    module_specifier,
                ));
            }
        }

        None
    }
}

// TODO: make a formal spec for this behavior
impl ModuleLoader for ModuleManager {
    fn resolve(
        &self,
        mut specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<deno_core::ModuleSpecifier, deno_core::error::AnyError> {
        // info!("resolving module: {} - {}", specifier, referrer);
        if let Ok(u) = Url::parse(specifier) {
            return Ok(u);
        };

        // TODO: remove this hardcoded overload
        if specifier == "botloader" {
            specifier = "/index";
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
            .join(format!("{specifier}.js").as_str())
            .unwrap();

        Ok(resolved)
    }

    fn load(
        &self,
        module_specifier: &deno_core::ModuleSpecifier,
        _maybe_referrer: Option<&deno_core::ModuleSpecifier>,
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
