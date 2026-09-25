pub mod active_period;
pub mod auth;
pub mod board;
pub mod checklist;
pub mod dashboard;
pub mod edge;
pub mod node;
pub mod poke;
pub mod topic;
pub mod user;

use serde::{Deserialize, Deserializer};

/// Pairs with `#[serde(default, deserialize_with = "deserialize_some")]` on an
/// `Option<Option<T>>` field: distinguishes "key omitted" (`None`) from "key
/// present with value `null`" (`Some(None)`) for PATCH-style partial updates
/// on nullable columns. Standard serde idiom, no new dependency.
pub(crate) fn deserialize_some<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}
