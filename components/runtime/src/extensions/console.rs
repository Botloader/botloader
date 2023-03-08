use deno_core::{op, Extension, OpState};
use guild_logger::entry::CreateLogEntry;
use runtime_models::internal::console::ConsoleLogMessage;
use vm::ScriptsStateStoreHandle;

use crate::RuntimeContext;
pub fn extension() -> Extension {
    Extension::builder("bl_console")
        .ops(vec![op_botloader_log::decl()])
        .build()
}

#[op]
pub fn op_botloader_log(state: &mut OpState, args: ConsoleLogMessage) {
    let script_store = state.borrow::<ScriptsStateStoreHandle>();

    let (name, line_col) = if let (Some(orig_name), Some(line)) = (args.file_name, args.line_number)
    {
        let col = args.col_number.unwrap_or_default();

        if let Some((src_name, src_line, src_col)) = script_store
            .borrow()
            .get_original_line_col(&orig_name, line, col)
        {
            (src_name, Some((src_line, src_col)))
        } else {
            (orig_name, Some((line, col)))
        }
    } else {
        (String::new(), None)
    };

    let ctx = state.borrow::<RuntimeContext>();

    ctx.guild_logger
        .log(CreateLogEntry::script_console(args.message, name, line_col));
}
