use std::future::Future;

use cargo_metadata::{Metadata, MetadataCommand};
use tokio::process::Command as AsyncCommand;

pub trait AsyncExecMetadata {
    fn exec_async(
        &self,
    ) -> impl Future<Output = Result<Metadata, cargo_metadata::Error>> + Send + Sync;
}

impl AsyncExecMetadata for MetadataCommand {
    async fn exec_async(&self) -> Result<Metadata, cargo_metadata::Error> {
        let command = self.cargo_command();
        let output = AsyncCommand::from(command).output().await?;

        if !output.status.success() {
            return Err(cargo_metadata::Error::CargoMetadata {
                stderr: String::from_utf8(output.stderr)?,
            });
        }

        let stdout = std::str::from_utf8(&output.stdout)?
            .lines()
            .find(|line| line.starts_with('{'))
            .ok_or(cargo_metadata::Error::NoJson)?;

        Self::parse(stdout)
    }
}

pub trait Discover {
    type Error;

    fn discover() -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn discover_async() -> impl Future<Output = Result<Self, Self::Error>> + Send + Sync
    where
        Self: Sized;
}

impl Discover for Metadata {
    type Error = cargo_metadata::Error;

    fn discover() -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        MetadataCommand::new().exec()
    }

    async fn discover_async() -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        MetadataCommand::new().exec_async().await
    }
}
