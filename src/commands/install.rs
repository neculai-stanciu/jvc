use std::{fs::rename, path::PathBuf};

use super::executor::Executor;
use crate::{
    archive::unarchive,
    client::package_client::get_client,
    config::{new_client_config, VersionRequirements},
    provider::Provider,
    version::Version,
};
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use log::{debug, warn};
use structopt::StructOpt;
#[derive(Debug, StructOpt)]
pub struct Install {
    /// Version to install
    pub version: u32,

    #[structopt(flatten)]
    pub requirements: VersionRequirements,
}

#[async_trait]
impl Executor for Install {
    async fn execute(self, config: crate::config::JvcConfig) -> Result<()> {
        let provider: &Provider = &config.java_provider;
        let client = get_client(provider);

        let versions = client.get_all_version(self.requirements.clone()).await?;
        let version = get_selected_version(versions, self.version).await?;

        let install_dir = &config.get_installation_dir().join(format!(
            "{}-{}",
            version.as_disk_version(),
            provider.as_str()
        ));

        let download_response = client
            .download(new_client_config(&config, version, self.requirements))
            .await?;
        debug!("Downloaded package {}", download_response.package_name);
        let archive_result = unarchive(
            download_response.package_name.as_ref(),
            download_response.download_path,
            install_dir.to_owned(),
        );

        let package_name = std::fs::read_dir(install_dir)?
            .next()
            .ok_or(anyhow!("Cannot get extracted package."))??
            .file_name()
            .into_string()
            .expect("Cannot extract archive name.");

        match archive_result {
            Ok(_) => rename_package(install_dir, package_name.as_ref())?,
            Err(e) => warn!("Cannot extract archive: {}", e),
        }
        Ok(())
    }
}

async fn get_selected_version(mut versions: Vec<Version>, version: u32) -> Result<Version> {
    let selected_version = versions
        .drain(..)
        .find(|v| v.value.eq_ignore_ascii_case(version.to_string().as_ref()))
        .ok_or(anyhow!(
            "Cannot find a version to match your selection {}",
            version.to_string().red()
        ))?;
    Ok(selected_version)
}

fn rename_package(install_dir: &PathBuf, package_name: &str) -> Result<()> {
    debug!(
        "Rename from {:?} to {:?}",
        install_dir
            .join(package_name.replace(".tar.gz", "").replace(".zip", ""))
            .as_path(),
        install_dir.join("installation").as_path()
    );

    rename(
        install_dir.join(package_name.replace(".tar.gz", "").replace(".zip", "")),
        install_dir.join("installation"),
    )?;

    Ok(())
}
