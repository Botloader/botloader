use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub mod discord_backend;
pub mod entry;
pub mod guild_subscriber_backend;

pub use entry::{LogEntry, LogLevel, ScriptContext};

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

    pub fn run(self) -> GuildLogger {
        let (tx, rx) = unbounded_channel();

        tokio::spawn(async move {
            let mut logger_task = LoggerTask {
                backends: self.backends,
                rx,
            };

            logger_task.run().await
        });

        GuildLogger { tx }
    }
}

#[derive(Clone)]
pub struct GuildLogger {
    tx: UnboundedSender<LogEntry>,
}

impl GuildLogger {
    pub fn log(&self, entry: LogEntry) {
        let _ = self.tx.send(entry);
    }
}
