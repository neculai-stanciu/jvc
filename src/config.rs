use crate::{loglevel::LogLevel, provider::Provider, version::Version};
use anyhow::Result;
use dirs::home_dir;
use log::debug;
use std::{
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct JvcConfig {
    /// The root directory for jvc. This will contain aliases, all downloaded versions and optional config.
    #[structopt(long = "jvc-dir", env = "JVC_DIR", global = true)]
    pub base_dir: Option<PathBuf>,

    /// Provider used to obtain java version. Possible options: adoptopenjdk.
    #[structopt(
        long = "provider",
        short = "p",
        default_value = "adoptopenjdk",
        env = "JVC_PROVIDER",
        global = true
    )]
    pub java_provider: Provider,

    /// The log level for jvc. Possible options: debug, info, error, silent
    #[structopt(
        long = "log-level",
        short = "ll",
        env = "JVC_LOGLEVEL",
        default_value = "info",
        global = true
    )]
    pub log_level: LogLevel,
}

#[derive(Debug, Clone, StructOpt)]
pub struct VersionRequirements {
    /// Java architecture
    ///
    /// Possible values Azul: x86, arm, mips, ppc, sparcv9
    /// Possible values Adoptopenjdk: x64, x32, ppc64, ppc64le, s390x, aarch64, arm, sparcv9, riscv64
    /// Default value: try detect based on machine info or
    #[structopt(long)]
    pub arch: Option<String>,

    /// Image type
    ///
    /// Possible values Azul: jdk, jre
    /// Possible values Adoptopenjdk: jdk, jre, testimage, debugimage, staticlibs
    /// Default value: jdk
    #[structopt(long, short = "it")]
    pub image_type: Option<String>,

    /// JVM implementation
    ///
    /// Possible values Azul:
    /// Possible values Adoptopenjdk: hotspot, openj9
    /// Default value: hotspot
    #[structopt(long, short = "ji")]
    pub jvm_impl: Option<String>,

    /// Heap size
    ///
    /// Possible values Azul:
    /// Possible values Adoptopenjdk: normal, large
    /// Default value: normal
    #[structopt(long)]
    pub heap_size: Option<String>,

    /// Release type
    ///
    /// Possible values Azul:
    /// Possible values Adoptopenjdk: ga, ea
    /// Default value: ga
    #[structopt(long)]
    pub release_type: Option<String>,

    /// Possible values Azul: N/A
    /// Possible values Adoptopenjdk: adoptopenjdk, openjdk
    /// Default value: adoptopenjdk for Adoptopenjdk
    #[structopt(long, short = "ve")]
    pub vendor: Option<String>,

    /// Project implementation
    ///
    /// Possible values Azul: N/A
    /// Possible values Adoptopenjdk: jdk, valhalla, metropolis, jfr, shenandoah
    /// Default value: N/A
    #[structopt(long)]
    pub project: Option<String>,

    /// Operating system values
    ///
    /// Possible values Azul: linux, linux_musl, macos, windows, solaris, qnx
    /// Possible values Adoptopenjdk:  linux, windows, mac, solaris, aix, alpine-linux
    /// Default value: try detect from machine or fail
    #[structopt(long)]
    pub os: Option<String>,
}

impl Default for VersionRequirements {
    fn default() -> Self {
        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "mac") {
            "mac"
        } else {
            "linux"
        };

        let arch = if cfg!(target_arch = "x86") {
            "x86"
        } else if cfg!(target_arch = "x86_64") {
            "x64"
        } else {
            "x64"
        };

        Self {
            arch: Some(arch.to_owned()),
            image_type: Some("jdk".to_owned()),
            jvm_impl: Some("hotspot".to_owned()),
            heap_size: Some("normal".to_owned()),
            release_type: Some("ga".to_owned()),
            vendor: Some("adoptopenjdk".to_owned()),
            project: Some("jdk".to_owned()),
            os: Some(os.to_owned()),
        }
    }
}

pub struct ClientConfig {
    pub requirements: VersionRequirements,
    pub download_dir: PathBuf,
    pub base_url: String,
    pub version: Version,
}

// not sure if needed
impl Default for JvcConfig {
    fn default() -> Self {
        Self {
            base_dir: None,
            java_provider: Provider::AdoptOpenJDK,
            log_level: LogLevel::Info,
        }
    }
}

impl JvcConfig {
    pub fn get_base_dir_or_default(&self) -> PathBuf {
        let path = self.base_dir.clone().unwrap_or(
            home_dir()
                .expect("Cannot get default base directory")
                .join(".jvc"),
        );
        create_all_dir_if_missing(path)
    }

