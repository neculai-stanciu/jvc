use crate::config::JvcConfig;
use anyhow::Result;
use async_trait::async_trait;
#[async_trait]
pub trait Executor {
    async fn execute(self, config: JvcConfig) -> Result<()>;
}
