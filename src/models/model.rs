use std::any::Any;
use std::fmt;

use serde::de::value::MapAccessDeserializer;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::exclusive::ExclusiveGateway;
use super::gate::Gate;
use super::generator::Generator;
use super::load_balancer::LoadBalancer;
use super::parallel::ParallelGateway;
use super::processor::Processor;
use super::stochastic_gate::StochasticGate;
use super::storage::Storage;
use super::ModelMessage;
use crate::input_modeling::uniform_rng::UniformRNG;

/// The `Model` trait defines everything required for a model to operate
/// within the discrete event simulation.  These requirements are based
/// largely on the Discrete Event System Specification (DEVS), but with a
/// small amount of plumbing (`as_any` and `id`) and a dedicated status
/// reporting method `status`.
pub trait Model {
    fn as_any(&self) -> &dyn Any;
    fn id(&self) -> String;
    fn status(&self) -> String;
    fn events_ext(
        &mut self,
        uniform_rng: &mut UniformRNG,
        incoming_message: ModelMessage,
    ) -> Vec<ModelMessage>;
    fn events_int(&mut self, uniform_rng: &mut UniformRNG) -> Vec<ModelMessage>;
    fn time_advance(&mut self, time_delta: f64);
    fn until_next_event(&self) -> f64;
}

impl Clone for Box<dyn Model> {
    fn clone(&self) -> Box<dyn Model> {
        if let Some(exclusive_gateway) = self.as_any().downcast_ref::<ExclusiveGateway>() {
            Box::new(exclusive_gateway.clone())
        } else if let Some(gate) = self.as_any().downcast_ref::<Gate>() {
            Box::new(gate.clone())
        } else if let Some(generator) = self.as_any().downcast_ref::<Generator>() {
            Box::new(generator.clone())
        } else if let Some(load_balancer) = self.as_any().downcast_ref::<LoadBalancer>() {
            Box::new(load_balancer.clone())
        } else if let Some(parallel_gateway) = self.as_any().downcast_ref::<ParallelGateway>() {
            Box::new(parallel_gateway.clone())
        } else if let Some(processor) = self.as_any().downcast_ref::<Processor>() {
            Box::new(processor.clone())
        } else if let Some(stochastic_gate) = self.as_any().downcast_ref::<StochasticGate>() {
            Box::new(stochastic_gate.clone())
        } else if let Some(storage) = self.as_any().downcast_ref::<Storage>() {
            Box::new(storage.clone())
        } else {
            panic!["Failed to clone component model"];
        }
    }
}

// TODO Consider typetag, instead of custom "dyn model"
// serialization and deserialization, after:
// https://github.com/dtolnay/typetag/issues/8
// https://github.com/dtolnay/typetag/issues/15
// https://github.com/dtolnay/typetag/pull/16

impl Serialize for Box<dyn Model> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(ref exclusive_gateway) = self.as_any().downcast_ref::<ExclusiveGateway>() {
            exclusive_gateway.serialize(serializer)
        } else if let Some(ref gate) = self.as_any().downcast_ref::<Gate>() {
            gate.serialize(serializer)
        } else if let Some(ref generator) = self.as_any().downcast_ref::<Generator>() {
            generator.serialize(serializer)
        } else if let Some(ref load_balancer) = self.as_any().downcast_ref::<LoadBalancer>() {
            load_balancer.serialize(serializer)
        } else if let Some(ref parallel_gateway) = self.as_any().downcast_ref::<ParallelGateway>() {
            parallel_gateway.serialize(serializer)
        } else if let Some(ref processor) = self.as_any().downcast_ref::<Processor>() {
            processor.serialize(serializer)
        } else if let Some(ref stochastic_gate) = self.as_any().downcast_ref::<StochasticGate>() {
            stochastic_gate.serialize(serializer)
        } else if let Some(ref storage) = self.as_any().downcast_ref::<Storage>() {
            storage.serialize(serializer)
        } else {
            panic!["Failed to serialize component model"];
        }
    }
}

struct ModelVisitor;

impl<'de> Visitor<'de> for ModelVisitor {
    type Value = Box<dyn Model>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A software delivery simulator component model")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Box<dyn Model>, V::Error>
    where
        V: MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "type" => {
                    let value = map.next_value::<String>()?;
                    match value.as_str() {
                        "ExclusiveGateway" => {
                            return Ok(Box::new(ExclusiveGateway::deserialize(
                                MapAccessDeserializer::new(map),
                            )?));
                        }
                        "Gate" => {
                            return Ok(Box::new(Gate::deserialize(MapAccessDeserializer::new(
                                map,
                            ))?));
                        }
                        "Generator" => {
                            return Ok(Box::new(Generator::deserialize(
                                MapAccessDeserializer::new(map),
                            )?));
                        }
                        "LoadBalancer" => {
                            return Ok(Box::new(LoadBalancer::deserialize(
                                MapAccessDeserializer::new(map),
                            )?));
                        }
                        "ParallelGateway" => {
                            return Ok(Box::new(ParallelGateway::deserialize(
                                MapAccessDeserializer::new(map),
                            )?));
                        }
                        "Processor" => {
                            return Ok(Box::new(Processor::deserialize(
                                MapAccessDeserializer::new(map),
                            )?));
                        }
                        "StochasticGate" => {
                            return Ok(Box::new(StochasticGate::deserialize(
                                MapAccessDeserializer::new(map),
                            )?));
                        }
                        "Storage" => {
                            return Ok(Box::new(Storage::deserialize(
                                MapAccessDeserializer::new(map),
                            )?));
                        }
                        &_ => {
                            return Err(de::Error::custom(
                                "A model type was not recognized during deserialization",
                            ));
                        }
                    }
                }
                &_ => {}
            }
        }
        Err(de::Error::custom("Failed to deserialize component model"))
    }
}

impl<'de> Deserialize<'de> for Box<dyn Model> {
    // TODO - Assumes the type field is at the beginning - make this order agnostic
    fn deserialize<D>(deserializer: D) -> Result<Box<dyn Model>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ModelVisitor)
    }
}
