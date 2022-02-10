use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::util::NotBigU64;

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/ClientHttpRequest.ts")]
#[serde(rename_all = "camelCase")]
pub struct ClientHttpRequest {
    pub path: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    #[serde(default)]
    #[ts(optional)]
    pub script_id: Option<NotBigU64>,
    #[serde(default)]
    #[ts(optional)]
    pub body_resource_id: Option<u32>,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/ClientHttpResponse.ts")]
#[serde(rename_all = "camelCase")]
pub struct ClientHttpResponse {
    pub headers: HashMap<String, String>,
    pub status_code: i32,
    pub body_resource_id: u32,
}
