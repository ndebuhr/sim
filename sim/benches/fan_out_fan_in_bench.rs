// Fan a single message out to N processors and then join them all back.
// benchmark execution time (not simulation time)

#![feature(test)]

extern crate test;

#[cfg(test)]
mod test_parallel_gateway {
    use sim::models::{DevsModel, Model, ModelMessage, ParallelGateway};
    use sim::simulator::{Connector, Message, Services, Simulation};
    use sim::utils::errors::SimulationError;
    use test::Bencher;

    /// the message that will be sent.
    fn initial_message(content: String) -> Message {
        Message::new(
            "manual".to_string(),
            "manual".to_string(),
            "PG_FAN_OUT".to_string(),
            "IN".to_string(),
            0.0,
            content,
        )
    }

    fn get_out_port_names(port_count: usize) -> Vec<String> {
        (0..port_count).map(|s| format!("OUT_{}", s)).collect()
    }
    fn get_in_port_names(port_count: usize) -> Vec<String> {
        (0..port_count).map(|s| format!("IN_{}", s)).collect()
    }

    fn get_parallel_gateway_fan_out(port_count: usize) -> Model {
        Model::new(
            "PG_FAN_OUT".to_string(),
            Box::new(ParallelGateway::new(
                vec!["IN".to_string()],
                get_out_port_names(port_count),
                false,
            )),
        )
    }

    fn get_parallel_gateway_fan_in(port_count: usize) -> Model {
        Model::new(
            "PG_FAN_IN".to_string(),
            Box::new(ParallelGateway::new(
                get_in_port_names(port_count),
                vec!["OUT".to_string()],
                false,
            )),
        )
    }
    fn get_models(port_count: usize) -> Vec<Model> {
        vec![
            get_parallel_gateway_fan_out(port_count),
            get_parallel_gateway_fan_in(port_count),
        ]
    }

    fn get_connectors(port_count: usize) -> Vec<Connector> {
        //Connect OUT_0 to IN_0
        (0..port_count)
            .map(|pi| {
                Connector::new(
                    format!("connector_{}", pi),
                    "PG_FAN_OUT".to_string(),
                    "PG_FAN_IN".to_string(),
                    format!("OUT_{}", pi),
                    format!("IN_{}", pi),
                )
            })
            .collect()
    }

    fn get_simulator(port_count: usize) -> Simulation {
        Simulation::post(get_models(port_count), get_connectors(port_count))
    }

    #[test]
    fn fi_fo_test() {
        let fan_size = 10usize;
        let mut sim = get_simulator(fan_size);
        sim.inject_input(initial_message("TESTING".to_string()));
        let result = sim.step();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), fan_size);
    }

    #[bench]
    fn fi_fo_bench_100(b: &mut Bencher) {
        let fan_size = 100usize;
        let mut sim = get_simulator(fan_size);
        sim.inject_input(initial_message("TESTING".to_string()));
        b.iter(|| sim.step());
    }
    #[bench]
    fn fi_fo_bench_1000(b: &mut Bencher) {
        let fan_size = 10000usize;
        let mut sim = get_simulator(fan_size);
        sim.inject_input(initial_message("TESTING".to_string()));
        b.iter(|| sim.step());
    }
    #[bench]
    fn fi_fo_bench_50000(b: &mut Bencher) {
        let fan_size = 50000usize;
        let mut sim = get_simulator(fan_size);
        sim.inject_input(initial_message("TESTING".to_string()));
        b.iter(|| sim.step());
    }

    // #[bench]
    // fn fi_fo_bench_80000(b: &mut Bencher) {
    //     let fan_size = 80000usize;
    //     let mut sim = get_simulator(fan_size);
    //     sim.inject_input(initial_message("TESTING".to_string()));
    //     b.iter(|| sim.step());
    // }
    
    // test test_parallel_gateway::fi_fo_bench_100    ... bench:          47.90 ns/iter (+/- 0.21)
    // test test_parallel_gateway::fi_fo_bench_1000   ... bench:          55.22 ns/iter (+/- 0.35)

    // test test_parallel_gateway::fi_fo_bench_100   ... bench:          47.90 ns/iter (+/- 0.56)
    // test test_parallel_gateway::fi_fo_bench_1000  ... bench:          55.53 ns/iter (+/- 4.41)
    // test test_parallel_gateway::fi_fo_bench_50000 ... bench:          59.44 ns/iter (+/- 1.39)

    // test test_parallel_gateway::fi_fo_bench_100   ... bench:          48.24 ns/iter (+/- 2.12)
    // test test_parallel_gateway::fi_fo_bench_1000  ... bench:          50.00 ns/iter (+/- 8.31)
    // test test_parallel_gateway::fi_fo_bench_50000 ... bench:          59.83 ns/iter (+/- 5.48)
    // test test_parallel_gateway::fi_fo_bench_80000 ... bench:          58.30 ns/iter (+/- 8.40)

    #[test]
    fn fo_test() {
        let fan_size = 10000usize;
        let mut model =
            ParallelGateway::new(vec!["IN".to_string()], get_out_port_names(fan_size), false);
        let in_message = ModelMessage {
            port_name: "IN".to_string(),
            content: "testing".to_string(),
        };

        let mut services = Services::default();

        let ext_result = &model.events_ext(&in_message, &mut services);
        assert!(ext_result.is_ok());
        let int_results = model.events_int(&mut services);
        assert!(int_results.is_ok());
        assert_eq!(int_results.unwrap().len(), fan_size);

        //instantaneous operation in simulation
        assert_eq!(services.global_time(), 0.0f64);
    }
    #[test]
    fn fi_test() {
        let fan_size = 10000usize;
        let mut model =
            ParallelGateway::new(get_in_port_names(fan_size), vec!["OUT".to_string()], false);
        let in_messages: Vec<ModelMessage> = (0..fan_size)
            .map(|i| ModelMessage {
                port_name: format!("IN_{}", i),
                content: "testing".to_string(),
            })
            .collect();

        let mut services = Services::default();

        let ext_results: Vec<Result<(), SimulationError>> = in_messages
            .iter()
            .map(|i| model.events_ext(i, &mut services))
            .collect();

        assert!(ext_results.iter().all(|r| r.is_ok()));
        //instantaneous simulation time.
        assert_eq!(services.global_time(), 0.0f64);

        //There's only one resulting message.  This is a fan in.
        let int_results = model.events_int(&mut services);
        assert!(int_results.is_ok());
        let mm = &int_results.unwrap().pop().unwrap();
        assert_eq!(mm.port_name, "OUT".to_string());

        //nothing left after the previous events_int call.
        let int_results = model.events_int(&mut services);
        assert!(int_results.is_ok());
        assert!(int_results.unwrap().is_empty());
    }

    
    // TODO do fo_bench
    // TODO smaller fan size for benchmark tests.
    #[bench]
    fn fi_bench(b: &mut Bencher) {
        let fan_size = 10000usize;
        let mut model =
            ParallelGateway::new(get_in_port_names(fan_size), vec!["OUT".to_string()], false);
        let in_messages: Vec<ModelMessage> = (0..fan_size)
            .map(|i| ModelMessage {
                port_name: format!("IN_{}", i),
                content: "testing".to_string(),
            })
            .collect();

        let mut services = Services::default();

        //Todo m
        b.iter(|| {
            let _: Vec<Result<(), SimulationError>> = in_messages
                .iter()
                .map(|i| model.events_ext(i, &mut services))
                .collect();
            let _ = model.events_int(&mut services);
        });
    }
}
