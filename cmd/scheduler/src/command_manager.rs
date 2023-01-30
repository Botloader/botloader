use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use common::DiscordConfig;
use guild_logger::{GuildLogger, LogEntry};
use stores::config::Script;
use tokio::sync::mpsc;
use tracing::{error, info};
use twilight_model::application::command::{
    Command as TwilightCommand, CommandOption as TwilightCommandOption,
    CommandOptionType as TwilightCommandOptionType, CommandType as TwilightCommandType,
};

use runtime_models::internal::script::{Command, CommandGroup, ScriptMeta};
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

use crate::scheduler;

#[derive(Clone, Debug)]
pub struct Handle {
    send_loaded_script: mpsc::UnboundedSender<LoadedScript>,
}

impl Handle {
    pub fn send(&self, script: LoadedScript) {
        self.send_loaded_script.send(script).ok();
    }
}

pub struct Manager {
    config_store: Arc<dyn scheduler::Store>,
    discord_config: Arc<DiscordConfig>,
    rcv_loaded_script: mpsc::UnboundedReceiver<LoadedScript>,
    pending_checks: Vec<PendingCheckGroup>,
    guild_logger: GuildLogger,

    cached_commands: HashMap<Id<GuildMarker>, String>,
}

pub fn create_manager_pair(
    config_store: Arc<dyn scheduler::Store>,
    discord_config: Arc<DiscordConfig>,
    guild_logger: GuildLogger,
) -> (Manager, Handle) {
    let (send, rcv) = mpsc::unbounded_channel();

    (
        Manager {
            config_store,
            discord_config,
            rcv_loaded_script: rcv,
            pending_checks: Vec::new(),
            guild_logger,
            cached_commands: HashMap::new(),
        },
        Handle {
            send_loaded_script: send,
        },
    )
}

impl Manager {
    pub async fn run(mut self) {
        let mut ticker = tokio::time::interval(Duration::from_secs(10));
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    self.handle_tick().await;
                },
                evt = self.rcv_loaded_script.recv() => {
                    if let Some(evt) = evt{
                        self.handle_evt(evt).await;
                    }else{
                        info!("all receivers dead, shutting down contrib manager");
                        return;
                    }
                },
            }
        }
    }

    async fn handle_evt(&mut self, evt: LoadedScript) {
        if let Some(item) = self
            .pending_checks
            .iter_mut()
            .find(|e| e.guild_id == evt.guild_id)
        {
            // guild queue already exists

            // check if this script is already in the queue, and if so overwrite it
            if let Some(qi) = item
                .items
                .iter_mut()
                .find(|v| v.meta.script_id.0 == evt.meta.script_id.0)
            {
                *qi = evt
            } else {
                item.items.push(evt);
            }
        } else {
            // create a new guild queue
            self.pending_checks.push(PendingCheckGroup {
                guild_id: evt.guild_id,
                items: vec![evt],
                started: Instant::now(),
            })
        }
    }

    async fn handle_tick(&mut self) {
        let old_list = std::mem::take(&mut self.pending_checks);

        for item in old_list {
            if item.started.elapsed() > Duration::from_secs(10) {
                if self.process_item(&item).await.is_err() {
                    // add back to queue if processing failed
                    self.pending_checks.push(item);
                }
            } else {
                self.pending_checks.push(item);
            }
        }
    }

    async fn process_item(&mut self, item: &PendingCheckGroup) -> Result<(), ()> {
        // TODO: only do this when they have actually changed
        // this will be more important when we need to scale it
        // but needs to be done before before we go beyond the 100 server mark
        self.update_guild_commands(item.guild_id).await?;
        Ok(())
    }

    async fn update_guild_commands(&mut self, guild_id: Id<GuildMarker>) -> Result<(), ()> {
        let all_guild_scripts = self
            .config_store
            .list_scripts(guild_id)
            .await
            .map_err(|err| {
                error!(%err, "failed retrieving guild scripts");
            })?;

        let merged = merge_script_commands(all_guild_scripts);

        let serialized = serde_json::to_string(&merged).unwrap();
        if let Some(current) = self.cached_commands.get(&guild_id) {
            if current == &serialized {
                self.guild_logger.log(LogEntry::info(
                    guild_id,
                    format!(
                        "skipped updating commands as there was no changes, top level commands: {}",
                        merged.len()
                    ),
                ));

                return Ok(());
            }
        }

        self.guild_logger.log(LogEntry::info(
            guild_id,
            format!(
                "updating guild commands, top level commands: {}",
                merged.len()
            ),
        ));

        let interaction_client = self.discord_config.interaction_client();

        if let Err(err) = interaction_client
            .set_guild_commands(guild_id, &merged)
            .await
        {
            error!(%err, "failed updating guild commands");

            self.guild_logger.log(LogEntry::error(
                guild_id,
                "failed updating guild commands".to_string(),
            ));
            // TODO: for now this returns an ok, in the future once we have
            // more validation we could return an err here and have it retry
            // (but not for client errors)
        }

        self.cached_commands.insert(guild_id, serialized);

        Ok(())
    }
}

