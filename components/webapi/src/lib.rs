use std::sync::Arc;

use axum::{
    error_handling::HandleErrorLayer,
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    BoxError, Router,
};

use routes::auth::AuthHandlers;
use stores::inmemory::web::InMemoryCsrfStore;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{error, info, Level};
use twilight_model::id::{marker::GuildMarker, Id};

mod app_state;
mod errors;
mod middlewares;
mod news_poller;
mod routes;
mod util;

use crate::middlewares::{
    bl_admin_only::bl_admin_only_mw, plugins::plugin_middleware,
    require_current_guild_admin_middleware, CorsLayer, NoSession, OptionalSession, SessionLayer,
};
use crate::{errors::ApiErrorResponse, middlewares::current_guild_injector_middleware};

type ApiResult<T> = Result<T, ApiErrorResponse>;

pub async fn run(
    common_conf: common::config::RunConfig,
    web_conf: WebConfig,
    setup_tracing_and_metrics: bool,
) {
    if setup_tracing_and_metrics {
        common::setup_tracing(&common_conf, "webapi");
        common::setup_metrics("0.0.0.0:7801");
    }

    let conf = common_conf.clone();

    info!("starting...");

    let state = app_state::init_app_state(&common_conf, &web_conf).await;

    let auth_handler =
        routes::auth::AuthHandlers::new(state.db.clone(), InMemoryCsrfStore::default());

    let session_layer = SessionLayer::new(state.db.clone(), state.discord_oauth_client.clone());
    let require_auth_layer = session_layer.require_auth_layer();
    let client_cache = session_layer.oauth_api_client_cache.clone();

    let common_middleware_stack = ServiceBuilder::new()
        .layer(axum_metrics_layer::MetricsLayer {
            name_prefix: "bl.webapi",
        })
        .layer(HandleErrorLayer::new(handle_mw_err_internal_err))
        // .layer(Extension(ConfigData {
        //     oauth_client: oatuh_client,
        //     frontend_base: common_conf.frontend_host_base.clone(),
        // }))
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(Level::INFO)))
        // .layer(Extension(bot_rpc_client))
        .layer(Extension(Arc::new(auth_handler)))
        // .layer(Extension(config_store))
        // .layer(Extension(session_store.clone()))
        .layer(Extension(client_cache))
        // .layer(Extension(news_handle))
        // .layer(Extension(discord_config))
        // .layer(Extension(state_client))
        .layer(Extension(Arc::new(state.stripe_client.clone())))
        .layer(Extension(OptionalSession::none()))
        .layer(session_layer)
        .layer(CorsLayer {
            run_config: conf.clone(),
        });

    let auth_guild_mw_stack = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(current_guild_injector_middleware))
        .layer(axum::middleware::from_fn(
            require_current_guild_admin_middleware,
        ));

    let authorized_admin_routes = Router::new()
        .route("/vm_workers", get(routes::admin::get_worker_statuses))
        .route(
            "/guild/:guild_id/status",
            get(routes::admin::get_guild_status),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            bl_admin_only_mw,
        ));

    let authorized_api_guild_routes = Router::new()
        .route("/reload_vm", post(routes::vm::reload_guild_vm))
        .route("/settings", get(routes::guilds::get_guild_settings))
        .route(
            "/premium_slots",
            get(routes::guilds::get_guild_premium_slots),
        )
        .route(
            "/scripts",
            get(routes::scripts::get_all_guild_scripts).put(routes::scripts::create_guild_script),
        )
        .route(
            "/scripts_with_plugins",
            get(routes::scripts::get_all_guild_scripts_with_plugins),
        )
        .route(
            "/scripts/:script_id",
            patch(routes::scripts::update_guild_script)
                .delete(routes::scripts::delete_guild_script),
        )
        .route(
            "/scripts/:script_id/validate_settings",
            post(routes::scripts::validate_script_settings),
        )
        .route(
            "/scripts/:script_id/update_plugin",
            post(routes::scripts::update_script_plugin),
        )
        .route("/add_plugin", post(routes::plugins::guild_add_plugin))
        .route("/full_guild", get(routes::guilds::get_full_guild))
        .layer(auth_guild_mw_stack);

    let authorized_api_routes =
        Router::new()
            .nest("/guilds/:guild", authorized_api_guild_routes)
            .nest("/admin", authorized_admin_routes)
            .route("/guilds", get(routes::guilds::list_user_guilds_route))
            .route(
                "/premium_slots/:slot_id/update_guild",
                post(routes::premium::update_premium_slot_guild),
            )
            .route(
                "/premium_slots",
                get(routes::premium::list_user_premium_slots),
            )
            .route(
                "/sessions",
                get(routes::sessions::get_all_sessions)
                    .delete(routes::sessions::del_session)
                    .put(routes::sessions::create_api_token),
            )
            .route("/sessions/all", delete(routes::sessions::del_all_sessions))
            .route("/current_user", get(routes::general::get_current_user))
            .route(
                "/user/plugins",
                get(routes::plugins::get_user_plugins).put(routes::plugins::create_plugin),
            )
            .route(
                "/user/plugins/:plugin_id",
                patch(routes::plugins::update_plugin_meta).layer(
                    axum::middleware::from_fn_with_state(state.clone(), plugin_middleware),
                ),
            )
            .route(
                "/user/plugins/:plugin_id/dev_version",
                patch(routes::plugins::update_plugin_dev_source).layer(
                    axum::middleware::from_fn_with_state(state.clone(), plugin_middleware),
                ),
            )
            .route(
                "/user/plugins/:plugin_id/publish_script_version",
                post(routes::plugins::publish_plugin_version).layer(
                    axum::middleware::from_fn_with_state(state.clone(), plugin_middleware),
                ),
            )
            .route(
                "/user/plugins/:plugin_id/images",
                post(routes::plugins::add_plugin_image).layer(
                    axum::middleware::from_fn_with_state(state.clone(), plugin_middleware),
                ),
            )
            .route(
                "/user/plugins/:plugin_id/images/:image_id",
                delete(routes::plugins::delete_plugin_image).layer(
                    axum::middleware::from_fn_with_state(state.clone(), plugin_middleware),
                ),
            )
            .route("/logout", post(AuthHandlers::handle_logout))
            .route(
                "/stripe/customer_portal",
                post(routes::stripe::handle_create_customer_portal_session),
            )
            .route(
                "/stripe/create_checkout_session",
                post(routes::stripe::handle_create_checkout_session),
            );

    let auth_routes_mw_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_mw_err_no_auth))
        .layer(require_auth_layer);

    let authorized_routes = Router::new()
        .nest("/api", authorized_api_routes)
        .layer(auth_routes_mw_stack);

    let public_routes = Router::new()
        .route("/error", get(routes::errortest::handle_errortest))
        .route("/login", get(AuthHandlers::handle_login))
        .route(
            "/media/plugins/:plugin_id/images/*image_id_specifier_with_extension",
            get(routes::plugins::get_plugin_image),
        )
        .route(
            "/api/plugins",
            get(routes::plugins::get_published_public_plugins),
        )
        .route(
            "/api/plugins/:plugin_id",
            get(routes::plugins::get_plugin).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                plugin_middleware,
            )),
        )
        .route("/api/news", get(routes::general::get_news))
        .route("/api/ws", get(routes::ws::ws_handler))
        .route(
            "/api/confirm_login",
            post(AuthHandlers::handle_confirm_login),
        )
        .route("/api/stripe/webhook", post(routes::stripe::handle_webhook));

    let app = public_routes
        .merge(authorized_routes)
        .layer(common_middleware_stack)
        .fallback(|| async { StatusCode::NOT_FOUND })
        .with_state(state);

    info!("Starting hype on address: {}", conf.listen_addr);

    let listener = tokio::net::TcpListener::bind(conf.listen_addr)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[allow(dead_code)]
