//! Latency measurement points along the event dispatch pipeline.
//!
//! Every stage records the *cumulative* latency since the event's source
//! timestamp (set broker-side for discord events). All stages sharing the
//! same reference point means a latency spike shows up at the first stage
//! it has accumulated by, without correlating individual events:
//!
//! - `broker_recv`: read off the broker tcp connection
//! - `scheduler_recv`: dequeued by the scheduler task
//! - `guild_handler_recv`: dequeued by the per-guild handler (includes guild
//!   activation for previously inactive guilds)
//! - `worker_sent`: handed to the worker socket (includes worker claim wait
//!   and vm creation request)
//!
//! The vm-side `dispatch_event_latency` (recorded when the js call is made)
//! and the scheduler-side `dispatch_event_acked_latency` complete the
//! picture.

use chrono::{DateTime, Utc};
use common::dispatch_event::EventSource;

pub(crate) fn record_stage(
    stage: &'static str,
    event_source: &'static str,
    source_timestamp: DateTime<Utc>,
) {
    metrics::histogram!(
        "dispatch_event_stage_latency",
        "stage" => stage,
        "event_source" => event_source,
    )
    .record(elapsed_millis(source_timestamp));
}

/// Fractional milliseconds since `since`; the early hops are normally
/// sub-millisecond so whole-ms resolution would round them all to zero.
pub(crate) fn elapsed_millis(since: DateTime<Utc>) -> f64 {
    let elapsed = Utc::now().signed_duration_since(since);
    elapsed
        .num_microseconds()
        .map(|us| us as f64 / 1000.0)
        .unwrap_or_else(|| elapsed.num_milliseconds() as f64)
}

pub(crate) fn event_source_label(source: EventSource) -> &'static str {
    match source {
        EventSource::Discord => "discord",
        EventSource::Timer => "timer",
    }
}
