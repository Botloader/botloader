use serde::{Deserialize, Deserializer};

pub mod discord;
pub mod events;
pub mod ops;
pub mod util;

pub(crate) fn deserialize_optional_field<'de, T, D>(
    deserializer: D,
) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}
