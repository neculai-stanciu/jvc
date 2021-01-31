use crate::{
    client::package_client::get_client,
    config::{JvcConfig, VersionRequirements},
    version::Version,
};
use std::{
    fs::read_link,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    vec,
};

use super::executor::Executor;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use colored::Colorize;
use log::debug;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct List {
    /// Activate remote listing for feature versions
    #[structopt(long = "remote", short = "rm")]
    pub remote: bool,

    #[structopt(flatten)]
    pub requirements: VersionRequirements,
}
#[async_trait]
impl Executor for List {
    async fn execute(self, config: JvcConfig) -> Result<()> {
        debug!(
            "running list command with provider {}",
            &config.java_provider.as_str()
        );
        let versions = if self.remote {
            debug!("try to retrieve list of available versions");
            let provider = &config.java_provider;
            let client = get_client(provider);
            client
                .get_all_version(self.requirements)
                .await
                .context("Cannot get available versions")?
        } else {
            let installation_dir = config.get_installation_dir();
            list_installed_versions(installation_dir)?
                .iter()
                .map(|v| Version::new_from_disk(&*v))
                .collect()
        };
        debug!("Prepare list from versions: {:?}", versions);
        let alias_versions = list_aliases_versions(config.aliases_dir())?;

        println!("###############################################################");
        for v in versions {
            let alias_versions = find_alias_version(&alias_versions, &v);
            if v.is_lts() {
                print!("{}", "*".blue());
            } else {
                print!(" ");
            }
            print!("- {} ", v.value);
            if !self.remote {
                print!("- {}", v.provider.as_str().green());
            } else {
                print!(" ");
            }
            for alias_v in alias_versions {
                print!(" [{}]", alias_v.alias_name)
            }
            println!()
        }

        println!("###############################################################");
        println!("{}", "All version marked with * are LTS version".blue());

        Ok(())
    }
}

// TODO: rewrite in a more functional style
pub fn list_installed_versions<P: AsRef<Path>>(installation_dir: P) -> Result<Vec<String>> {
    let mut vec = vec![];
    for result_entry in installation_dir.as_ref().read_dir()? {
        let entry =
            result_entry.context(anyhow!("Error when try to read installation directory."))?;
        if entry.file_name() == ".downloads" {
            continue;
        }

        let path = entry.path();
        let version_number = path
            .file_name()
            .ok_or_else(|| Error::from(ErrorKind::NotFound))?
            .to_str()
            .ok_or_else(|| Error::from(ErrorKind::NotFound))?
            .to_owned();
        vec.push(version_number);
    }
    Ok(vec)
}

#[derive(Debug, Clone)]
pub struct AliasVersion {
    pub version: Option<Version>,
    pub is_invalid: bool,
    pub alias_name: String,
    pub alias_path: PathBuf,
    pub target_path: Option<PathBuf>,
}

impl AliasVersion {
    pub fn contains_version(&self, ver: &Version) -> bool {
        if let Some(av) = &self.version {
            if av == ver {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn new(target_path: PathBuf, alias_path: PathBuf) -> Self {
        let current_path = target_path.as_path();
        let real_file_name = target_path
            .file_name()
            .expect("Cannot get name for version")
            .to_str()
            .expect("Cannot get name for version");

        Self {
            alias_name: alias_path
                .clone()
                .file_name()
                .expect("Cannot get alias file name")
                .to_str()
                .expect("Cannot get alias file name")
                .to_string(),
            alias_path,
            is_invalid: false,
            target_path: Some(current_path.to_path_buf()),
            version: Some(Version::new_from_disk(real_file_name)),
        }
    }
    pub fn new_invalid_alias(alias_path: PathBuf) -> Self {
        Self {
            alias_name: alias_path
                .file_name()
                .expect("Cannot get alias file name")
                .to_str()
                .expect("Cannot get alias file name")
                .to_string(),
            alias_path,
            is_invalid: true,
            target_path: None,
            version: None,
        }
    }
}

pub fn list_aliases_versions<P: AsRef<Path>>(aliases_dir: P) -> Result<Vec<AliasVersion>> {
    let mut vec = vec![];
    for entry in aliases_dir.as_ref().read_dir()? {
        let element = entry?;
        let type_info = element.file_type()?;
        if type_info.is_symlink() {
            let version = match read_link(element.path()) {
                Ok(target_path) => AliasVersion::new(target_path, element.path()),
                Err(_e) => AliasVersion::new_invalid_alias(element.path()),
            };
            vec.push(version);
        };
    }

    Ok(vec)
}
fn find_alias_version(alias_versions: &[AliasVersion], v: &Version) -> Vec<AliasVersion> {
    let result = alias_versions
        .iter()
        .filter(|alias_v| {
            if alias_v.contains_version(v) {
                true
            } else {
                false
            }
        })
        .map(|ver| ver.clone())
        .collect();

    result
}
#[allow(dead_code)]
pub struct VersionPath {
    path: PathBuf,
    version: Version,
}

impl VersionPath {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub fn find_version(to_version: u8, config: &JvcConfig) -> Result<Option<VersionPath>> {
    let all_versions = list_installed_versions(config.get_installation_dir())?;
    debug!("Identified the following versions: {:?}", all_versions);

    let selected_version = all_versions.iter().find(|v| {
        to_version
            == Version::new_from_disk(&**v)
                .to_version_number()
                .expect("Cannot parse installed version")
    });
    let installation_dir = config.get_installation_dir();
    if let Some(ver) = selected_version {
        Ok(Some(VersionPath {
            path: installation_dir.join(ver.to_string()),
            version: Version::new_from_disk(&*ver),
        }))
    } else {
        Ok(None)
    }
}
