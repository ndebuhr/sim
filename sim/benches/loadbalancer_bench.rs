#![feature(test)]

extern crate test;

#[cfg(test)]
mod test_loadbalancer {
    use std::collections::HashMap;
    use test::Bencher;
    use sim::models::{DevsModel, LoadBalancer, Model, ModelMessage};
    use sim::simulator::Services;
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use log::info;

    fn job_message(content: String) -> ModelMessage {
        ModelMessage {
            port_name: "job".to_string(),
            content: content,
        }
    }

    fn get_loadbalancer() -> (Model,HashSet<String>) {
        let fpports = vec!["A".to_string(),
                           "B".to_string(),
                           "C".to_string()];
        let fpports_set = HashSet::from_iter(fpports.clone().into_iter());
        (Model::new(
            "bench_storage".to_string(),
            Box::new(LoadBalancer::new(
                "job".to_string(),
                fpports,
                false,
            ))
        ), fpports_set)
        
    }

    #[test]
    ///verify that the storage model stores and retrieves a stored value.
    fn loadbalancer_test() {
        let (mut model, mut expected_port_set) = get_loadbalancer();
        
        let expected_message = "value001".to_string();
        let job_message = job_message(expected_message.clone());
        let mut services = Services::default();

        for i in 0..expected_port_set.len(){
            info!("Checking for each port in round robin {}", i);

            let ext_result = &model.events_ext(&job_message, &mut services);
            assert!(ext_result.is_ok());
            //expect an internal message routed to "A" port
            let int_results = model.events_int(&mut services);
            assert!(int_results.is_ok());
            match int_results {
                Ok(vl) => {
                    assert_eq!(vl.len(), 1);
                    assert_eq!(vl[0].content, expected_message.clone());
                    assert!(expected_port_set.contains(&vl[0].port_name), "The expected port `{}` was not found.", vl[0].port_name);
                    expected_port_set.remove(&vl[0].port_name);
                },
                Err(_) => assert!(int_results.is_err())
            }
        }
        assert!(expected_port_set.is_empty(),"Not all the ports expected have been used. in round robin.");
    }

    #[bench]
    fn loadbalancer_bench(b: &mut Bencher) {
        let (mut model, mut expected_port_set) = get_loadbalancer();

        let expected_message = "value001".to_string();
        let job_message = job_message(expected_message.clone());
        let mut services = Services::default();

        b.iter(|| {
            for i in 0..expected_port_set.len() {
                let ext_result = &model.events_ext(&job_message, &mut services);
                let int_result = model.events_int(&mut services);
            }
        });
    }
    // test test_loadbalancer::loadbalancer_bench ... bench:         549.13 ns/iter (+/- 6.46)
}