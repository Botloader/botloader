use std::{cell::RefCell, rc::Rc};

use deno_core::{v8_set_flags, JsRuntime, OpDecl};
use stores::config::Script;
use tscompiler::CompiledItem;
use url::Url;

pub mod moduleloader;
pub mod vm;
pub mod vmthread;

/// Represents a value passed to or from JavaScript.
///
/// Currently aliased as serde_json's Value type.
pub type JsValue = serde_json::Value;

/// Polymorphic error type able to represent different error domains.
///
/// Ops can't return this directly, they return `runtime::error::OpError` instead.
pub type AnyError = anyhow::Error;

pub static BOTLOADER_CORE_SNAPSHOT: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/BOTLOADER_SNAPSHOT.bin"));

pub fn prepend_script_source_header(source: &str, script: Option<&Script>) -> String {
    let mut result = gen_script_source_header(script);
    result.push_str(source);
    result.push_str("\nscript.run();");

    result
}

fn gen_script_source_header(script: Option<&Script>) -> String {
    match script {
        None => {
            r#"import {Script} from "/script"; const script = new Script("unknown.ts", 0, null);"#
                .to_string()
        }
        Some(s) => {
            format!(
                r#"import {{Script}} from "/script";const script = new Script("{}.ts", {}, {});"#,
                s.name,
                s.id,
                s.plugin_id
                    .map(|v| format!("'{v}'"))
                    .unwrap_or("null".to_string()),
            )
        }
    }
}

pub fn init_v8_flags(v8_flags: &[String]) {
    let v8_flags_includes_help = v8_flags
        .iter()
        .any(|flag| flag == "-help" || flag == "--help");

    // Keep in sync with `standalone.rs`.
    let v8_flags = vec!["UNUSED_BUT_NECESSARY_ARG0".to_owned()]
        .into_iter()
        .chain(v8_flags.iter().cloned())
        .collect::<Vec<_>>();
    let unrecognized_v8_flags = v8_set_flags(v8_flags)
        .into_iter()
        .skip(1)
        .collect::<Vec<_>>();

    if !unrecognized_v8_flags.is_empty() {
        for f in unrecognized_v8_flags {
            eprintln!("error: V8 did not recognize flag '{f}'");
        }
        std::process::exit(1);
    }
    if v8_flags_includes_help {
        std::process::exit(0);
    }
}

#[derive(Clone)]
pub struct ScriptState {
    pub script: Script,
    pub url: url::Url,
    pub state: ScriptLoadState,
    pub compiled: Option<CompiledItem>,
}

impl ScriptState {
    pub fn can_run(&self) -> bool {
        matches!(self.state, ScriptLoadState::Unloaded) && self.compiled.is_some()
    }
}

#[derive(Clone)]
pub enum ScriptLoadState {
    Unloaded,
    Loaded,
    Failed,
    FailedCompilation,
}

pub type ScriptsStateStoreHandle = Rc<RefCell<ScriptsStateStore>>;

#[derive(Clone)]
pub struct ScriptsStateStore {
    pub scripts: Vec<ScriptState>,
}

impl ScriptsStateStore {
    pub fn new() -> Self {
        Self {
            scripts: Vec::new(),
        }
    }

    pub fn new_rc() -> ScriptsStateStoreHandle {
        Rc::new(RefCell::new(Self::new()))
    }

    pub fn clear(&mut self) {
        self.scripts.clear();
    }

    pub fn compile_add_script(&mut self, script: Script) -> Result<ScriptState, String> {
        let prefixed_source = prepend_script_source_header(&script.original_source, Some(&script));

        match tscompiler::compile_typescript(
            &prefixed_source,
            script_url(&script, "ts").to_string(),
        ) {
            Ok(compiled) => {
                let item = ScriptState {
                    compiled: Some(compiled),
                    url: script_url(&script, "js"),
                    script,
                    state: ScriptLoadState::Unloaded,
                };

                self.scripts.push(item.clone());

                Ok(item)
            }
            Err(e) => {
                let item = ScriptState {
                    compiled: None,
                    url: script_url(&script, "js"),
                    script,
                    state: ScriptLoadState::FailedCompilation,
                };

                self.scripts.push(item.clone());

                Err(e)
            }
        }
    }

    pub fn set_state(&mut self, script_id: u64, new_state: ScriptLoadState) {
        if let Some(current) = self.get_script_mut(script_id) {
            current.state = new_state;
        }
    }

    pub fn get_script(&self, script_id: u64) -> Option<&ScriptState> {
        self.scripts.iter().find(|v| v.script.id == script_id)
    }

    pub fn get_script_mut(&mut self, script_id: u64) -> Option<&mut ScriptState> {
        self.scripts.iter_mut().find(|v| v.script.id == script_id)
    }
}

impl Default for ScriptsStateStore {
    fn default() -> Self {
        Self::new()
    }
}

fn script_url(script: &Script, suffix: &str) -> Url {
    if let Some(plugin_id) = &script.plugin_id {
        return Url::parse(&format!(
            "file:///plugins/{}/{}.{suffix}",
            plugin_id, script.name
        ))
        .unwrap();
    }

    Url::parse(&format!("file:///guild_scripts/{}.{suffix}", script.name)).unwrap()
}

pub struct BlCoreOptions {
    cloned_load_states: Rc<RefCell<ScriptsStateStore>>,
}

deno_core::extension!(bl_core,
  js = ["src/botloader-core-rt.js",],
  options = {
    options: BlCoreOptions,
  },
  middleware = disable_unsafe_builtin_ops,
  state = |state, options| {
    state.put::<Rc<RefCell<ScriptsStateStore>>>(options.options.cloned_load_states);
  },
);

/// Built in deno_core ops that scripts must not be able to call.
///
/// Scripts have access to `Deno.core.ops`, so every op deno_core registers is callable by user
/// code. Review the builtin op list (`ops_builtin.rs` and `ops_builtin_v8.rs` in deno_core) for
/// new ops whenever deno_core is updated.
///
/// Ops in here must not be used by the deno_core JS that botloader relies on, as they throw when
/// called (fast calls become no-ops).
pub const DISABLED_BUILTIN_OPS: &[&str] = &[
    // panics, which aborts the whole worker process
    "op_panic",
    // writes straight to the worker process stdout/stderr, scripts use our own console
    "op_print",
    // wasm streaming compilation, botloader doesn't support it
    "op_wasm_streaming_feed",
    "op_wasm_streaming_set_url",
    // synchronous module loading for node's require(), the `_with_source` variant compiles and
    // evaluates arbitrary source under any module specifier, bypassing the module loader
    "op_import_sync",
    "op_import_sync_with_source",
    // node:vm support, also panics on unexpected arguments
    "op_compile_function",
    // lazily loaded extension scripts, botloader has none. `op_set_captured_bootstrap` would let
    // scripts replace the `__bootstrap` object those scripts get evaluated with.
    "op_load_ext_script",
    "op_set_captured_bootstrap",
    // stores stack traces in rust memory that is not covered by the v8 heap limit
    "op_leak_tracing_enable",
    "op_leak_tracing_submit",
];

fn disable_unsafe_builtin_ops(op: OpDecl) -> OpDecl {
    if DISABLED_BUILTIN_OPS.contains(&op.name) {
        op.disable()
    } else {
        op
    }
}

pub fn init_v8_platform() {
    JsRuntime::init_platform(None);
}
