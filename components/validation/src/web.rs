use lazy_static::lazy_static;
use regex::Regex;
use stores::config::{CreatePlugin, CreateScript, UpdatePluginMeta, UpdateScript};

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

impl Validator for CreatePlugin {
    fn validate(&self, ctx: &mut ValidationContext) {
        check_plugin_name(ctx, &self.name);
        check_plugin_short_description(ctx, &self.short_description);
        check_plugin_long_description(ctx, &self.long_description);
    }
}

impl Validator for UpdatePluginMeta {
    fn validate(&self, ctx: &mut ValidationContext) {
        if let Some(name) = &self.name {
            check_plugin_name(ctx, name)
        }

        if let Some(short_description) = &self.short_description {
            check_plugin_short_description(ctx, short_description);
        }

        if let Some(long_description) = &self.long_description {
            check_plugin_long_description(ctx, long_description);
        }
    }
}

fn check_plugin_short_description(ctx: &mut ValidationContext, short_desc: &str) {
    if short_desc.chars().count() > 150 {
        ctx.push_error(
            "short_description",
            "short description can be max 150 characters long".to_string(),
        );
    }
}

fn check_plugin_long_description(ctx: &mut ValidationContext, long_desc: &str) {
    if long_desc.chars().count() > 4000 {
        ctx.push_error(
            "long_description",
            "long description can be max 4000 characters long".to_string(),
        );
    }
}

fn check_plugin_name(ctx: &mut ValidationContext, name: &str) {
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
