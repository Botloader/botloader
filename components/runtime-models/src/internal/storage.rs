use serde::{Deserialize, Serialize};
// use stores::bucketstore::{self, SetCondition};
use ts_rs::TS;

use crate::util::{NotBigU64, PluginId};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucket.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpStorageBucket {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketValue.ts")]
#[serde(rename_all = "camelCase")]
pub enum OpStorageBucketValue {
    Json(#[ts(type = "any")] serde_json::Value),
    Double(f64),
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketSetValue.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpStorageBucketSetValue {
    pub bucket_name: String,
    pub key: String,
    pub value: OpStorageBucketValue,
    #[serde(default)]
    #[ts(optional)]
    pub ttl: Option<u32>,

    pub plugin_id: Option<PluginId>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketSetIf.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpStorageBucketSetIf {
    pub bucket_name: String,
    pub key: String,
    pub value: OpStorageBucketValue,
    #[serde(default)]
    #[ts(optional)]
    pub ttl: Option<u32>,
    pub cond: OpStorageBucketSetCondition,

    pub plugin_id: Option<PluginId>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketEntryId.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpStorageBucketEntryId {
    pub bucket_name: String,
    pub key: String,

    pub plugin_id: Option<PluginId>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketList.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpStorageBucketList {
    pub bucket_name: String,
    #[serde(default)]
    #[ts(optional)]
    pub key_pattern: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub after: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub limit: Option<u32>,

    pub plugin_id: Option<PluginId>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketListOrder.ts")]
pub enum OpStorageBucketListOrder {
    Ascending,
    Descending,
}

// impl From<OpStorageBucketListOrder> for bucketstore::SortedOrder {
//     fn from(v: OpStorageBucketListOrder) -> Self {
//         match v {
//             OpStorageBucketListOrder::Ascending => Self::Ascending,
//             OpStorageBucketListOrder::Descending => Self::Descending,
//         }
//     }
// }

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketSortedList.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpStorageBucketSortedList {
    pub bucket_name: String,
    pub plugin_id: Option<PluginId>,

    #[serde(default)]
    #[ts(optional)]
    pub offset: Option<u32>,

    #[serde(default)]
    #[ts(optional)]
    pub limit: Option<u32>,

    pub order: OpStorageBucketListOrder,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketIncr.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpStorageBucketIncr {
    pub bucket_name: String,
    pub key: String,
    pub plugin_id: Option<PluginId>,

    pub amount: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketEntry.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpStorageBucketEntry {
    pub plugin_id: Option<PluginId>,
    pub bucket_name: String,
    pub key: String,
    pub value: OpStorageBucketValue,
    pub expires_at: Option<NotBigU64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketSetCondition.ts")]
pub enum OpStorageBucketSetCondition {
    IfExists,
    IfNotExists,
}
