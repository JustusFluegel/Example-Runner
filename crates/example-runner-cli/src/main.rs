mod cargo_metadata_async;
mod config_args;
mod example_config;
mod read_example_configuration;
mod struct_merge;
mod templates;

use std::process::Stdio;

use cargo_metadata::MetadataCommand;
use clap::Parser;
use futures::{
    stream::{FuturesOrdered, FuturesUnordered},
    StreamExt,
};
use miette::Diagnostic;
use read_example_configuration::ExamplesConfigurationError;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::{
    cargo_metadata_async::AsyncExecMetadata, example_config::ExampleConfigFinalized,
    read_example_configuration::ExamplesConfiguration,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Errors if any examples aren't configured (either explicitly or via a default config)
    #[arg(long)]
    error_on_unconfigured: bool,
    /// Errors if any runner configurations are present for unknown examples
    #[arg(long)]
    error_on_unknown: bool,
    /// Run examples in parallel?
    #[arg(short, long, default_value = "false")]
    parallel: bool,
}

#[derive(thiserror::Error, Debug, Diagnostic)]
enum AppError {
    #[diagnostic(
        code(app::metadata_fetch_failed),
        help("Check if you are inside a cargo workspace")
    )]
    #[error(transparent)]
    MetadataFetch(#[from] cargo_metadata::Error),
    #[diagnostic(transparent)]
    #[error(transparent)]
    ExampleConfigurationsRead(#[from] ExamplesConfigurationError),
    #[diagnostic(
        code(app::unconfigured_example),
        help("Check the output above for unconfigured examples and configure them")
    )]
    #[error("An unconfigured example was encountered")]
    UnconfiguredExample,
    #[diagnostic(
        code(app::unknown_example),
        help(
            "Check the output above for unconfigured examples and remove the configuration of them"
        )
    )]
    #[error("An unconfigured example was encountered")]
    UnknownExample,
    #[error(transparent)]
    ExampleRun(#[from] std::io::Error),
    #[error(transparent)]
    StdioJoin(#[from] tokio::task::JoinError),
    #[diagnostic(
        code(app::unknown_example),
        help("Check the output above for unsucessful examples and fix them")
    )]
    #[error("An unsucessful example run was encountered")]
    ExampleUnsuccessful,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    Ok(main_wrapper().await?)
}

