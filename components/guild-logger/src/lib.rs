use entry::CreateLogEntry;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    oneshot,
};

pub mod discord_backend;
pub mod entry;
pub mod guild_subscriber_backend;

pub use entry::{LogEntry, LogLevel, ScriptContext};
use twilight_model::id::{marker::GuildMarker, Id};

#[async_trait::async_trait]
pub trait GuildLoggerBackend {
    async fn handle_entry(&self, entry: LogEntry);
}

enum LogCommand {
    LogMessage(LogEntry),
    Flush(oneshot::Sender<()>),
}

struct LoggerTask {
    backends: Vec<Arc<dyn GuildLoggerBackend + Send + Sync>>,
    rx: UnboundedReceiver<LogCommand>,
}

impl LoggerTask {
    async fn run(&mut self) {
        loop {
            while let Some(next) = self.rx.recv().await {
                match next {
                    LogCommand::LogMessage(msg) => self.handle_next(msg).await,
                    LogCommand::Flush(top) => {
                        // flush all pending messages in the channel and send a response when there's no more pending
                        let mut additional_flush_waiters = Vec::new();
                        while let Ok(cmd) = self.rx.try_recv() {
                            match cmd {
                                LogCommand::LogMessage(msg) => self.handle_next(msg).await,
                                LogCommand::Flush(waiter) => additional_flush_waiters.push(waiter),
                            }
                        }

                        for waiter in additional_flush_waiters {
                            let _ = waiter.send(());
                        }
                        let _ = top.send(());
                    }
                }
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
    tx: UnboundedSender<LogCommand>,
}

impl LogSender {
    pub fn log(&self, entry: LogEntry) {
        let _ = self.tx.send(LogCommand::LogMessage(entry));
    }

    pub fn with_guild(&self, guild_id: Id<GuildMarker>) -> GuildLogSender {
        GuildLogSender {
            tx: self.tx.clone(),
            guild_id,
        }
    }

    // flush all pending messages in the channel
    pub async fn flush(&self) {
        let (tx, rx) = oneshot::channel();
        if self.tx.send(LogCommand::Flush(tx)).is_ok() {
            let _ = rx.await;
        }
    }
}

#[derive(Clone)]
pub struct GuildLogSender {
    tx: UnboundedSender<LogCommand>,
    guild_id: Id<GuildMarker>,
}

impl GuildLogSender {
    pub fn log(&self, entry: CreateLogEntry) {
        let _ = self.tx.send(LogCommand::LogMessage(LogEntry {
            guild_id: self.guild_id,
            level: entry.level,
            message: entry.message,
            script_context: entry.script_context,
        }));
    }

    pub fn log_raw(&self, entry: LogEntry) {
        let _ = self.tx.send(LogCommand::LogMessage(entry));
    }
}
