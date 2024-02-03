use cargo_metadata::MetadataCommand;
use miette::IntoDiagnostic;
use serde::Deserialize;
use tokio::process::Command as AsyncCommand;

#[derive(Deserialize, Debug)]
struct Example {
    name: String,
    runner: RunnerOptionVariants,
}

#[derive(Deserialize, Debug)]
enum RunnerType {
    #[serde(rename = "inherit")]
    Inherit,
    #[serde(rename = "explicit")]
    Explicit,
    #[serde(rename = "no_run")]
    NoRun,
    #[serde(rename = "ignore")]
    Ignore,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum RunnerOptionVariants {
    Direct(RunnerType),
    Options(RunnerOptions),
}

#[derive(Deserialize, Debug)]
struct RunnerOptions {
    r#type: RunnerType,
    arguments: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Manifest {
    example: Option<Vec<Example>>,
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
            .find(|line| line.starts_with("{"))
            .ok_or(cargo_metadata::Error::NoJson)
            .into_diagnostic()?,
    )
    .into_diagnostic()?;

    let workspace_packages = metadata.workspace_packages();
    for package in workspace_packages {
        let examples = toml::from_str::<Manifest>(
            &tokio::fs::read_to_string(&package.manifest_path)
                .await
                .into_diagnostic()?,
        )
        .into_diagnostic()?;
        dbg!(examples);
        for example in package.targets.iter().filter(|target| target.is_example()) {
            dbg!(example);
        }
    }
    Ok(())
}
