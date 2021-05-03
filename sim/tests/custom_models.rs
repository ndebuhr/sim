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
    #[derive(Deserialize, Serialize, Clone, Debug, AsModel)]
    enum ModelType {
        // Built-in models
        Gate(Gate),
        Generator(Generator),
        // Crate user's models
        CustomModel(CustomModel),
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
