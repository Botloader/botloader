use std::{borrow::Cow, convert::Infallible, pin::Pin, task::Poll, time::Duration};

use axum::{
    extract::{
        ws::{CloseCode, CloseFrame, Message, WebSocket, WebSocketUpgrade},
        Extension,
    },
    response::IntoResponse,
};
use discordoauthwrapper::{ClientCache, DiscordOauthApiClient, TwilightApiProvider};
use futures::{stream::SelectAll, Stream, StreamExt};
use guild_logger::LogEntry;
use oauth2::basic::BasicClient;
use serde::{Deserialize, Serialize};
use stores::web::SessionStore;
use twilight_model::{guild::Permissions, id::GuildId, user::CurrentUser};

use crate::{middlewares::LoggedInSession, ConfigData};

pub async fn ws_headler<ST: SessionStore + Clone + Send + Sync + 'static>(
    ws: WebSocketUpgrade,
    Extension(session_store): Extension<ST>,
    Extension(config_data): Extension<ConfigData>,
    Extension(bot_rpc): Extension<botrpc::Client>,
    Extension(client_cache): Extension<
        ClientCache<TwilightApiProvider, oauth2::basic::BasicClient, ST>,
    >,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| {
        handle_socket_upgrade(
            socket,
            session_store,
            config_data.oauth_client,
            bot_rpc,
            client_cache,
        )
    })
}

async fn handle_socket_upgrade<ST: SessionStore + Send + Clone + 'static>(
    socket: WebSocket,
    session_store: ST,
    oauth_client: BasicClient,
    bot_rpc: botrpc::Client,
    client_cache: ClientCache<TwilightApiProvider, oauth2::basic::BasicClient, ST>,
) {
    WsConn::new(socket, session_store, oauth_client, bot_rpc, client_cache)
        .run()
        .await;
}

struct WsConn<ST> {
    session_store: ST,
    socket: WebSocket,
    oauth_client: BasicClient,
    bot_rpc: botrpc::Client,
    client_cache: ClientCache<TwilightApiProvider, oauth2::basic::BasicClient, ST>,

    active_log_streams: SelectAll<GuildLogStream>,

    state: WsState<ST>,
}

type WsResult = Result<(), WsCloseReason>;

