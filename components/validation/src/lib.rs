use std::fmt::Display;

use serde::Serialize;

pub mod runtime;
pub mod web;

#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub msg: String,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}: {}", self.field, self.msg))
    }
}

pub fn validate<T: Validator>(val: &T) -> Result<(), Vec<ValidationError>> {
    let mut ctx = ValidationContext {
        errs: Vec::new(),
        field_stack: Vec::new(),
    };
    val.validate(&mut ctx);
    if ctx.errs.is_empty() {
        Ok(())
    } else {
        Err(ctx.errs)
    }
}

pub struct ValidationContext {
    errs: Vec<ValidationError>,
    field_stack: Vec<String>,
}

impl ValidationContext {
    fn push_error(&mut self, field: &str, msg: String) {
        let mut prefix = self.field_stack.join(".");
        if !prefix.is_empty() {
            prefix.push('.');
        }

        self.errs.push(ValidationError {
            field: field.to_string(),
            msg: format!("{}{}", prefix, msg),
        });
    }

    pub fn push_field(&mut self, field: String) {
        self.field_stack.push(field);
    }

    pub fn pop_field(&mut self) {
        self.field_stack.pop();
    }
}

pub trait Validator {
    fn validate(&self, ctx: &mut ValidationContext);
}
