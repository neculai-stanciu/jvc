use std::fs::rename;

use crate::config::JvcConfig;

use super::{
    executor::Executor,
    list::{find_version, VersionPath},
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::warn;
use structopt::StructOpt;
use symlink::{remove_symlink_dir, symlink_dir};

#[derive(Debug, StructOpt)]
pub struct Alias {
    pub to_version: String,
    pub name: String,
}

#[async_trait]
impl Executor for Alias {
    async fn execute(self, config: crate::config::JvcConfig) -> Result<()> {
        validate_alias_name(self.name.as_str())?;

        create_alias(self.name.as_str(), self.to_version.as_str(), &config)
    }
}

fn create_alias(name: &str, to_version: &str, config: &JvcConfig) -> Result<()> {
    if is_alias(to_version) {
        warn!("We will override your existing alias.");
        override_alias(name, to_version, &config)
    } else {
        let version = to_version.parse::<u8>()?;
        let applicable_version =
            find_version(version, &config)?.ok_or(anyhow!("Cannot find requested version!"))?;

        apply_alias(&config, name, applicable_version)
    }
}

fn apply_alias(config: &JvcConfig, name: &str, applicable_version: VersionPath) -> Result<()> {
    let aliases_dir = config.aliases_dir();
    let version_dir = applicable_version.path();
    let alias_dir = aliases_dir.join(name);

    if alias_dir.exists() {
        remove_symlink_dir(&alias_dir)?;
    }
    symlink_dir(version_dir, alias_dir.as_path())?;

    Ok(())
}

fn override_alias(name: &str, to_version: &str, config: &JvcConfig) -> Result<()> {
    let aliases_dir = config.aliases_dir();
    let actual_alias_dir = aliases_dir.join(to_version);
    let required_alias_dir = aliases_dir.join(name);

    rename(actual_alias_dir.as_path(), required_alias_dir.as_path())?;

    Ok(())
}

/// alias name should not be parsable to u8
fn validate_alias_name(alias_name: &str) -> Result<()> {
    if let Ok(_) = alias_name.parse::<u8>() {
        Err(anyhow!("Alias name should not be a version number"))
    } else {
        Ok(())
    }
}

fn is_alias(version: &str) -> bool {
    if let Ok(_) = version.parse::<u8>() {
        false
    } else {
        true
    }
}
