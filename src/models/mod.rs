pub mod exclusive;
pub mod gate;
pub mod generator;
pub mod load_balancer;
pub mod model;
pub mod parallel;
pub mod processor;
pub mod stochastic_gate;
pub mod storage;

#[derive(Debug, Clone)]
pub struct ModelMessage {
    pub port_name: String,
    pub message: String,
}
