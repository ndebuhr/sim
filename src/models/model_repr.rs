use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelExtra {
    pub id: String,
    #[serde(rename="type")]
    pub model_type: String,
    #[serde(flatten)]
    pub extra: Value
}