async fn main_wrapper() -> Result<(), AppError> {
    let args = Args::parse();

    let metadata = MetadataCommand::new().exec_async().await?;

    let ExamplesConfiguration {
        examples,
        unconfigured,
        unknown,
    } = ExamplesConfiguration::from_metadata(&metadata).await?;

    let unconfigured_examples_present = !unconfigured.is_empty();
    for (package_id, target) in unconfigured {
        println!(
            "Unconfigured example found in crate {}: {}",
            metadata
                .packages
                .iter()
                .find(|package| package.id == package_id)
                .unwrap()
                .name,
            target.name
        )
    }
    let unknown_examples_present = !unknown.is_empty();
    for (package_id, example_name) in unknown {
        println!(
            "Unknown example configured in crate {}: {}",
            metadata
                .packages
                .iter()
                .find(|package| package.id == package_id)
                .unwrap()
                .name,
            example_name
        )
    }

    if args.error_on_unconfigured && unconfigured_examples_present {
        return Err(AppError::UnconfiguredExample);
    }

    if args.error_on_unknown && unknown_examples_present {
        return Err(AppError::UnknownExample);
    }

    let to_be_run = examples.iter().flat_map(|(id, configurations)| {
        let package = metadata
            .packages
            .iter()
            .find(|package| package.id == id.0)
            .unwrap();
        configurations.iter().enumerate().map(|(i, configuration)| {
            let mut command = tokio::process::Command::new("cargo");
            command
                .arg("run")
                .arg("-p")
                .arg(&package.name.clone())
                .arg("--example")
                .arg(&id.1.name.clone())
                .arg("--")
                .args(configuration.args.arguments.clone())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::null());
            (
                (
                    package.name.clone(),
                    id.1.name.clone(),
                    configuration
                        .args
                        .label
                        .as_ref()
                        .map(ToOwned::to_owned)
                        .unwrap_or_else(|| i.to_string()),
                ),
                command,
                configuration.to_owned(),
            )
        })
    });

    type StdioHandles = (
        tokio::task::JoinHandle<Result<(), tokio::io::Error>>,
        tokio::task::JoinHandle<Result<(), tokio::io::Error>>,
    );

    fn run_command(
        ((package_name, target_name, config_name), mut command, configuration): (
            (String, String, String),
            tokio::process::Command,
            ExampleConfigFinalized,
        ),
    ) -> std::io::Result<(
        tokio::process::Child,
        StdioHandles,
        ExampleConfigFinalized,
        (String, String, String),
    )> {
        let mut child = command.spawn()?;

        let stdout = child.stdout.take().unwrap();
        let mut root_stdout = tokio::io::stdout();
        let package_name_copy = package_name.clone();
        let target_name_copy = target_name.clone();
        let config_name_copy = config_name.clone();
        let stdout_handle = tokio::spawn(async move {
            let mut buf_reader = tokio::io::BufReader::new(stdout).lines();
            while let Some(line) = buf_reader.next_line().await? {
                root_stdout
                    .write_all(
                        format!(
                            "[{package} {example} <{config}>] {line}\n",
                            package = package_name_copy,
                            example = target_name_copy,
                            config = config_name_copy
                        )
                        .as_bytes(),
                    )
                    .await?;
            }

            Ok::<_, std::io::Error>(())
        });

        let stderr = child.stderr.take().unwrap();
        let mut root_stderr = tokio::io::stderr();
        let package_name_copy = package_name.clone();
        let target_name_copy = target_name.clone();
        let config_name_copy = config_name.clone();
        let stderr_handle = tokio::spawn(async move {
            let mut buf_reader = tokio::io::BufReader::new(stderr).lines();
            while let Some(line) = buf_reader.next_line().await? {
                root_stderr
                    .write_all(
                        format!(
                            "[{package} {example} <{config}>] {line}\n",
                            package = package_name_copy,
                            example = target_name_copy,
                            config = config_name_copy
                        )
                        .as_bytes(),
                    )
                    .await?;
            }

            Ok::<_, std::io::Error>(())
        });

        Ok((
            child,
            (stdout_handle, stderr_handle),
            configuration,
            (package_name, target_name, config_name),
        ))
    }

    let results = if args.parallel {
        let mut handles = to_be_run.map(run_command).collect::<Result<Vec<_>, _>>()?;

        let exit_codes = handles
            .iter_mut()
            .map(|(child, _, _, _)| async move { child.wait().await })
            .collect::<FuturesOrdered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        exit_codes
            .into_iter()
            .zip(handles.into_iter())
            .map(|(exit_code, (_, handles, config, names))| async move {
                handles.0.await??;
                handles.1.await??;
                Ok::<_, AppError>((exit_code, config, names))
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let mut results = Vec::new();

        for command in to_be_run {
            let (mut child, handles, config, names) = run_command(command)?;
            results.push((child.wait().await?, config, names));
            handles.0.await??;
            handles.1.await??;
        }

        results
    };

    let mut unsucessful = false;
    for (exit_status, config, (package, example, config_label)) in results {
        if !config.args.expected_exit_status.matches(&exit_status) {
            unsucessful = true;
            println!("Example run [{package} {example} <{config_label}>] executed unsucessfully with {exit_status}!", );
        } else {
            println!("Example run [{package} {example} <{config_label}>] suceeded!");
        }
    }

    if unsucessful {
        Err(AppError::ExampleUnsuccessful)
    } else {
        Ok(())
    }
}
