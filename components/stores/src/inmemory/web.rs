use dashmap::{mapref::entry::Entry, DashMap};
use oauth2::CsrfToken;

use crate::web::gen_token;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("oauth token not found")]
    OauthTokenNotFound,
}

#[derive(Default)]
pub struct InMemoryCsrfStore {
    tokens: DashMap<String, ()>,
}

impl InMemoryCsrfStore {
    pub async fn generate_csrf_token(&self) -> CsrfToken {
        // although very very low chance, handle the case where we generate 2 identical tokens
        loop {
            let token = gen_token();
            match self.tokens.entry(token.clone()) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(e) => {
                    e.insert(());
                    return CsrfToken::new(token);
                }
            }
        }
    }

    pub async fn check_csrf_token(&self, token: &str) -> bool {
        self.tokens.remove(token).is_some()
    }
}
