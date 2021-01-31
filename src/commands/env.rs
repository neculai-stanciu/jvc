use std::{env::temp_dir, path::PathBuf, process};

use crate::{
    config::JvcConfig,
    shell::{
        detect_shell,
        shell::{Shell, AVAILABLE_SHELLS},
    },
};

use super::executor::Executor;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Env {
    /// Shell to use. Try to detect if this option is missing.
    #[structopt(long)]
    #[structopt(possible_values = AVAILABLE_SHELLS)]
    shell: Option<Box<dyn Shell>>,
}

#[async_trait]
impl Executor for Env {
    async fn execute(self, config: JvcConfig) -> Result<()> {
        let shell: Box<dyn Shell> = self.shell.or_else(&detect_shell).ok_or(anyhow!(
            "Cannot detect your shell. Please provide your shell as option."
        ))?;
        let shell_path = make_symlink(&config);
        let bin_path = shell_path.join("installation").join("bin");

        let env_path = shell.export_path(&bin_path).await;
        println!("{}", env_path);

        let shell_path_as_str = shell_path
            .to_str()
            .ok_or(anyhow!("Cannot set shell path."))?;
        let base_dir = config.get_base_dir_or_default();
        let jvc_dir_as_str = base_dir
            .to_str()
            .ok_or(anyhow!("Cannot read base directory."))?;

        let jvc_shell_path = shell.set_env_var("JVC_SHELL_PATH", shell_path_as_str).await;
        let jvc_dir = shell.set_env_var("JVC_DIR", jvc_dir_as_str).await;
        let jvc_loglevel = shell
            .set_env_var("JVC_LOGLEVEL", config.log_level.into())
            .await;
        let jvc_provider = shell
            .set_env_var("JVC_PROVIDER", &config.java_provider.as_str())
            .await;

        println!("{}", jvc_shell_path);
        println!("{}", jvc_dir);
        println!("{}", jvc_loglevel);
        println!("{}", jvc_provider);

        Ok(())
    }
}

pub fn make_symlink(config: &JvcConfig) -> PathBuf {
    let sys_temp_dir = temp_dir();
    let mut temp_dir = create_symlink_path(&sys_temp_dir);

    while temp_dir.exists() {
        temp_dir = create_symlink_path(&sys_temp_dir);
    }

    symlink::symlink_dir(config.default_version_dir(), &temp_dir).expect("Cannot create symlink");
    temp_dir
}

fn create_symlink_path(sys_temp_dir: &PathBuf) -> PathBuf {
    let temp_dir_name = format!(
        "jvc_shell_{}_{}",
        process::id(),
        Utc::now().timestamp_millis()
    );
    sys_temp_dir.join(temp_dir_name)
}
