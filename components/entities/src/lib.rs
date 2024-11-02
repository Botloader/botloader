use std::{fmt::Display, ops::Deref};

use sea_orm::{DbErr, TryFromU64};
use twilight_model::id::Id;

pub mod prelude;

pub mod bucket_store;
pub mod discord_oauth_tokens;
pub mod guild_meta_configs;
pub mod guild_plugin_subscriptions;
pub mod guild_scripts;
pub mod guild_whitelist;
pub mod images;
pub mod interval_timers;
pub mod joined_guilds;
pub mod plugin_images;
pub mod plugin_versions;
pub mod plugins;
pub mod premium_slots;
pub mod scheduled_tasks;
pub mod user_meta;
pub mod web_sessions;

pub struct TwilightId<T>(pub Id<T>);

impl<T> From<Id<T>> for TwilightId<T> {
    fn from(value: Id<T>) -> Self {
        TwilightId(value)
    }
}

impl<T> std::fmt::Debug for TwilightId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TwilightId").field(&self.0).finish()
    }
}

impl<T> PartialEq for TwilightId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for TwilightId<T> {}

impl<T> Clone for TwilightId<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for TwilightId<T> {
    type Target = Id<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Display for TwilightId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<TwilightId<T>> for sea_orm::Value {
    fn from(source: TwilightId<T>) -> Self {
        (source.0.get() as i64).into()
    }
}

impl<T> TryFromU64 for TwilightId<T> {
    fn try_from_u64(n: u64) -> Result<Self, DbErr> {
        Id::new_checked(n)
            .ok_or(DbErr::TryIntoErr {
                from: "u64",
                into: "TwilightId",
                source: "id is 0".into(),
            })
            .map(TwilightId)
    }
}

impl<T> sea_orm::TryGetable for TwilightId<T> {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> std::result::Result<Self, sea_orm::TryGetError> {
        <i64 as sea_orm::TryGetable>::try_get_by(res, idx).map(|v| TwilightId(Id::new(v as u64)))
    }
}

impl<T> sea_orm::sea_query::ValueType for TwilightId<T> {
    fn try_from(v: sea_orm::Value) -> std::result::Result<Self, sea_orm::sea_query::ValueTypeErr> {
        <i64 as sea_orm::sea_query::ValueType>::try_from(v).map(|v| TwilightId(Id::new(v as u64)))
    }

    fn type_name() -> std::string::String {
        stringify!(TwilightId).to_owned()
    }

    fn array_type() -> sea_orm::sea_query::ArrayType {
        sea_orm::sea_query::ArrayType::BigInt
    }

    fn column_type() -> sea_orm::sea_query::ColumnType {
        sea_orm::prelude::ColumnType::BigInteger
    }
}
