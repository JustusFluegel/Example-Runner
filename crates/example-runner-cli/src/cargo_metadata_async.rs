use cargo_metadata::{Metadata, MetadataCommand};
use tokio::process::Command as AsyncCommand;

pub trait AsyncExecMetadata {
    fn exec_async(
        &self,
    ) -> impl std::future::Future<Output = Result<Metadata, cargo_metadata::Error>> + Send + Sync;
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
