use deno_core::{op2, OpState};
use guild_logger::entry::CreateLogEntry;
use runtime_models::internal::console::ConsoleLogMessage;

use crate::RuntimeContext;

deno_core::extension!(bl_console, ops = [op_botloader_log,]);

#[op2]
pub fn op_botloader_log(state: &mut OpState, #[serde] args: ConsoleLogMessage) {
    let (name, line_col) = if let (Some(orig_name), Some(line)) = (args.file_name, args.line_number)
    {
        let col = args.col_number.unwrap_or_default();
        (orig_name, Some((line, col)))
    } else {
        (String::new(), None)
    };

    let ctx = state.borrow::<RuntimeContext>();

    ctx.guild_logger
        .log(CreateLogEntry::script_console(args.message, name, line_col));
}
