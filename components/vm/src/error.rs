use deno_core::error::JsError;

pub fn handle_js_error(err: anyhow::Error) -> anyhow::Error {
    match err.downcast::<JsError>() {
        Ok(v) => {
            let mut msg = v.exception_message.clone();

            let frame = v.frames.first();
            if let Some(frame) = frame {
                if let (Some(f_), Some(l), Some(c)) =
                    (&frame.file_name, frame.line_number, frame.column_number)
                {
                    msg.push_str(&format!("\n    at {}:{}:{}", f_, l, c));
                }
            }
            anyhow::Error::msg(msg)
        }
        Err(orig) => orig,
    }
}
