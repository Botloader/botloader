use deno_core::error::AnyError;
use deno_core::JsRuntimeForSnapshot;
use deno_core::RuntimeOptions;

use std::env;
use std::path::PathBuf;

deno_core::extension!(bl_core, js = ["src/botloader-core.js",],);

fn main() {
    // let core_extension = deno_core::Extension::builder("bl_core")
    //     .js(deno_core::include_js_files!(bl_core "src/botloader-core.js",).into())
    //     .build();

    let o = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let snapshot_path = o.join("BOTLOADER_SNAPSHOT.bin");
    let options = RuntimeOptions {
        // will_snapshot: true,
        extensions: vec![bl_core::ext()],

        // extensions_with_js: vec![core_extension],
        ..Default::default()
    };
    let isolate = JsRuntimeForSnapshot::new(options);

    let snapshot = isolate.snapshot();
    let snapshot_slice: &[u8] = &snapshot;
    println!("Snapshot size: {}", snapshot_slice.len());
    std::fs::write(&snapshot_path, snapshot_slice).unwrap();
    println!("Snapshot written to: {} ", snapshot_path.display());
}

pub fn in_mem_source_load_fn(src: &'static str) -> Box<dyn Fn() -> Result<String, AnyError>> {
    Box::new(move || Ok(src.to_string()))
}
