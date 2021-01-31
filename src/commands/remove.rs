use std::{fs::remove_dir_all, path::Path};

use super::{executor::Executor, list::find_version};
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use structopt::StructOpt;
#[derive(Debug, StructOpt)]
pub struct Remove {
    pub version: String,
}

#[async_trait]
impl Executor for Remove {
    async fn execute(self, config: crate::config::JvcConfig) -> Result<()> {
        let version = self.version.parse::<u8>()?;
        let applicable_version =
            find_version(version, &config)?.ok_or(anyhow!("Cannot find requested version!"))?;

        info!("Removing version: {}", version);
        debug!("Path to version: {:?}", applicable_version.path());
        remove_version(applicable_version.path())?;
        Ok(())
    }
}

fn remove_version<T: AsRef<Path>>(path: T) -> Result<()> {
    remove_dir_all(path.as_ref()).map_err(|_e| anyhow!("Cannot get path"))
}

#[cfg(test)]
mod tests {

    use super::Remove;
    use crate::{commands::executor::Executor, config::JvcConfig};
    use anyhow::Result;

    #[tokio::test]
    async fn test_remove_execution_for_big_version() -> Result<()> {
        let remove = Remove {
            version: "1000".to_owned(),
        };
        let result: Result<()> = remove.execute(JvcConfig::default()).await;
        assert!(result.is_err());
        Ok(())
    }
}
