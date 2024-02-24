use tscompiler::compile_typescript;

use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

// Example custom build script.
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=src/ts/*");

    // let files = vec!["op_wrappers", "core_util", "jack"];

    let compiled_files = compile_folder(Path::new("./src/ts/"));

    // create a lazy static file with a mapping of all the modules
    let header = format!(
        r#"
    ::lazy_static::lazy_static! {{
        pub static ref MODULE_MAP: [(::url::Url, &'static str);{}] = [
            (::url::Url::parse("file:///script_globals.js").unwrap(), "export {{}}"),
            "#,
        compiled_files.len() + 1
    );
    let footer = r#"];
    }"#;

    let mut body = String::new();

    for (i, f) in compiled_files.iter().enumerate() {
        body.push_str(&format!(
            r#"(::url::Url::parse("file:///{f}.js").unwrap(), include_js!("{f}.js"))"#
        ));
        if i != compiled_files.len() - 1 {
            body.push(',');
        }
        body.push('\n');
    }

    let full = format!("{header}{body}{footer}");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("module_map.rs"), full).unwrap();
}

// compiles a folder of typescript files recursively, returning a list of files its compiled
fn compile_folder(path: &Path) -> Vec<String> {
    let relative_path = path.strip_prefix("./src/ts/").unwrap();
    let mut result = Vec::<String>::new();
    let entries = std::fs::read_dir(path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .unwrap();

    let mut loaded_files = Vec::new();
    for file in &entries {
        let filename = file.file_name().unwrap().to_str().unwrap();
        if filename.ends_with(".d.ts") || !filename.ends_with(".ts") {
            let data = std::fs::metadata(file).unwrap();
            if data.is_dir() {
                let mut compiled = compile_folder(file);
                result.append(&mut compiled);
            }
            continue;
        }

        let mut result = File::open(file).unwrap();
        let mut contents = String::new();
        result.read_to_string(&mut contents).unwrap();

        loaded_files.push((filename.strip_suffix(".ts").unwrap(), contents));
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.join(Path::new("js").join(relative_path));
    fs::create_dir_all(&target_dir).unwrap();

    for (name, file) in loaded_files {
        let output = compile_typescript(&file, format!("{}.ts", file)).unwrap();
        fs::write(target_dir.join(format!("{name}.js")), output.output).unwrap();

        // we manually construct the joined path because on windows the std::path separator is \ vs what we want, /
        let path = relative_path.join(name);
        let components = path
            .components()
            .map(|v| v.as_os_str().to_string_lossy())
            .collect::<Vec<_>>();

        let joined = components.join("/");

        result.push(joined);
    }

    result
}
