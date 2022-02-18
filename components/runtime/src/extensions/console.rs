use deno_core::{op_sync, Extension, OpState};
use guild_logger::LogEntry;
use runtime_models::internal::console::ConsoleLogMessage;
use vm::{AnyError, ScriptsStateStoreHandle};

use crate::RuntimeContext;
pub fn extension() -> Extension {
    Extension::builder()
        .ops(vec![("op_botloader_log", op_sync(console_log))])
        .build()
}

pub fn console_log(state: &mut OpState, args: ConsoleLogMessage, _: ()) -> Result<(), AnyError> {
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

    ctx.guild_logger.log(LogEntry::script_console(
        ctx.guild_id,
        args.message,
        name,
        line_col,
    ));

    Ok(())
}
