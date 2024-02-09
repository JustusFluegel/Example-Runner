mod cargo_metadata_async;
mod config_args;
mod example_config;
mod read_example_configuration;
mod struct_merge;
mod templates;

use cargo_metadata::MetadataCommand;
use clap::Parser;
use miette::{Diagnostic, IntoDiagnostic};
use read_example_configuration::ExamplesConfigurationError;

use crate::{
    cargo_metadata_async::AsyncExecMetadata, read_example_configuration::ExamplesConfiguration,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    error_on_unconfigured: bool,
    #[arg(long)]
    error_on_unknown: bool,
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
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    main_wrapper().await.into_diagnostic()
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

    dbg!(examples);

    Ok(())
}
