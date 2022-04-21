use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use twilight_model::user::CurrentUser;

pub struct ApiClient {
    token: String,
    client: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(token: String, base_url: String) -> Self {
        Self {
            token,
            client: Default::default(),
            base_url,
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let req = self
            .client
            .request(reqwest::Method::GET, format!("{}/{}", self.base_url, path))
            .header("Authorization", &self.token);

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await?;
            return Err(anyhow::anyhow!(
                "API request failed: status {status}, body: {body}",
            ));
        }

        Ok(resp.json().await?)
    }

    pub async fn get_self_user(&self) -> anyhow::Result<CurrentUser> {
        self.get("api/current_user").await
    }
}