async fn todo_route() -> &'static str {
    "todo"
}

async fn handle_mw_err_internal_err(err: BoxError) -> ApiErrorResponse {
    error!("internal error occured: {}", err);

    ApiErrorResponse::InternalError
}

async fn handle_mw_err_no_auth(err: BoxError) -> impl IntoResponse {
    match err.downcast::<NoSession>() {
        Ok(_) => ApiErrorResponse::SessionExpired,
        Err(_) => ApiErrorResponse::InternalError,
    }
}

#[derive(Clone, clap::Parser, Debug)]
pub struct WebConfig {
    #[clap(long, env = "BL_NEWS_CHANNELS", default_value = "")]
    pub(crate) news_channels: String,

    #[clap(long, env = "BL_NEWS_GUILD")]
    pub(crate) news_guild: Option<Id<GuildMarker>>,

    #[clap(
        long,
        env = "BL_BROKER_API_ADDR",
        default_value = "http://localhost:7449"
    )]
    pub(crate) broker_api_addr: String,

    #[clap(long, env = "STRIPE_PUBLIC_KEY")]
    pub(crate) stripe_public_key: Option<String>,

    #[clap(long, env = "STRIPE_PRIVATE_KEY")]
    pub(crate) stripe_private_key: Option<String>,

    #[clap(long, env = "STRIPE_WEBHOOK_SECRET")]
    pub(crate) stripe_webhook_secret: Option<String>,

    #[clap(long, env = "STRIPE_PREMIUM_PRODUCT_ID")]
    pub(crate) stripe_premium_product_id: Option<String>,

    #[clap(long, env = "STRIPE_PREMIUM_PRICE_ID")]
    pub(crate) stripe_premium_price_id: Option<String>,

    #[clap(long, env = "STRIPE_LITE_PRODUCT_ID")]
    pub(crate) stripe_lite_product_id: Option<String>,

    #[clap(long, env = "STRIPE_LITE_PRICE_ID")]
    pub(crate) stripe_lite_price_id: Option<String>,
}
