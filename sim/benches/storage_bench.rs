#![feature(test)]

extern crate test;

#[cfg(test)]
mod test_models {
    use sim::models::{DevsModel, Model, ModelMessage, Storage};
    use sim::simulator::Services;
    use test::Bencher;

    #[bench]
    fn storage_bench(b: &mut Bencher) {
        let mut model = Model::new(
            "bench_storage".to_string(),
            Box::new(Storage::new(
                "put".to_string(),
                "get".to_string(),
                "stored".to_string(),
                false,
            )),
        );
        let message = ModelMessage {
            port_name: "store".to_string(),
            content: "value001".to_string(),
        };

        b.iter(|| {
            let mut services = Services::default();
            let ext_result = model.events_int(&mut services);
            let int_result = model.events_ext(&message, &mut services);
        });
    }

    #[test]
    fn storage_test() {
        let mut model = Model::new(
            "bench_storage".to_string(),
            Box::new(Storage::new(
                "put".to_string(),
                "get".to_string(),
                "stored".to_string(),
                false,
            )),
        );
        let message = ModelMessage {
            port_name: "put".to_string(),
            content: "value001".to_string(),
        };

        let mut services = Services::default();
        let ext_result = model.events_ext(&message, &mut services);
        assert!(ext_result.is_ok());

        let int_results = model.events_int(& mut services);
        assert!(int_results.is_ok());
        
        let message = ModelMessage {
            port_name: "get".to_string(),
            content: "ignored".to_string(),
        };
        let ext_result = model.events_ext(&message, &mut services);
        assert!(ext_result.is_ok());

        let int_results = model.events_int(& mut services);
        assert!(int_results.is_ok());
        
    }
}

// warning: `sim` (bench "storage_bench") generated 2 warnings
//     Finished `bench` profile [optimized] target(s) in 0.04s
//      Running benches/storage_bench.rs (target/release/deps/storage_bench-016615949a18d51a)
// 26.23022772606383 ns/iter (+/- 0.5808988530585104)
