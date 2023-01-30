use deno_core::error::JsError;

use regex::Regex;

use crate::{ScriptsStateStore, ScriptsStateStoreHandle};
use lazy_static::lazy_static;

pub fn source_map_error(
    loaded_scripts: &ScriptsStateStoreHandle,
    err: anyhow::Error,
) -> anyhow::Error {
    match err.downcast::<JsError>() {
        Ok(v) => {
            if let Some(stack) = v.stack {
                let borrow = loaded_scripts.borrow();
                let transformed = parse_transform_stack(&borrow, &stack);
                anyhow::anyhow!("{}\n{}", v.exception_message, transformed)
            } else {
                v.into()
            }
        }
        Err(orig) => orig,
    }
}

fn parse_transform_stack(scripts: &ScriptsStateStore, stack: &str) -> String {
    let mut output = String::new();

    for line in stack.split('\n') {
        if let Some(new_line) = parse_transform_stack_line(scripts, line) {
            output.push_str(&new_line)
        } else {
            output.push_str(line)
        }
        output.push('\n');
    }

    output
}

fn parse_transform_stack_line(scripts: &ScriptsStateStore, line: &str) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"\(?(file:///[\w/\.]+):(\d+):(\d+)\)?"#).unwrap();
    }

    let segments = line
        .split(' ')
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();

    let (main_part, func) = if segments.len() == 2 {
        (segments[1], None)
    } else if segments.len() == 3 {
        (segments[2], Some(segments[1]))
    } else {
        return None;
    };

    let cap = RE.captures(main_part)?;

    let file = cap.get(1)?.as_str();
    let line: u32 = cap.get(2)?.as_str().parse().ok()?;
    let col: u32 = cap.get(3)?.as_str().parse().ok()?;

    let (new_file, src_line, src_col) = scripts.get_original_line_col(file, line, col)?;
    let mut output = String::new();
    output.push_str("    at ");
    if let Some(f) = func {
        output.push_str(f);
        output.push(' ');
    }

    output.push_str(&format!("({new_file}:{src_line}:{src_col})"));
    Some(output)
}
