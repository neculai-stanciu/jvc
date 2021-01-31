use super::{alias::Alias, executor::Executor};
use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use log::{debug, info};
use structopt::StructOpt;
#[derive(Debug, StructOpt)]
pub struct Default {
    pub to_version: String,
}

#[async_trait]
impl Executor for Default {
    async fn execute(self, config: crate::config::JvcConfig) -> Result<()> {
        debug!("Creating default version for: {}", self.to_version);
        let provider = config.java_provider.as_str();
        Alias {
            name: "default".into(),
            to_version: self.to_version.clone(),
        }
        .execute(config)
        .await?;

        info!(
            "Selected version {} is now default with provider {}",
            self.to_version.blue(),
            provider.blue()
        );
        Ok(())
    }
}
