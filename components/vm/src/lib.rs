use std::{cell::RefCell, rc::Rc};

use deno_core::{v8_set_flags, SourceMapGetter};
use stores::config::Script;
use tscompiler::CompiledItem;
use url::Url;

pub mod error;
pub mod moduleloader;
pub mod vm;
pub mod vmthread;

/// Represents a value passed to or from JavaScript.
///
/// Currently aliased as serde_json's Value type.
pub type JsValue = serde_json::Value;

/// Polymorphic error type able to represent different error domains.
pub type AnyError = deno_core::error::AnyError;

pub static BOTLOADER_CORE_SNAPSHOT: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/BOTLOADER_SNAPSHOT.bin"));

pub fn prepend_script_source_header(source: &str, script: Option<&Script>) -> String {
    let mut result = gen_script_source_header(script);
    result.push_str(source);
    result.push_str("\nscript.run();");

    result
}

const SCRIPT_HEADER_NUM_LINES: u32 = 4;

#[test]
fn hmm() {
    let res = gen_script_source_header(None);
    assert!(res.lines().count() == 4);
}

fn gen_script_source_header(script: Option<&Script>) -> String {
    match script {
        None => r#"
        import {Script} from "/script";
        const script = new Script(0, null);
"#
        .to_string(),
        Some(h) => {
            format!(
                r#"
                import {{Script}} from "/script";
                const script = new Script({}, {});
"#,
                h.id,
                h.plugin_id
                    .map(|v| v.to_string())
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

// impl ScriptState {
//     fn get_original_line_col(&self, line_no: u32, col: u32) -> Option<(u32, u32)> {
//         dbg!(line_no);
//         self.compiled
//             .source_map
//             .lookup_token(line_no, col)
//             .map(|token| (token.get_src_line() + 1, token.get_src_col()))
//     }
// }

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

    pub fn get_original_line_col(
        &self,
        res: &str,
        line: u32,
        col: u32,
    ) -> Option<(String, u32, u32)> {
        let excl_js = res.strip_suffix(".js").unwrap();
        Some((
            format!("{excl_js}.ts"),
            line - (SCRIPT_HEADER_NUM_LINES - 1),
            col,
        ))

        // return (res.)

        // if let Some(script_load) = self.scripts.iter().find(|v| v.url.as_str() == res).cloned() {
        //     if let Some((line, col)) = script_load.get_original_line_col(line, col) {
        //         let excl_js = script_load.url.as_str().strip_suffix(".js").unwrap();

        //         return Some((format!("{excl_js}.ts"), line, col));
        //     }
        // }

        // None
    }

    pub fn compile_add_script(&mut self, script: Script) -> Result<ScriptState, String> {
        match tscompiler::compile_typescript(&prepend_script_source_header(
            &script.original_source,
            Some(&script),
        )) {
            Ok(compiled) => {
                let item = ScriptState {
                    compiled: Some(compiled),
                    url: script_url(&script),
                    script,
                    state: ScriptLoadState::Unloaded,
                };

                self.scripts.push(item.clone());

                Ok(item)
            }
            Err(e) => {
                let item = ScriptState {
                    compiled: None,
                    url: script_url(&script),
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

pub(crate) struct ScriptStateStoreWrapper(pub(crate) ScriptsStateStoreHandle);

impl SourceMapGetter for ScriptStateStoreWrapper {
    fn get_source_map(&self, file_name: &str) -> Option<Vec<u8>> {
        let state = self.0.borrow();
        if let Some(script_load) = state
            .scripts
            .iter()
            .find(|v| v.url.as_str() == file_name)
            .cloned()
        {
            if let Some(compiled) = script_load.compiled {
                return Some(compiled.source_map_raw.as_bytes().into());
            }
        }

        None
    }

    fn get_source_line(&self, file_name: &str, line_number: usize) -> Option<String> {
        Some(format!("{file_name}:{line_number}"))
    }
}

fn script_url(script: &Script) -> Url {
    if let Some(plugin_id) = &script.plugin_id {
        return Url::parse(&format!("file:///plugins/{}/{}.js", plugin_id, script.name)).unwrap();
    }

    Url::parse(&format!("file:///guild_scripts/{}.js", script.name)).unwrap()
}

// deno_core::extension!(bl_core, js = ["src/botloader-core.js",],);

pub struct BlCoreOptions {
    cloned_load_states: Rc<RefCell<ScriptsStateStore>>,
}

// parameters = [FP: FetchPermissions],
//   op_fetch<FP>,
//   op_fetch_send,
//   op_fetch_response_upgrade,
//   op_utf8_to_byte_string,
//   op_fetch_custom_client<FP>,

deno_core::extension!(bl_core,
  js = ["src/botloader-core-rt.js",],
  options = {
    options: BlCoreOptions,
  },
  state = |state, options| {
    state.put::<Rc<RefCell<ScriptsStateStore>>>(options.options.cloned_load_states);
  },
);
