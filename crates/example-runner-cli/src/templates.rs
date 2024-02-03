use std::collections::HashMap;

use serde::Deserialize;

use crate::example_config::ExampleConfig;

#[derive(Debug, Deserialize)]
pub struct ExampleRunnerConfig {
    pub templates: Option<HashMap<String, ExampleConfig>>,
}
