use async_trait::async_trait;

use super::shell::Shell;

#[derive(Debug)]
pub struct WindowsCMD;

impl WindowsCMD {
    async fn set_var(&self, name: &str, value: &str) -> String {
        format!("SET {}={}", name, value)
    }
}

#[async_trait]
impl Shell for WindowsCMD {
    async fn into_clap_shell(&self) -> structopt::clap::Shell {
        panic!("Shell completions is not supported in Windows CMD")
    }

    async fn export_path(&self, path: &std::path::PathBuf) -> String {
        let current_path = std::env::var_os("PATH").expect("Can't read PATH env var");
        let mut split_paths: Vec<_> = std::env::split_paths(&current_path).collect();
        split_paths.insert(0, path.to_path_buf());
        let new_path = std::env::join_paths(split_paths).expect("Can't join paths");
        let binary_path = new_path.to_str().expect("Can't read PATH");
        // self.set_path_var(binary_path).await;
        self.set_var("PATH", binary_path).await
    }

    async fn set_env_var(&self, name: &str, value: &str) -> String {
        self.set_var(name, value).await
    }
}
