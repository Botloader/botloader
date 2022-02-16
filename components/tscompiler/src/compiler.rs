use std::sync::{Arc, Mutex};

use swc_common::{
    self, chain,
    errors::{Diagnostic, Emitter, Handler},
    sync::Lrc,
    FileName, Globals, Mark, SourceMap,
};
// use swc_ecmascript::ast::Module;
use swc_ecmascript::{
    ast::EsVersion,
    codegen::{text_writer::JsWriter, Emitter as CodeEmitter},
    parser::TsConfig,
    transforms::{fixer, helpers, hygiene, resolver_with_mark, typescript::strip},
};
use swc_ecmascript::{
    codegen::Config as CodeGenConfig,
    parser::{lexer::Lexer, Capturing, Parser, StringInput, Syntax},
    visit::FoldWith,
};

pub fn compile_typescript(input: &str) -> Result<CompiledItem, String> {
    compile_typescript_inner(input)
}

fn compile_typescript_inner(input: &str) -> Result<CompiledItem, String> {
    let mut result_buf = Vec::new();

    swc_common::GLOBALS.set(&Globals::new(), || {
        helpers::HELPERS.set(&helpers::Helpers::default(), || {
            let global_mark = Mark::fresh(Mark::root());

            let cm: Lrc<SourceMap> = Default::default();

            let buf = Arc::new(Mutex::new(Vec::new()));
            let handler = Handler::with_emitter_writer(
                Box::new(VecLocked { buf: buf.clone() }),
                Some(cm.clone()),
            );

            let fm = cm.new_source_file(FileName::Custom("script.ts".into()), input.into());

            let lexer = Lexer::new(
                Syntax::Typescript(TsConfig {
                    ..Default::default()
                }),
                EsVersion::Es2020,
                StringInput::from(&*fm),
                None,
            );

            let capturing = Capturing::new(lexer);

            let mut parser = Parser::new_from(capturing);

            for e in parser.take_errors() {
                e.into_diagnostic(&handler).emit();
            }

            let mut module = match parser.parse_module() {
                Ok(m) => m,
                Err(e) => {
                    e.into_diagnostic(&handler).emit();
                    let errs = buf.lock().unwrap();
                    return Err(String::from_utf8(errs.clone()).unwrap());
                }
            };

            let mut pass = chain!(
                resolver_with_mark(global_mark),
                strip(global_mark),
                hygiene(),
                // compat::es2021::es2021(),
                // export_namespace_from(),
                // compat::reserved_words::reserved_words(),
                fixer(None),
            );

            module = module.fold_with(&mut pass);
            let mut src_map = Vec::new();

            {
                let writer = JsWriter::new(cm.clone(), "\n", &mut result_buf, Some(&mut src_map));

                let mut emitter = CodeEmitter {
                    cfg: CodeGenConfig {
                        ..Default::default()
                    },
                    cm: cm.clone(),
                    comments: None,
                    wr: Box::new(writer),
                };

                // TODO: handle the io error? how would there be a io error since its a in mem buffer though?
                emitter.emit_module(&module).unwrap();
            }

            let proper_source_map = cm.build_source_map(&mut src_map);

            // i really hope this dosen't produce any invalid utf8 stuff :eyes:
            Ok(CompiledItem {
                output: String::from_utf8(result_buf).unwrap(),
                source_map: proper_source_map,
            })
        })
    })
}

struct CollectingEmitter {
    messages: Arc<Mutex<Vec<Diagnostic>>>,
}

impl Emitter for CollectingEmitter {
    fn emit(&mut self, db: &swc_common::errors::DiagnosticBuilder<'_>) {
        let mut messages = self.messages.lock().unwrap();
        // let mut_brw = self.messages.borrow_mut();
        messages.push((**db).clone());
        // println!("[SWC]: {:?}", db);
    }
}

struct VecLocked {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl std::io::Write for VecLocked {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut inner = self.buf.lock().unwrap();
        std::io::Write::write(&mut *inner, buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut inner = self.buf.lock().unwrap();
        std::io::Write::flush(&mut *inner)
    }
    // fn emit(&mut self, db: &swc_common::errors::DiagnosticBuilder<'_>) {
    //     let mut messages = self.messages.lock().unwrap();
    //     // let mut_brw = self.messages.borrow_mut();
    //     messages.push((**db).clone());
    //     // println!("[SWC]: {:?}", db);
    // }
}

#[derive(Debug, Clone)]
pub struct CompiledItem {
    pub output: String,
    pub source_map: sourcemap::SourceMap,
}
