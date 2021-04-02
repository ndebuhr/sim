use serde::de;
use serde::Deserializer;
use super::model_trait::AsModel;
use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

pub type ModelConstructor = fn(serde_yaml::Value) -> Option<Box<dyn AsModel>>;
lazy_static! {
    static ref CONSTRUCTORS: Mutex<HashMap<String, ModelConstructor>> = Mutex::new(HashMap::new());
}

pub fn create<'de, D: Deserializer<'de>>(model_type: &str, extra_fields: serde_yaml::Value) -> Result<Box<dyn AsModel>, D::Error> {
    const VARIANTS: &'static [&'static str] = &[
        &"Generator", &"ExclusiveGateway", &"Processor", &"Storage"
    ];
    match model_type {
        "Generator" => {
            let generator = serde_yaml::from_value::<super::Generator>(extra_fields).map_err(de::Error::custom)?;
            Ok(Box::new(generator))
        },
        "ExclusiveGateway" => {
            let exclusive_gateway = serde_yaml::from_value::<super::ExclusiveGateway>(extra_fields).map_err(de::Error::custom)?;
            Ok(Box::new(exclusive_gateway))
        },
        "Processor" => {
            let processor = serde_yaml::from_value::<super::Processor>(extra_fields).map_err(de::Error::custom)?;
            Ok(Box::new(processor))
        },
        "Storage" => {
            let storage = serde_yaml::from_value::<super::Storage>(extra_fields).map_err(de::Error::custom)?;
            Ok(Box::new(storage))
        },
        other => {
            Err(de::Error::unknown_variant(other, VARIANTS))
        }
    }
}
