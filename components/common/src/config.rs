use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

#[derive(Clone, clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct RunConfig {
    #[clap(short, long, env = "DISCORD_BOT_TOKEN")]
    pub discord_token: String,

    #[clap(long, env = "DISCORD_CLIENT_ID")]
    pub client_id: String,

    #[clap(long, env = "DISCORD_CLIENT_SECRET")]
    pub client_secret: String,

    /// points to the frontend's host base, this can be seperate from the api server(webapi)
    ///
    /// example: api may run on https://api.botlabs.io and the frontend could use https://botlabs.io
    /// in this case, the frontend host base is https://botlabs.io
    #[clap(
        long,
        env = "FRONTEND_HOST_BASE",
        default_value = "http://localhost:3000"
    )]
    pub frontend_host_base: String,

    #[clap(long, env = "DATABASE_URL")]
    pub database_url: String,

    #[clap(long, env = "WEBAPI_LISTEN_ADDR", default_value = "127.0.0.1:7447")]
    pub listen_addr: String,

    #[clap(
        long,
        env = "BOT_RPC_CONNECT_ADDR",
        default_value = "http://127.0.0.1:7448"
    )]
    pub bot_rpc_connect_addr: String,

    #[clap(long, env = "BOT_RPC_LISTEN_ADDR", default_value = "127.0.0.1:7448")]
    pub bot_rpc_listen_addr: String,

    #[clap(long, env = "BL_USER_SCRIPT_HTTP_PROXY")]
    pub user_script_http_proxy: Option<String>,

    /// Export traces to an otlp compatible client (such as grafana agent) at the provided url
    #[clap(long, env = "BL_OTLP_GRPC_URL")]
    pub otlp_grpc_url: Option<String>,

    #[clap(long, env = "BL_SENTRY_DSN")]
    pub sentry_dsn: Option<String>,
}

impl RunConfig {
    pub fn get_discord_oauth2_client(&self) -> BasicClient {
        BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string()).unwrap(),
            Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap()),
        )
        // Set the URL the user will be redirected to after the authorization process.
        .set_redirect_uri(
            RedirectUrl::new(format!("{}/confirm_login", self.frontend_host_base)).unwrap(),
        )
    }
}
