use std::sync::Arc;

use aide::{
    axum::{routing as api_routing, ApiRouter},
    openapi::OpenApi,
};
use axum::{
    error_handling::HandleErrorLayer,
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    BoxError,
};

use routes::auth::AuthHandlers;
use stores::inmemory::web::InMemoryCsrfStore;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{info, Level};
use twilight_model::id::{marker::GuildMarker, Id};

mod app_state;
mod discord_schemas;
mod errors;
mod middlewares;
mod news_poller;
mod routes;
mod util;

use crate::middlewares::{
    bl_admin_only::bl_admin_only_mw, plugins::plugin_middleware,
    require_current_guild_admin_middleware, session_mw, CorsLayer, NoSession, RequireAuthLayer,
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

    let common_middleware_stack = ServiceBuilder::new()
        .layer(axum_metrics_layer::MetricsLayer {
            name_prefix: "bl.webapi",
        })
        // .layer(Extension(ConfigData {
        //     oauth_client: oatuh_client,
        //     frontend_base: common_conf.frontend_host_base.clone(),
        // }))
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(Level::INFO)))
        // .layer(Extension(bot_rpc_client))
        .layer(Extension(Arc::new(auth_handler)))
        // .layer(Extension(config_store))
        // .layer(Extension(session_store.clone()))
        // .layer(Extension(news_handle))
        // .layer(Extension(discord_config))
        // .layer(Extension(state_client))
        .layer(Extension(Arc::new(state.stripe_client.clone())))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            session_mw,
        ))
        .layer(CorsLayer {
            run_config: conf.clone(),
        });

    let auth_guild_mw_stack = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(current_guild_injector_middleware))
        .layer(axum::middleware::from_fn(
            require_current_guild_admin_middleware,
        ));

    let authorized_admin_routes = ApiRouter::new()
        .api_route(
            "/vm_workers",
            api_routing::get(routes::admin::get_worker_statuses),
        )
        .api_route(
            "/guild/{guild_id}/status",
            api_routing::get(routes::admin::get_guild_status),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            bl_admin_only_mw,
        ));

    let authorized_api_guild_routes = ApiRouter::new()
        .api_route("/reload_vm", api_routing::post(routes::vm::reload_guild_vm))
        .api_route("/settings", api_routing::get(routes::guilds::get_guild_settings))
        .api_route(
            "/premium_slots",
            api_routing::get(routes::guilds::get_guild_premium_slots),
        )
        .api_route(
            "/scripts",
            api_routing::get(routes::scripts::get_all_guild_scripts)
                .put(routes::scripts::create_guild_script),
        )
        .api_route(
            "/scripts_with_plugins",
            api_routing::get(routes::scripts::get_all_guild_scripts_with_plugins),
        )
        .api_route(
            "/scripts/{script_id}",
            api_routing::patch(routes::scripts::update_guild_script)
                .delete(routes::scripts::delete_guild_script),
        )
        .api_route(
            "/scripts/{script_id}/validate_settings",
            api_routing::post(routes::scripts::validate_script_settings),
        )
        .api_route(
            "/scripts/{script_id}/update_plugin",
            api_routing::post(routes::scripts::update_script_plugin),
        )
        .api_route(
            "/add_plugin",
            api_routing::post(routes::plugins::guild_add_plugin),
        )
        .api_route("/full_guild", api_routing::get(routes::guilds::get_full_guild))
        .layer(auth_guild_mw_stack);

    let authorized_api_routes =
        ApiRouter::new()
            .nest("/guilds/{guild}", authorized_api_guild_routes)
            .nest("/admin", authorized_admin_routes)
            .api_route(
                "/guilds",
                api_routing::get(routes::guilds::list_user_guilds_route),
            )
            .api_route(
                "/premium_slots/{slot_id}/update_guild",
                api_routing::post(routes::premium::update_premium_slot_guild),
            )
            .api_route(
                "/premium_slots",
                api_routing::get(routes::premium::list_user_premium_slots),
            )
            .api_route(
                "/sessions",
                api_routing::get(routes::sessions::get_all_sessions)
                    .delete(routes::sessions::del_session)
                    .put(routes::sessions::create_api_token),
            )
            .api_route(
                "/sessions/all",
                api_routing::delete(routes::sessions::del_all_sessions),
            )
            .api_route(
                "/current_user",
                api_routing::get(routes::general::get_current_user),
            )
            .api_route(
                "/user/plugins",
                api_routing::get(routes::plugins::get_user_plugins)
                    .put(routes::plugins::create_plugin),
            )
            .api_route(
                "/user/plugins/{plugin_id}",
                api_routing::patch(routes::plugins::update_plugin_meta).layer(
                    axum::middleware::from_fn_with_state(state.clone(), plugin_middleware),
                ),
            )
            .api_route(
                "/user/plugins/{plugin_id}/dev_version",
                api_routing::patch(routes::plugins::update_plugin_dev_source).layer(
                    axum::middleware::from_fn_with_state(state.clone(), plugin_middleware),
                ),
            )
            .api_route(
                "/user/plugins/{plugin_id}/publish_script_version",
                api_routing::post(routes::plugins::publish_plugin_version).layer(
                    axum::middleware::from_fn_with_state(state.clone(), plugin_middleware),
                ),
            )
            .api_route(
                "/user/plugins/{plugin_id}/images",
                api_routing::post(routes::plugins::add_plugin_image).layer(
                    axum::middleware::from_fn_with_state(state.clone(), plugin_middleware),
                ),
            )
            .api_route(
                "/user/plugins/{plugin_id}/images/{image_id}",
                api_routing::delete(routes::plugins::delete_plugin_image).layer(
                    axum::middleware::from_fn_with_state(state.clone(), plugin_middleware),
                ),
            )
            .api_route("/logout", api_routing::post(AuthHandlers::handle_logout))
            .api_route(
                "/stripe/customer_portal",
                api_routing::post(routes::stripe::handle_create_customer_portal_session),
            )
            .api_route(
                "/stripe/create_checkout_session",
                api_routing::post(routes::stripe::handle_create_checkout_session),
            );

    let auth_routes_mw_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_mw_err_no_auth))
        .layer(RequireAuthLayer {});

    let authorized_routes = ApiRouter::new()
        .nest("/api", authorized_api_routes)
        .layer(auth_routes_mw_stack);

    let public_routes = ApiRouter::new()
        .route("/error", get(routes::errortest::handle_errortest))
        .route("/login", get(AuthHandlers::handle_login))
        .route(
            "/media/plugins/{plugin_id}/images/{*image_id_specifier_with_extension}",
            get(routes::plugins::get_plugin_image),
        )
        .api_route(
            "/api/plugins",
            api_routing::get(routes::plugins::get_published_public_plugins),
        )
        .api_route(
            "/api/plugins/{plugin_id}",
            api_routing::get(routes::plugins::get_plugin).layer(
                axum::middleware::from_fn_with_state(state.clone(), plugin_middleware),
            ),
        )
        .api_route("/api/news", api_routing::get(routes::general::get_news))
        .route("/api/ws", get(routes::ws::ws_handler))
        .api_route(
            "/api/confirm_login",
            api_routing::post(AuthHandlers::handle_confirm_login),
        )
        .route("/api/stripe/webhook", post(routes::stripe::handle_webhook));

    let mut api = OpenApi {
        info: aide::openapi::Info {
            title: "Botloader HTTP API".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    let app = public_routes
        .merge(authorized_routes)
        .finish_api(&mut api)
        .route("/api/openapi.json", get(serve_openapi))
        .layer(Extension(Arc::new(api)))
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

async fn serve_openapi(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoResponse {
    axum::Json(api)
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
