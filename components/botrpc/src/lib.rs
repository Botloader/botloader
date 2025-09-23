#![doc = include_str!("../README.md")]

pub mod client;
// pub mod proto;

pub use client::Client;

pub mod types;

use axum::{Json, Router};
use service::service;
use service::Empty;
use service::GetClientResponse;

use types::GuildSpecifier;
use types::GuildStatusResponse;
use types::VmWorkerStatusResponse;

service!(
    BotServiceServer, BotServiceClient,
    POST reload_vm: (GuildSpecifier) -> Empty,
    POST purge_guild_cache: (GuildSpecifier) -> Empty,
    GET vm_worker_status: () -> Json<VmWorkerStatusResponse>,
    GET guild_status: (GuildSpecifier) -> Json<GuildStatusResponse>,
    GET stream_guild_logs: (GuildSpecifier) -> service::SseStream<guild_logger::LogEntry>,
);
