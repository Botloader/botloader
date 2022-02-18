use serde::{Deserialize, Serialize};
use stores::bucketstore::{self, SetCondition};
use ts_rs::TS;

use crate::util::NotBigU64;

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

impl From<bucketstore::StoreValue> for OpStorageBucketValue {
    fn from(v: bucketstore::StoreValue) -> Self {
        match v {
            bucketstore::StoreValue::Json(s) => Self::Json(s),
            bucketstore::StoreValue::Float(f) => Self::Double(f),
        }
    }
}

impl From<OpStorageBucketValue> for bucketstore::StoreValue {
    fn from(v: OpStorageBucketValue) -> Self {
        match v {
            OpStorageBucketValue::Json(s) => Self::Json(s),
            OpStorageBucketValue::Double(f) => Self::Float(f),
        }
    }
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
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketEntryId.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpStorageBucketEntryId {
    pub bucket_name: String,
    pub key: String,
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
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketListOrder.ts")]
pub enum OpStorageBucketListOrder {
    Ascending,
    Descending,
}

impl From<OpStorageBucketListOrder> for bucketstore::SortedOrder {
    fn from(v: OpStorageBucketListOrder) -> Self {
        match v {
            OpStorageBucketListOrder::Ascending => Self::Ascending,
            OpStorageBucketListOrder::Descending => Self::Descending,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketSortedList.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpStorageBucketSortedList {
    pub bucket_name: String,

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
    pub amount: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketEntry.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpStorageBucketEntry {
    bucket_name: String,
    key: String,
    value: OpStorageBucketValue,
    expires_at: Option<NotBigU64>,
}

impl From<bucketstore::Entry> for OpStorageBucketEntry {
    fn from(v: bucketstore::Entry) -> Self {
        Self {
            bucket_name: v.bucket,
            key: v.key,
            value: v.value.into(),
            expires_at: v.expires_at.map(|e| NotBigU64(e.timestamp_millis() as u64)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/StorageBucketSetCondition.ts")]
pub enum OpStorageBucketSetCondition {
    IfExists,
    IfNotExists,
}

impl From<OpStorageBucketSetCondition> for SetCondition {
    fn from(v: OpStorageBucketSetCondition) -> Self {
        match v {
            OpStorageBucketSetCondition::IfExists => Self::IfExists,
            OpStorageBucketSetCondition::IfNotExists => Self::IfNotExists,
        }
    }
}
