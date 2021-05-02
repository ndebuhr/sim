use serde::{Deserialize, Serialize};
use sim::input_modeling::random_variable::*;
use sim::models::*;
use sim::simulator::*;
use sim::SimulationError;

#[test]
fn custom_models() {
    // Implement a custom model type.
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct CustomModel;

    impl AsModel for CustomModel {
        fn status(&self) -> String {
            "testing".to_string()
        }

        fn events_ext(
            &mut self,
            _: ModelMessage,
            _: &mut Services,
        ) -> Result<Vec<ModelMessage>, SimulationError> {
            Ok(vec![])
        }

        fn events_int(&mut self, _: &mut Services) -> Result<Vec<ModelMessage>, SimulationError> {
            Ok(vec![])
        }

        fn time_advance(&mut self, _: f64) {}

        fn until_next_event(&self) -> f64 {
            10.0
        }
    }

    // Create a new enum that implements AsModel and includes the custom model.
    #[derive(Deserialize, Serialize, Clone, Debug)]
    enum ModelType {
        // Built-in models
        Gate(Gate),
        Generator(Generator),
        // Crate user's models
        CustomModel(CustomModel),
    }

    // We could probably write a macro to generate this boilerplate. We can't
    // use enum_dispatch since this is across a crate boundary.
    impl AsModel for ModelType {
        fn status(&self) -> String {
            match self {
                Self::Gate(gate) => gate.status(),
                Self::Generator(gen) => gen.status(),
                Self::CustomModel(m) => m.status(),
            }
        }

        fn events_ext(
            &mut self,
            msg: ModelMessage,
            svc: &mut Services,
        ) -> Result<Vec<ModelMessage>, SimulationError> {
            match self {
                Self::Gate(gate) => gate.events_ext(msg, svc),
                Self::Generator(gen) => gen.events_ext(msg, svc),
                Self::CustomModel(m) => m.events_ext(msg, svc),
            }
        }

        fn events_int(&mut self, svc: &mut Services) -> Result<Vec<ModelMessage>, SimulationError> {
            match self {
                Self::Gate(gate) => gate.events_int(svc),
                Self::Generator(gen) => gen.events_int(svc),
                Self::CustomModel(m) => m.events_int(svc),
            }
        }

        fn time_advance(&mut self, time: f64) {
            match self {
                Self::Gate(gate) => gate.time_advance(time),
                Self::Generator(gen) => gen.time_advance(time),
                Self::CustomModel(m) => m.time_advance(time),
            }
        }

        fn until_next_event(&self) -> f64 {
            match self {
                Self::Gate(gate) => gate.until_next_event(),
                Self::Generator(gen) => gen.until_next_event(),
                Self::CustomModel(m) => m.until_next_event(),
            }
        }
    }

    let models = [
        Model::new(
            String::from("generator-01"),
            ModelType::Generator(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 0.5 },
                None,
                String::from("job"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("custom-01"),
            ModelType::CustomModel(CustomModel),
        ),
    ];

    let connectors = [Connector::new(
        "connector-01".to_string(),
        "generator-01".to_string(),
        "custom-01".to_string(),
        "job".to_string(),
        "testing".to_string(),
    )];

    let _sim = Simulation::post(models.to_vec(), connectors.to_vec());
}
