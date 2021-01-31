use super::executor::Executor;
use anyhow::Result;
use async_trait::async_trait;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Package {}

#[async_trait]
impl Executor for Package {
    async fn execute(self, config: crate::config::JvcConfig) -> Result<()> {
        println!("Called pkg: {:#?} list info: {:#?}", config, self);
        Ok(())
    }
}
