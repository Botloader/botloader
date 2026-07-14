//! Schema mirrors for twilight types exposed by the api.
//!
//! Twilight types can't implement `JsonSchema` (foreign trait on foreign type),
//! so response types reference these mirrors with `#[schemars(with = "...")]`.
//! They exist only for schema generation and must be kept in sync with what the
//! twilight types actually serialize.

use schemars::JsonSchema;

/// Mirror of [twilight_model::user::CurrentUser]
#[derive(JsonSchema)]
#[schemars(rename = "CurrentUser")]
#[allow(dead_code)]
pub struct CurrentUserSchema {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub bot: bool,
    pub mfa_enabled: bool,
    pub locale: Option<String>,
    pub verified: Option<bool>,
    pub email: Option<String>,
    pub flags: Option<u64>,
    pub premium_type: Option<u8>,
    pub public_flags: Option<u64>,
    pub accent_color: Option<u32>,
    pub banner: Option<String>,
}

/// Mirror of [twilight_model::user::CurrentUserGuild]
#[derive(JsonSchema)]
#[schemars(rename = "CurrentUserGuild")]
#[allow(dead_code)]
pub struct CurrentUserGuildSchema {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub owner: bool,
    /// Permissions bitflags serialized as a string
    pub permissions: String,
    pub features: Vec<String>,
}