impl<ST: SessionStore + Clone + 'static> WsConn<ST> {
    fn new(
        socket: WebSocket,
        session_store: ST,
        oauth_client: BasicClient,
        bot_rpc: botrpc::Client,
        client_cache: ClientCache<TwilightApiProvider, oauth2::basic::BasicClient, ST>,
    ) -> Self {
        Self {
            socket,
            session_store,
            oauth_client,
            bot_rpc,
            client_cache,
            active_log_streams: SelectAll::new(),
            state: WsState::UnAuth,
        }
    }

    async fn run(&mut self) {
        let mut ping_ticker = tokio::time::interval(Duration::from_secs(30));

        loop {
            // SelectAll returns Ready(None) when empty
            // so if we didn't have this check this thread
            // would get pinned at 100%
            if !self.active_log_streams.is_empty() {
                tokio::select! {
                    item = self.active_log_streams.next() => {
                        if !self.handle_log_stream_item(item).await {
                            return;
                        }
                    },
                    ws = self.socket.recv() => {
                        if !self.handle_ws_rcv(ws).await {
                            return;
                        }
                    },
                    _ = ping_ticker.tick() => {
                        if !self.send_ping().await{
                            return;
                        }
                    },
                }
            } else {
                tokio::select! {
                    ws = self.socket.recv() => {
                        if !self.handle_ws_rcv(ws).await {
                            return;
                        }
                    },
                    _ = ping_ticker.tick() => {
                        if !self.send_ping().await{
                            return;
                        }
                    },
                }
            }
        }
    }

    async fn handle_log_stream_item(
        &mut self,
        item: Option<Result<LogEntry, tonic::Status>>,
    ) -> bool {
        match item {
            Some(Ok(item)) => self.handle_inner_log_item(item).await,
            Some(Err(_)) => {
                self.close(WsCloseReason::BotRpcError).await;
                false
            }
            _ => true, // There can't be a none since we have the is_empty check in the caller
        }
    }

    async fn handle_inner_log_item(&mut self, item: LogEntry) -> bool {
        if let Err(reason) = self.send_event(WsEvent::ScriptLogMessage(item)).await {
            self.close(reason).await;
            false
        } else {
            true
        }
    }

    async fn handle_ws_rcv(&mut self, ws_msg: Option<Result<Message, axum::Error>>) -> bool {
        match ws_msg {
            None | Some(Err(_)) => false,
            Some(Ok(msg)) => {
                if let Err(reason) = self.handle_ws_message(msg).await {
                    self.close(reason).await;
                    false
                } else {
                    true
                }
            }
        }
    }

    async fn close(&mut self, reason: WsCloseReason) {
        if matches!(reason, WsCloseReason::ClientDisconnected) {
            // connection is already closed
            return;
        }

        let code = reason.code();
        let desc = reason.description();

        self.socket
            .send(Message::Close(Some(CloseFrame {
                code,
                reason: Cow::from(desc),
            })))
            .await
            .ok(); // we don't really care about the error here
    }

    async fn handle_ws_message(&mut self, msg: Message) -> WsResult {
        match msg {
            Message::Text(s) => {
                let cmd: WsCommand =
                    serde_json::from_str(&s).map_err(|_| WsCloseReason::JsonDecodeError)?;
                self.handle_ws_command(cmd).await
            }
            Message::Binary(b) => {
                let cmd: WsCommand =
                    serde_json::from_slice(&b).map_err(|_| WsCloseReason::JsonDecodeError)?;
                self.handle_ws_command(cmd).await
            }
            Message::Ping(d) => self.send(Message::Pong(d)).await,
            Message::Pong(_) => Ok(()),
            Message::Close(_) => Err(WsCloseReason::ClientDisconnected),
        }
    }

    async fn handle_ws_command(&mut self, cmd: WsCommand) -> WsResult {
        match &self.state {
            WsState::UnAuth => self.handle_ws_command_unauth(cmd).await,
            WsState::Authorized(_) => self.handle_ws_command_auth(cmd).await,
        }
    }
    async fn handle_ws_command_unauth(&mut self, cmd: WsCommand) -> WsResult {
        match cmd {
            WsCommand::Authorize(token) => self.do_login(token).await,
            _ => Err(WsCloseReason::NotAuthorized),
        }
    }

    async fn do_login(&mut self, token: String) -> WsResult {
        if let Some(session) = self
            .session_store
            .get_session(&token)
            .await
            .map_err(|_| WsCloseReason::InternalError)?
        {
            let api_client = self
                .client_cache
                .fetch(session.user.id, || {
                    Result::<_, Infallible>::Ok(DiscordOauthApiClient::new_twilight(
                        session.user.id,
                        session.oauth_token.access_token.clone(),
                        self.oauth_client.clone(),
                        self.session_store.clone(),
                    ))
                })
                .unwrap();

            let logged_in_session = LoggedInSession::new(session, api_client);

            self.send_event(WsEvent::AuthSuccess(logged_in_session.session.user.clone()))
                .await?;

            self.state = WsState::Authorized(AuthorizedWsState {
                session: logged_in_session,
            });

            Ok(())
        } else {
            Err(WsCloseReason::BadToken)
        }
    }

    async fn handle_ws_command_auth(&mut self, cmd: WsCommand) -> WsResult {
        match cmd {
            WsCommand::SubscribeLogs(g) => self.subscribe_logs(g).await,
            WsCommand::UnSubscribeLogs(g) => self.unsubscribe_logs(g).await,

            WsCommand::Authorize(_) => Err(WsCloseReason::AuthWhenAuthorized),
        }
    }

    async fn subscribe_logs(&mut self, guild_id: GuildId) -> WsResult {
        if self
            .active_log_streams
            .iter()
            .any(|s| s.guild_id == guild_id)
        {
            // already subscribed
            return Ok(());
        }

        self.check_guild_acces(guild_id).await?;

        let stream = self
            .bot_rpc
            .guild_log_stream(guild_id)
            .await
            .map_err(|_| WsCloseReason::BotRpcError)?;

        self.active_log_streams.push(GuildLogStream {
            guild_id,
            inner: Box::pin(stream),
        });

        self.emit_subscriptions().await
    }

    async fn unsubscribe_logs(&mut self, guild_id: GuildId) -> WsResult {
        let current_streams = std::mem::replace(&mut self.active_log_streams, SelectAll::new());

        self.active_log_streams = current_streams
            .into_iter()
            .filter(|e| e.guild_id == guild_id)
            .collect();

        self.emit_subscriptions().await
    }

    async fn emit_subscriptions(&mut self) -> WsResult {
        let ids = self
            .active_log_streams
            .iter()
            .map(|item| item.guild_id)
            .collect::<Vec<_>>();

        self.send_event(WsEvent::SubscriptionsUpdated(ids)).await
    }

    async fn check_guild_acces(&mut self, guild_id: GuildId) -> WsResult {
        let session = match &self.state {
            WsState::Authorized(s) => s,
            _ => panic!("can't check guild access when not authorized"),
        };

        let user_guilds = session
            .session
            .api_client
            .current_user_guilds()
            .await
            .map_err(|_| WsCloseReason::InternalError)?;

        if let Some(ug) = user_guilds.into_iter().find(|e| e.id == guild_id) {
            if ug
                .permissions
                .intersects(Permissions::ADMINISTRATOR | Permissions::MANAGE_GUILD)
            {
                return Ok(());
            }

            if ug.owner {
                return Ok(());
            }

            Err(WsCloseReason::GuildMissingAccess)
        } else {
            Err(WsCloseReason::UnknownGuild)
        }
    }

    async fn send_ping(&mut self) -> bool {
        match self.send(Message::Ping(vec![69])).await {
            Ok(_) => true,
            Err(reason) => {
                self.close(reason).await;
                false
            }
        }
    }

    async fn send(&mut self, msg: Message) -> WsResult {
        if self.socket.send(msg).await.is_err() {
            // client disconnected
            Err(WsCloseReason::ClientDisconnected)
        } else {
            Ok(())
        }
    }

    async fn send_event(&mut self, evt: WsEvent) -> WsResult {
        let encoded = serde_json::to_string(&evt).map_err(|_| WsCloseReason::JsonEncodeError)?;
        self.send(Message::Text(encoded)).await
    }
}

