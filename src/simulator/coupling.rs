use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Connectors are configured to connect models through their ports.  During
/// simulation, models exchange messages (as per the Discrete Event System
/// Specification) via these connectors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connector {
    id: String,
    #[serde(rename = "sourceID")]
    source_id: String,
    #[serde(rename = "targetID")]
    target_id: String,
    source_port: String,
    target_port: String,
}

impl Connector {
    pub fn new(
        id: String,
        source_id: String,
        target_id: String,
        source_port: String,
        target_port: String,
    ) -> Self {
        Self {
            id,
            source_id,
            target_id,
            source_port,
            target_port,
        }
    }

    /// This accessor method returns the model ID of the connector source model.
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// This accessor method returns the source port of the connector.
    pub fn source_port(&self) -> &str {
        &self.source_port
    }

    /// This accessor method returns the model ID of the connector target model.
    pub fn target_id(&self) -> &str {
        &self.target_id
    }

    /// This accessor method returns the target port of the connector.
    pub fn target_port(&self) -> &str {
        &self.target_port
    }
}

/// Messages are the mechanism of information exchange for models in a
/// a simulation.  The message must contain origin information (source model
/// ID and source model port), destination information (target model ID and
/// target model port), and the text/content of the message.
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    source_id: String,
    source_port: String,
    target_id: String,
    target_port: String,
    time: f64,
    content: String,
}

impl Message {
    /// This constructor method builds a `Message`, which is passed between
    /// simulation models
    pub fn new(
        source_id: String,
        source_port: String,
        target_id: String,
        target_port: String,
        time: f64,
        content: String,
    ) -> Self {
        Self {
            source_id,
            source_port,
            target_id,
            target_port,
            time,
            content,
        }
    }

    /// This accessor method returns the model ID of a message source.
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// This accessor method returns the source port of a message.
    pub fn source_port(&self) -> &str {
        &self.source_port
    }

    /// This accessor method returns the model ID of a message target.
    pub fn target_id(&self) -> &str {
        &self.target_id
    }

    /// This accessor method returns the target port of a message.
    pub fn target_port(&self) -> &str {
        &self.target_port
    }

    /// This accessor method returns the transmission time of a message.
    pub fn time(&self) -> &f64 {
        &self.time
    }

    /// This accessor method returns the content of a message.
    pub fn content(&self) -> &str {
        &self.content
    }
}
