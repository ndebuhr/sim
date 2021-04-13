use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelRepr {
    pub id: String,
    #[serde(rename = "type")]
    pub model_type: String,
    #[serde(flatten)]
    pub extra: serde_yaml::Value,
}
