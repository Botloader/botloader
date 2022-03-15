use std::sync::Arc;

use axum::{
    error_handling::HandleErrorLayer,
    extract::Extension,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    BoxError, Router,
};
use clap::Parser;
use oauth2::basic::BasicClient;
use routes::auth::AuthHandlers;
use stores::{inmemory::web::InMemoryCsrfStore, postgres::Postgres};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

mod errors;
mod middlewares;
mod routes;
mod util;

use crate::errors::ApiErrorResponse;
use crate::middlewares::{
    CorsLayer, CurrentGuildLayer, NoSession, RequireCurrentGuildAuthLayer, SessionLayer,
};

#[derive(Clone)]
pub struct ConfigData {
    oauth_client: BasicClient,
}

type CurrentSessionStore = Postgres;
type CurrentConfigStore = Postgres;
type AuthHandlerData = AuthHandlers<InMemoryCsrfStore, CurrentSessionStore>;
type ApiResult<T> = Result<T, ApiErrorResponse>;

#[tokio::main]
async fn main() {
    common::common_init(Some("0.0.0.0:7801"));
    let conf = common::config::RunConfig::parse();

    info!("starting...");

    let oatuh_client = conf.get_discord_oauth2_client();

    let postgres_store = Postgres::new_with_url(&conf.database_url).await.unwrap();
    let config_store: CurrentConfigStore = postgres_store.clone();
    let session_store: CurrentSessionStore = postgres_store.clone();
    let bot_rpc_client = botrpc::Client::new(conf.bot_rpc_connect_addr.clone())
        .await
        .expect("failed connecting to bot rpc");

    let auth_handler: AuthHandlerData =
        routes::auth::AuthHandlers::new(session_store.clone(), InMemoryCsrfStore::default());

    let session_layer = SessionLayer::new(session_store.clone(), oatuh_client.clone());
    let require_auth_layer = session_layer.require_auth_layer();
    let client_cache = session_layer.oauth_api_client_cache.clone();

    let common_middleware_stack = ServiceBuilder::new() // Process at most 100 requests concurrently
        .layer(HandleErrorLayer::new(handle_mw_err_internal_err))
        .layer(Extension(ConfigData {
            oauth_client: oatuh_client,
        }))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(bot_rpc_client))
        .layer(Extension(Arc::new(auth_handler)))
        .layer(Extension(config_store))
        .layer(Extension(session_store.clone()))
        .layer(Extension(client_cache))
        .layer(session_layer)
        .layer(CorsLayer {
            run_config: conf.clone(),
        });

    let auth_guild_mw_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_mw_err_internal_err))
        .layer(CurrentGuildLayer {
            session_store: session_store.clone(),
        })
        .layer(RequireCurrentGuildAuthLayer);

    let authorized_api_guild_routes = Router::new()
        .route(
            "/reload_vm",
            post(routes::vm::reload_guild_vm::<CurrentSessionStore>),
        )
        .route(
            "/settings",
            get(routes::guilds::get_guild_settings::<CurrentSessionStore>),
        )
        .route(
            "/scripts",
            get(routes::scripts::get_all_guild_scripts).put(routes::scripts::create_guild_script),
        )
        .route(
            "/scripts/:script_id",
            patch(routes::scripts::update_guild_script)
                .delete(routes::scripts::delete_guild_script),
        )
        .layer(auth_guild_mw_stack);

    let authorized_api_routes =
        Router::new()
            .nest("/guilds/:guild", authorized_api_guild_routes)
            .route(
                "/guilds",
                get(routes::guilds::list_user_guilds_route::<
                    CurrentSessionStore,
                    CurrentConfigStore,
                >),
            )
            .route(
                "/premium_slots/:slot_id/update_guild",
                post(
                    routes::premium::update_premium_slot_guild::<
                        CurrentSessionStore,
                        CurrentConfigStore,
                    >,
                ),
            )
            .route(
                "/premium_slots",
                get(routes::premium::list_user_premium_slots::<
                    CurrentSessionStore,
                    CurrentConfigStore,
                >),
            )
            .route(
                "/sessions",
                get(routes::sessions::get_all_sessions::<CurrentSessionStore>)
                    .delete(routes::sessions::del_session::<CurrentSessionStore>)
                    .put(routes::sessions::create_api_token::<CurrentSessionStore>),
            )
            .route(
                "/sessions/all",
                delete(routes::sessions::del_all_sessions::<CurrentSessionStore>),
            )
            .route(
                "/current_user",
                get(routes::general::get_current_user::<CurrentSessionStore>),
            )
            .route("/logout", post(AuthHandlerData::handle_logout));

    let auth_routes_mw_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_mw_err_no_auth))
        .layer(require_auth_layer);

    let authorized_routes = Router::new()
        .nest("/api", authorized_api_routes)
        .layer(auth_routes_mw_stack);

    let public_routes = Router::new()
        .route("/error", get(routes::errortest::handle_errortest))
        .route("/login", get(AuthHandlerData::handle_login))
        .route(
            "/api/ws",
            get(routes::ws::ws_headler::<CurrentSessionStore>),
        )
        .route(
            "/api/confirm_login",
            post(AuthHandlerData::handle_confirm_login),
        );

    let app = public_routes
        .merge(authorized_routes)
        .layer(common_middleware_stack.clone());

    let make_service = app.into_make_service();

    // run it with hyper on configured address
    info!("Starting hype on address: {}", conf.listen_addr);
    let addr = conf.listen_addr.parse().unwrap();
    axum::Server::bind(&addr)
        .serve(make_service)
        .with_graceful_shutdown(common::shutdown::wait_shutdown_signal())
        .await
        .unwrap();
}

#[allow(dead_code)]
async fn todo_route() -> &'static str {
    "todo"
}

async fn handle_mw_err_internal_err(err: BoxError) -> impl IntoResponse {
    error!("internal error occured: {}", err);

    ApiErrorResponse::InternalError
}

async fn handle_mw_err_no_auth(err: BoxError) -> impl IntoResponse {
    match err.downcast::<NoSession>() {
        Ok(_) => ApiErrorResponse::SessionExpired,
        Err(_) => ApiErrorResponse::InternalError,
    }
}
