use super::shell::Shell;
use async_trait::async_trait;
#[derive(Debug)]
pub struct PowerShell;

#[async_trait]
impl Shell for PowerShell {
    async fn into_clap_shell(&self) -> structopt::clap::Shell {
        structopt::clap::Shell::PowerShell
    }

    async fn export_path(&self, path: &std::path::PathBuf) -> String {
        let current_path = std::env::var_os("PATH").expect("Can't read PATH env var");
        let mut split_paths: Vec<_> = std::env::split_paths(&current_path).collect();
        split_paths.insert(0, path.to_path_buf());
        let new_path = std::env::join_paths(split_paths).expect("Can't join paths");
        self.set_env_var("PATH", new_path.to_str().expect("Can't read PATH"))
            .await
    }

    async fn set_env_var(&self, name: &str, value: &str) -> String {
        format!(r#"$env:{} = "{}""#, name, value)
    }
}
