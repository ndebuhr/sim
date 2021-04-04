use serde::de;
use serde::Deserializer;
use super::model_trait::AsModel;
use std::collections::HashMap;

use lazy_static::lazy_static;

use std::sync::Mutex;

pub type ModelConstructor = fn(serde_yaml::Value) -> Option<Box<dyn AsModel>>;
lazy_static! {
    static ref CONSTRUCTORS: Mutex<HashMap<&'static str, ModelConstructor>> = {
        let mut m = HashMap::new();
        m.insert("Generator", super::Generator::from_value as ModelConstructor);
        m.insert("ExclusiveGateway", super::ExclusiveGateway::from_value as ModelConstructor);
        m.insert("Processor", super::Processor::from_value as ModelConstructor);
        m.insert("Storage", super::Storage::from_value as ModelConstructor);
        m.insert("Gate", super::Gate::from_value as ModelConstructor);
        m.insert("LoadBalancer", super::LoadBalancer::from_value as ModelConstructor);
        m.insert("ParallelGateway", super::ParallelGateway::from_value as ModelConstructor);
        m.insert("StochasticGate", super::StochasticGate::from_value as ModelConstructor);
        Mutex::new(m)
    };
    static ref VARIANTS: Vec<&'static str> = {
        CONSTRUCTORS.lock().unwrap().iter().map(|(k, _)| k).map(|&x| x).collect::<Vec<_>>()
    };
}

pub fn register(model_type: &'static str, model_constructor: ModelConstructor) {
    CONSTRUCTORS.lock().unwrap().insert(model_type, model_constructor);
}

pub fn create<'de, D: Deserializer<'de>>(model_type: &str, extra_fields: serde_yaml::Value) -> Result<Box<dyn AsModel>, D::Error> {
    match CONSTRUCTORS.lock().unwrap().get(model_type) {
        Some(constructor) => {
            match constructor(extra_fields) {
                Some(model) => Ok(model),
                None => Err(de::Error::unknown_variant(model_type, &VARIANTS))
            }
        },
        None => {
            Err(de::Error::unknown_variant(model_type, &VARIANTS))
        }
    }
}
