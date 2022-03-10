use std::collections::HashMap;
use std::f64::INFINITY;

use sim::models::{Model, ModelRecord};
use sim::output_analysis::IndependentSample;
use sim::simulator::{Connector, Message, WebSimulation};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

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
            "job": "processed job"
        },
        "serviceTime": {
            "exp": {
                "lambda": 3.0
            }
        },
        "state": {
            "phase": "Passive",
            "untilNextEvent": 0.0,
            "queue": [
                "job 1", "job 2", "job 3", "job 4", "job 5", "job 6", "job 7", "job 8", "job 9", "job 10",
                "job 11", "job 12", "job 13", "job 14", "job 15", "job 16", "job 17", "job 18", "job 19", "job 20",
                "job 21", "job 22", "job 23", "job 24", "job 25", "job 26", "job 27", "job 28", "job 29", "job 30",
                "job 31", "job 32", "job 33", "job 34", "job 35", "job 36", "job 37", "job 38", "job 39", "job 40",
                "job 41", "job 42", "job 43", "job 44", "job 45", "job 46", "job 47", "job 48", "job 49", "job 50",
                "job 51", "job 52", "job 53", "job 54", "job 55", "job 56", "job 57", "job 58", "job 59", "job 60",
                "job 61", "job 62", "job 63", "job 64", "job 65", "job 66", "job 67", "job 68", "job 69", "job 70",
                "job 71", "job 72", "job 73", "job 74", "job 75", "job 76", "job 77", "job 78", "job 79", "job 80",
                "job 81", "job 82", "job 83", "job 84", "job 85", "job 86", "job 87", "job 88", "job 89", "job 90",
                "job 91", "job 92", "job 93", "job 94", "job 95", "job 96", "job 97", "job 98", "job 99", "job 100"
            ],
            "records": []
        }
    },
    {
        "type": "Storage",
        "id": "storage-01",
        "portsIn": {
            "put": "store",
            "get": "read"
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
            "job": "processed job"
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
                "job 0", "job 1", "job 2", "job 3", "job 4", "job 5", "job 6", "job 7", "job 8", "job 9"
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
            "job": "processed job"
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
            "job": "processed job"
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
            "job": "processed job"
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
            "job": "processed job"
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
            "job": "processed job"
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
            "put": "store",
            "get": "read"
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
    job: "processed job"
  serviceTime:
    exp:
      lambda: 0.333333
  queueCapacity: 14
  id: "processor-01"
- portsIn:
    put: "store"
    get: "read"
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
    job: "processed job"
  serviceTime:
    exp:
      lambda: 0.333333
  queueCapacity: 14
- type: "Storage"
  id: "storage-01"
  portsIn:
    put: "store"
    get: "read"
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
    job: "job"
  storeRecords: true
  serviceTime:
    exp:
      lambda: 0.1649
- type: "Processor"
  id: "processor-02"
  portsIn:
    job: "job"
  portsOut:
    job: "job"
  storeRecords: true
  serviceTime:
    exp:
      lambda: 0.1236
- type: "Processor"
  id: "processor-03"
  portsIn:
    job: "job"
  portsOut:
    job: "job"
  storeRecords: true
  serviceTime:
    exp:
      lambda: 0.1026
- type: "Storage"
  id: "storage-01"
  portsIn:
    put: "store"
    get: "read"
  portsOut:
    stored: "stored"
- type: "Storage"
  id: "storage-02"
  portsIn:
    put: "store"
    get: "read"
  portsOut:
    stored: "stored"
- type: "Storage"
  id: "storage-03"
  portsIn:
    put: "store"
    get: "read"
  portsOut:
    stored: "stored"
- type: "Storage"
  id: "sys-storage"
  portsIn:
    put: "store"
    get: "read"
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
    loop {
        // Refresh the models, but maintain the Uniform RNG for replication independence
        web.reset();
        web.put_yaml(models, connectors);
        web.step_until_json(480.0);
        // Loop through (processor_number, average_processing_time)
        ["processor-01", "processor-02", "processor-03"]
            .iter()
            .enumerate()
            .for_each(|(processor_index, processor)| {
                let records: Vec<ModelRecord> =
                    serde_json::from_str(&web.get_records_json(processor)).unwrap();
                // Times: Arrival, Processing Start, Departure
                let mut job_timestamps: HashMap<String, (Option<f64>, Option<f64>, Option<f64>)> =
                    HashMap::new();
                records.iter().for_each(|record| {
                    let job = job_timestamps
                        .entry(record.subject.clone())
                        .or_insert((None, None, None));
                    if record.action == "Arrival" {
                        job.0 = Some(record.time);
                    } else if record.action == "Processing Start" {
                        job.1 = Some(record.time);
                    } else if record.action == "Departure" {
                        job.2 = Some(record.time);
                    }
                });
                let waiting_times = job_timestamps
                    .iter()
                    .filter_map(|job_timestamps| match &job_timestamps.1 {
                        (Some(arrival), Some(processing_start), _) => {
                            Some(processing_start - arrival)
                        }
                        (_, _, _) => None,
                    })
                    .collect();
                let waiting_times_sample = IndependentSample::post(waiting_times).unwrap();
                if !waiting_times_sample.point_estimate_mean().is_nan() {
                    average_waiting_times[processor_index]
                        .push(waiting_times_sample.point_estimate_mean());
                }
                let ci = IndependentSample::post(average_waiting_times[processor_index].clone())
                    .unwrap()
                    .confidence_interval_mean(0.05)
                    .unwrap();
                if ci.half_width() < 1.0 && ci.half_width() > 0.0 {
                    cis_sufficient_precision[processor_index] = true;
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
