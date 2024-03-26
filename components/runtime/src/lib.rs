use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use common::DiscordConfig;
use deno_core::{op2, Extension, Op, OpState, ResourceId, ResourceTable};
use guild_logger::{entry::CreateLogEntry, GuildLogSender};
use runtime_models::internal::script::{ScriptMeta, SettingsOptionValue};
use stores::{config::PremiumSlotTier, Db};
use tokio::sync::mpsc;
use tracing::info;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use vm::{AnyError, JsValue};

use crate::limits::RateLimiters;

pub mod extensions;
pub mod jsmodules;
pub mod limits;

pub fn create_extensions(ctx: CreateRuntimeContext) -> Vec<Extension> {
    let mut http_client_builder: reqwest::ClientBuilder = reqwest::ClientBuilder::new();
    if let Some(proxy_addr) = &ctx.script_http_client_proxy {
        info!("using http client proxy: {}", proxy_addr);
        let proxy = reqwest::Proxy::all(proxy_addr).expect("valid http proxy address");
        http_client_builder = http_client_builder.proxy(proxy);
    } else {
        #[cfg(not(debug_assertions))]
        tracing::warn!("no proxy set in release!");
    }

    let http_client = http_client_builder.build().expect("valid http client");
    let premium_tier = *ctx.premium_tier.read().unwrap();
    let core_ctx = CoreRuntimeContext {
        event_tx: ctx.event_tx.clone(),
        settings_values: ctx.settings_values,
    };

    if let Some(guild_id) = ctx.guild_id {
        let rt_ctx = RuntimeContext {
            guild_id,
            bot_state: ctx.bot_state.clone(),
            discord_config: ctx.discord_config.clone(),
            guild_logger: ctx.guild_logger.clone(),
            script_http_client_proxy: ctx.script_http_client_proxy.clone(),
            event_tx: ctx.event_tx.clone(),
            premium_tier,
            main_tokio_runtime: ctx.main_tokio_runtime,

            db: ctx.db,
        };

        vec![
            bl_script_core::init_ops_and_esm(core_ctx, rt_ctx, http_client, premium_tier),
            extensions::storage::bl_storage::init_ops_and_esm(),
            extensions::discord::bl_discord::init_ops_and_esm(),
            extensions::console::bl_console::init_ops_and_esm(),
            extensions::httpclient::bl_http::init_ops_and_esm(),
            extensions::tasks::bl_tasks::init_ops_and_esm(),
        ]
    } else {
        vec![
            bl_script_core_no_guild::init_ops_and_esm(core_ctx, http_client, premium_tier),
            extensions::storage::bl_storage::init_ops_and_esm(),
            extensions::discord::bl_discord::init_ops_and_esm(),
            extensions::console::bl_console::init_ops_and_esm(),
            extensions::httpclient::bl_http::init_ops_and_esm(),
            extensions::tasks::bl_tasks::init_ops_and_esm(),
        ]
    }
}

deno_core::extension!(
    bl_script_core,
    ops = [
        op_botloader_script_start,
        op_get_current_bot_user,
        op_get_current_guild_id,
        op_get_run_mode,
        op_get_settings,
    ],
    options = {
        ctx: CoreRuntimeContext,
        rt_ctx: RuntimeContext,
        http_client: reqwest::Client,
        premium_tier: Option<PremiumSlotTier>,
    },
    middleware = |op_decl|match op_decl.name {
        // we have our own custom print function
        "op_print" => disabled_op::DECL,
        "op_wasm_streaming_feed" => disabled_op::DECL,
        "op_wasm_streaming_set_url" => disabled_op::DECL,
        _ => op_decl,
    },
    state = |state, options| {
        state.put(options.ctx);
        state.put(options.rt_ctx);
        state.put(options.http_client);
        state.put(Rc::new(RateLimiters::new(options.premium_tier)));
    },

);

deno_core::extension!(
    bl_script_core_no_guild,
    ops = [
        op_botloader_script_start,
        op_get_current_bot_user,
        op_get_current_guild_id,
        op_get_run_mode,
        op_get_settings,
    ],
    options = {
        ctx: CoreRuntimeContext,
        http_client: reqwest::Client,
        premium_tier: Option<PremiumSlotTier>,
    },
    middleware = |op_decl|match op_decl.name {
        // we have our own custom print function
        "op_print" => disabled_op::DECL,
        "op_wasm_streaming_feed" => disabled_op::DECL,
        "op_wasm_streaming_set_url" => disabled_op::DECL,
        _ => op_decl,
    },
    state = |state, options| {
        state.put(options.ctx);
        state.put(options.http_client);
        state.put(Rc::new(RateLimiters::new(options.premium_tier)));
    },

);

pub fn in_mem_source_load_fn(src: &'static str) -> Box<dyn Fn() -> Result<String, AnyError>> {
    Box::new(move || Ok(src.to_string()))
}

#[op2(fast)]
pub fn disabled_op() -> Result<(), AnyError> {
    Err(anyhow::anyhow!("this op is disabled"))
}

#[derive(Clone)]
pub struct CoreRuntimeContext {
    pub event_tx: mpsc::UnboundedSender<RuntimeEvent>,
    pub settings_values: Vec<ScriptSettingsValues>,
}

