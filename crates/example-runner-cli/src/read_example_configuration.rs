use std::collections::{HashMap, HashSet};

use cargo_metadata::{Metadata, PackageId, Target};
use miette::Diagnostic;
use serde::{Deserialize, Deserializer};

use crate::{
    example_config::{
        ExampleConfigFinalized, RunnerOptionVariants, RunnerOptions, RunnerType,
        TemplateResolveError,
    },
    templates::{PackageExampleRunnerConfig, WorkspaceExampleRunnerConfig},
};

pub struct ExamplesConfiguration<'a> {
    pub examples: HashMap<(PackageId, &'a Target), HashSet<ExampleConfigFinalized>>,
    pub unconfigured: HashSet<(PackageId, &'a Target)>,
    pub unknown: HashSet<(PackageId, String)>,
}

#[derive(Deserialize, Debug, Default)]
struct WorkspaceMetadata {
    #[serde(default)]
    example_runner: WorkspaceExampleRunnerConfig,
}

#[derive(Deserialize, Debug, Default)]
struct PackageMetadata {
    #[serde(rename = "example_runner", default)]
    package: PackageConfig,
}

fn configurations_deserialize<'de, D>(deserializer: D) -> Result<HashSet<RunnerOptions>, D::Error>
where
    D: Deserializer<'de>,
{
    let map_1 = HashSet::<RunnerOptionVariants>::deserialize(deserializer)?;

    Ok(map_1.into_iter().map(Into::into).collect())
}

fn examples_deserialize<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, ExtendedExampleConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    let map_1 = HashMap::<String, ExampleConfigVariants>::deserialize(deserializer)?;

    Ok(map_1.into_iter().map(|(k, v)| (k, v.into())).collect())
}
#[derive(Deserialize, Debug, Default)]
struct PackageConfig {
    #[serde(flatten, default)]
    example_runner: PackageExampleRunnerConfig,
    #[serde(deserialize_with = "examples_deserialize")]
    examples: HashMap<String, ExtendedExampleConfig>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ExampleConfigVariants {
    Extended(ExtendedExampleConfig),
    #[serde(deserialize_with = "configurations_deserialize")]
    Simple(HashSet<RunnerOptions>),
}

impl From<ExampleConfigVariants> for ExtendedExampleConfig {
    fn from(value: ExampleConfigVariants) -> Self {
        match value {
            ExampleConfigVariants::Extended(e) => e,
            ExampleConfigVariants::Simple(s) => ExtendedExampleConfig {
                extend_configurations: false,
                configurations: s,
            },
        }
    }
}

#[derive(Deserialize, Debug)]
struct ExtendedExampleConfig {
    extend_configurations: bool,
    #[serde(deserialize_with = "configurations_deserialize")]
    configurations: HashSet<RunnerOptions>,
}

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum ExamplesConfigurationError {
    #[error(transparent)]
    #[diagnostic(
        code(examples_config::config_parse),
        help("Check your configuration files for syntax errors")
    )]
    ConfigParse(#[from] serde_json::Error),
    #[diagnostic(transparent)]
    #[error(transparent)]
    TemplateResolveError(#[from] TemplateResolveError),
}

impl<'a> ExamplesConfiguration<'a> {
    pub async fn from_metadata(metadata: &'a Metadata) -> Result<Self, ExamplesConfigurationError> {
        let mut examples = HashMap::new();
        let mut unconfigured_examples = HashSet::new();
        let mut unknown_examples: HashSet<(PackageId, String)> = HashSet::new();

        let mut workspace_config = serde_json::from_value::<Option<WorkspaceMetadata>>(
            metadata.workspace_metadata.clone(),
        )?
        .unwrap_or_default()
        .example_runner;

        workspace_config.default = workspace_config
            .default
            .into_iter()
            .map(|mut default_config| {
                default_config.resolve_templates(&workspace_config.templates)?;
                Ok::<_, TemplateResolveError>(default_config)
            })
            .collect::<Result<HashSet<_>, _>>()?;

        let workspace_default_configs = workspace_config
            .default
            .into_iter()
            .map(|config| {
                config
                    .extract_config()
                    .with_default_type(RunnerType::Explicit)
            })
            .collect::<HashSet<_>>();

        for package in metadata.workspace_packages() {
            let PackageConfig {
                mut example_runner,
                examples: mut examples_configs,
            } = serde_json::from_value::<Option<PackageMetadata>>(package.metadata.clone())?
                .unwrap_or_default()
                .package;

            example_runner.default = example_runner
                .default
                .into_iter()
                .map(|mut default_config| {
                    default_config
                        .resolve_templates(&example_runner.templates)
                        .or_else(|_| {
                            default_config.resolve_templates(&workspace_config.templates)
                        })?;
                    Ok::<_, TemplateResolveError>(default_config)
                })
                .collect::<Result<HashSet<_>, _>>()?;

            let package_default_configs = example_runner.default.into_iter().map(|config| {
                config
                    .extract_config()
                    .with_default_type(RunnerType::Explicit)
            });

            let fallback_configs =
                if package_default_configs.len() > 0 && !example_runner.extend_workspace_defaults {
                    package_default_configs.collect::<HashSet<_>>()
                } else if package_default_configs.len() > 0 {
                    workspace_default_configs
                        .iter()
                        .cloned()
                        .chain(package_default_configs)
                        .collect::<HashSet<_>>()
                } else {
                    workspace_default_configs.clone()
                };

            for target in package.targets.iter().filter(|target| target.is_example()) {
                let explicit_configs = examples_configs.remove(&target.name);

                if explicit_configs.is_none() && fallback_configs.is_empty() {
                    unconfigured_examples.insert((package.id.clone(), target));
                    continue;
                }

                let example_configurations = explicit_configs
                    .map(|configs| -> Result<_, _> {
                        let explicit_configs = configs
                            .configurations
                            .into_iter()
                            .map(|mut config| {
                                config
                                    .resolve_templates(&example_runner.templates)
                                    .or_else(|_| {
                                        config.resolve_templates(&workspace_config.templates)
                                    })?;
                                Ok::<_, TemplateResolveError>(
                                    config
                                        .extract_config()
                                        .with_default_type(RunnerType::Explicit),
                                )
                            })
                            .collect::<Result<HashSet<_>, _>>()?;

                        Ok::<_, TemplateResolveError>(if configs.extend_configurations {
                            explicit_configs
                                .into_iter()
                                .chain(fallback_configs.iter().cloned())
                                .collect()
                        } else {
                            explicit_configs
                        })
                    })
                    .transpose()?
                    .unwrap_or_else(|| fallback_configs.clone());

                examples.insert((package.id.clone(), target), example_configurations);
            }

            for config in examples_configs.into_keys() {
                unknown_examples.insert((package.id.clone(), config));
            }
        }

        Ok(Self {
            examples,
            unconfigured: unconfigured_examples,
            unknown: unknown_examples,
        })
    }
}
