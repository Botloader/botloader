use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use guild_logger::LogEntry;
use serde::Deserialize;
use tracing::info;

pub struct State {
    completions: i32,
    err: Option<String>,
    last_completion: Instant,
    expecting_completions: i32,
}

pub struct Tracker {
    pub state: RwLock<State>,
}

#[derive(Deserialize)]
pub enum IntegrationTestMessage {
    ScriptComplete,
}

pub enum CompletionStatus {
    Pending,
    Timeout,
    Complete(i32),
    Error(String),
}

impl Tracker {
    pub fn new(num_scripts: i32) -> Self {
        Self {
            state: RwLock::new(State {
                completions: 0,
                expecting_completions: num_scripts,
                err: None,
                last_completion: Instant::now(),
            }),
        }
    }

    fn handle_console_log(&self, entry: LogEntry) {
        if let Some(payload) = entry.message.strip_prefix("INTEGRATION_TEST:") {
            let decoded: IntegrationTestMessage = serde_json::from_str(payload).unwrap();
            self.handle_ig_test_message(entry, decoded);
        } else {
            info!(
                "CONSOLELOG: {}: {}",
                entry
                    .script_context
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                entry.message
            );
        }
    }

    fn handle_ig_test_message(&self, _entry: LogEntry, msg: IntegrationTestMessage) {
        match msg {
            IntegrationTestMessage::ScriptComplete => {
                let mut locked = self.state.write().unwrap();
                locked.completions += 1;
                locked.last_completion = Instant::now();
                info!(
                    "script completed: remaining: {}",
                    locked.expecting_completions - locked.completions
                )
            }
        }
    }

    fn handle_err(&self, err: String) {
        let mut locked = self.state.write().unwrap();
        locked.err = Some(err);
    }

    pub fn is_complete(self: &Arc<Self>) -> CompletionStatus {
        let locked = self.state.read().unwrap();

        if let Some(err) = &locked.err {
            return CompletionStatus::Error(err.clone());
        }

        if locked.expecting_completions <= locked.completions {
            return CompletionStatus::Complete(locked.completions);
        }

        if locked.last_completion.elapsed() > Duration::from_secs(120) {
            return CompletionStatus::Timeout;
        }

        CompletionStatus::Pending
    }
}

#[async_trait::async_trait]
impl guild_logger::GuildLoggerBackend for Tracker {
    async fn handle_entry(&self, entry: LogEntry) {
        match entry.level {
            guild_logger::LogLevel::Critical => self.handle_err(entry.message),
            guild_logger::LogLevel::Error => self.handle_err(entry.message),
            guild_logger::LogLevel::Warn => {}
            guild_logger::LogLevel::Info => {}
            guild_logger::LogLevel::ConsoleLog => self.handle_console_log(entry),
        }
    }
}
