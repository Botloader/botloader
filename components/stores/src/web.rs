use std::{fmt::Debug, str::FromStr, time::Duration};

use oauth2::{basic::BasicTokenType, EmptyExtraTokenFields, StandardTokenResponse, TokenResponse};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use twilight_model::{
    id::{marker::UserMarker, Id},
    user::CurrentUser,
    util::ImageHash,
};

use crate::Db;

const USER_API_KEY_LIMIT: i64 = 100;

pub type OauthToken = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum SessionType {
    User,
    ApiKey,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("oauth token not found")]
    OauthTokenNotFound,

    #[error("reached limit of api keys: {0} (limit {1})")]
    ApiKeyLimitReached(u64, u64),

    #[error(transparent)]
    Sql(#[from] sqlx::Error),
}

impl Db {
    async fn get_api_key_count(&self, user_id: Id<UserMarker>) -> Result<i64, Error> {
        let result = sqlx::query!(
            "SELECT count(*) FROM web_sessions WHERE user_id = $1 AND kind = $2;",
            user_id.get() as i64,
            i16::from(SessionType::ApiKey),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or_default())
    }

    pub async fn set_user_oatuh_token(
        &self,
        oauth2_token: DiscordOauthToken,
    ) -> Result<DiscordOauthToken, Error> {
        Ok(sqlx::query_as!(
            DbOauthToken,
            "INSERT INTO discord_oauth_tokens (user_id, discord_bearer_token, \
             discord_refresh_token, discord_token_expires_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id) DO UPDATE SET 
            discord_bearer_token = $2,
            discord_refresh_token = $3,
            discord_token_expires_at = $4
            RETURNING user_id, discord_bearer_token, discord_refresh_token, \
             discord_token_expires_at;",
            oauth2_token.user_id.get() as i64,
            oauth2_token.access_token,
            oauth2_token.refresh_token,
            oauth2_token.token_expires,
        )
        .fetch_one(&self.pool)
        .await?
        .into())
    }

    pub async fn set_oauth_create_session(
        &self,
        oauth2_token: DiscordOauthToken,
        user: CurrentUser,
        kind: SessionType,
    ) -> Result<Session, Error> {
        self.set_user_oatuh_token(oauth2_token).await?;
        self.create_session(user, kind).await
    }

    pub async fn create_session(
        &self,
        user: CurrentUser,
        kind: SessionType,
    ) -> Result<Session, Error> {
        if matches!(kind, SessionType::ApiKey) {
            let count = self.get_api_key_count(user.id).await?;
            if count > USER_API_KEY_LIMIT {
                return Err(Error::ApiKeyLimitReached(
                    count as u64,
                    USER_API_KEY_LIMIT as u64,
                ));
            }
        }

        let oauth_token = sqlx::query_as!(
            DbOauthToken,
            "SELECT user_id, discord_bearer_token, discord_refresh_token, discord_token_expires_at
            FROM discord_oauth_tokens WHERE user_id = $1",
            user.id.get() as i64,
        )
        .fetch_one(&self.pool)
        .await?;

        let token = gen_token();

        let resp = sqlx::query_as!(
            DbSession,
            "INSERT INTO web_sessions (token, kind, user_id, discriminator, username, avatar, \
             created_at) VALUES ($1, $2, $3, $4, $5, $6, now())
            RETURNING token, kind, user_id, discriminator, username, avatar, created_at;",
            &token,
            i16::from(kind),
            user.id.get() as i64,
            user.discriminator as i16,
            user.name,
            user.avatar
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Session {
            oauth_token: oauth_token.into(),
            created_at: resp.created_at,
            token,
            kind,
            user,
        })
    }

    pub async fn get_oauth_token(
        &self,
        user_id: Id<UserMarker>,
    ) -> Result<DiscordOauthToken, Error> {
        Ok(sqlx::query_as!(
            DbOauthToken,
            "SELECT user_id, discord_bearer_token, discord_refresh_token, discord_token_expires_at
            FROM discord_oauth_tokens WHERE user_id = $1",
            user_id.get() as i64,
        )
        .fetch_one(&self.pool)
        .await?
        .into())
    }

