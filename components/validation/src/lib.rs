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

pub fn validate<T: Validator>(
    val: &T,
    ctx_data: &T::ContextData,
) -> Result<(), Vec<ValidationError>> {
    let mut ctx = ValidationContext::new();

    val.validate(&mut ctx, ctx_data);
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
    pub fn new() -> Self {
        ValidationContext {
            errs: Vec::new(),
            field_stack: Vec::new(),
        }
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationContext {
    fn push_field_error(&mut self, field: &str, msg: impl Into<String>) {
        let mut prefix = self.field_stack.join(".");
        if !prefix.is_empty() {
            prefix.push('.');
        }

        self.errs.push(ValidationError {
            field: format!("{prefix}{field}"),
            msg: msg.into(),
        });
    }

    fn push_error(&mut self, msg: impl Into<String>) {
        let fields = self.field_stack.join(".");

        self.errs.push(ValidationError {
            field: fields,
            msg: msg.into(),
        });
    }

    pub fn push_field(&mut self, field: impl Into<String>) {
        self.field_stack.push(field.into());
    }

    pub fn push_index(&mut self, index: usize) {
        self.push_field(index.to_string())
    }

    pub fn pop_field(&mut self) {
        self.field_stack.pop();
    }
}

pub trait Validator {
    type ContextData;

    fn validate(&self, ctx: &mut ValidationContext, context_data: &Self::ContextData);
}
