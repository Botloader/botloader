use lazy_static::lazy_static;
use regex::Regex;
use runtime_models::internal::script::{
    SettingsOption, SettingsOptionDefinition, SettingsOptionList, SettingsOptionType,
    SettingsOptionValue,
};
use stores::config::{CreatePlugin, CreateScript, Script, UpdatePluginMeta, UpdateScript};

use crate::{ValidationContext, Validator};

impl Validator for CreateScript {
    type ContextData = ();

    fn validate(&self, ctx: &mut ValidationContext, _: &()) {
        check_script_name(ctx, &self.name);
        check_script_source(ctx, "original_source", &self.original_source);
    }
}

impl Validator for UpdateScript {
    type ContextData = Script;

    fn validate(&self, ctx: &mut ValidationContext, current_script: &Script) {
        if let Some(name) = &self.name {
            check_script_name(ctx, name);
        }

        if let Some(source) = &self.original_source {
            check_script_source(ctx, "original_source", source);
        }

        if let Some(values) = &self.settings_values {
            let definitions = self
                .settings_definitions
                .as_ref()
                .or(current_script.settings_definitions.as_ref());

            ctx.push_field("settings_values".to_owned());

            for value in values {
                if let Some(definitions) = definitions {
                    let Some(definition) = definitions.iter().find(|v| v.name() == value.name)
                    else {
                        ctx.push_field_error(&value.name, "unknown option".to_owned());
                        continue;
                    };

                    ctx.push_field(value.name.clone());
                    value.validate(ctx, definition);
                    ctx.pop_field();
                } else {
                    ctx.push_field_error(&value.name, "unknown option".to_owned());
                }
            }

            // check missing required fields
            // if let Some(definitions) = definitions {
            //     for definition in definitions.iter().filter(|def| def.required()) {
            //         if !values.iter().any(|v| v.name == definition.name()) {
            //             ctx.push_error(format!("missing required field: {}", definition.name()));
            //         }
            //     }
            // }

            ctx.pop_field();
        }
    }
}

impl Validator for SettingsOptionValue {
    type ContextData = SettingsOptionDefinition;

    fn validate(&self, ctx: &mut ValidationContext, context_data: &Self::ContextData) {
        match context_data {
            SettingsOptionDefinition::Option(option) => {
                validate_settings_option_value_option(ctx, &self.value, option)
            }
            SettingsOptionDefinition::List(list) => {
                validate_settings_option_value_list(ctx, &self.value, list);
            }
        }
    }
}

pub(crate) fn validate_settings_option_value_option(
    ctx: &mut ValidationContext,
    value: &serde_json::Value,
    definition: &SettingsOption,
) {
    match &definition.kind {
        SettingsOptionType::String {
            max_length,
            min_length,
        } => {
            let Some(value) = value.as_str() else {
                ctx.push_error(format!("expected string, got {}", value));
                return;
            };
        }
        SettingsOptionType::Float { min, max } => {
            let Some(value) = value.as_f64() else {
                ctx.push_error(format!("expected double, got {}", value));
                return;
            };
        }
        SettingsOptionType::Integer { min, max } => {
            let Some(value) = value.as_i64() else {
                ctx.push_error(format!("expected integer, got {}", value));
                return;
            };
        }
        SettingsOptionType::Integer64 { min, max } => {
            let Some(value) = value.as_i64() else {
                ctx.push_error(format!("expected integer, got {}", value));
                return;
            };
        }
        SettingsOptionType::Channel { types } => {
            let Some(value) = value.as_str() else {
                ctx.push_error(format!("expected string, got {}", value));
                return;
            };
        }
        SettingsOptionType::Channels {
            types,
            max_length,
            min_length,
        } => {
            let Some(value) = value.as_array() else {
                ctx.push_error(format!("expected array, got {}", value));
                return;
            };

            for (i, item) in value.iter().enumerate() {
                let Some(id) = item.as_str() else {
                    ctx.push_field_error(&i.to_string(), format!("expected string, got: {}", item));
                    continue;
                };
            }
        }
        SettingsOptionType::Role { assignable } => {
            let Some(value) = value.as_str() else {
                ctx.push_error(format!("expected string, got {}", value));
                return;
            };
        }
        SettingsOptionType::Roles {
            assignable,
            max_length,
            min_length,
        } => {
            let Some(value) = value.as_array() else {
                ctx.push_error(format!("expected array, got {}", value));
                return;
            };

            for (i, item) in value.iter().enumerate() {
                let Some(id) = item.as_str() else {
                    ctx.push_field_error(&i.to_string(), format!("expected string, got: {}", item));
                    continue;
                };
            }
        }
    }
}