    pub async fn get_session(&self, token: &str) -> Result<Option<Session>, Error> {
        let session = match sqlx::query_as!(
            DbSession,
            "SELECT token, kind, user_id, discriminator, username, avatar, created_at FROM \
             web_sessions WHERE token = $1;",
            token
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => s,
            Err(sqlx::Error::RowNotFound) => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        let oauth_token = sqlx::query_as!(
            DbOauthToken,
            "SELECT user_id, discord_bearer_token, discord_refresh_token, discord_token_expires_at
            FROM discord_oauth_tokens WHERE user_id = $1",
            session.user_id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(Session {
            token: token.to_string(),
            kind: SessionType::from(session.kind),
            oauth_token: oauth_token.into(),
            created_at: session.created_at,
            user: session.into(),
        }))
    }

    pub async fn get_all_sessions(&self, user_id: Id<UserMarker>) -> Result<Vec<Session>, Error> {
        let oauth_token: DiscordOauthToken = sqlx::query_as!(
            DbOauthToken,
            "SELECT user_id, discord_bearer_token, discord_refresh_token, discord_token_expires_at
            FROM discord_oauth_tokens WHERE user_id = $1",
            user_id.get() as i64,
        )
        .fetch_one(&self.pool)
        .await?
        .into();

        let sessions = sqlx::query_as!(
            DbSession,
            "SELECT token, kind, user_id, discriminator, username, avatar, created_at FROM \
             web_sessions WHERE user_id = $1",
            user_id.get() as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(sessions
            .into_iter()
            .map(|e| Session {
                token: e.token.clone(),
                kind: e.kind.into(),
                oauth_token: oauth_token.clone(),
                created_at: e.created_at,
                user: e.into(),
            })
            .collect())
    }

    pub async fn del_session(&self, token: &str) -> Result<bool, Error> {
        let res = sqlx::query!("DELETE FROM web_sessions WHERE token= $1", token,)
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected() > 0)
    }

    pub async fn del_all_sessions(&self, user_id: Id<UserMarker>) -> Result<(), Error> {
        sqlx::query!(
            "DELETE FROM discord_oauth_tokens WHERE user_id= $1",
            user_id.get() as i64
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

struct DbOauthToken {
    user_id: i64,
    discord_bearer_token: String,
    discord_refresh_token: String,
    discord_token_expires_at: chrono::DateTime<chrono::Utc>,
}

impl From<DbOauthToken> for DiscordOauthToken {
    fn from(db_t: DbOauthToken) -> Self {
        Self {
            access_token: db_t.discord_bearer_token,
            refresh_token: db_t.discord_refresh_token,
            token_expires: db_t.discord_token_expires_at,
            user_id: Id::new(db_t.user_id as u64),
        }
    }
}

struct DbSession {
    token: String,
    kind: i16,
    user_id: i64,
    discriminator: i16,
    username: String,
    avatar: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<DbSession> for CurrentUser {
    fn from(db_u: DbSession) -> Self {
        Self {
            avatar: if !db_u.avatar.is_empty() {
                Some(ImageHash::from_str(&db_u.avatar).ok()).flatten()
            } else {
                None
            },
            bot: false,
            discriminator: db_u.discriminator as u16,
            email: None,
            flags: None,
            id: Id::new(db_u.user_id as u64),
            locale: None,
            mfa_enabled: false,
            name: db_u.username,
            premium_type: None,
            public_flags: None,
            verified: None,
            accent_color: None,
            banner: None,
            global_name: None,
        }
    }
}

impl From<SessionType> for i16 {
    fn from(st: SessionType) -> Self {
        match st {
            SessionType::User => 1,
            SessionType::ApiKey => 2,
        }
    }
}

impl From<i16> for SessionType {
    fn from(st: i16) -> Self {
        match st {
            1 => SessionType::User,
            2 => SessionType::ApiKey,
            _ => panic!("unknown variant of sessiontype: {st}"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub oauth_token: DiscordOauthToken,
    pub token: String,
    pub kind: SessionType,
    pub user: CurrentUser,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DiscordOauthToken {
    pub user_id: Id<UserMarker>,
    pub access_token: String,
    pub refresh_token: String,
    pub token_expires: chrono::DateTime<chrono::Utc>,
}

impl DiscordOauthToken {
    pub fn new(user_id: Id<UserMarker>, oauth2_token: OauthToken) -> DiscordOauthToken {
        DiscordOauthToken {
            access_token: oauth2_token.access_token().secret().clone(),
            refresh_token: oauth2_token
                .refresh_token()
                .map(|at| at.secret().clone())
                .unwrap_or_default(),
            token_expires: chrono::Utc::now()
                + chrono::Duration::from_std(
                    oauth2_token
                        .expires_in()
                        .unwrap_or_else(|| Duration::from_secs(60 * 60 * 24 * 7)),
                )
                .unwrap(),
            user_id,
        }
    }
}

impl Debug for DiscordOauthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiscordOauthToken")
            .field("user", &self.user_id)
            .field("access_token", &"<yoinked>")
            .field("refresh_token", &"<yoinked>")
            .field("token_expires", &self.token_expires)
            .finish()
    }
}

pub fn gen_token() -> String {
    let random_bytes: Vec<u8> = (0..32).map(|_| thread_rng().gen::<u8>()).collect();
    base64::encode_config(random_bytes, base64::URL_SAFE_NO_PAD)
}
