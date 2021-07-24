use super::model_trait::ReportableModel;
use serde::de;
use serde::Deserializer;
use std::collections::HashMap;

use lazy_static::lazy_static;

use std::sync::Mutex;

pub type ModelConstructor = fn(serde_yaml::Value) -> Option<Box<dyn ReportableModel>>;
lazy_static! {
    static ref CONSTRUCTORS: Mutex<HashMap<&'static str, ModelConstructor>> = {
        let mut m = HashMap::new();
        m.insert("Batcher", super::Batcher::from_value as ModelConstructor);
        m.insert(
            "ExclusiveGateway",
            super::ExclusiveGateway::from_value as ModelConstructor,
        );
        m.insert("Gate", super::Gate::from_value as ModelConstructor);
        m.insert(
            "Generator",
            super::Generator::from_value as ModelConstructor,
        );
        m.insert(
            "LoadBalancer",
            super::LoadBalancer::from_value as ModelConstructor,
        );
        m.insert(
            "ParallelGateway",
            super::ParallelGateway::from_value as ModelConstructor,
        );
        m.insert(
            "Processor",
            super::Processor::from_value as ModelConstructor,
        );
        m.insert(
            "StochasticGate",
            super::StochasticGate::from_value as ModelConstructor,
        );
        m.insert(
            "Stopwatch",
            super::Stopwatch::from_value as ModelConstructor,
        );
        m.insert("Storage", super::Storage::from_value as ModelConstructor);
        Mutex::new(m)
    };
    static ref VARIANTS: Vec<&'static str> = {
        CONSTRUCTORS
            .lock()
            .unwrap()
            .iter()
            .map(|(k, _)| k)
            .copied()
            .collect::<Vec<_>>()
    };
}

pub fn register(model_type: &'static str, model_constructor: ModelConstructor) {
    CONSTRUCTORS
        .lock()
        .unwrap()
        .insert(model_type, model_constructor);
}

pub fn create<'de, D: Deserializer<'de>>(
    model_type: &str,
    extra_fields: serde_yaml::Value,
) -> Result<Box<dyn ReportableModel>, D::Error> {
    let model = match CONSTRUCTORS.lock().unwrap().get(model_type) {
        Some(constructor) => constructor(extra_fields),
        None => None,
    };
    match model {
        Some(model) => Ok(model),
        None => Err(de::Error::unknown_variant(model_type, &VARIANTS)),
    }
}
