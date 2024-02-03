use std::collections::{HashMap, HashSet};

use cargo_metadata::{Metadata, PackageId, Target};
use miette::Diagnostic;
use serde::Deserialize;

use crate::{
    example_config::{
        Example, ExampleConfigFinalized, RunnerOptions, RunnerType, TemplateResolveError,
    },
    templates::ExampleRunnerConfig,
};

#[derive(Deserialize, Debug)]
struct Manifest {
    example: Option<Vec<Example>>,
    example_runner: Option<ExampleRunnerConfig>,
}

#[derive(Deserialize, Debug)]
struct WorkspaceManifest {
    workspace: Option<WorkspaceConfig>,
}

#[derive(Deserialize, Debug)]
struct WorkspaceConfig {
    example_runner: Option<ExampleRunnerConfig>,
}

pub struct ExamplesConfiguration<'a> {
    pub examples: HashMap<(PackageId, &'a Target), ExampleConfigFinalized>,
    pub unconfigured: HashSet<(PackageId, &'a Target)>,
}

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum ExamplesConfigurationError {
    #[error(transparent)]
    WorkspaceCargoTomlReadFailed(#[from] std::io::Error),
    #[diagnostic(
        code(examples_config::invalid_toml),
        help("Check your Cargo.toml for syntax errors")
    )]
    #[error(transparent)]
    InvalidTomlInConfig(#[from] toml::de::Error),
    #[diagnostic(transparent)]
    #[error(transparent)]
    TemplateResolveError(#[from] TemplateResolveError),
}

impl<'a> ExamplesConfiguration<'a> {
    pub async fn from_metadata(metadata: &'a Metadata) -> Result<Self, ExamplesConfigurationError> {
        let mut examples = HashMap::new();
        let mut unconfigured_examples = HashSet::new();

        let workspace_packages = metadata.workspace_packages();

        let mut workspace_config = metadata.workspace_root.clone();
        workspace_config.push("Cargo.toml");

        let workspace_manifest = toml::from_str::<WorkspaceManifest>(
            &tokio::fs::read_to_string(&workspace_config).await?,
        )?;

        let workspace_config = workspace_manifest
            .workspace
            .and_then(|workspace_config| workspace_config.example_runner);

        let workspace_templates = workspace_config
            .as_ref()
            .and_then(|example_runner| example_runner.templates.as_ref());

        let workspace_fallback_config = workspace_config
            .as_ref()
            .and_then(|example_runner| example_runner.default.as_ref())
            .map(|options| options.clone().resolve_templates(workspace_templates))
            .transpose()?
            .map(|config| config.finalize_fallback_to_type(RunnerType::Explicit));

        for package in workspace_packages {
            let manifest = toml::from_str::<Manifest>(
                &tokio::fs::read_to_string(&package.manifest_path).await?,
            )?;

            let mut additional_configs = manifest
                .example
                .into_iter()
                .flatten()
                .map(|example| (example.name.clone(), example))
                .collect::<HashMap<_, _>>();

            let package_templates = manifest
                .example_runner
                .as_ref()
                .and_then(|example_runner| example_runner.templates.as_ref());

            let package_fallback_config = manifest
                .example_runner
                .as_ref()
                .and_then(|example_runner| example_runner.default.as_ref())
                .map(|options| {
                    options
                        .clone()
                        .resolve_templates(package_templates)
                        .or_else(|err| {
                            err.chain(|options| options.resolve_templates(workspace_templates))?
                        })
                })
                .transpose()?
                .map(|config| config.finalize_fallback_to_type(RunnerType::Explicit))
                .map(|config| {
                    if let Some(fallback_config) = &workspace_fallback_config {
                        config.fallback(fallback_config)
                    } else {
                        config
                    }
                });

            let fallback_config = package_fallback_config
                .as_ref()
                .or(workspace_fallback_config.as_ref());

            for target in package.targets.iter().filter(|target| target.is_example()) {
                let Some(config) = additional_configs
                    .remove(&target.name)
                    .and_then(|example| {
                        Some(
                            RunnerOptions::from(example.runner?)
                                .resolve_templates(package_templates)
                                .or_else(|err| {
                                    err.chain(|options| {
                                        options.resolve_templates(workspace_templates)
                                    })?
                                }),
                        )
                    })
                    .transpose()?
                    .map(|config| {
                        if let Some(fallback_config) = fallback_config {
                            config.fallback(fallback_config)
                        } else {
                            config
                        }
                    })
                    .map(|config| config.finalize_fallback_to_type(RunnerType::Explicit))
                    .or_else(|| fallback_config.cloned())
                else {
                    unconfigured_examples.insert((package.id.clone(), target));
                    continue;
                };

                examples.insert((package.id.clone(), target), config);
            }
        }

        Ok(ExamplesConfiguration {
            examples,
            unconfigured: unconfigured_examples,
        })
    }
}
