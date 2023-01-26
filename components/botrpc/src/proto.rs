use twilight_model::id::Id;

tonic::include_proto!("botrpc");

impl From<guild_logger::LogEntry> for GuildLogItem {
    fn from(entry: guild_logger::LogEntry) -> Self {
        Self {
            guild_id: entry.guild_id.get(),
            level: LogLevel::from(entry.level) as i32,
            message: entry.message,
            script_context: entry.script_context.map(Into::into),
        }
    }
}

impl From<GuildLogItem> for guild_logger::LogEntry {
    fn from(entry: GuildLogItem) -> Self {
        Self {
            guild_id: Id::new(entry.guild_id),
            level: match entry.level {
                0 => guild_logger::LogLevel::Critical,
                1 => guild_logger::LogLevel::Error,
                2 => guild_logger::LogLevel::Warn,
                3 => guild_logger::LogLevel::Info,
                4 => guild_logger::LogLevel::ConsoleLog,
                _ => panic!("invalid loglevel value"),
            },
            message: entry.message,
            script_context: entry.script_context.map(Into::into),
        }
    }
}

impl From<guild_logger::ScriptContext> for ScriptContext {
    fn from(ctx: guild_logger::ScriptContext) -> Self {
        Self {
            filename: ctx.filename,
            line_col: ctx.line_col.map(Into::into),
        }
    }
}

impl From<ScriptContext> for guild_logger::ScriptContext {
    fn from(ctx: ScriptContext) -> Self {
        Self {
            filename: ctx.filename,
            line_col: ctx.line_col.map(Into::into),
        }
    }
}

impl From<LineCol> for (u32, u32) {
    fn from(l: LineCol) -> Self {
        (l.line, l.column)
    }
}

impl From<(u32, u32)> for LineCol {
    fn from(l: (u32, u32)) -> Self {
        Self {
            line: l.0,
            column: l.1,
        }
    }
}

impl From<guild_logger::LogLevel> for LogLevel {
    fn from(l: guild_logger::LogLevel) -> Self {
        match l {
            guild_logger::LogLevel::Critical => Self::Critical,
            guild_logger::LogLevel::Error => Self::Error,
            guild_logger::LogLevel::Warn => Self::Warn,
            guild_logger::LogLevel::Info => Self::Info,
            guild_logger::LogLevel::ConsoleLog => Self::ConsoleLog,
        }
    }
}

impl From<LogLevel> for guild_logger::LogLevel {
    fn from(l: LogLevel) -> Self {
        match l {
            LogLevel::Critical => Self::Critical,
            LogLevel::Error => Self::Error,
            LogLevel::Warn => Self::Warn,
            LogLevel::Info => Self::Info,
            LogLevel::ConsoleLog => Self::ConsoleLog,
        }
    }
}
