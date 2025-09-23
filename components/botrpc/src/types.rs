use twilight_model::id::{marker::GuildMarker, Id};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuildSpecifier {
    pub guild_id: Id<GuildMarker>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScriptContext {
    pub filename: String,
    pub line_col: LineCol,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LineCol {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VmWorkerStatusResponse {
    pub workers: Vec<VmWorkerStatus>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VmWorkerStatus {
    pub worker_id: u32,
    pub currently_claimed_by_guild_id: Option<u64>,
    pub last_claimed_by_guild_id: Option<u64>,
    pub claimed_last_ms_ago: u64,
    pub returned_last_ms_ago: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuildStatusResponse {
    pub current_claimed_worker_id: Option<u32>,
    pub last_claimed_worker_id: Option<u32>,
    pub claimed_last_since_ms: u64,
    pub returned_last_since_ms: u64,
    pub pending_acks: u32,
}
