use std::fmt::Display;

use serde::{Deserialize, Serialize};
use twilight_model::id::{marker::GuildMarker, Id};

#[derive(Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub guild_id: Id<GuildMarker>,
    pub message: String,
    pub script_context: Option<ScriptContext>,
    pub level: LogLevel,
}

impl LogEntry {
    pub fn critical(guild_id: Id<GuildMarker>, msg: String) -> Self {
        Self {
            guild_id,
            message: msg,
            level: LogLevel::Critical,
            script_context: None,
        }
    }

    pub fn error(guild_id: Id<GuildMarker>, msg: String) -> Self {
        Self {
            guild_id,
            message: msg,
            level: LogLevel::Error,
            script_context: None,
        }
    }

    pub fn info(guild_id: Id<GuildMarker>, msg: String) -> Self {
        Self {
            guild_id,
            message: msg,
            level: LogLevel::Info,
            script_context: None,
        }
    }

    pub fn script_error(
        guild_id: Id<GuildMarker>,
        msg: String,
        filename: String,
        line_col: Option<LineCol>,
    ) -> Self {
        Self {
            guild_id,
            script_context: Some(ScriptContext { filename, line_col }),
            message: msg,
            level: LogLevel::Error,
        }
    }
    pub fn script_warning(
        guild_id: Id<GuildMarker>,
        msg: String,
        filename: String,
        line_col: Option<LineCol>,
    ) -> Self {
        Self {
            guild_id,
            script_context: Some(ScriptContext { filename, line_col }),
            message: msg,
            level: LogLevel::Warn,
        }
    }
    pub fn script_console(
        guild_id: Id<GuildMarker>,
        msg: String,
        filename: String,
        line_col: Option<LineCol>,
    ) -> Self {
        Self {
            guild_id,
            script_context: Some(ScriptContext { filename, line_col }),
            message: msg,
            level: LogLevel::ConsoleLog,
        }
    }
    pub fn script_info(
        guild_id: Id<GuildMarker>,
        msg: String,
        filename: String,
        line_col: Option<LineCol>,
    ) -> Self {
        Self {
            guild_id,
            script_context: Some(ScriptContext { filename, line_col }),
            message: msg,
            level: LogLevel::Info,
        }
    }
}

pub type LineCol = (u32, u32);

#[derive(Clone, Serialize, Deserialize)]
pub struct ScriptContext {
    pub filename: String,
    pub line_col: Option<LineCol>,
}

impl Display for ScriptContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.filename)?;
        if let Some((line, col)) = self.line_col {
            write!(f, ":{}:{}", line, col)?;
        }

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Critical,
    Error,
    Warn,
    Info,
    ConsoleLog,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "CRIT"),
            Self::Error => write!(f, "ERRO"),
            Self::Warn => write!(f, "WARN"),
            Self::ConsoleLog => write!(f, "CLOG"),
            Self::Info => write!(f, "INFO"),
        }
    }
}