static GROUP_DESC_PLACEHOLDER: &str = "no description";

struct PendingCheckGroup {
    guild_id: Id<GuildMarker>,
    started: Instant,
    items: Vec<LoadedScript>,
}

pub fn to_twilight_commands(
    guild_id: Id<GuildMarker>,
    commands: &[Command],
    groups: &[CommandGroup],
) -> Vec<TwilightCommand> {
    // handle top level commands
    let mut result = commands
        .iter()
        .filter(|c| c.group.is_none())
        .map(|cmd| TwilightCommand {
            name: cmd.name.clone(),
            description: if matches!(
                cmd.kind,
                runtime_models::internal::interaction::CommandType::Chat
            ) {
                cmd.description.clone()
            } else {
                String::new()
            },
            application_id: None,
            options: cmd.options.iter().map(|opt| opt.clone().into()).collect(),
            guild_id: None,
            id: None,
            kind: cmd.kind.into(),
            version: Id::new(1),
            dm_permission: None,
            default_member_permissions: None,
            description_localizations: Default::default(),
            name_localizations: Default::default(),
            nsfw: None,
        })
        .collect::<Vec<_>>();

    let mut groups = groups
        .iter()
        .map(|cg| group_to_twilight_command(guild_id, cg))
        .collect::<Vec<_>>();

    // add the commands to the groups and sub groups
    for cmd in commands.iter() {
        if let Some(cmd_group) = &cmd.group {
            // find (or create) the group
            let group = match groups.iter_mut().find(|g| *cmd_group == g.name) {
                Some(g) => g,
                None => {
                    // group not found, make a new one
                    groups.push(make_unknown_group(guild_id, cmd_group.clone()));

                    // return mut reference to the new group
                    let len = groups.len();
                    &mut groups[len - 1]
                }
            };

            // check if this belongs to a subgroup
            if let Some(cmd_sub_group) = &cmd.sub_group {
                match group.options.iter_mut().find(|sg| {
                    &sg.name == cmd_sub_group

                    // matches!(sg, TwilightCommandOption::SubCommandGroup(OptionsCommandOptionData {
                    //     name,
                    //     ..
                    // }) if name == cmd_sub_group)
                }) {
                    Some(g) => {
                        // add the cmd to the existing sub group
                        if let Some(options) = &mut g.options {
                            options.push(cmd.clone().into());
                        } else {
                            g.options = Some(vec![cmd.clone().into()]);
                        }
                    }
                    None => {
                        // sub group not found, make a new one and add the cmd to it
                        group.options.push(TwilightCommandOption {
                            name: cmd_sub_group.clone(),
                            description: GROUP_DESC_PLACEHOLDER.to_string(),
                            options: Some(vec![cmd.clone().into()]),
                            description_localizations: Default::default(),
                            name_localizations: Default::default(),
                            kind: TwilightCommandOptionType::SubCommandGroup,
                            autocomplete: None,
                            channel_types: None,
                            choices: None,
                            max_length: None,
                            max_value: None,
                            min_length: None,
                            min_value: None,
                            required: None,
                        });
                        // group.options.push(TwilightCommandOption::SubCommandGroup(
                        //     OptionsCommandOptionData {
                        //         name: cmd_sub_group.clone(),
                        //         description: GROUP_DESC_PLACEHOLDER.to_string(),
                        //         options: vec![cmd.clone().into()],
                        //         description_localizations: Default::default(),
                        //         name_localizations: Default::default(),
                        //     },
                        // ));
                    }
                };
            } else {
                // belongs to normal group (not sub group)
                group.options.push(cmd.clone().into())
            }
        }
    }

    result.append(&mut groups);
    result
}

