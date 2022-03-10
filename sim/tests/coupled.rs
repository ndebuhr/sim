use sim::input_modeling::ContinuousRandomVariable;
use sim::models::*;
use sim::output_analysis::*;
use sim::simulator::*;

fn get_message_number(message: &str) -> &str {
    message.split_whitespace().last().unwrap()
}

#[test]
fn closure_under_coupling() {
    let atomic_models = vec![
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
        Model::new(
            String::from("storage-01"),
            Box::new(Storage::new(
                String::from("store"),
                String::from("read"),
                String::from("stored"),
                false,
            )),
        ),
    ];
    let atomic_connectors = vec![
        Connector::new(
            String::from("connector-01"),
            String::from("generator-01"),
            String::from("processor-01"),
            String::from("job"),
            String::from("job"),
        ),
        Connector::new(
            String::from("connector-02"),
            String::from("processor-01"),
            String::from("storage-01"),
            String::from("processed"),
            String::from("store"),
        ),
    ];
    let coupled_models = vec![
        Model::new(
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
        ),
        Model::new(
            String::from("storage-02"),
            Box::new(Storage::new(
                String::from("store"),
                String::from("read"),
                String::from("stored"),
                false,
            )),
        ),
    ];
    let coupled_connectors = vec![
        Connector::new(
            String::from("connector-01"),
            String::from("coupled-01"),
            String::from("storage-02"),
            String::from("start"),
            String::from("store"),
        ),
        Connector::new(
            String::from("connector-02"),
            String::from("coupled-01"),
            String::from("storage-02"),
            String::from("stop"),
            String::from("store"),
        ),
    ];
    let response_times_confidence_intervals: Vec<ConfidenceInterval<f64>> = [
        (atomic_models, atomic_connectors),
        (coupled_models, coupled_connectors),
    ]
    .iter()
    .enumerate()
    .map(|(index, (models, connectors))| {
        let mut simulation = Simulation::post(models.to_vec(), connectors.to_vec());
        let message_records: Vec<Message> = simulation.step_n(1000).unwrap();
        let arrivals: Vec<(&f64, &str)>;
        let departures: Vec<(&f64, &str)>;
        match index {
            0 => {
                arrivals = message_records
                    .iter()
                    .filter(|message_record| message_record.target_id() == "processor-01")
                    .map(|message_record| (message_record.time(), message_record.content()))
                    .collect();
                departures = message_records
                    .iter()
                    .filter(|message_record| message_record.target_id() == "storage-01")
                    .map(|message_record| (message_record.time(), message_record.content()))
                    .collect();
            }
            _ => {
                arrivals = message_records
                    .iter()
                    .filter(|message_record| message_record.target_id() == "storage-02")
                    .filter(|message_record| message_record.source_port() == "start")
                    .map(|message_record| (message_record.time(), message_record.content()))
                    .collect();
                departures = message_records
                    .iter()
                    .filter(|message_record| message_record.target_id() == "storage-02")
                    .filter(|message_record| message_record.source_port() == "stop")
                    .map(|message_record| (message_record.time(), message_record.content()))
                    .collect();
            }
        }
        let response_times: Vec<f64> = departures
            .iter()
            .map(|departure| {
                departure.0
                    - arrivals
                        .iter()
                        .find(|arrival| {
                            get_message_number(&arrival.1) == get_message_number(&departure.1)
                        })
                        .unwrap()
                        .0
            })
            .collect();
        let mut response_times_sample = SteadyStateOutput::post(response_times);
        response_times_sample
            .confidence_interval_mean(0.001)
            .unwrap()
    })
    .collect();
    // Ensure confidence intervals overlap
    assert![
        response_times_confidence_intervals[0].lower()
            < response_times_confidence_intervals[1].upper()
    ];
    assert![
        response_times_confidence_intervals[1].lower()
            < response_times_confidence_intervals[0].upper()
    ];
}
