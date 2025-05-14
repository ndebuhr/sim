#![feature(test)]

extern crate test;

#[cfg(test)]
mod test_models {
    use sim::models::{DevsModel, Model, ModelMessage, Storage};
    use sim::simulator::Services;
    use test::Bencher;
    
    fn put_message(content: String) -> ModelMessage{
        ModelMessage {
            port_name: "put".to_string(),
            content: content,
        }
    }
    fn get_message() -> ModelMessage {
        ModelMessage {
            port_name: "get".to_string(),
            content: "NA01".to_string(),
        }
    }

    fn get_storage() -> Model {
        Model::new(
            "bench_storage".to_string(),
            Box::new(Storage::new(
                "put".to_string(),
                "get".to_string(),
                "stored".to_string(),
                false,
            )),
        )
    }

    #[test]
    ///verify that the storage model stores and retrieves a stored value.
    fn storage_test() {
        let mut model = get_storage();
        let expected_message = "value001".to_string();
        let put_message = put_message(expected_message.clone());
        let get_message = get_message();
        let mut services = Services::default();
        let ext_result = model.events_ext(&put_message, &mut services);
        assert!(ext_result.is_ok());

        let int_results = model.events_int(& mut services);
        assert!(int_results.is_ok());

        let ext_result = model.events_ext(&get_message, &mut services);
        assert!(ext_result.is_ok());

        let int_results = model.events_int(& mut services);
        match int_results {
            Ok(vl) => {
                assert_eq!(vl.len(), 1);
                assert_eq!(vl[0].content, expected_message.clone());
            },
            Err(_) => assert!(int_results.is_err())
        }
        assert_eq!(services.global_time(), 0f64)
        
    }


    #[bench]
    fn storage_bench(b: &mut Bencher) {
        let mut model = get_storage();
        
        let put_message = put_message("value001".to_string());
        let get_message = get_message(); 

        b.iter(|| {
            let mut services = Services::default();
            let ext_result = model.events_ext(&put_message, &mut services);
            let int_result = model.events_int(&mut services);
            let ext_result = model.events_ext(&get_message, &mut services);
            let int_result = model.events_int(&mut services);
            
        });
    }
// test test_models::storage_bench ... bench:         165.55 ns/iter (+/- 2.20)
// test test_models::storage_bench ... bench:         165.97 ns/iter (+/- 2.58)    
// test test_models::storage_bench ... bench:         166.18 ns/iter (+/- 3.14)    
}
