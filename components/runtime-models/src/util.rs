use std::fmt::Display;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

macro_rules! impl_primitives {
    ($($($ty:ty),* => $l:literal),*) => { $($(
        impl TS for $ty {
            type WithoutGenerics = Self;
            type OptionInnerType = Self;
            fn name() -> String { $l.to_owned() }
            fn inline() -> String { <Self as ::ts_rs::TS>::name() }
            fn inline_flattened() -> String { panic!("{} cannot be flattened", <Self as ::ts_rs::TS>::name()) }
            fn decl() -> String { panic!("{} cannot be declared", <Self as ::ts_rs::TS>::name()) }
            fn decl_concrete() -> String { panic!("{} cannot be declared", <Self as ::ts_rs::TS>::name()) }
        }
    )*)* };
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub struct NotBigU64(pub u64);

impl From<u64> for NotBigU64 {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<NotBigU64> for u64 {
    fn from(value: NotBigU64) -> Self {
        value.0
    }
}

impl Display for NotBigU64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub struct NotBigI64(pub i64);

impl Display for NotBigI64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct PluginId(pub u64);

impl TryFrom<String> for PluginId {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse().map_err(|_| "Invalid plugin id").map(PluginId)
    }
}

impl From<PluginId> for String {
    fn from(value: PluginId) -> Self {
        value.0.to_string()
    }
}

impl From<PluginId> for u64 {
    fn from(value: PluginId) -> Self {
        value.0
    }
}

impl_primitives! {
    NotBigU64, NotBigI64 => "number",
    PluginId => "string"
}
