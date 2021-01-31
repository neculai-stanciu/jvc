use async_trait::async_trait;
use std::{path::PathBuf, str::FromStr};

use super::{bash::Bash, powershell::PowerShell, windows_cmd::WindowsCMD, zsh::Zsh};

#[async_trait]
pub trait Shell: std::fmt::Debug + Send + Sync {
    async fn into_clap_shell(&self) -> structopt::clap::Shell;
    async fn export_path(&self, path: &PathBuf) -> String;
    async fn set_env_var(&self, name: &str, value: &str) -> String;
}

#[cfg(windows)]
pub const AVAILABLE_SHELLS: &[&str; 4] = &["cmd", "powershell", "bash", "zsh"];

#[cfg(unix)]
pub const AVAILABLE_SHELLS: &[&str; 3] = &["bash", "zsh", "powershell"];

impl FromStr for Box<dyn Shell> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cmd" => Ok(Box::from(WindowsCMD)),
            "zsh" => Ok(Box::from(Zsh)),
            "bash" => Ok(Box::from(Bash)),
            "powershell" => Ok(Box::from(PowerShell)),
            shell_type => Err(format!("Cannot identify shell type: {:?}", shell_type)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Shell, AVAILABLE_SHELLS};
    #[test]
    fn available_shells_should_be_parsable_from_string() {
        for shell_name in AVAILABLE_SHELLS {
            let shell = shell_name.parse::<Box<dyn Shell>>();
            assert!(shell.is_ok())
        }
    }
}
