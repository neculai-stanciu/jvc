use std::path::PathBuf;

use super::shell::Shell;
use async_trait::async_trait;
use log::warn;

#[derive(Debug)]
pub struct Bash;

#[async_trait]
impl Shell for Bash {
    async fn into_clap_shell(&self) -> structopt::clap::Shell {
        structopt::clap::Shell::Bash
    }

    async fn export_path(&self, path: &PathBuf) -> String {
        if let Some(new_path) = path.to_str() {
            format!("export PATH={:?}:$PATH", new_path)
        } else {
            warn!("Cannot construct path for Path variable");
            "".to_owned()
        }
    }

    async fn set_env_var(&self, name: &str, value: &str) -> String {
        format!("export {}={:?}", name, value)
    }
}
