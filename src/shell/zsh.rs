use super::shell::Shell;
use async_trait::async_trait;
use log::warn;

#[derive(Debug)]
pub struct Zsh;

#[async_trait]
impl Shell for Zsh {
    async fn into_clap_shell(&self) -> structopt::clap::Shell {
        structopt::clap::Shell::Zsh
    }

    async fn export_path(&self, path: &std::path::PathBuf) -> String {
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
