use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub struct NotBigU64(pub u64);

impl ts_rs::TS for NotBigU64 {
    const EXPORT_TO: Option<&'static str> = None;
    fn decl() -> String {
        format!("type {}{} = {};", "NotBigU64", "", "number")
    }
    fn name() -> String {
        "number".to_owned()
    }
    fn inline() -> String {
        "number".to_string()
    }
    fn dependencies() -> Vec<ts_rs::Dependency> {
        vec![]
    }
    fn transparent() -> bool {
        false
    }
}

impl From<u64> for NotBigU64 {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Display for NotBigU64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub struct NotBigI64(pub i64);

impl ts_rs::TS for NotBigI64 {
    const EXPORT_TO: Option<&'static str> = None;
    fn decl() -> String {
        format!("type {}{} = {};", "NotBigI64", "", "number")
    }
    fn name() -> String {
        "number".to_owned()
    }
    fn inline() -> String {
        "number".to_string()
    }
    fn dependencies() -> Vec<ts_rs::Dependency> {
        vec![]
    }
    fn transparent() -> bool {
        false
    }
}

impl Display for NotBigI64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}
