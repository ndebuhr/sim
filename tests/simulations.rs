use std::f64::INFINITY;

use serde::{Deserialize, Serialize};
use sim::input_modeling::random_variable::*;
use sim::models::*;
use sim::output_analysis::*;
use sim::simulator::*;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProcessorMetrics {
    queue_size: usize,
    last_arrival: Option<(String, f64)>,
    last_service_start: Option<(String, f64)>,
    last_completion: Option<(String, f64)>,
    is_utilized: bool,
}

fn epsilon() -> f64 {
    0.34
}

fn get_message_number(message: &str) -> &str {
    message.split_whitespace().last().unwrap()
}

#[test]
fn poisson_generator_processor_with_capacity() {
    let models = [
        Model::new(
            String::from("generator-01"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 0.5 },
                None,
                String::from("job"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("processor-01"),
            Box::new(Processor::new(
                ContinuousRandomVariable::Exp { lambda: 0.333333 },
                14,
                String::from("job"),
                String::from("processed"),
                false,
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
                false,
            )),
        ),
    ];
    let connectors = [
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
    // A Poisson generator (mean of 0.5) arrival pattern (exponential interarrival with mean 2)
    // A processor with exponential processing time, mean processing time 3.0, and queue capacity 14
    // A stage for processed job collection
    let mut simulation = Simulation::post(models.to_vec(), connectors.to_vec());
    // Sample size will be reduced during output analysis - initialization bias reduction through deletion
    let message_records: Vec<Message> = simulation.step_n(3000).unwrap();
    let departures: Vec<(&f64, &str)> = message_records
        .iter()
        .filter(|message_record| message_record.target_id() == "storage-01")
        .map(|message_record| (message_record.time(), message_record.content()))
        .collect();
    let arrivals: Vec<(&f64, &str)> = message_records
        .iter()
        .filter(|message_record| message_record.target_id() == "processor-01")
        .map(|message_record| (message_record.time(), message_record.content()))
        .collect();
    // Response Times
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
    // Response times are not independent
    // Varying queue size leads to auto-correlation
    // To combat this, use steady state output analysis with deletion+batching
    let mut response_times_sample = SteadyStateOutput::post(response_times);
    let response_times_confidence_interval = response_times_sample
        .confidence_interval_mean(0.001)
        .unwrap();
    // average number of jobs in the processor divided by the effective arrival rate (Little's Formula)
    let expected = (172285188.0 / 14316139.0) / (4766600.0 / 14316169.0);
    assert!(response_times_confidence_interval.lower() < expected);
    assert!(response_times_confidence_interval.upper() > expected);

    // Effective Arrival Rate
    let last_processed_job = get_message_number(&departures.iter().last().unwrap().1);
    let count_generated = arrivals
        .iter()
        .position(|arrival| get_message_number(&arrival.1) == last_processed_job)
        .unwrap()
        + 1;
    let count_processed = departures.len();
    // Effective arrival rate as the generated rate multiplied by the percent of jobs "served" (not ignored due to a full queue)
    let effective_arrival_rate = 0.5 * ((count_processed as f64) / (count_generated as f64));
    let expected = 4766600.0 / 14316169.0;
    assert!((effective_arrival_rate - expected).abs() / expected < epsilon());
}

#[test]
fn processor_from_queue_response_time_is_correct() {
    let models = r#"
[
    {
        "type": "Processor",
        "id": "processor-01",
        "portsIn": {
            "job": "job"
        },
        "portsOut": {
            "processedJob": "processed job"
        },
        "serviceTime": {
            "exp": {
                "lambda": 3.0
            }
        },
        "state": {
            "eventList": [
                {
                    "time": 0.0,
                    "event": "BeginProcessing"
                }
            ],
            "untilJobCompletion": 0.0,
            "queue": [
                "job 1", "job 2", "job 3", "job 4", "job 5",
                "job 6", "job 7", "job 8", "job 9", "job 10",
                "job 11", "job 12", "job 13", "job 14", "job 15",
                "job 16", "job 17", "job 18", "job 19", "job 20",
                "job 21", "job 22", "job 23", "job 24", "job 25",
                "job 26", "job 27", "job 28", "job 29", "job 30",
                "job 31", "job 32", "job 33", "job 34", "job 35",
                "job 36", "job 37", "job 38", "job 39", "job 40",
                "job 41", "job 42", "job 43", "job 44", "job 45",
                "job 46", "job 47", "job 48", "job 49", "job 50",
                "job 51", "job 52", "job 53", "job 54", "job 55",
                "job 56", "job 57", "job 58", "job 59", "job 60",
                "job 61", "job 62", "job 63", "job 64", "job 65",
                "job 66", "job 67", "job 68", "job 69", "job 70",
                "job 71", "job 72", "job 73", "job 74", "job 75",
                "job 76", "job 77", "job 78", "job 79", "job 80",
                "job 81", "job 82", "job 83", "job 84", "job 85",
                "job 86", "job 87", "job 88", "job 89", "job 90",
                "job 91", "job 92", "job 93", "job 94", "job 95",
                "job 96", "job 97", "job 98", "job 99", "job 100"
            ],
            "phase": "Active"
        }
    },
    {
        "type": "Storage",
        "id": "storage-01",
        "portsIn": {
            "store": "store",
            "read": "read"
        },
        "portsOut": {
            "stored": "stored"
        }
    }
]"#;
    let connectors = r#"
[
    {
        "id": "connector-01",
        "sourceID": "processor-01",
        "targetID": "storage-01",
        "sourcePort": "processed job",
        "targetPort": "store"
    }
]"#;
    let mut web = WebSimulation::post_json(models, connectors);
    let average_batch_completion_time = (0..200) // 100 jobs, and 2 steps per job
        .map(|simulation_step| {
            // Get expected Option<String> message at each step
            if (simulation_step + 1) % 2 == 0 {
                Some(format!["processed job {:?}", (simulation_step + 1) / 2])
            } else {
                None
            }
        })
        .map(|expected_output| {
            // Run simulation and capture output messages
            // Assert based on None vs. Some(String) message expectations
            let messages_json = web.step_json();
            let messages_set: Vec<Message> = serde_json::from_str(&messages_json).unwrap();
            match expected_output {
                None => {
                    assert![messages_set.is_empty()];
                    INFINITY
                }
                Some(output) => {
                    let first_message = messages_set.first().unwrap();
                    assert![first_message.content() == output];
                    *first_message.time()
                }
            }
        })
        .enumerate()
        .filter(|(index, _)| {
            // Get only job completion times, skipping events with no job completion
            (index + 1) % 2 == 0
        })
        .map(|(_, job_completion_time)| job_completion_time)
        .enumerate()
        .fold(
            (Vec::new(), 0.0),
            |mut batch_completion_times, (job_index, job_completion_time)| {
                // Compile batch completion times - 50 batches of 2 jobs each
                // batch_completion_times.1 is the global time of the last batch completion
                if (job_index + 1) % 2 == 0 {
                    batch_completion_times
                        .0
                        .push(job_completion_time - batch_completion_times.1);
                    batch_completion_times.1 = job_completion_time
                }
                batch_completion_times
            },
        )
        .0
        .iter()
        // Take the average completion time across the 20 batches
        .sum::<f64>()
        / 50.0;
    let expectation = 1.0 / 3.0; // Exponential with lambda=3.0
    assert!(
        (average_batch_completion_time - 2.0 * expectation).abs() / (2.0 * expectation) < epsilon()
    );
}

#[test]
fn processor_network_no_job_loss() {
    let models = r#"
[
    {
        "type": "Processor",
        "id": "processor-0",
        "portsIn": {
            "job": "job"
        },
        "portsOut": {
            "processedJob": "processed job"
        },
        "serviceTime": {
            "exp": {
                "lambda": 1.0
            }
        },
        "state": {
            "eventList": [
                {
                    "time": 0.0,
                    "event": "BeginProcessing"
                }
            ],
            "untilJobCompletion": 0.0,
            "queue": [
                "job 0",
                "job 1",
                "job 2",
                "job 3",
                "job 4",
                "job 5",
                "job 6",
                "job 7",
                "job 8",
                "job 9"
            ],
            "phase": "Active"
        }
    },
    {
        "type": "Processor",
        "id": "processor-1",
        "portsIn": {
            "job": "job"
        },
        "portsOut": {
            "processedJob": "processed job"
        },
        "serviceTime": {
            "exp": {
                "lambda": 2.0
            }
        }
    },
    {
        "type": "Processor",
        "id": "processor-2",
        "portsIn": {
            "job": "job"
        },
        "portsOut": {
            "processedJob": "processed job"
        },
        "serviceTime": {
            "exp": {
                "lambda": 3.0
            }
        }
    },
    {
        "type": "Processor",
        "id": "processor-3",
        "portsIn": {
            "job": "job"
        },
        "portsOut": {
            "processedJob": "processed job"
        },
        "serviceTime": {
            "exp": {
                "lambda": 5.0
            }
        }
    },
    {
        "type": "Processor",
        "id": "processor-4",
        "portsIn": {
            "job": "job"
        },
        "portsOut": {
            "processedJob": "processed job"
        },
        "serviceTime": {
            "exp": {
                "lambda": 7.0
            }
        }
    },
    {
        "type": "Processor",
        "id": "processor-5",
        "portsIn": {
            "job": "job"
        },
        "portsOut": {
            "processedJob": "processed job"
        },
        "serviceTime": {
            "exp": {
                "lambda": 11.0
            }
        }
    },
    {
        "type": "Storage",
        "id": "storage-0",
        "portsIn": {
            "store": "store",
            "read": "read"
        },
        "portsOut": {
            "stored": "stored"
        }
    }
]"#;
    let connectors = r#"
[
    {
        "id": "connector-01",
        "sourceID": "processor-0",
        "targetID": "processor-1",
        "sourcePort": "processed job",
        "targetPort": "job"
    },
    {
        "id": "connector-02",
        "sourceID": "processor-1",
        "targetID": "processor-4",
        "sourcePort": "processed job",
        "targetPort": "job"
    },
    {
        "id": "connector-03",
        "sourceID": "processor-4",
        "targetID": "storage-0",
        "sourcePort": "processed job",
        "targetPort": "store"
    },
    {
        "id": "connector-04",
        "sourceID": "processor-1",
        "targetID": "processor-3",
        "sourcePort": "processed job",
        "targetPort": "job"
    },
    {
        "id": "connector-05",
        "sourceID": "processor-0",
        "targetID": "processor-2",
        "sourcePort": "processed job",
        "targetPort": "job"
    },
    {
        "id": "connector-06",
        "sourceID": "processor-2",
        "targetID": "processor-5",
        "sourcePort": "processed job",
        "targetPort": "job"
    },
    {
        "id": "connector-07",
        "sourceID": "processor-3",
        "targetID": "processor-5",
        "sourcePort": "processed job",
        "targetPort": "job"
    },
    {
        "id": "connector-08",
        "sourceID": "processor-5",
        "targetID": "storage-0",
        "sourcePort": "processed job",
        "targetPort": "store"
    }
]"#;
    let mut web = WebSimulation::post_json(models, connectors);
    let mut message_records: Vec<Message> = Vec::new();
    // Needs to be around 360+ steps (10 jobs, 3 network paths, across 6 processors, and 2 events per processing cycle)
    for _x in 0..720 {
        let messages_json = web.step_json();
        let messages_set: Vec<Message> = serde_json::from_str(&messages_json).unwrap();
        message_records.extend(messages_set);
    }
    let storage_arrivals_count = message_records
        .iter()
        .filter(|message_record| message_record.target_id() == "storage-0")
        .count();
    let expected = 3 * 10; // 10 jobs traversing three paths through network
    assert!(storage_arrivals_count == expected);
}

#[test]
fn simulation_serialization_deserialization_field_ordering() {
    // Confirm field order does not matter for yaml deserialization
    let models = r#"
- id: "generator-01"
  portsIn: {}
  portsOut:
    job: "job"
  messageInterdepartureTime:
    exp:
      lambda: 0.5
  type: "Generator"
- type: "Processor"
  portsIn:
    job: "job"
  portsOut:
    processedJob: "processed job"
  serviceTime:
    exp:
      lambda: 0.333333
  queueCapacity: 14
  id: "processor-01"
- portsIn:
    store: "store"
    read: "read"
  portsOut:
    stored: "stored"
  type: "Storage"
  id: "storage-01"
"#;
    let connectors = r#"
- sourcePort: "job"
  targetPort: "job"
  id: "connector-01"
  sourceID: "generator-01"
  targetID: "processor-01"
- id: "connector-02"
  targetPort: "store"
  targetID: "storage-01"
  sourcePort: "processed job"
  sourceID: "processor-01"
"#;
    WebSimulation::post_yaml(models, connectors);
}

#[test]
fn step_until_activities() {
    let models = [
        Model::new(
            String::from("generator-01"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 0.5 },
                None,
                String::from("job"),
                false,
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
                false,
            )),
        ),
    ];
    let connectors = [Connector::new(
        String::from("connector-01"),
        String::from("generator-01"),
        String::from("storage-01"),
        String::from("job"),
        String::from("store"),
    )];
    let mut generations_count: Vec<f64> = Vec::new();
    let mut simulation = Simulation::default();
    // 10 replications
    for _ in 0..10 {
        // Refresh the models, but maintain the Uniform RNG for replication independence
        simulation.reset();
        simulation.put(models.to_vec(), connectors.to_vec());
        let messages = simulation.step_until(100.0).unwrap();
        generations_count.push(messages.len() as f64);
    }
    let generations_per_replication = IndependentSample::post(generations_count).unwrap();
    let generations_per_replication_ci = generations_per_replication
        .confidence_interval_mean(0.001)
        .unwrap();
    let expected = 50.0; // 50 interarrivals - 1/0.5 mean and 100 duration
    assert!(generations_per_replication_ci.lower() < expected);
    assert!(generations_per_replication_ci.upper() > expected);
}

