#![feature(test)]

extern crate test;

#[cfg(test)]
mod testy {
    use sim::input_modeling::ContinuousRandomVariable;
    use sim::models::{Model, Processor};
    use sim::simulator::{Connector, Message, Simulation};
    use test::Bencher;

    #[bench]
    fn ping_pong_bench(b: &mut Bencher) {
        let bench_iterations = 5;

        //run the ping pong a bunch of times and collect runtime metrics.
        let (initial_messages, mut simulation) = ping_pong_sim();

        initial_messages
            .iter()
            .for_each(|m| simulation.inject_input(m.clone()));

        b.iter(|| simulation.step_n(bench_iterations).unwrap());
    }

    fn ping_pong_sim() -> ([Message; 1], Simulation) {
        let models = [
            Model::new(
                String::from("player-01"),
                Box::new(Processor::new(
                    ContinuousRandomVariable::Exp { lambda: 0.9 },
                    None,
                    String::from("receive"),
                    String::from("send"),
                    false,
                    None,
                )),
            ),
            Model::new(
                String::from("player-02"),
                Box::new(Processor::new(
                    ContinuousRandomVariable::Exp { lambda: 0.9 },
                    None,
                    String::from("receive"),
                    String::from("send"),
                    false,
                    None,
                )),
            ),
        ];

        let connectors = [
            Connector::new(
                String::from("p1 to p2"),
                String::from("player-01"),
                String::from("player-02"),
                String::from("send"),
                String::from("receive"),
            ),
            Connector::new(
                String::from("p2 to p1"),
                String::from("player-02"),
                String::from("player-01"),
                String::from("send"),
                String::from("receive"),
            ),
        ];

        let initial_messages = [Message::new(
            "manual".to_string(),
            "manual".to_string(),
            "player-01".to_string(),
            "receive".to_string(),
            0.0,
            "Ball".to_string(),
        )];

        let simulation = Simulation::post(models.to_vec(), connectors.to_vec());
        (initial_messages, simulation)
    }
    // test testy::ping_pong_bench ... bench:       2,321.04 ns/iter (+/- 22.68)
}
