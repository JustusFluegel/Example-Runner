use std::collections::HashMap;

use serde::Deserialize;

use crate::example_config::{ExampleConfig, RunnerOptions};

#[derive(Debug, Deserialize)]
pub struct ExampleRunnerConfig {
    pub templates: Option<HashMap<String, ExampleConfig>>,
    pub default: Option<RunnerOptions>,
}
