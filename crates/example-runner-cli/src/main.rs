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
    #[arg(short, long)]
    error_on_unconfigured: bool,
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

    if args.error_on_unconfigured && unconfigured_examples_present {
        return Err(AppError::UnconfiguredExample);
    }

    dbg!(examples);

    Ok(())
}
