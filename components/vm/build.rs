use deno_core::error::AnyError;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;

use std::env;
use std::path::PathBuf;

fn main() {
    let core_extension = deno_core::Extension::builder()
        .js(deno_core::include_js_files!(
          prefix "bl:core",
          "src/botloader-core.js",
        ))
        .build();

    let o = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let snapshot_path = o.join("BOTLOADER_SNAPSHOT.bin");
    let options = RuntimeOptions {
        will_snapshot: true,
        extensions: vec![core_extension],
        ..Default::default()
    };
    let mut isolate = JsRuntime::new(options);

    let snapshot = isolate.snapshot();
    let snapshot_slice: &[u8] = &*snapshot;
    println!("Snapshot size: {}", snapshot_slice.len());
    std::fs::write(&snapshot_path, snapshot_slice).unwrap();
    println!("Snapshot written to: {} ", snapshot_path.display());
}

pub fn in_mem_source_load_fn(src: &'static str) -> Box<dyn Fn() -> Result<String, AnyError>> {
    Box::new(move || Ok(src.to_string()))
}