fn make_unknown_group(guild_id: Id<GuildMarker>, name: String) -> TwilightCommand {
    TwilightCommand {
        application_id: None,
        description: GROUP_DESC_PLACEHOLDER.to_string(),
        guild_id: Some(guild_id),
        id: None,
        kind: TwilightCommandType::ChatInput,
        options: Vec::new(),
        name,
        version: Id::new(1),
        default_member_permissions: None,
        dm_permission: None,
        description_localizations: Default::default(),
        name_localizations: Default::default(),
        nsfw: None,
    }
}

pub fn group_to_twilight_command(
    guild_id: Id<GuildMarker>,
    group: &CommandGroup,
) -> TwilightCommand {
    // handle sub groups
    let opts = group
        .sub_groups
        .iter()
        .map(|sg| TwilightCommandOption {
            name: sg.name.clone(),
            description: sg.description.clone(),
            options: None,
            description_localizations: Default::default(),
            name_localizations: Default::default(),
            kind: TwilightCommandOptionType::SubCommandGroup,
            autocomplete: None,
            channel_types: None,
            choices: None,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            required: None,
        })
        .collect::<Vec<_>>();

    TwilightCommand {
        application_id: None,
        guild_id: Some(guild_id),
        description: group.description.clone(),
        id: None,
        kind: TwilightCommandType::ChatInput,
        name: group.name.clone(),
        options: opts,
        version: Id::new(1),
        default_member_permissions: None,
        dm_permission: None,
        description_localizations: Default::default(),
        name_localizations: Default::default(),
        nsfw: None,
    }
}

fn merge_script_commands(scripts: Vec<Script>) -> Vec<TwilightCommand> {
    let mut result = Vec::new();

    for script in scripts {
        for cmd in script.contributes.commands {
            if let Some(existing) = result
                .iter_mut()
                .find(|v: &&mut TwilightCommand| v.name == cmd.name)
            {
                merge_command(existing, cmd);
            } else {
                result.push(cmd);
            }
        }
    }

    result
}

// merges src into dst
fn merge_command(dst: &mut TwilightCommand, src: TwilightCommand) {
    if dst.description == GROUP_DESC_PLACEHOLDER && src.description != GROUP_DESC_PLACEHOLDER {
        dst.description = src.description;
    }

    for opt in &dst.options {
        if !matches!(
            opt.kind,
            TwilightCommandOptionType::SubCommand | TwilightCommandOptionType::SubCommandGroup
        ) {
            // We can only merge sub commands
            return;
        }
    }

    for opt in src.options {
        if !matches!(
            opt.kind,
            TwilightCommandOptionType::SubCommand | TwilightCommandOptionType::SubCommandGroup
        ) {
            // We can only merge sub commands
            return;
        }

        let src_opt_name = &opt.name;
        if let Some(dst_opt) = dst.options.iter_mut().find(|v| &v.name == src_opt_name) {
            // we need to merge these options
            match (dst_opt.kind, opt.kind) {
                (
                    TwilightCommandOptionType::SubCommandGroup,
                    TwilightCommandOptionType::SubCommandGroup,
                ) => merge_subgroups(dst_opt, opt),
                _ => {
                    // we can only merge subgroups, how would we merge subcommands?
                    continue;
                }
            }
        } else {
            // no conflict
            dst.options.push(opt);
        }
    }
}

fn merge_subgroups(dst: &mut TwilightCommandOption, src: TwilightCommandOption) {
    if dst.description == GROUP_DESC_PLACEHOLDER && src.description != GROUP_DESC_PLACEHOLDER {
        dst.description = src.description;
    }

    for opt in src.options.as_ref().unwrap_or(&Vec::new()) {
        if let Some(dst_options) = &mut dst.options {
            if dst_options.iter().any(|v| v.name == opt.name) {
                // we command merge sub commands themselves
                continue;
            }

            // but we can add them to the group if there is no conflict!
            dst_options.push(opt.clone());
        } else {
            dst.options = src.options.clone();
        }
    }
}

pub struct LoadedScript {
    pub guild_id: Id<GuildMarker>,
    pub meta: ScriptMeta,
}
