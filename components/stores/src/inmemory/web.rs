use std::{convert::Infallible, sync::Arc};

use async_trait::async_trait;
use dashmap::{mapref::entry::Entry, DashMap};
use oauth2::CsrfToken;
use twilight_model::{
    id::{marker::UserMarker, Id},
    user::CurrentUser,
};

use crate::web::{gen_token, CsrfStore, DiscordOauthToken, Session, SessionType};

#[derive(Default, Clone)]
pub struct InMemorySessionStore {
    sessions: Arc<DashMap<String, BareSession>>,
    tokens: Arc<DashMap<Id<UserMarker>, DiscordOauthToken>>,
}
pub struct BareSession {
    pub token: String,
    pub user: CurrentUser,
    pub kind: SessionType,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("oauth token not found")]
    OauthTokenNotFound,
}

#[async_trait]
impl crate::web::SessionStore for InMemorySessionStore {
    type Error = Error;

    async fn set_user_oatuh_token(
        &self,
        oauth_token: DiscordOauthToken,
    ) -> Result<DiscordOauthToken, Self::Error> {
        let user_id = oauth_token.user_id;
        self.tokens.insert(user_id, oauth_token.clone());
        Ok(oauth_token)
    }

    async fn set_oauth_create_session(
        &self,
        oauth_token: DiscordOauthToken,
        user: CurrentUser,
        kind: SessionType,
    ) -> Result<Session, Self::Error> {
        self.set_user_oatuh_token(oauth_token).await?;
        self.create_session(user, kind).await
    }

    async fn create_session(
        &self,
        user: CurrentUser,
        kind: SessionType,
    ) -> Result<Session, Self::Error> {
        let oauth_token = match self.tokens.get(&user.id) {
            Some(t) => t,
            None => return Err(Error::OauthTokenNotFound),
        };

        loop {
            let token = gen_token();

            match self.sessions.entry(token.clone()) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(e) => {
                    let bare_session = BareSession {
                        token: token.clone(),
                        user: user.clone(),
                        created_at: chrono::Utc::now(),
                        kind,
                    };

                    let session = Session {
                        oauth_token: oauth_token.clone(),
                        created_at: bare_session.created_at,
                        token,
                        kind,
                        user,
                    };

                    e.insert(bare_session);
                    return Ok(session);
                }
            }
        }
    }

    async fn get_oauth_token(
        &self,
        user_id: Id<UserMarker>,
    ) -> Result<DiscordOauthToken, Self::Error> {
        let token = match self.tokens.get(&user_id) {
            Some(s) => s,
            None => return Err(Error::OauthTokenNotFound),
        };

        Ok(token.clone())
    }

    async fn get_session(&self, token: &str) -> Result<Option<Session>, Self::Error> {
        let bare_session = match self.sessions.get(token) {
            Some(s) => s,
            None => return Ok(None),
        };

        let token = match self.tokens.get(&bare_session.user.id) {
            Some(s) => s,
            None => return Err(Error::OauthTokenNotFound),
        };

        Ok(Some(Session {
            oauth_token: token.clone(),
            token: bare_session.token.clone(),
            kind: bare_session.kind,
            user: bare_session.user.clone(),
            created_at: bare_session.created_at,
        }))
    }

    async fn get_all_sessions(&self, user_id: Id<UserMarker>) -> Result<Vec<Session>, Self::Error> {
        let token = match self.tokens.get(&user_id) {
            Some(s) => s,
            None => return Ok(vec![]),
        };

        Ok(self
            .sessions
            .iter()
            .filter(|e| e.user.id == user_id)
            .map(|e| Session {
                oauth_token: token.clone(),
                token: e.token.clone(),
                kind: e.kind,
                user: e.user.clone(),
                created_at: e.created_at,
            })
            .collect())
    }

    async fn del_session(&self, token: &str) -> Result<bool, Self::Error> {
        Ok(self.sessions.remove(token).is_some())
    }

    async fn del_all_sessions(&self, user_id: Id<UserMarker>) -> Result<(), Self::Error> {
        self.sessions.retain(|_, v| v.user.id != user_id);
        Ok(())
    }
}

#[derive(Default)]
pub struct InMemoryCsrfStore {
    tokens: DashMap<String, ()>,
}

#[async_trait]
impl CsrfStore for InMemoryCsrfStore {
    type Error = Infallible;

    async fn generate_csrf_token(&self) -> Result<CsrfToken, Self::Error> {
        // altough very very low chance, handle the case where we generate 2 identical tokens
        loop {
            let token = gen_token();
            match self.tokens.entry(token.clone()) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(e) => {
                    e.insert(());
                    return Ok(CsrfToken::new(token));
                }
            }
        }
    }

    async fn check_csrf_token(&self, token: &str) -> Result<bool, Self::Error> {
        Ok(self.tokens.remove(token).is_some())
    }
}