#[test]
fn simulation_serialization_deserialization_round_trip() {
    // Confirm a round trip deserialization-serialization
    let s_models = r#"
- type: "Generator"
  id: "generator-01"
  portsIn: {}
  portsOut:
    job: "job"
  messageInterdepartureTime:
    exp:
      lambda: 0.5
- type: "Processor"
  id: "processor-01"
  portsIn:
    job: "job"
  portsOut:
    processedJob: "processed job"
  serviceTime:
    exp:
      lambda: 0.333333
  queueCapacity: 14
- type: "Storage"
  id: "storage-01"
  portsIn:
    store: "store"
    read: "read"
  portsOut:
    stored: "stored"
"#;
    let s_connectors = r#"
- id: "connector-01"
  sourceID: "generator-01"
  targetID: "processor-01"
  sourcePort: "job"
  targetPort: "job"
- id: "connector-02"
  sourceID: "processor-01"
  targetID: "storage-01"
  sourcePort: "processed job"
  targetPort: "store"
"#;
    let models: Vec<Model> = serde_yaml::from_str(s_models).unwrap();
    let connectors: Vec<Connector> = serde_yaml::from_str(s_connectors).unwrap();
    WebSimulation::post_yaml(
        &serde_yaml::to_string(&models).unwrap(),
        &serde_yaml::to_string(&connectors).unwrap(),
    );
}

