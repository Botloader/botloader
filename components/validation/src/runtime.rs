use lazy_static::lazy_static;
use regex::Regex;
use runtime_models::internal::{
    interaction::CommandType,
    script::{
        Command, CommandGroup, CommandOption, CommandSubGroup, SettingsOption,
        SettingsOptionDefinition, SettingsOptionList,
    },
};

use crate::{
    web::{validate_settings_option_value_list, validate_settings_option_value_option},
    ValidationContext, Validator,
};

impl Validator for Command {
    type ContextData = ();

    fn validate(&self, ctx: &mut ValidationContext, _: &()) {
        check_name_field(ctx, "name", &self.name);

        if matches!(self.kind, CommandType::Chat) {
            check_description_field(ctx, "description", &self.description);
        } else if !self.description.is_empty() {
            ctx.push_field_error(
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
            ctx.push_field_error("options", "max 25 options".to_string());
        }

        let mut found_optional = false;
        for option in &self.options {
            ctx.push_field("options".to_string());
            option.validate(ctx, &());
            ctx.pop_field();

            if !option.required {
                found_optional = true;
            } else if found_optional {
                ctx.push_field_error("options", "optional options has to be last".to_string());
            }
        }
    }
}

impl Validator for CommandOption {
    type ContextData = ();

    fn validate(&self, ctx: &mut ValidationContext, _: &()) {
        check_name_field(ctx, "name", &self.name);
        check_description_field(ctx, "description", &self.description);
    }
}

impl Validator for CommandGroup {
    type ContextData = ();

    fn validate(&self, ctx: &mut ValidationContext, _: &()) {
        check_name_field(ctx, "name", &self.name);
        check_description_field(ctx, "description", &self.description);

        for sub_group in &self.sub_groups {
            ctx.push_field("sub_groups".to_string());
            sub_group.validate(ctx, &());
            ctx.pop_field();
        }
    }
}

impl Validator for CommandSubGroup {
    type ContextData = ();

    fn validate(&self, ctx: &mut ValidationContext, _: &()) {
        check_name_field(ctx, "name", &self.name);
        check_description_field(ctx, "description", &self.description);
    }
}

fn check_name_field(ctx: &mut ValidationContext, field: &str, value: &str) {
    if value.chars().count() < 1 {
        ctx.push_field_error(field, "has to be atleast 1 character".to_string());
    }
    if value.chars().count() > 32 {
        ctx.push_field_error(field, "can be max 32 characters long".to_string());
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^[\w-]+$"#).unwrap();
    }

    if !RE.is_match(value) {
        ctx.push_field_error(field, "can only contain 'word' characters".to_string());
    }

    if value.to_lowercase() != value {
        ctx.push_field_error(field, "has to be lower-case".to_string());
    }
}

fn check_description_field(ctx: &mut ValidationContext, field: &str, value: &str) {
    if value.chars().count() < 1 {
        ctx.push_field_error(field, "has to be atleast 1 character".to_string());
    }
    if value.chars().count() > 100 {
        ctx.push_field_error(field, "can be max 100 characters long".to_string());
    }
}

impl Validator for SettingsOptionDefinition {
    type ContextData = ();

    fn validate(&self, ctx: &mut ValidationContext, context_data: &Self::ContextData) {
        match self {
            SettingsOptionDefinition::Option(opt) => {
                ctx.push_field(&opt.name);
                opt.validate(ctx, context_data);
                ctx.pop_field();
            }
            SettingsOptionDefinition::List(list) => {
                ctx.push_field(&list.name);
                list.validate(ctx, context_data);
                ctx.pop_field();
            }
        }
    }
}

impl Validator for SettingsOption {
    type ContextData = ();

    fn validate(&self, ctx: &mut ValidationContext, _context_data: &Self::ContextData) {
        validate_option_name(ctx, &self.name);

        if self.description.chars().count() > 1000 {
            ctx.push_field_error("description", "max 1000 characters");
        }

        // validate default value
        if let Some(default_value) = &self.default_value {
            validate_settings_option_value_option(ctx, default_value, self, None);
        }
    }
}

impl Validator for SettingsOptionList {
    type ContextData = ();

    fn validate(&self, ctx: &mut ValidationContext, context_data: &Self::ContextData) {
        validate_option_name(ctx, &self.name);

        if self.template.len() > 10 {
            ctx.push_error("list objects can have max 10 fields")
        }

        if self.description.chars().count() > 1000 {
            ctx.push_error("option description can be max 1000 characters");
        }

        for opt in &self.template {
            ctx.push_field(&opt.name);
            opt.validate(ctx, context_data);
            ctx.pop_field();
        }

        // check for name duplicates
        for (i, opt) in self.template.iter().enumerate() {
            for (j, inner_opt) in self.template.iter().enumerate() {
                if opt.name == inner_opt.name && i != j {
                    ctx.push_error(format!("option {} appears multiple times", opt.name))
                }
            }
        }

        // validate list default value
        if let Some(default_value) = &self.default_value {
            validate_settings_option_value_list(ctx, default_value, self, None);
        }
    }
}

fn validate_option_name(ctx: &mut ValidationContext, name: &str) {
    if name.chars().count() > 42 {
        ctx.push_field_error("name", "name can be max 42 characters long".to_string());
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^[\w_-]*$"#).unwrap();
    }
    if !RE.is_match(name) {
        ctx.push_field_error(
            "name",
            "name can only contain 'word characters', '-' and '_'".to_string(),
        );
    }
}
