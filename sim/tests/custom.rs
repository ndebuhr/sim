use std::f64::INFINITY;

use serde::{Deserialize, Serialize};
use sim::input_modeling::ContinuousRandomVariable;
use sim::models::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use sim::models::{Generator, Model, ModelMessage, ModelRecord};
use sim::simulator::{Connector, Message, Services, Simulation, WebSimulation};
use sim::utils::errors::SimulationError;
use sim_derive::{register, SerializableModel};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

#[cfg(feature = "simx")]
use simx::event_rules;

wasm_bindgen_test_configure!(run_in_browser);

/// The passive model does nothing
#[derive(Debug, Clone, Serialize, Deserialize, SerializableModel)]
#[serde(rename_all = "camelCase")]
pub struct Passive {
    ports_in: PortsIn,
    #[serde(default)]
    state: State,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsIn {
    job: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct State {
    records: Vec<ModelRecord>,
}

#[cfg_attr(feature = "simx", event_rules)]
impl Passive {
    pub fn new(job_port: String) -> Self {
        Self {
            ports_in: PortsIn { job: job_port },
            state: State {
                records: Vec::new(),
            },
        }
    }
}

#[cfg_attr(feature = "simx", event_rules)]
impl DevsModel for Passive {
    fn events_ext(
        &mut self,
        _incoming_message: &ModelMessage,
        _services: &mut Services,
    ) -> Result<(), SimulationError> {
        Ok(())
    }

    fn events_int(
        &mut self,
        _services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        Ok(Vec::new())
    }

    fn time_advance(&mut self, _time_delta: f64) {
        // No future events list to advance
    }

    fn until_next_event(&self) -> f64 {
        // No future events list, as a source of finite until_next_event
        // values
        INFINITY
    }
}

impl Reportable for Passive {
    fn status(&self) -> String {
        "Passive".into()
    }

    fn records(&self) -> &Vec<ModelRecord> {
        &self.state.records
    }
}

impl ReportableModel for Passive {}

#[test]
fn step_n_with_custom_passive_model() -> Result<(), SimulationError> {
    let models = [
        Model::new(
            String::from("generator-01"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 0.5 },
                None,
                String::from("job"),
                false,
            )),
        ),
        Model::new(
            String::from("passive-01"),
            Box::new(Passive::new(String::from("job"))),
        ),
    ];
    let connectors = [Connector::new(
        String::from("connector-01"),
        String::from("generator-01"),
        String::from("passive-01"),
        String::from("job"),
        String::from("job"),
    )];
    let mut simulation = Simulation::post(models.to_vec(), connectors.to_vec());
    // 1 initialization event, and 2 events per generation
    let messages = simulation.step_n(9)?;
    let generations_count = messages.len();
    let expected = 4; // 4 interarrivals from 9 steps
    assert_eq!(generations_count, expected);
    Ok(())
}

#[test]
#[wasm_bindgen_test]
fn step_n_with_custom_passive_model_wasm() {
    let models = r#"
- type: "Generator"
  id: "generator-01"
  portsIn: {}
  portsOut:
    job: "job"
  messageInterdepartureTime:
    exp:
      lambda: 0.5
- type: "Passive"
  id: "passive-01"
  portsIn:
    job: "job"
"#;
    let connectors = r#"
- id: "connector-01"
  sourceID: "generator-01"
  targetID: "passive-01"
  sourcePort: "job"
  targetPort: "job"
"#;
    register![Passive];
    let mut simulation = WebSimulation::post_yaml(&models, &connectors);
    // 1 initialization event, and 2 events per generation
    let messages: Vec<Message> = serde_json::from_str(&simulation.step_n_json(9)).unwrap();
    let generations_count = messages.len();
    let expected = 4; // 4 interarrivals from 9 steps
    assert_eq!(generations_count, expected);
}
