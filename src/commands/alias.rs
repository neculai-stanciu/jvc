use std::fs::{read_link, rename};

use crate::{config::JvcConfig, version::Version};

use super::{
    executor::Executor,
    list::{find_version, VersionPath},
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use colored::Colorize;
use log::{debug, info, warn};
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
    let version_name = applicable_version
        .path()
        .file_name()
        .ok_or(anyhow!("Cannot get file name!"))?
        .to_str()
        .ok_or(anyhow!("Cannot convert to utf8 string"))?;

    let version = Version::new_from_disk(version_name);

    let symlink_exists = read_link(alias_dir.as_path()).ok();
    debug!(
        "Aliases dir: {:?} - exists {}",
        alias_dir.as_path(),
        symlink_exists.is_some()
    );

    if symlink_exists.is_some() {
        debug!("Try to remove: {:?}", alias_dir.as_path());

        remove_symlink_dir(&alias_dir)?;
    }
    symlink_dir(version_dir, alias_dir.as_path())?;

    info!(
        "Created alias {} for version {} and provider {}",
        name.green(),
        version.value.green(),
        version.provider.as_str().green()
    );

    Ok(())
}

fn override_alias(name: &str, to_version: &str, config: &JvcConfig) -> Result<()> {
    debug!("Try to override alias {} to version {}", name, to_version);

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
