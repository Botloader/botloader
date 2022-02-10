use vm::moduleloader::ModuleEntry;

macro_rules! include_js {
    ($f:tt) => {
        include_str!(concat!(env!("OUT_DIR"), concat!("/js/", $f)))
    };
}

include!(concat!(env!("OUT_DIR"), "/module_map.rs"));

pub fn create_module_map() -> Vec<ModuleEntry> {
    MODULE_MAP
        .iter()
        .map(|(name, source)| ModuleEntry {
            source: *source,
            specifier: name.clone(),
        })
        .collect()
}