#[derive(Clone)]
pub struct RuntimeContext {
    pub guild_id: Id<GuildMarker>,
    pub bot_state: dbrokerapi::state_client::Client,
    pub discord_config: Arc<DiscordConfig>,
    pub guild_logger: GuildLogSender,
    pub script_http_client_proxy: Option<String>,
    pub event_tx: mpsc::UnboundedSender<RuntimeEvent>,
    pub premium_tier: Option<PremiumSlotTier>,
    pub main_tokio_runtime: tokio::runtime::Handle,

    pub db: Db,
}

#[derive(Clone)]
pub struct CreateRuntimeContext {
    pub guild_id: Option<Id<GuildMarker>>,
    pub bot_state: dbrokerapi::state_client::Client,
    pub discord_config: Arc<DiscordConfig>,
    pub guild_logger: GuildLogSender,
    pub script_http_client_proxy: Option<String>,
    pub event_tx: mpsc::UnboundedSender<RuntimeEvent>,
    pub premium_tier: Arc<RwLock<Option<PremiumSlotTier>>>,
    pub main_tokio_runtime: tokio::runtime::Handle,
    pub settings_values: Vec<ScriptSettingsValues>,

    pub db: Db,
}

#[derive(Clone)]
pub struct ScriptSettingsValues {
    pub script_id: u64,
    pub settings_values: Vec<SettingsOptionValue>,
}

#[op2]
#[serde]
pub fn op_get_current_bot_user(
    state: &mut OpState,
) -> Result<runtime_models::internal::user::User, AnyError> {
    let ctx = state.borrow::<RuntimeContext>();
    Ok(ctx.discord_config.bot_user.clone().into())
}

#[op2]
#[string]
pub fn op_get_current_guild_id(state: &mut OpState) -> Result<String, AnyError> {
    let ctx = state.borrow::<RuntimeContext>();
    Ok(ctx.guild_id.to_string())
}

#[op2]
#[string]
pub fn op_get_run_mode(state: &mut OpState) -> String {
    if state.has::<RuntimeContext>() {
        "normal".to_string()
    } else {
        "validation".to_string()
    }
}

#[op2]
pub fn op_botloader_script_start(
    state: &mut OpState,
    #[serde] args: JsValue,
) -> Result<(), AnyError> {
    let des: ScriptMeta = serde_json::from_value(args)?;

    info!(
        "running script! {}, commands: {}",
        des.script_id.0,
        des.commands.len() + des.command_groups.len()
    );

    let ctx = state.borrow::<RuntimeContext>();

    if let Err(err) = validate_script_meta(&des) {
        // error!(%err, "script meta validation failed");
        ctx.guild_logger.log(CreateLogEntry::script_error(
            format!("script meta validation failed: {err}"),
            format!("{}", des.script_id),
            None,
        ));
        return Err(err);
    }

    let _ = ctx.event_tx.send(RuntimeEvent::ScriptStarted(des));

    Ok(())
}

#[op2]
#[serde]
pub fn op_get_settings(
    state: &mut OpState,
    #[number] script_id: u64,
) -> Result<Vec<SettingsOptionValue>, AnyError> {
    let core_ctx = state.borrow::<CoreRuntimeContext>();
    let Some(script_settings_entry) = core_ctx
        .settings_values
        .iter()
        .find(|v| v.script_id == script_id)
    else {
        return Ok(Vec::new());
    };

    Ok(script_settings_entry.settings_values.clone())
}

pub(crate) fn validate_script_meta(meta: &ScriptMeta) -> Result<(), anyhow::Error> {
    let mut out_buf = String::new();

    for command in &meta.commands {
        if let Err(verrs) = validation::validate(command, &()) {
            for verr in verrs {
                out_buf.push_str(format!("\ncommand {}: {}", command.name, verr).as_str());
            }
        }
    }

    for group in &meta.command_groups {
        if let Err(verrs) = validation::validate(group, &()) {
            for verr in verrs {
                out_buf.push_str(format!("\ncommand group {}: {}", group.name, verr).as_str());
            }
        }
    }

    for option in &meta.settings {
        if let Err(validation_errors) = validation::validate(option, &()) {
            for error in validation_errors {
                out_buf.push_str(format!("\nsettings options: {}", error).as_str());
            }
        }
    }

    if out_buf.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("script validation failed: {}", out_buf))
    }
}

pub fn try_insert_resource_table<T: deno_core::Resource>(
    table: &mut ResourceTable,
    v: T,
) -> Result<ResourceId, AnyError> {
    let count = table.names().count();

    // todo: give this a proper limit
    if count > 100 {
        return Err(anyhow::anyhow!(
            "exhausted resource table limit, make sure to close your resources when you're done \
             with them."
        ));
    }

    Ok(table.add(v))
}

pub enum RuntimeEvent {
    ScriptStarted(ScriptMeta),
    NewTaskScheduled,
    InvalidRequestsExceeded,
}

impl RuntimeEvent {
    pub fn span_name(&self) -> &'static str {
        match self {
            RuntimeEvent::ScriptStarted(_) => "RuntimeEvent::ScriptStarted",
            RuntimeEvent::NewTaskScheduled => "RuntimeEvent::NewTaskScheduled",
            RuntimeEvent::InvalidRequestsExceeded => "RuntimeEvent::InvalidRequestsExceeded",
        }
    }
}

pub fn get_rt_ctx(state: &Rc<RefCell<OpState>>) -> RuntimeContext {
    let state = state.borrow();
    state.borrow::<RuntimeContext>().clone()
}
