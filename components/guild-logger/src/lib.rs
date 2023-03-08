use entry::CreateLogEntry;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub mod discord_backend;
pub mod entry;
pub mod guild_subscriber_backend;

pub use entry::{LogEntry, LogLevel, ScriptContext};
use twilight_model::id::{marker::GuildMarker, Id};

#[async_trait::async_trait]
pub trait GuildLoggerBackend {
    async fn handle_entry(&self, entry: LogEntry);
}

struct LoggerTask {
    backends: Vec<Arc<dyn GuildLoggerBackend + Send + Sync>>,
    rx: UnboundedReceiver<LogEntry>,
}

impl LoggerTask {
    async fn run(&mut self) {
        loop {
            while let Some(next) = self.rx.recv().await {
                self.handle_next(next).await;
            }
        }
    }

    async fn handle_next(&self, entry: LogEntry) {
        for backend in &self.backends {
            backend.handle_entry(entry.clone()).await;
        }
    }
}

#[derive(Default)]
pub struct GuildLoggerBuilder {
    backends: Vec<Arc<dyn GuildLoggerBackend + Send + Sync>>,
}

impl GuildLoggerBuilder {
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
        }
    }

    pub fn add_backend<T: GuildLoggerBackend + Send + Sync + 'static>(
        mut self,
        backend: Arc<T>,
    ) -> Self {
        self.backends.push(backend);
        self
    }

    pub fn run(self) -> LogSender {
        let (tx, rx) = unbounded_channel();

        tokio::spawn(async move {
            let mut logger_task = LoggerTask {
                backends: self.backends,
                rx,
            };

            logger_task.run().await
        });

        LogSender { tx }
    }
}

#[derive(Clone)]
pub struct LogSender {
    tx: UnboundedSender<LogEntry>,
}

impl LogSender {
    pub fn log(&self, entry: LogEntry) {
        let _ = self.tx.send(entry);
    }

    pub fn with_guild(&self, guild_id: Id<GuildMarker>) -> GuildLogSender {
        GuildLogSender {
            tx: self.tx.clone(),
            guild_id,
        }
    }
}

#[derive(Clone)]
pub struct GuildLogSender {
    tx: UnboundedSender<LogEntry>,
    guild_id: Id<GuildMarker>,
}

impl GuildLogSender {
    pub fn log(&self, entry: CreateLogEntry) {
        let _ = self.tx.send(LogEntry {
            guild_id: self.guild_id,
            level: entry.level,
            message: entry.message,
            script_context: entry.script_context,
        });
    }

    pub fn log_raw(&self, entry: LogEntry) {
        let _ = self.tx.send(entry);
    }
}
