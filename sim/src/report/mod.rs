use crate::simulator::{Simulation};

pub trait Report {
    fn generate_dot_graph(&self) -> String;
}

impl Report for Simulation {
    fn generate_dot_graph(&self) -> String {
        let models = self.get_models();
        let connectors = self.get_connectors();

        let mut dot_string = String::from("digraph DAG {\n");

        // Add nodes
        for model in models {
            dot_string.push_str(&format!("  \"{}\" [shape=box];\n", model.id()));
        }

        // Add edges
        for connector in connectors {
            dot_string.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                connector.source_id(),
                connector.target_id(),
                connector.id()
            ));
        }

        dot_string.push_str("}\n");
        dot_string
    }
}
