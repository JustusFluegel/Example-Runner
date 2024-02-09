use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::example_config::{ExampleConfig, RunnerOptions};

#[derive(Debug, Deserialize, Default)]
pub struct WorkspaceExampleRunnerConfig {
    #[serde(default)]
    pub templates: HashMap<String, ExampleConfig>,
    #[serde(default)]
    pub default: HashSet<RunnerOptions>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PackageExampleRunnerConfig {
    #[serde(default)]
    pub templates: HashMap<String, ExampleConfig>,
    #[serde(default)]
    pub extend_workspace_defaults: bool,
    #[serde(default)]
    pub default: HashSet<RunnerOptions>,
}
