use std::{sync::Arc, time::Duration};

use stores::Db;

use crate::job::{Job, JobSpawner, OutputFuture};

pub struct PluginStatsJobSpawner {
    pub db: Db,
}

impl JobSpawner for PluginStatsJobSpawner {
    fn name(&self) -> &'static str {
        "plugin_stats"
    }

    fn spawn(&self) -> Arc<dyn Job> {
        Arc::new(PluginStatsJob {
            db: self.db.clone(),
        })
    }

    fn interval(&self) -> std::time::Duration {
        Duration::from_secs(600)
    }
}

pub struct PluginStatsJob {
    db: Db,
}

impl Job for PluginStatsJob {
    fn status(&self) -> String {
        "Running".to_string()
    }

    fn run(self: std::sync::Arc<Self>) -> OutputFuture {
        Box::pin(async move {
            self.db.batch_update_plugins_stats().await?;

            Ok(())
        })
    }
}
