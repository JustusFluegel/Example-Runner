mod example_config;
mod templates;

use core::task;
use std::collections::HashMap;

use cargo_metadata::{MetadataCommand, Package, Target};
use example_config::{Example, ExampleConfig, RunnerOptions};
use miette::IntoDiagnostic;
use serde::Deserialize;
use templates::ExampleRunnerConfig;
use tokio::process::Command as AsyncCommand;

use crate::example_config::TemplateResolveError;

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

#[derive(Debug)]
struct RunnerExample<'a> {
    package: &'a Package,
    target: &'a Target,
    options: Option<ExampleConfig>,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let output = AsyncCommand::from(MetadataCommand::new().cargo_command())
        .output()
        .await
        .into_diagnostic()?;

    if !output.status.success() {
        return Err(cargo_metadata::Error::CargoMetadata {
            stderr: String::from_utf8(output.stderr).into_diagnostic()?,
        })
        .into_diagnostic();
    }

    let metadata = MetadataCommand::parse(
        std::str::from_utf8(&output.stdout)
            .into_diagnostic()?
            .lines()
            .find(|line| line.starts_with('{'))
            .ok_or(cargo_metadata::Error::NoJson)
            .into_diagnostic()?,
    )
    .into_diagnostic()?;

    let mut examples = Vec::new();

    let workspace_packages = metadata.workspace_packages();

    let mut workspace_config = metadata.workspace_root.clone();
    workspace_config.push("Cargo.toml");

    let workspace_manifest = toml::from_str::<WorkspaceManifest>(
        &tokio::fs::read_to_string(&workspace_config)
            .await
            .into_diagnostic()?,
    )
    .into_diagnostic()?;

    for package in workspace_packages {
        let manifest = toml::from_str::<Manifest>(
            &tokio::fs::read_to_string(&package.manifest_path)
                .await
                .into_diagnostic()?,
        )
        .into_diagnostic()?;

        let mut additional_configs = manifest
            .example
            .into_iter()
            .flatten()
            .map(|example| (example.name.clone(), example))
            .collect::<HashMap<_, _>>();

        for target in package.targets.iter().filter(|target| target.is_example()) {
            let options = additional_configs
                .remove(&target.name)
                .map(|example| {
                    let step_1 = RunnerOptions::from(example.runner).resolve_templates(
                        manifest
                            .example_runner
                            .as_ref()
                            .and_then(|example_runner| example_runner.templates.as_ref()),
                    );

                    match step_1 {
                        Err(TemplateResolveError::NoSuchTemplate { options }) => options
                            .resolve_templates(
                                workspace_manifest
                                    .workspace
                                    .as_ref()
                                    .and_then(|workspace_config| {
                                        workspace_config.example_runner.as_ref()
                                    })
                                    .and_then(|example_runner| example_runner.templates.as_ref()),
                            ),
                        v => v,
                    }
                })
                .transpose()?;

            examples.push(RunnerExample {
                package,
                target,
                options,
            });
        }
    }

    dbg!(examples);

    Ok(())
}
