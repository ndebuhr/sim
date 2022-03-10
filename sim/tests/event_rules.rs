#[cfg(feature = "simx")]
use {
    sim::input_modeling::ContinuousRandomVariable,
    sim::models::{
        Batcher, Coupled, DevsModel, ExternalOutputCoupling, Generator, InternalCoupling, Model,
        Processor,
    },
    std::fs,
};

#[cfg(feature = "simx")]
fn strip_whitespace(string: String) -> String {
    string.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
#[cfg(feature = "simx")]
fn batcher_event_rules() {
    let batcher = Batcher::new(String::from("job"), String::from("job"), 0.5, 10, false);

    let batcher_event_rules = fs::read_to_string("tests/data/batcher_event_rules.json")
        .expect("Unable to read tests/batcher_event_rules.json");

    assert_eq!(
        strip_whitespace(batcher.event_rules()),
        strip_whitespace(batcher_event_rules)
    );
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

    let generator_event_rules = fs::read_to_string("tests/data/generator_event_rules.json")
        .expect("Unable to read tests/generator_event_rules.json");

    assert_eq!(
        strip_whitespace(generator.event_rules()),
        strip_whitespace(generator_event_rules)
    );
}

#[test]
#[cfg(feature = "simx")]
fn coupled_event_rules() {
    let coupled = Model::new(
        String::from("coupled-01"),
        Box::new(Coupled::new(
            Vec::new(),
            vec![String::from("start"), String::from("stop")],
            vec![
                Model::new(
                    String::from("generator-01"),
                    Box::new(Generator::new(
                        ContinuousRandomVariable::Exp { lambda: 0.007 },
                        None,
                        String::from("job"),
                        false,
                    )),
                ),
                Model::new(
                    String::from("processor-01"),
                    Box::new(Processor::new(
                        ContinuousRandomVariable::Exp { lambda: 0.011 },
                        Some(14),
                        String::from("job"),
                        String::from("processed"),
                        false,
                    )),
                ),
            ],
            Vec::new(),
            vec![
                ExternalOutputCoupling {
                    source_id: String::from("generator-01"),
                    source_port: String::from("job"),
                    target_port: String::from("start"),
                },
                ExternalOutputCoupling {
                    source_id: String::from("processor-01"),
                    source_port: String::from("processed"),
                    target_port: String::from("stop"),
                },
            ],
            vec![InternalCoupling {
                source_id: String::from("generator-01"),
                target_id: String::from("processor-01"),
                source_port: String::from("job"),
                target_port: String::from("job"),
            }],
        )),
    );
    let coupled_event_rules = fs::read_to_string("tests/data/coupled_event_rules.json")
        .expect("Unable to read tests/coupled_event_rules.json");

    assert_eq!(
        strip_whitespace(coupled.event_rules()),
        strip_whitespace(coupled_event_rules)
    );
}
