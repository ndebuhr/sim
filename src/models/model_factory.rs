use serde::de;
use serde::Deserializer;
use super::model_trait::AsModel;
use std::collections::HashMap;

use lazy_static::lazy_static;

fn generator_new(value: serde_yaml::Value) -> Option<Box<dyn AsModel>> {
    match serde_yaml::from_value::<super::Generator>(value) {
        Ok(generator) => Some(Box::new(generator)),
        Err(_) => None
    }
}

fn exclusive_gateway_new(value: serde_yaml::Value) -> Option<Box<dyn AsModel>> {
    match serde_yaml::from_value::<super::ExclusiveGateway>(value) {
        Ok(exclusive_gateway) => Some(Box::new(exclusive_gateway)),
        Err(_) => None
    }
}

fn processor_new(value: serde_yaml::Value) -> Option<Box<dyn AsModel>> {
    match serde_yaml::from_value::<super::Processor>(value) {
        Ok(processor) => Some(Box::new(processor)),
        Err(_) => None
    }
}

fn storage_new(value: serde_yaml::Value) -> Option<Box<dyn AsModel>> {
    match serde_yaml::from_value::<super::Storage>(value) {
        Ok(storage) => Some(Box::new(storage)),
        Err(_) => None
    }
}

pub type ModelConstructor = fn(serde_yaml::Value) -> Option<Box<dyn AsModel>>;
lazy_static! {
    static ref CONSTRUCTORS: HashMap<&'static str, ModelConstructor> = {
        let mut m = HashMap::new();
        m.insert("Generator", generator_new as ModelConstructor);
        m.insert("ExclusiveGateway", exclusive_gateway_new as ModelConstructor);
        m.insert("Processor", processor_new as ModelConstructor);
        m.insert("Storage", storage_new as ModelConstructor);
        m
    };
}

pub fn create<'de, D: Deserializer<'de>>(model_type: &str, extra_fields: serde_yaml::Value) -> Result<Box<dyn AsModel>, D::Error> {
    const VARIANTS: &'static [&'static str] = &[
        &"Generator", &"ExclusiveGateway", &"Processor", &"Storage"
    ];
    match CONSTRUCTORS.get(model_type) {
        Some(constructor) => {
            match constructor(extra_fields) {
                Some(model) => Ok(model),
                None => Err(de::Error::unknown_variant(model_type, VARIANTS))
            }
        },
        None => {
            Err(de::Error::unknown_variant(model_type, VARIANTS))
        }
    }
    // match model_type {
    //     "Generator" => {
    //         let generator = serde_yaml::from_value::<super::Generator>(extra_fields).map_err(de::Error::custom)?;
    //         Ok(Box::new(generator))
    //     },
    //     "ExclusiveGateway" => {
    //         let exclusive_gateway = serde_yaml::from_value::<super::ExclusiveGateway>(extra_fields).map_err(de::Error::custom)?;
    //         Ok(Box::new(exclusive_gateway))
    //     },
    //     "Processor" => {
    //         let processor = serde_yaml::from_value::<super::Processor>(extra_fields).map_err(de::Error::custom)?;
    //         Ok(Box::new(processor))
    //     },
    //     "Storage" => {
    //         let storage = serde_yaml::from_value::<super::Storage>(extra_fields).map_err(de::Error::custom)?;
    //         Ok(Box::new(storage))
    //     },
    //     other => {
    //     }
    // }
}
