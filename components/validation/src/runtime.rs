use lazy_static::lazy_static;
use regex::Regex;
use runtime_models::internal::{
    command_interaction::CommandType,
    script::{Command, CommandGroup, CommandOption, CommandSubGroup},
};

use crate::{ValidationContext, Validator};

impl Validator for Command {
    fn validate(&self, ctx: &mut ValidationContext) {
        check_name_field(ctx, "name", &self.name);

        if matches!(self.kind, CommandType::Chat) {
            check_description_field(ctx, "description", &self.description);
        } else if !self.description.is_empty() {
            ctx.push_error(
                "description",
                "description has to be empty for user and message commands".to_string(),
            );
        }

        if let Some(group) = &self.group {
            check_name_field(ctx, "group", group);
        }

        if let Some(group) = &self.sub_group {
            check_name_field(ctx, "sub_group", group);
        }

        if self.options.len() > 25 {
            ctx.push_error("options", "max 25 options".to_string());
        }

        let mut found_optional = false;
        for option in &self.options {
            ctx.push_field("options".to_string());
            option.validate(ctx);
            ctx.pop_field();

            if !option.required {
                found_optional = true;
            } else if found_optional {
                ctx.push_error("options", "optional options has to be last".to_string());
            }
        }
    }
}

impl Validator for CommandOption {
    fn validate(&self, ctx: &mut ValidationContext) {
        check_name_field(ctx, "name", &self.name);
        check_description_field(ctx, "description", &self.description);
    }
}

impl Validator for CommandGroup {
    fn validate(&self, ctx: &mut ValidationContext) {
        check_name_field(ctx, "name", &self.name);
        check_description_field(ctx, "description", &self.description);

        for sub_group in &self.sub_groups {
            ctx.push_field("sub_groups".to_string());
            sub_group.validate(ctx);
            ctx.pop_field();
        }
    }
}

impl Validator for CommandSubGroup {
    fn validate(&self, ctx: &mut ValidationContext) {
        check_name_field(ctx, "name", &self.name);
        check_description_field(ctx, "description", &self.description);
    }
}

fn check_name_field(ctx: &mut ValidationContext, field: &str, value: &str) {
    if value.chars().count() < 1 {
        ctx.push_error(field, "has to be atleast 1 character".to_string());
    }
    if value.chars().count() > 32 {
        ctx.push_error(field, "can be max 32 characters long".to_string());
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^[\w-]+$"#).unwrap();
    }

    if !RE.is_match(value) {
        ctx.push_error(field, "can only contain 'word' characters".to_string());
    }

    if value.to_lowercase() != value {
        ctx.push_error(field, "has to be lower-case".to_string());
    }
}

fn check_description_field(ctx: &mut ValidationContext, field: &str, value: &str) {
    if value.chars().count() < 1 {
        ctx.push_error(field, "has to be atleast 1 character".to_string());
    }
    if value.chars().count() > 100 {
        ctx.push_error(field, "can be max 100 characters long".to_string());
    }
}
