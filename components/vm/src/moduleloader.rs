use std::borrow::Cow;

use deno_core::{
    error::ModuleLoaderError, ModuleLoadOptions, ModuleLoadReferrer, ModuleLoadResponse,
    ModuleLoader, ModuleSource, ModuleType, ResolutionKind,
};
use deno_error::JsErrorBox;
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
                    None,
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
                    deno_core::ModuleSourceCode::Bytes(deno_core::ModuleCodeBytes::Boxed(
                        Box::from(compiled.output.as_bytes()),
                    )),
                    module_specifier,
                    None,
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
    ) -> Result<deno_core::ModuleSpecifier, ModuleLoaderError> {
        // info!("resolving module: {} - {}", specifier, referrer);
        if let Ok(u) = Url::parse(specifier) {
            return Ok(u);
        };

        // TODO: remove this hardcoded overload
        if specifier == "botloader" {
            specifier = "/index";
        }

        let parsed_referrer = Url::parse(referrer).map_err(|e| {
            JsErrorBox::generic(format!(
                "failed parsing referrer url: {e} ({specifier} - {referrer})"
            ))
        })?;

        let resolved = parsed_referrer
            .join(format!("{specifier}.js").as_str())
            .map_err(|e| {
                JsErrorBox::generic(format!(
                    "failed resolving module: {e} ({specifier} - {referrer})"
                ))
            })?;

        Ok(resolved)
    }

    fn load(
        &self,
        module_specifier: &deno_core::ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        ModuleLoadResponse::Sync(
            if let Some(l) = self.try_load_std_module(module_specifier) {
                Ok(l)
            } else if let Some(l) = self.try_load_script_module(module_specifier) {
                Ok(l)
            } else {
                Err(JsErrorBox::generic(format!(
                    "failed finding module {module_specifier:?}"
                )))
            },
        )
    }

    fn get_source_map(&self, file_name: &str) -> Option<Cow<'_, [u8]>> {
        let store = self.guild_scripts.borrow();
        let script = store
            .scripts
            .iter()
            .find(|v| v.url.as_str() == file_name)?;

        let compiled = script.compiled.as_ref()?;
        Some(Cow::Owned(compiled.source_map_raw.as_bytes().to_vec()))
    }

    fn get_source_mapped_source_line(&self, file_name: &str, line_number: usize) -> Option<String> {
        Some(format!("{file_name}:{line_number}"))
    }
}

pub struct ModuleEntry {
    pub specifier: Url,
    pub source: &'static str,
}
