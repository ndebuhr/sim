use sim::input_modeling::random_variable::*;
use sim::models::*;
use sim::output_analysis::*;
use sim::simulator::*;

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
    assert_eq![outputs.iter().sum::<usize>(), 200];
    // 3 bins, 2 dof, 0.01 alpha
    let chi_square_critical = 9.21;
    assert![chi_square < chi_square_critical];
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
        assert_eq![*server_arrival_count, 3];
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
    assert_eq![messages[0].content(), "42"];
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
    assert_eq![alpha_passes, beta_passes];
    assert_eq![beta_passes, delta_passes];
    assert_eq![delta_passes, storage_passes];
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
    assert_eq![
        simulation.status("generator-01").unwrap(),
        "Generating jobs"
    ];
    assert_eq![
        simulation.status("load-balancer-01").unwrap(),
        "Listening for requests"
    ];
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