#[allow(clippy::large_enum_variant)]
enum WsState<ST> {
    UnAuth,
    Authorized(AuthorizedWsState<ST>),
}

struct AuthorizedWsState<ST> {
    session: LoggedInSession<ST>,
}

/// Event is something that is transferred from server -> client
#[derive(Serialize)]
#[serde(tag = "t", content = "d")]
enum WsEvent {
    AuthSuccess(CurrentUser),
    SubscriptionsUpdated(Vec<GuildId>),
    ScriptLogMessage(LogEntry),
    // GeneralLogMEssage(String)
}

/// Command is something that is from client -> server
#[derive(Deserialize)]
#[serde(tag = "t", content = "d")]
enum WsCommand {
    Authorize(String),

    // below commands requires authorization
    SubscribeLogs(GuildId),
    UnSubscribeLogs(GuildId),
}

#[derive(Serialize)]
enum WsCloseReason {
    // normal client disconnect for whatever reason
    ClientDisconnected,

    // someting very bad happened, probably related to the database or botrpc
    InternalError,

    // failed decoding command
    JsonDecodeError,

    // failed encoding event
    JsonEncodeError,

    // bad token used for auth
    BadToken,

    // when trying to use commands that requires auth
    NotAuthorized,

    // tried to authorize when already authorized, this is unsuported
    AuthWhenAuthorized,

    // unknown guild, guild may still exist but the authorized user is not on it
    UnknownGuild,

    // missing access to the guild
    GuildMissingAccess,

    // an error occured cummincating with the bot
    BotRpcError,
}

impl WsCloseReason {
    fn code(&self) -> CloseCode {
        match self {
            WsCloseReason::ClientDisconnected => 1000,
            WsCloseReason::InternalError => 1011,
            WsCloseReason::JsonDecodeError => 1007,
            WsCloseReason::JsonEncodeError => 4005,
            WsCloseReason::BadToken => 4001,
            WsCloseReason::NotAuthorized => 4000,
            WsCloseReason::AuthWhenAuthorized => 4002,
            WsCloseReason::UnknownGuild => 4003,
            WsCloseReason::GuildMissingAccess => 4004,
            WsCloseReason::BotRpcError => 4006,
        }
    }

    fn description(&self) -> &'static str {
        match self {
            WsCloseReason::ClientDisconnected => "client disconnected",
            WsCloseReason::InternalError => "internal error occured",
            WsCloseReason::JsonDecodeError => "failed decoding json payload",
            WsCloseReason::JsonEncodeError => "failed encoding json payload",
            WsCloseReason::BadToken => "bad auth token provided",
            WsCloseReason::NotAuthorized => "not authorized",
            WsCloseReason::AuthWhenAuthorized => "already authorized",
            WsCloseReason::UnknownGuild => "unknown guild",
            WsCloseReason::GuildMissingAccess => "missing access to guild",
            WsCloseReason::BotRpcError => "error on communication with bot",
        }
    }
}

struct GuildLogStream {
    guild_id: GuildId,
    inner: Pin<Box<dyn Stream<Item = Result<LogEntry, tonic::Status>> + Send>>,
}

impl Stream for GuildLogStream {
    type Item = Result<LogEntry, tonic::Status>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx)
    }
}
