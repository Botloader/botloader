use std::{rc::Rc, str::FromStr};

use lazy_static::lazy_static;
use regex::Regex;
use runtime_models::internal::script::{
    SettingsOption, SettingsOptionDefinition, SettingsOptionList, SettingsOptionType,
    SettingsOptionValue,
};
use stores::config::{CreatePlugin, CreateScript, Script, UpdatePluginMeta, UpdateScript};
use twilight_model::id::Id;

use crate::{ValidationContext, Validator};

impl Validator for CreateScript {
    type ContextData = ();

    fn validate(&self, ctx: &mut ValidationContext, _: &()) {
        check_script_name(ctx, &self.name);
        check_script_source(ctx, "original_source", &self.original_source);
    }
}

pub struct GuildData {
    pub channels: Vec<twilight_model::channel::Channel>,
    pub roles: Vec<twilight_model::guild::Role>,
}

pub struct ScriptValidationContextData {
    pub script: Script,
    pub guild_data: Option<Rc<GuildData>>,
}

impl Validator for UpdateScript {
    type ContextData = ScriptValidationContextData;

    fn validate(&self, ctx: &mut ValidationContext, ctx_data: &ScriptValidationContextData) {
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
                .or(ctx_data.script.settings_definitions.as_ref());

            ctx.push_field("settings_values".to_owned());

            for value in values {
                if let Some(definitions) = definitions {
                    let Some(definition) = definitions.iter().find(|v| v.name() == value.name)
                    else {
                        ctx.push_field_error(&value.name, "unknown option".to_owned());
                        continue;
                    };

                    ctx.push_field(value.name.clone());
                    value.validate(
                        ctx,
                        &SettingsValueValidationContext {
                            definition: definition.clone(),
                            guild_data: ctx_data.guild_data.clone(),
                        },
                    );
                    ctx.pop_field();
                } else {
                    ctx.push_field_error(&value.name, "unknown option".to_owned());
                }
            }

            ctx.pop_field();
        }
    }
}

pub struct SettingsValueValidationContext {
    pub definition: SettingsOptionDefinition,
    pub guild_data: Option<Rc<GuildData>>,
}

impl Validator for SettingsOptionValue {
    type ContextData = SettingsValueValidationContext;

    fn validate(&self, ctx: &mut ValidationContext, context_data: &Self::ContextData) {
        match &context_data.definition {
            SettingsOptionDefinition::Option(option) => validate_settings_option_value_option(
                ctx,
                &self.value,
                option,
                context_data.guild_data.as_deref(),
            ),
            SettingsOptionDefinition::List(list) => {
                validate_settings_option_value_list(
                    ctx,
                    &self.value,
                    list,
                    context_data.guild_data.as_deref(),
                );
            }
        }
    }
}

