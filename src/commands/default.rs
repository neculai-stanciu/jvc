use super::{alias::Alias, executor::Executor};
use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use structopt::StructOpt;
#[derive(Debug, StructOpt)]
pub struct Default {
    pub to_version: String,
}

#[async_trait]
impl Executor for Default {
    async fn execute(self, config: crate::config::JvcConfig) -> Result<()> {
        debug!("Creating default version for: {}", self.to_version);

        Alias {
            name: "default".into(),
            to_version: self.to_version.clone(),
        }
        .execute(config)
        .await?;

        Ok(())
    }
}
