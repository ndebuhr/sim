#[cfg(feature = "simx")]
use std::fs;

#[cfg(feature = "simx")]
use sim::input_modeling::ContinuousRandomVariable;
#[cfg(feature = "simx")]
use sim::models::{Batcher, DevsModel, Generator};

#[test]
#[cfg(feature = "simx")]
fn batcher_event_rules() {
    let batcher = Batcher::new(String::from("job"), String::from("job"), 0.5, 10, false);

    let batcher_event_rules = fs::read_to_string("tests/batcher_event_rules.json")
        .expect("Unable to read tests/batcher_event_rules.json");

    assert_eq!(batcher.event_rules(), batcher_event_rules);
}

#[test]
#[cfg(feature = "simx")]
fn generator_event_rules() {
    let generator = Generator::new(
        ContinuousRandomVariable::Exp { lambda: 0.5 },
        None,
        String::from("job"),
        false,
    );

    let generator_event_rules = fs::read_to_string("tests/generator_event_rules.json")
        .expect("Unable to read tests/generator_event_rules.json");

    assert_eq!(generator.event_rules(), generator_event_rules);
}