pub(crate) fn validate_settings_option_value_list(
    ctx: &mut ValidationContext,
    value: &serde_json::Value,
    context_data: &SettingsOptionList,
) {
    let serde_json::Value::Array(v) = value else {
        ctx.push_error("Invalid type, expected array");
        return;
    };

    for (i, item) in v.iter().enumerate() {
        ctx.push_index(i);

        let Some(object) = item.as_object() else {
            ctx.push_error("item is not a object");
            ctx.pop_field();
            continue;
        };

        // validate all fields against the template
        for (key, value) in object {
            let Some(definition) = context_data.template.iter().find(|v| &v.name == key) else {
                ctx.push_field_error(key, "unknown field");
                continue;
            };

            ctx.push_field(key);
            validate_settings_option_value_option(ctx, value, definition);
            ctx.pop_field();
        }

        // check required fields
        for field in context_data.template.iter().filter(|v| v.required) {
            if !object.contains_key(&field.name) {
                ctx.push_error(format!("missing required field: {}", field.name))
            }
        }

        ctx.pop_field();
    }
}

impl Validator for CreatePlugin {
    type ContextData = ();

    fn validate(&self, ctx: &mut ValidationContext, _: &()) {
        check_plugin_name(ctx, &self.name);
        check_plugin_short_description(ctx, &self.short_description);
        check_plugin_long_description(ctx, &self.long_description);
    }
}

impl Validator for UpdatePluginMeta {
    type ContextData = ();

    fn validate(&self, ctx: &mut ValidationContext, _: &()) {
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
        ctx.push_field_error(
            "short_description",
            "short description can be max 150 characters long".to_string(),
        );
    }
}

fn check_plugin_long_description(ctx: &mut ValidationContext, long_desc: &str) {
    if long_desc.chars().count() > 4000 {
        ctx.push_field_error(
            "long_description",
            "long description can be max 4000 characters long".to_string(),
        );
    }
}

fn check_plugin_name(ctx: &mut ValidationContext, name: &str) {
    if name.chars().count() > 32 {
        ctx.push_field_error("name", "name can be max 32 characters long".to_string());
    }

    if name.chars().count() < 3 {
        ctx.push_field_error("name", "name can be minimum 3 characters long".to_string());
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^[\w_-]*$"#).unwrap();
    }
    if !RE.is_match(name) {
        ctx.push_field_error(
            "name",
            "name can only contain 'a-z', '-' and '_'".to_string(),
        );
    }
}

fn check_script_name(ctx: &mut ValidationContext, name: &str) {
    if name.chars().count() > 32 {
        ctx.push_field_error("name", "name can be max 32 characters long".to_string());
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^[\w_-]*$"#).unwrap();
    }
    if !RE.is_match(name) {
        ctx.push_field_error(
            "name",
            "name can only contain 'a-z', '-' and '_'".to_string(),
        );
    }
}

pub fn check_script_source(ctx: &mut ValidationContext, field_name: &str, source: &str) {
    if source.len() > 100_000 {
        ctx.push_field_error(field_name, "source can be max 100KiB".to_string());
    }
}