pub(crate) fn validate_settings_option_value_option(
    ctx: &mut ValidationContext,
    value: &serde_json::Value,
    definition: &SettingsOption,
    guild_data: Option<&GuildData>,
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

            if let Some(max_length) = max_length {
                if value.chars().count() > *max_length as usize {
                    ctx.push_error(format!("can be a maximum of {} characters", max_length))
                }
            }

            if let Some(min_length) = min_length {
                if value.chars().count() < *min_length as usize {
                    ctx.push_error(format!("has to be at least {} characters", min_length))
                }
            }

            if definition.required && value.is_empty() {
                ctx.push_error("cannot be empty")
            }
        }
        SettingsOptionType::Float { min, max } => {
            let Some(value) = value.as_f64() else {
                ctx.push_error(format!("expected double, got {}", value));
                return;
            };

            if let Some(max) = max {
                if value > *max {
                    ctx.push_error(format!("max {}", max));
                }
            }

            if let Some(min) = min {
                if value < *min {
                    ctx.push_error(format!("min {}", min));
                }
            }
        }
        SettingsOptionType::Integer { min, max } => {
            let Some(value) = value.as_i64() else {
                ctx.push_error(format!("expected integer, got {}", value));
                return;
            };

            if let Some(max) = max {
                if value > max.0 {
                    ctx.push_error(format!("max {}", max));
                }
            }

            if let Some(min) = min {
                if value < min.0 {
                    ctx.push_error(format!("min {}", min));
                }
            }
        }
        SettingsOptionType::Integer64 { min, max } => {
            let Some(value) = value.as_i64() else {
                ctx.push_error(format!("expected integer, got {}", value));
                return;
            };

            if let Some(max) = max {
                if let Ok(max) = max.parse() {
                    if value > max {
                        ctx.push_error(format!("max {}", max));
                    }
                }
            }

            if let Some(min) = min {
                if let Ok(min) = min.parse() {
                    if value > min {
                        ctx.push_error(format!("min {}", min));
                    }
                }
            }
        }
        SettingsOptionType::Channel { types } => {
            let Some(value) = value.as_str() else {
                ctx.push_error(format!("expected string, got {}", value));
                return;
            };

            let Ok(id) = Id::from_str(value) else {
                ctx.push_error(format!("expected snowflake, got: {}", value));
                return;
            };

            if let Some(guild_data) = guild_data {
                let Some(channel) = guild_data.channels.iter().find(|v| v.id == id) else {
                    ctx.push_error(format!("channel not found: {}", value));
                    return;
                };

                if let Some(types) = types {
                    let rt_type =
                        runtime_models::discord::channel::ChannelType::try_from(channel.kind);
                    if let Ok(rt_type) = rt_type {
                        if !types.contains(&rt_type) {
                            ctx.push_error(format!("channel type not allowed: {:?}", channel.kind));
                        }
                    } else {
                        ctx.push_error(format!("channel type not allowed: {:?}", channel.kind));
                    }
                }
            }
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

            if let Some(min) = min_length {
                if *min as usize > value.len() {
                    ctx.push_error(format!("you have to select at least {} channel(s)", min));
                }
            }

            if let Some(max) = max_length {
                if (*max as usize) < value.len() {
                    ctx.push_error(format!("you can select a maximum of {} channel(s)", max));
                }
            }

            for (i, item) in value.iter().enumerate() {
                let Some(id) = item.as_str() else {
                    ctx.push_field_error(&i.to_string(), format!("expected string, got: {}", item));
                    continue;
                };

                if id.parse::<u64>().is_err() {
                    ctx.push_field_error(
                        &i.to_string(),
                        format!("expected snowflake, got: {}", item),
                    );
                    continue;
                }

                let Ok(id) = Id::from_str(id) else {
                    ctx.push_field_error(
                        &i.to_string(),
                        format!("expected snowflake, got: {}", item),
                    );
                    return;
                };

                if let Some(guild_data) = guild_data {
                    let Some(channel) = guild_data.channels.iter().find(|v| v.id == id) else {
                        ctx.push_field_error(&i.to_string(), format!("channel not found: {}", id));
                        return;
                    };

                    if let Some(types) = types {
                        let rt_type =
                            runtime_models::discord::channel::ChannelType::try_from(channel.kind);
                        if let Ok(rt_type) = rt_type {
                            if !types.contains(&rt_type) {
                                ctx.push_error(format!(
                                    "channel type not allowed: {:?}",
                                    channel.kind
                                ));
                            }
                        } else {
                            ctx.push_error(format!("channel type not allowed: {:?}", channel.kind));
                        }
                    }
                }
            }
        }
        SettingsOptionType::Role { assignable: _ } => {
            let Some(value) = value.as_str() else {
                ctx.push_error(format!("expected string, got {}", value));
                return;
            };

            if value.parse::<u64>().is_err() {
                ctx.push_error(format!("expected snowflake, got: {}", value));
            }
        }
        SettingsOptionType::Roles {
            assignable: _,
            max_length,
            min_length,
        } => {
            let Some(value) = value.as_array() else {
                ctx.push_error(format!("expected array, got {}", value));
                return;
            };

            if let Some(min) = min_length {
                if *min as usize > value.len() {
                    ctx.push_error(format!("you have to select at least {} role(s)", min));
                }
            }

            if let Some(max) = max_length {
                if (*max as usize) < value.len() {
                    ctx.push_error(format!("you can select a maximum of {} role(s)", max));
                }
            }

            for (i, item) in value.iter().enumerate() {
                let Some(id) = item.as_str() else {
                    ctx.push_field_error(&i.to_string(), format!("expected string, got: {}", item));
                    continue;
                };

                if id.parse::<u64>().is_err() {
                    ctx.push_field_error(
                        &i.to_string(),
                        format!("expected snowflake, got: {}", item),
                    );
                    continue;
                }
            }
        }
        SettingsOptionType::Boolean => {
            if let Some(value_str) = value.as_str() {
                let lower = value_str.to_lowercase();
                if lower != "true"
                    && lower != "false"
                    && lower != "0"
                    && lower != "1"
                    && lower != "yes"
                    && lower != "no"
                {
                    ctx.push_error(format!(
                        "Unexpected boolean value, got {} expected one of true,false,yes,no,0,1",
                        value
                    ));
                }
            }

            if let Some(_) = value.as_bool() {
                return;
            }

            if let Some(_) = value.as_i64() {
                return;
            }

            ctx.push_error(format!("value is not a valid boolean"));
        }
    }
}

pub(crate) fn validate_settings_option_value_list(
    ctx: &mut ValidationContext,
    value: &serde_json::Value,
    context_data: &SettingsOptionList,
    guild_data: Option<&GuildData>,
) {
    let serde_json::Value::Array(v) = value else {
        ctx.push_error("Invalid type, expected array");
        return;
    };

    for (i, item) in v.iter().enumerate() {
        ctx.push_index(i);

        // let Some(arr) = item.as_array() else {
        //     ctx.push_error("item is not a object");
        //     ctx.pop_field();
        //     continue;
        // };

        match serde_json::from_value::<Vec<SettingsOptionValue>>(item.clone()) {
            Ok(fields) => {
                for field in &fields {
                    let Some(definition) =
                        context_data.template.iter().find(|v| v.name == field.name)
                    else {
                        ctx.push_field_error(&field.name, "unknown field");
                        continue;
                    };

                    ctx.push_field(&field.name);
                    validate_settings_option_value_option(
                        ctx,
                        &field.value,
                        definition,
                        guild_data,
                    );
                    ctx.pop_field();
                }

                // check required fields
                for field_def in context_data.template.iter().filter(|v| v.required) {
                    if !fields.iter().any(|v| v.name == field_def.name) {
                        ctx.push_field_error(&field_def.name, "required")
                    }
                }
            }
            Err(err) => {
                ctx.push_error(format!("failed deserializing fields: {}", err));
            }
        }

        // validate all fields against the template
        // for (key, value) in object {
        //     let Some(definition) = context_data.template.iter().find(|v| &v.name == key) else {
        //         ctx.push_field_error(key, "unknown field");
        //         continue;
        //     };

        //     ctx.push_field(key);
        //     validate_settings_option_value_option(ctx, value, definition);
        //     ctx.pop_field();
        // }

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