    pub fn clean_up_downloads_dir(&self) -> Result<()> {
        let download_dir = self.get_download_dir();
        fs::remove_dir_all(download_dir)?;

        Ok(())
    }

    pub fn get_installation_dir(&self) -> PathBuf {
        create_all_dir_if_missing(self.get_base_dir_or_default().join("java-versions"))
    }

    pub fn get_download_dir(&self) -> PathBuf {
        create_all_dir_if_missing(
            self.get_base_dir_or_default()
                .join("java-versions")
                .join(".downloads"),
        )
    }

    pub fn default_version_dir(&self) -> PathBuf {
        self.aliases_dir().join("default")
    }

    pub fn aliases_dir(&self) -> PathBuf {
        create_all_dir_if_missing(self.get_base_dir_or_default().join("aliases"))
    }
}
fn create_all_dir_if_missing<T: AsRef<Path>>(p: T) -> T {
    match create_dir_all(p.as_ref()) {
        Ok(_) => p,
        Err(e) => {
            debug!(
                "cannot create all dirs for path: {} \n {}",
                p.as_ref().to_str().unwrap_or("cannot display path"),
                e
            );
            p
        }
    }
}

pub fn new_client_config(
    config: &JvcConfig,
    version: Version,
    requirements: VersionRequirements,
) -> ClientConfig {
    let provider = &config.java_provider;
    match provider {
        Provider::AdoptOpenJDK => ClientConfig {
            base_url: "https://api.adoptopenjdk.net/v3".to_owned(),
            requirements,
            download_dir: config.get_download_dir(),
            version,
        },
        Provider::Azul => ClientConfig {
            base_url: "https://api.azul.com/zulu/download/community/v1.0".to_owned(),
            requirements,
            download_dir: config.get_download_dir(),
            version,
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::version;

    use std::time::{Duration, Instant};
    use version::Version;

    use super::{create_all_dir_if_missing, new_client_config, JvcConfig, VersionRequirements};

    #[test]
    fn create_new_client_for_azul_should_be_ok() {
        let client = new_client_config(
            &JvcConfig {
                base_dir: dirs::home_dir(),
                java_provider: crate::provider::Provider::Azul,
                log_level: crate::loglevel::LogLevel::Debug,
            },
            Version::new(10, false, crate::provider::Provider::Azul),
            VersionRequirements::default(),
        );
        assert_eq!(
            client.base_url,
            "https://api.azul.com/zulu/download/community/v1.0".to_owned()
        );
        assert_eq!(
            client.requirements.jvm_impl,
            VersionRequirements::default().jvm_impl
        )
    }

    #[test]
    fn create_new_client_for_adoptopenjdk_should_be_ok() {
        let client = new_client_config(
            &JvcConfig {
                base_dir: dirs::home_dir(),
                java_provider: crate::provider::Provider::AdoptOpenJDK,
                log_level: crate::loglevel::LogLevel::Debug,
            },
            Version::new(10, false, crate::provider::Provider::AdoptOpenJDK),
            VersionRequirements::default(),
        );
        assert_eq!(
            client.base_url,
            "https://api.adoptopenjdk.net/v3".to_owned()
        );
        assert_eq!(
            client.requirements.jvm_impl,
            VersionRequirements::default().jvm_impl
        )
    }

    #[test]
    fn aliases_dir_should_be_ok() {
        let aliases_dir = JvcConfig::default().aliases_dir();
        assert!(aliases_dir.as_path().to_str().unwrap().contains("aliases"));
    }

    #[test]
    fn download_dir_should_contain_downloads() {
        let downloads_path = JvcConfig::default().get_download_dir();
        let downloads_as_str = downloads_path.as_path().to_str().unwrap();
        assert!(downloads_as_str.contains("downloads"));
        assert!(downloads_as_str.contains("java-versions"))
    }
    #[test]
    fn create_all_dir_if_missing_should_work() {
        let config = JvcConfig::default();
        let downloads_path = config.get_download_dir();
        let timestamp = Instant::now();
        let first_dir = timestamp.elapsed().as_millis().to_string();
        let next_dir = timestamp
            .elapsed()
            .checked_add(Duration::new(3, 999))
            .unwrap()
            .as_millis()
            .to_string();
        let random_folder_names = downloads_path
            .join(first_dir.clone())
            .join(next_dir.clone());
        create_all_dir_if_missing(random_folder_names);
        assert!(downloads_path.join(first_dir.clone()).is_dir());
        assert!(downloads_path.join(first_dir).join(next_dir).is_dir());

        let result = config.clean_up_downloads_dir();
        assert!(result.is_ok());
    }
}