#[test]
fn non_stationary_generation() {
    let models = [
        Model::new(
            String::from("generator-01"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 0.0957 },
                None,
                String::from("job"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("processor-01"),
            Box::new(Processor::new(
                ContinuousRandomVariable::Exp { lambda: 0.1659 },
                14,
                String::from("job"),
                String::from("processed"),
                false,
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
                false,
            )),
        ),
    ];
    let connectors = [
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
    let mut simulation = Simulation::default();
    let mut message_records: Vec<Message> = Vec::new();
    let mut arrivals_count: Vec<f64> = Vec::new();
    // 10 replications
    for _ in 0..10 {
        // Refresh the models, but maintain the Uniform RNG for replication independence
        simulation.reset();
        simulation.put(models.to_vec(), connectors.to_vec());
        let messages = simulation.step_until(480.0).unwrap();
        let arrivals: Vec<&Message> = messages
            .iter()
            .filter(|message| message.target_id() == "processor-01")
            .collect();
        arrivals_count.push(arrivals.len() as f64);
        message_records.extend(messages);
    }
    let arrivals_ci = IndependentSample::post(arrivals_count)
        .unwrap()
        .confidence_interval_mean(0.05)
        .unwrap();
    // Confirm empirical CI and simulation output CI overlap
    let empirical_arrivals = IndependentSample::post(vec![47.0, 42.0, 45.0, 34.0, 37.0]).unwrap();
    let empirical_arrivals_ci = empirical_arrivals.confidence_interval_mean(0.05).unwrap();
    assert!(
        arrivals_ci.lower() < empirical_arrivals_ci.upper()
            && arrivals_ci.upper() > empirical_arrivals_ci.lower()
    );
}

#[test]
fn exclusive_gateway_proportions_chi_square() {
    let models = [
        Model::new(
            String::from("generator-01"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 5.0 },
                None,
                String::from("job"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("exclusive-01"),
            Box::new(ExclusiveGateway::new(
                vec![String::from("in")],
                vec![
                    String::from("s01"),
                    String::from("s02"),
                    String::from("s03"),
                ],
                IndexRandomVariable::WeightedIndex {
                    weights: vec![6, 3, 1],
                },
                false,
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
                false,
            )),
        ),
        Model::new(
            String::from("storage-02"),
            Box::new(Storage::new(
                String::from("store"),
                String::from("read"),
                String::from("stored"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("storage-03"),
            Box::new(Storage::new(
                String::from("store"),
                String::from("read"),
                String::from("stored"),
                false,
                false,
            )),
        ),
    ];
    let connectors = [
        Connector::new(
            String::from("connector-01"),
            String::from("generator-01"),
            String::from("exclusive-01"),
            String::from("job"),
            String::from("in"),
        ),
        Connector::new(
            String::from("connector-02"),
            String::from("exclusive-01"),
            String::from("storage-01"),
            String::from("s01"),
            String::from("store"),
        ),
        Connector::new(
            String::from("connector-03"),
            String::from("exclusive-01"),
            String::from("storage-02"),
            String::from("s02"),
            String::from("store"),
        ),
        Connector::new(
            String::from("connector-04"),
            String::from("exclusive-01"),
            String::from("storage-03"),
            String::from("s03"),
            String::from("store"),
        ),
    ];
    let mut simulation = Simulation::post(models.to_vec(), connectors.to_vec());
    let mut message_records: Vec<Message> = Vec::new();
    // 601 steps means 200 processed jobs (3 steps per gateway passthrough)
    // 1 initialization step
    for _x in 0..601 {
        let messages_set: Vec<Message> = simulation.step().unwrap();
        message_records.extend(messages_set);
    }
    let outputs = vec![
        message_records
            .iter()
            .filter(|message_record| message_record.target_id() == "storage-01")
            .count(),
        message_records
            .iter()
            .filter(|message_record| message_record.target_id() == "storage-02")
            .count(),
        message_records
            .iter()
            .filter(|message_record| message_record.target_id() == "storage-03")
            .count(),
    ];
    let per_class_expected = [120, 60, 20];
    let chi_square = outputs
        .iter()
        .enumerate()
        .fold(0.0, |acc, (index, per_class_observed)| {
            acc + (*per_class_observed as f64 - per_class_expected[index] as f64).powi(2)
                / (per_class_expected[index] as f64)
        });
    assert![outputs.iter().sum::<usize>() == 200];
    // 3 bins, 2 dof, 0.01 alpha
    let chi_square_critical = 9.21;
    assert![chi_square < chi_square_critical];
}

#[test]
fn ci_half_width_for_average_waiting_time() {
    let models = r#"
- type: "Generator"
  id: "generator-01"
  portsIn: {}
  portsOut:
    job: "job"
  messageInterdepartureTime:
    exp:
      lambda: 0.0957
  thinning:
    function:
      polynomial:
        coefficients: [1, -0.000422]
- type: "ExclusiveGateway"
  id: "exclusive-01"
  portsIn:
    flowPaths:
    - "in"
  portsOut:
    flowPaths:
    - "p01"
    - "p02"
    - "p03"
  portWeights:
    weightedIndex:
      weights: [6, 3, 1]
- type: "Processor"
  id: "processor-01"
  portsIn:
    job: "job"
    snapshot: "snapshot"
    history: "history"
  portsOut:
    processedJob: "job"
    snapshot: "snapshot"
    history: "history"
  metricsOutput: true
  serviceTime:
    exp:
      lambda: 0.1649
- type: "Processor"
  id: "processor-02"
  portsIn:
    job: "job"
    snapshot: "snapshot"
    history: "history"
  portsOut:
    processedJob: "job"
    snapshot: "snapshot"
    history: "history"
  metricsOutput: true
  serviceTime:
    exp:
      lambda: 0.1236
- type: "Processor"
  id: "processor-03"
  portsIn:
    job: "job"
    snapshot: "snapshot"
    history: "history"
  portsOut:
    processedJob: "job"
    snapshot: "snapshot"
    history: "history"
  metricsOutput: true
  serviceTime:
    exp:
      lambda: 0.1026
- type: "Storage"
  id: "storage-01"
  portsIn:
    store: "store"
    read: "read"
  portsOut:
    stored: "stored"
- type: "Storage"
  id: "storage-02"
  portsIn:
    store: "store"
    read: "read"
  portsOut:
    stored: "stored"
- type: "Storage"
  id: "storage-03"
  portsIn:
    store: "store"
    read: "read"
  portsOut:
    stored: "stored"
- type: "Storage"
  id: "sys-storage"
  portsIn:
    store: "store"
    read: "read"
  portsOut:
    stored: "stored"
"#;
    let connectors = r#"
- id: "connector-01"
  sourceID: "generator-01"
  targetID: "exclusive-01"
  sourcePort: "job"
  targetPort: "in"
- id: "connector-02"
  sourceID: "exclusive-01"
  targetID: "processor-01"
  sourcePort: "p01"
  targetPort: "job"
- id: "connector-03"
  sourceID: "exclusive-01"
  targetID: "processor-02"
  sourcePort: "p02"
  targetPort: "job"
- id: "connector-04"
  sourceID: "exclusive-01"
  targetID: "processor-03"
  sourcePort: "p03"
  targetPort: "job"
- id: "connector-05"
  sourceID: "processor-01"
  targetID: "storage-01"
  sourcePort: "job"
  targetPort: "store"
- id: "connector-06"
  sourceID: "processor-02"
  targetID: "storage-02"
  sourcePort: "job"
  targetPort: "store"
- id: "connector-07"
  sourceID: "processor-03"
  targetID: "storage-03"
  sourcePort: "job"
  targetPort: "store"
- id: "connector-08"
  sourceID: "processor-01"
  targetID: "sys-storage"
  sourcePort: "history"
  targetPort: "store"
- id: "connector-09"
  sourceID: "processor-02"
  targetID: "sys-storage"
  sourcePort: "history"
  targetPort: "store"
- id: "connector-10"
  sourceID: "processor-03"
  targetID: "sys-storage"
  sourcePort: "history"
  targetPort: "store"
"#;
    let mut web = WebSimulation::default();
    // Average waiting times across processors (first dimension) and replication average (second dimension)
    let mut average_waiting_times: [Vec<f64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut cis_sufficient_precision = [false, false, false];
    // Replicate as needed for a CI half width of 1.0
    let mut waiting_times: Vec<IndependentSample<f64>>;
    loop {
        // Refresh the models, but maintain the Uniform RNG for replication independence
        web.reset();
        web.put_yaml(models, connectors);
        web.step_until_json(480.0);
        waiting_times = Vec::new();
        for processor_number in ["01", "02", "03"].iter() {
            let metrics_history_request = Message::new(
                String::from("manual"),
                String::from("manual"),
                format!["processor-{}", processor_number],
                String::from("history"),
                web.get_global_time(),
                String::from(""),
            );
            web.inject_input_json(&serde_json::to_string(&metrics_history_request).unwrap());
            let messages_json = web.step_json();
            let messages: Vec<Message> = serde_json::from_str(&messages_json).unwrap();
            let metrics_message = messages
                .iter()
                .find(|message| message.source_port() == "history")
                .unwrap();
            let metrics_history: Vec<ProcessorMetrics> =
                serde_json::from_str(&metrics_message.content()).unwrap();
            let processor_waiting_times: Vec<f64> = metrics_history
                .iter()
                .map(|snapshot| &snapshot.last_service_start)
                .filter_map(|service_start_snapshot| service_start_snapshot.as_ref())
                .map(|service_start_snapshot| {
                    let associated_arrival = metrics_history
                        .iter()
                        .map(|snapshot| &snapshot.last_arrival)
                        .filter_map(|arrival_snapshot| arrival_snapshot.as_ref())
                        .find(|arrival_snapshot| arrival_snapshot.0 == service_start_snapshot.0)
                        .unwrap();
                    service_start_snapshot.1 - associated_arrival.1
                })
                .collect();
            waiting_times.push(IndependentSample::post(processor_waiting_times).unwrap());
        }

        waiting_times
            .iter()
            .enumerate()
            .for_each(|(index, waiting_times_sample)| {
                if !waiting_times_sample.point_estimate_mean().is_nan() {
                    average_waiting_times[index].push(waiting_times_sample.point_estimate_mean());
                }
                let ci = IndependentSample::post(average_waiting_times[index].clone())
                    .unwrap()
                    .confidence_interval_mean(0.05)
                    .unwrap();
                if ci.half_width() < 1.0 && ci.half_width() > 0.0 {
                    cis_sufficient_precision[index] = true;
                }
            });

        if cis_sufficient_precision
            .iter()
            .all(|sufficient| *sufficient)
        {
            break;
        }
    }
}

#[test]
fn gate_blocking_proportions() {
    // Deactivation/activation switch at a much higher frequency than job arrival, to limit autocorrelation and initialization bias
    let models = [
        Model::new(
            String::from("generator-01"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 10.0 },
                None,
                String::from("job"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("generator-02"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 10.0 },
                None,
                String::from("job"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("generator-03"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 1.0 },
                None,
                String::from("job"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("gate-01"),
            Box::new(Gate::new(
                String::from("job"),
                String::from("activation"),
                String::from("deactivation"),
                String::from("job"),
                false,
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
                false,
            )),
        ),
    ];
    let connectors = [
        Connector::new(
            String::from("connector-01"),
            String::from("generator-01"),
            String::from("gate-01"),
            String::from("job"),
            String::from("activation"),
        ),
        Connector::new(
            String::from("connector-02"),
            String::from("generator-02"),
            String::from("gate-01"),
            String::from("job"),
            String::from("deactivation"),
        ),
        Connector::new(
            String::from("connector-03"),
            String::from("generator-03"),
            String::from("gate-01"),
            String::from("job"),
            String::from("job"),
        ),
        Connector::new(
            String::from("connector-04"),
            String::from("gate-01"),
            String::from("storage-01"),
            String::from("job"),
            String::from("store"),
        ),
    ];
    let mut simulation = Simulation::default();
    let mut passed: Vec<f64> = Vec::new();
    // 10 replications and 10000 steps is more or less arbitrary here
    for _ in 0..10 {
        // Refresh the models, but maintain the Uniform RNG for replication independence
        simulation.reset();
        simulation.put(models.to_vec(), connectors.to_vec());
        let mut message_records: Vec<Message> = Vec::new();
        for _x in 0..1000 {
            let messages_set: Vec<Message> = simulation.step().unwrap();
            message_records.extend(messages_set);
        }
        let arrivals = message_records
            .iter()
            .filter(|message_record| {
                message_record.source_id() == "generator-03"
                    && message_record.target_id() == "gate-01"
            })
            .count();
        let departures = message_records
            .iter()
            .filter(|message_record| message_record.target_id() == "storage-01")
            .count();
        if arrivals > 0 {
            passed.push(departures as f64 / arrivals as f64);
        }
    }
    let passed_ci = IndependentSample::post(passed)
        .unwrap()
        .confidence_interval_mean(0.01)
        .unwrap();
    // With no "processing" delay for the gate, we can expect the blocked/unblocked proportions to be 50%
    assert![passed_ci.lower() < 0.5 && 0.5 < passed_ci.upper()];
}

#[test]
fn load_balancer_round_robin_outputs() {
    // Deactivation/activation switch at a much higher frequency than job arrival, to limit autocorrelation and initialization bias
    let models = [
        Model::new(
            String::from("generator-01"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 0.01 },
                None,
                String::from("job"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("load-balancer-01"),
            Box::new(LoadBalancer::new(
                String::from("request"),
                vec![
                    String::from("server-1"),
                    String::from("server-2"),
                    String::from("server-3"),
                ],
                false,
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
                false,
            )),
        ),
        Model::new(
            String::from("storage-02"),
            Box::new(Storage::new(
                String::from("store"),
                String::from("read"),
                String::from("stored"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("storage-03"),
            Box::new(Storage::new(
                String::from("store"),
                String::from("read"),
                String::from("stored"),
                false,
                false,
            )),
        ),
    ];
    let connectors = [
        Connector::new(
            String::from("connector-01"),
            String::from("generator-01"),
            String::from("load-balancer-01"),
            String::from("job"),
            String::from("request"),
        ),
        Connector::new(
            String::from("connector-02"),
            String::from("load-balancer-01"),
            String::from("storage-01"),
            String::from("server-1"),
            String::from("store"),
        ),
        Connector::new(
            String::from("connector-03"),
            String::from("load-balancer-01"),
            String::from("storage-02"),
            String::from("server-2"),
            String::from("store"),
        ),
        Connector::new(
            String::from("connector-04"),
            String::from("load-balancer-01"),
            String::from("storage-03"),
            String::from("server-3"),
            String::from("store"),
        ),
    ];
    let mut simulation = Simulation::post(models.to_vec(), connectors.to_vec());
    // 28 steps means 9 processed jobs
    // 3 steps per processed job
    // 1 step for initialization
    let message_records: Vec<Message> = simulation.step_n(28).unwrap();
    let outputs = vec![
        message_records
            .iter()
            .filter(|message_record| message_record.target_id() == "storage-01")
            .count(),
        message_records
            .iter()
            .filter(|message_record| message_record.target_id() == "storage-02")
            .count(),
        message_records
            .iter()
            .filter(|message_record| message_record.target_id() == "storage-03")
            .count(),
    ];
    outputs.iter().for_each(|server_arrival_count| {
        assert![*server_arrival_count == 3];
    });
}

#[test]
fn injection_initiated_stored_value_exchange() {
    let models = [
        Model::new(
            String::from("storage-01"),
            Box::new(Storage::new(
                String::from("store"),
                String::from("read"),
                String::from("stored"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("storage-02"),
            Box::new(Storage::new(
                String::from("store"),
                String::from("read"),
                String::from("stored"),
                false,
                false,
            )),
        ),
    ];
    let connectors = [
        Connector::new(
            String::from("connector-01"),
            String::from("storage-02"),
            String::from("storage-01"),
            String::from("stored"),
            String::from("store"),
        ),
        Connector::new(
            String::from("connector-02"),
            String::from("storage-01"),
            String::from("storage-02"),
            String::from("stored"),
            String::from("store"),
        ),
    ];
    let mut simulation = Simulation::post(models.to_vec(), connectors.to_vec());
    let stored_value = Message::new(
        String::from("manual"),
        String::from("manual"),
        String::from("storage-01"),
        String::from("store"),
        simulation.get_global_time(),
        String::from("42"),
    );
    simulation.inject_input(stored_value);
    simulation.step().unwrap();
    let transfer_request = Message::new(
        String::from("manual"),
        String::from("manual"),
        String::from("storage-01"),
        String::from("read"),
        simulation.get_global_time(),
        String::from(""),
    );
    simulation.inject_input(transfer_request);
    simulation.step().unwrap();
    let read_request = Message::new(
        String::from("manual"),
        String::from("manual"),
        String::from("storage-02"),
        String::from("read"),
        simulation.get_global_time(),
        String::from(""),
    );
    simulation.inject_input(read_request);
    let messages: Vec<Message> = simulation.step().unwrap();
    assert![messages[0].content() == "42"];
}

#[test]
fn parallel_gateway_splits_and_joins() {
    let models = [
        Model::new(
            String::from("generator-01"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 5.0 },
                None,
                String::from("job"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("parallel-01"),
            Box::new(ParallelGateway::new(
                vec![String::from("in")],
                vec![
                    String::from("alpha"),
                    String::from("beta"),
                    String::from("delta"),
                ],
                false,
                false,
            )),
        ),
        Model::new(
            String::from("parallel-02"),
            Box::new(ParallelGateway::new(
                vec![
                    String::from("alpha"),
                    String::from("beta"),
                    String::from("delta"),
                ],
                vec![String::from("out")],
                false,
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
                false,
            )),
        ),
    ];
    let connectors = [
        Connector::new(
            String::from("connector-01"),
            String::from("generator-01"),
            String::from("parallel-01"),
            String::from("job"),
            String::from("in"),
        ),
        Connector::new(
            String::from("connector-02"),
            String::from("parallel-01"),
            String::from("parallel-02"),
            String::from("alpha"),
            String::from("alpha"),
        ),
        Connector::new(
            String::from("connector-03"),
            String::from("parallel-01"),
            String::from("parallel-02"),
            String::from("beta"),
            String::from("beta"),
        ),
        Connector::new(
            String::from("connector-04"),
            String::from("parallel-01"),
            String::from("parallel-02"),
            String::from("delta"),
            String::from("delta"),
        ),
        Connector::new(
            String::from("connector-05"),
            String::from("parallel-02"),
            String::from("storage-01"),
            String::from("out"),
            String::from("store"),
        ),
    ];
    let mut simulation = Simulation::post(models.to_vec(), connectors.to_vec());
    let message_records: Vec<Message> = simulation.step_n(101).unwrap();
    let alpha_passes = message_records
        .iter()
        .filter(|message_record| message_record.target_port() == "alpha")
        .count();
    let beta_passes = message_records
        .iter()
        .filter(|message_record| message_record.target_port() == "beta")
        .count();
    let delta_passes = message_records
        .iter()
        .filter(|message_record| message_record.target_port() == "delta")
        .count();
    let storage_passes = message_records
        .iter()
        .filter(|message_record| message_record.target_port() == "store")
        .count();
    assert![alpha_passes == beta_passes];
    assert![beta_passes == delta_passes];
    assert![delta_passes == storage_passes];
    assert![storage_passes > 0];
}

#[test]
fn match_status_reporting() {
    let models = [
        Model::new(
            String::from("generator-01"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 5.0 },
                None,
                String::from("job"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("load-balancer-01"),
            Box::new(LoadBalancer::new(
                String::from("request"),
                vec![
                    String::from("alpha"),
                    String::from("beta"),
                    String::from("delta"),
                ],
                false,
                false,
            )),
        ),
    ];
    let connectors = [];
    let simulation = Simulation::post(models.to_vec(), connectors.to_vec());
    assert![simulation.status("generator-01").unwrap() == "Generating jobs"];
    assert![simulation.status("load-balancer-01").unwrap() == "Listening for requests"];
}

#[test]
fn stochastic_gate_blocking() {
    let models = [
        Model::new(
            String::from("generator-01"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 5.0 },
                None,
                String::from("job"),
                false,
                false,
            )),
        ),
        Model::new(
            String::from("stochastic-gate-01"),
            Box::new(StochasticGate::new(
                BooleanRandomVariable::Bernoulli { p: 0.2 },
                String::from("job"),
                String::from("job"),
                false,
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
                false,
            )),
        ),
    ];
    let connectors = [
        Connector::new(
            String::from("connector-01"),
            String::from("generator-01"),
            String::from("stochastic-gate-01"),
            String::from("job"),
            String::from("job"),
        ),
        Connector::new(
            String::from("connector-02"),
            String::from("stochastic-gate-01"),
            String::from("storage-01"),
            String::from("job"),
            String::from("store"),
        ),
    ];
    let mut simulation = Simulation::post(models.to_vec(), connectors.to_vec());
    let message_records: Vec<Message> = simulation.step_n(101).unwrap();
    let mut results: Vec<f64> = Vec::new();
    message_records
        .iter()
        .filter(|message_record| message_record.target_id() == "storage-01")
        .for_each(|_pass| results.push(1.0));
    let passes = results.len();
    message_records
        .iter()
        .enumerate()
        .filter(|(index, message_record)| {
            message_record.target_id() == "stochastic-gate-01" && *index > passes
        })
        .for_each(|_fail| results.push(0.0));
    let sample = IndependentSample::post(results).unwrap();
    assert![sample.confidence_interval_mean(0.01).unwrap().lower() < 0.2];
    assert![sample.confidence_interval_mean(0.01).unwrap().upper() > 0.2];
}
