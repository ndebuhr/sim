use std::f64::INFINITY;

use sim::models::processor::Job as ProcessorJob;
use sim::models::*;
use sim::output_analysis::*;
use sim::simulator::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[test]
#[wasm_bindgen_test]
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
            "phase": "Passive",
            "untilNextEvent": 0.0,
            "untilJobCompletion": 0.0,
            "queue": [
                {"content": "job 1", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 2", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 3", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 4", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 5", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 6", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 7", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 8", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 9", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 10", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 11", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 12", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 13", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 14", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 15", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 16", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 17", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 18", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 19", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 20", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 21", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 22", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 23", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 24", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 25", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 26", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 27", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 28", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 29", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 30", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 31", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 32", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 33", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 34", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 35", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 36", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 37", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 38", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 39", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 40", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 41", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 42", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 43", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 44", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 45", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 46", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 47", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 48", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 49", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 50", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 51", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 52", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 53", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 54", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 55", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 56", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 57", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 58", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 59", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 60", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 61", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 62", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 63", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 64", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 65", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 66", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 67", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 68", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 69", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 70", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 71", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 72", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 73", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 74", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 75", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 76", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 77", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 78", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 79", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 80", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 81", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 82", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 83", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 84", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 85", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 86", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 87", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 88", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 89", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 90", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 91", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 92", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 93", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 94", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 95", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 96", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 97", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 98", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 99", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 100", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0}
            ],
            "records": []
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
                Some(format!["job {:?}", (simulation_step + 1) / 2])
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
                    assert_eq![first_message.content(), output];
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
                                 // Epsilon of 0.34
    assert!((average_batch_completion_time - 2.0 * expectation).abs() / (2.0 * expectation) < 0.34);
}

#[test]
#[wasm_bindgen_test]
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
            "phase": "Passive",
            "untilNextEvent": 0.0,
            "untilJobCompletion": 0.0,
            "queue": [
                {"content": "job 0", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 1", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 2", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 3", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 4", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 5", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 6", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 7", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 8", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0},
                {"content": "job 9", "arrivalTime": 0.0, "processingStartTime": 0.0, "departureTime": 0.0}
            ],
            "records": []
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
    assert_eq!(storage_arrivals_count, expected);
}

#[test]
#[wasm_bindgen_test]
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
#[wasm_bindgen_test]
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
#[wasm_bindgen_test]
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
  portsOut:
    processedJob: "job"
  storeRecords: true
  serviceTime:
    exp:
      lambda: 0.1649
- type: "Processor"
  id: "processor-02"
  portsIn:
    job: "job"
  portsOut:
    processedJob: "job"
  storeRecords: true
  serviceTime:
    exp:
      lambda: 0.1236
- type: "Processor"
  id: "processor-03"
  portsIn:
    job: "job"
  portsOut:
    processedJob: "job"
  storeRecords: true
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
  sourcePort: "records"
  targetPort: "store"
- id: "connector-09"
  sourceID: "processor-02"
  targetID: "sys-storage"
  sourcePort: "records"
  targetPort: "store"
- id: "connector-10"
  sourceID: "processor-03"
  targetID: "sys-storage"
  sourcePort: "records"
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
        // Loop through (processor_number, average_processing_time)
        for processor in ["processor-01", "processor-02", "processor-03"].iter() {
            let records_request = Message::new(
                String::from("manual"),
                String::from("manual"),
                processor.to_string(),
                String::from("records"),
                web.get_global_time(),
                String::from(""),
            );
            web.inject_input_json(&serde_json::to_string(&records_request).unwrap());
            let messages_json = web.step_json();
            let messages: Vec<Message> = serde_json::from_str(&messages_json).unwrap();
            let metrics_message = messages
                .iter()
                .find(|message| message.source_port() == "records")
                .unwrap();
            let records: Vec<ProcessorJob> =
                serde_json::from_str(&metrics_message.content()).unwrap();
            let processor_waiting_times: Vec<f64> = records
                .iter()
                .filter(|job| job.arrival_time < INFINITY)
                .map(|job| &job.departure_time - &job.processing_start_time)
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
