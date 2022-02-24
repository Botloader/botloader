use lazy_static::lazy_static;
use regex::Regex;
use stores::config::{CreateScript, UpdateScript};

use crate::{ValidationContext, Validator};

impl Validator for CreateScript {
    fn validate(&self, ctx: &mut ValidationContext) {
        check_script_name(ctx, &self.name);
        check_script_source(ctx, &self.original_source);
    }
}

impl Validator for UpdateScript {
    fn validate(&self, ctx: &mut ValidationContext) {
        if let Some(name) = &self.name {
            check_script_name(ctx, name);
        }

        if let Some(source) = &self.original_source {
            check_script_source(ctx, source);
        }
    }
}

fn check_script_name(ctx: &mut ValidationContext, name: &str) {
    if name.chars().count() > 32 {
        ctx.push_error("name", "name can be max 32 characters long".to_string());
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^[\w_-]*$"#).unwrap();
    }
    if !RE.is_match(name) {
        ctx.push_error(
            "name",
            "name can only contain 'a-z', '-' and '_'".to_string(),
        );
    }
}

fn check_script_source(ctx: &mut ValidationContext, source: &str) {
    if source.len() > 100_000 {
        ctx.push_error("original_source", "source can be max 100KiB".to_string());
    }
}
