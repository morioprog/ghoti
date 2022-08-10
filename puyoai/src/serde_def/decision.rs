use puyoai_core::decision::Decision;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeAs, SerializeAs};

#[derive(Serialize, Deserialize)]
#[serde(remote = "Decision")]
pub struct DecisionDef {
    #[serde(getter = "Decision::axis_x")]
    x: usize,
    #[serde(getter = "Decision::rot")]
    r: usize,
}

impl From<DecisionDef> for Decision {
    fn from(def: DecisionDef) -> Decision {
        Decision::new(def.x, def.r)
    }
}

impl SerializeAs<Decision> for DecisionDef {
    fn serialize_as<S>(value: &Decision, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        DecisionDef::serialize(value, serializer)
    }
}

impl<'de> DeserializeAs<'de, Decision> for DecisionDef {
    fn deserialize_as<D>(deserializer: D) -> Result<Decision, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DecisionDef::deserialize(deserializer)
    }
}
