use std::{cell::RefCell, num::NonZeroU32, rc::Rc, sync::Arc};

use common::DiscordConfig;
use deno_core::{op, Extension, OpState, ResourceId, ResourceTable};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota,
};
use guild_logger::{GuildLogger, LogEntry};
use runtime_models::internal::script::ScriptMeta;
use stores::{bucketstore::BucketStore, config::ConfigStore, timers::TimerStore};
use tokio::sync::mpsc;
use tracing::{info, instrument};
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use vm::{vm::VmRole, AnyError, JsValue};

pub mod extensions;
pub mod jsmodules;

pub fn create_extensions(ctx: CreateRuntimeContext) -> Vec<Extension> {
    let mut http_client_builder = reqwest::ClientBuilder::new();
    if let Some(proxy_addr) = &ctx.script_http_client_proxy {
        info!("using http client proxy: {}", proxy_addr);
        let proxy = reqwest::Proxy::all(proxy_addr).expect("valid http proxy address");
        http_client_builder = http_client_builder.proxy(proxy);
    } else {
        #[cfg(not(debug_assertions))]
        tracing::warn!("no proxy set in release!");
    }
    let http_client = http_client_builder.build().expect("valid http client");

    let core_extension = Extension::builder()
        .ops(vec![
            // botloader stuff
            op_botloader_script_start::decl(),
            op_get_current_bot_user::decl(),
            op_get_current_guild_id::decl(),
        ])
        .state(move |state| {
            state.put(RuntimeContext {
                guild_id: ctx.guild_id,
                bot_state: ctx.bot_state.clone(),
                discord_config: ctx.discord_config.clone(),
                role: ctx.role,
                guild_logger: ctx.guild_logger.clone(),
                script_http_client_proxy: ctx.script_http_client_proxy.clone(),
                event_tx: ctx.event_tx.clone(),

                bucket_store: ctx.bucket_store.clone(),
                config_store: ctx.config_store.clone(),
                timer_store: ctx.timer_store.clone(),
            });
            state.put(http_client.clone());

            state.put(Rc::new(RateLimiters::new()));

            Ok(())
        })
        .middleware(|deno_op| match deno_op.name {
            // we have our own custom print function
            "op_print" => disabled_op::decl(),
            "op_wasm_streaming_feed" => disabled_op::decl(),
            "op_wasm_streaming_set_url" => disabled_op::decl(),
            _ => deno_op,
        })
        .build();

    vec![
        core_extension,
        extensions::storage::extension(),
        extensions::discord::extension(),
        extensions::console::extension(),
        extensions::httpclient::extension(),
        extensions::tasks::extension(),
    ]
}

pub fn in_mem_source_load_fn(src: &'static str) -> Box<dyn Fn() -> Result<String, AnyError>> {
    Box::new(move || Ok(src.to_string()))
}

#[op]
pub fn disabled_op() -> Result<(), AnyError> {
    Err(anyhow::anyhow!("this op is disabled"))
}

#[derive(Clone)]
pub struct RuntimeContext {
    pub guild_id: Id<GuildMarker>,
    pub bot_state: dbrokerapi::state_client::Client,
    pub discord_config: Arc<DiscordConfig>,
    pub role: VmRole,
    pub guild_logger: GuildLogger,
    pub script_http_client_proxy: Option<String>,
    pub event_tx: mpsc::UnboundedSender<RuntimeEvent>,

    pub bucket_store: Arc<dyn BucketStore>,
    pub config_store: Arc<dyn ConfigStore>,
    pub timer_store: Arc<dyn TimerStore>,
}

#[derive(Clone)]
pub struct CreateRuntimeContext {
    pub guild_id: Id<GuildMarker>,
    pub bot_state: dbrokerapi::state_client::Client,
    pub discord_config: Arc<DiscordConfig>,
    pub role: VmRole,
    pub guild_logger: GuildLogger,
    pub script_http_client_proxy: Option<String>,
    pub event_tx: mpsc::UnboundedSender<RuntimeEvent>,

    pub bucket_store: Arc<dyn BucketStore>,
    pub config_store: Arc<dyn ConfigStore>,
    pub timer_store: Arc<dyn TimerStore>,
}

#[op]
pub fn op_get_current_bot_user(
    state: &mut OpState,
) -> Result<runtime_models::internal::user::User, AnyError> {
    let ctx = state.borrow::<RuntimeContext>();
    Ok(ctx.discord_config.bot_user.clone().into())
}

#[op]
pub fn op_get_current_guild_id(state: &mut OpState) -> Result<String, AnyError> {
    let ctx = state.borrow::<RuntimeContext>();
    Ok(ctx.guild_id.to_string())
}

#[op]
pub fn op_botloader_script_start(state: &mut OpState, args: JsValue) -> Result<(), AnyError> {
    let des: ScriptMeta = serde_json::from_value(args)?;

    info!(
        "running script! {}, commands: {}",
        des.script_id.0,
        des.commands.len() + des.command_groups.len()
    );

    let ctx = state.borrow::<RuntimeContext>();

    if let Err(err) = validate_script_meta(&des) {
        // error!(%err, "script meta valication failed");
        ctx.guild_logger.log(LogEntry::script_error(
            ctx.guild_id,
            format!("script meta validation failed: {}", err),
            format!("{}", des.script_id),
            None,
        ));
        return Err(err);
    }

    let _ = ctx.event_tx.send(RuntimeEvent::ScriptStarted(des));

    Ok(())
}

pub(crate) fn validate_script_meta(meta: &ScriptMeta) -> Result<(), anyhow::Error> {
    let mut outbuf = String::new();

    for command in &meta.commands {
        if let Err(verrs) = validation::validate(command) {
            for verr in verrs {
                outbuf.push_str(format!("\ncommand {}: {}", command.name, verr).as_str());
            }
        }
    }

    for group in &meta.command_groups {
        if let Err(verrs) = validation::validate(group) {
            for verr in verrs {
                outbuf.push_str(format!("\ncommand group {}: {}", group.name, verr).as_str());
            }
        }
    }

    if outbuf.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("failed validating script: {}", outbuf))
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

pub type RateLimiter = governor::RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

#[derive(Debug)]
pub enum RatelimiterType {
    UserHttp,
    Tasks,
}

pub struct RateLimiters {
    user_http: RateLimiter,
    tasks: RateLimiter,
}

impl RateLimiters {
    #[instrument(skip(op_state))]
    async fn ops_until_ready(op_state: Rc<RefCell<OpState>>, typ: RatelimiterType) {
        // is the block here needed? i honestly have no idea but better to be on the safe side i guess
        let ratelimiters = { op_state.borrow().borrow::<Rc<RateLimiters>>().clone() };
        ratelimiters.until_ready(typ).await
    }
}

impl RateLimiters {
    fn new() -> Self {
        Self {
            user_http: RateLimiter::direct(Quota::per_second(NonZeroU32::new(2).unwrap())),
            tasks: RateLimiter::direct(Quota::per_second(NonZeroU32::new(2).unwrap())),
        }
    }

    async fn until_ready(&self, typ: RatelimiterType) {
        match typ {
            RatelimiterType::UserHttp => self.user_http.until_ready().await,
            RatelimiterType::Tasks => self.tasks.until_ready().await,
        }
    }
}

pub enum RuntimeEvent {
    ScriptStarted(ScriptMeta),
    NewTaskScheduled,
    InvalidRequestsExceeded,
}

pub fn get_rt_ctx(state: &Rc<RefCell<OpState>>) -> RuntimeContext {
    let state = state.borrow();
    state.borrow::<RuntimeContext>().clone()
}
